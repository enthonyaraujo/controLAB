//! Grafo de fluxo de sinal (SFG) e fórmula de ganho de Mason.
//!
//! Um SFG é um grafo dirigido em que cada **nó é um sinal** e cada
//! **ramo multiplica** o sinal de origem pelo seu ganho; o valor de um nó
//! é a soma de tudo que chega nele. Um diagrama de blocos vira SFG assim:
//!
//! * cada bloco G vira um ramo com ganho G;
//! * cada entrada de somador vira um ramo com ganho +1 ou −1;
//! * ramificações não viram nada: só repetem o mesmo sinal.
//!
//! Sobre o SFG, a fórmula de Mason dá a função de transferência sem
//! precisar das regras de redução de blocos (ver `mason.rs`).

mod build;
mod johnson;
mod mason;
mod paths;

use thiserror::Error;

pub use build::build;
pub use mason::{GainPath, MAX_COMBINATIONS, MAX_LOOPS, MasonReport, mason};

use crate::algebra::{CancelToken, RationalFn, with_cancel};
use crate::diagram::{Diagram, DiagramError, NodeId, validate};

/// Um nó do SFG (um sinal).
#[derive(Clone, Debug)]
pub struct SfgNode {
    /// Rótulo para a lista didática: "X", "Σ1", "G1" (sinal na saída do
    /// bloco G1), "Y".
    pub label: String,
    /// Nó do diagrama que gera este sinal (`None` em grafos montados à mão).
    pub origin: Option<NodeId>,
}

/// Um ramo do SFG.
#[derive(Clone, Debug)]
pub struct SfgEdge {
    pub from: usize,
    pub to: usize,
    pub gain: RationalFn,
}

#[derive(Clone, Debug, Default)]
pub struct Sfg {
    pub nodes: Vec<SfgNode>,
    pub edges: Vec<SfgEdge>,
    /// Nó de entrada (não pode ter ramos chegando).
    pub source: usize,
    /// Nó de saída.
    pub sink: usize,
}

impl Sfg {
    pub fn add_node(&mut self, label: impl Into<String>, origin: Option<NodeId>) -> usize {
        self.nodes.push(SfgNode {
            label: label.into(),
            origin,
        });
        self.nodes.len() - 1
    }

    /// Adiciona um ramo. Ramos paralelos (mesma origem e destino) são
    /// somados num só — regra básica de SFG: a e b em paralelo = a + b.
    /// Se a soma der zero, o ramo some.
    pub fn add_edge(&mut self, from: usize, to: usize, gain: RationalFn) {
        match self.edges.iter().position(|e| e.from == from && e.to == to) {
            Some(i) => {
                let total = &self.edges[i].gain + &gain;
                if total.is_zero() {
                    self.edges.remove(i);
                } else {
                    self.edges[i].gain = total;
                }
            }
            None if !gain.is_zero() => self.edges.push(SfgEdge { from, to, gain }),
            None => {}
        }
    }

    /// O ramo de `from` para `to`, se existir (é único, ver `add_edge`).
    pub fn edge(&self, from: usize, to: usize) -> Option<&SfgEdge> {
        self.edges.iter().find(|e| e.from == from && e.to == to)
    }

    /// Lista de sucessores de cada nó, em ordem crescente.
    pub(crate) fn successors(&self) -> Vec<Vec<usize>> {
        let mut succ = vec![Vec::new(); self.nodes.len()];
        for e in &self.edges {
            succ[e.from].push(e.to);
        }
        for s in &mut succ {
            s.sort_unstable();
        }
        succ
    }
}

#[derive(Debug, Clone, PartialEq, Error)]
pub enum SolveError {
    #[error("o diagrama tem {} problema(s)", .0.len())]
    Invalid(Vec<DiagramError>),
    #[error(
        "Δ = 0: há um laço algébrico de ganho 1 (realimentação sem nenhum bloco), \
         e o sistema não tem solução única"
    )]
    DegenerateDelta,
    #[error("cálculo cancelado")]
    Cancelled,
    #[error("diagrama complexo demais: mais de {limit} {what}")]
    TooComplex { what: &'static str, limit: usize },
}

/// Valida o diagrama, monta o SFG e aplica Mason.
pub fn solve(d: &Diagram) -> Result<MasonReport, SolveError> {
    solve_with(d, &CancelToken::new())
}

/// Igual a [`solve`], mas pode ser interrompido por outra thread com
/// [`CancelToken::cancel`]; nesse caso devolve [`SolveError::Cancelled`].
pub fn solve_with(d: &Diagram, cancel: &CancelToken) -> Result<MasonReport, SolveError> {
    let gains = validate(d).map_err(SolveError::Invalid)?;
    let sfg = build(d, &gains);
    with_cancel(cancel, || mason(sfg, cancel))
}
