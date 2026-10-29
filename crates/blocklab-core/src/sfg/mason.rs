//! Fórmula de ganho de Mason.
//!
//! ```text
//!         Σₖ Pₖ · Δₖ
//!   T  =  ──────────
//!             Δ
//!
//!   Δ  = 1 − Σ Lᵢ + Σ LᵢLⱼ − Σ LᵢLⱼLₖ + ...
//! ```
//!
//! * **Pₖ** — ganho do k-ésimo caminho direto (produto dos ramos).
//! * **Lᵢ** — ganho do i-ésimo laço.
//! * **Δ** — o *determinante do grafo*. A primeira soma corre sobre todos
//!   os laços; a segunda, sobre todos os **pares de laços que não se
//!   tocam** (sem nó em comum); a terceira, sobre os trios mutuamente
//!   disjuntos, e assim por diante, alternando o sinal.
//! * **Δₖ** — o mesmo Δ, mas só com os laços que não tocam o caminho k.
//!
//! Por que funciona: o SFG é um sistema linear x = A·x + b·X. Resolvendo
//! por Cramer, o denominador comum é det(I − A), e expandir esse
//! determinante pela definição (permutações = coleções de ciclos
//! disjuntos) dá exatamente Δ. O numerador é um cofator, que vira
//! Σ Pₖ Δₖ. O teste `mason_props.rs` confere isso contra a resolução
//! direta do sistema por eliminação de Gauss.
//!
//! Uma forma compacta (usada no código): Δ é a soma, sobre **todos** os
//! conjuntos S de laços mutuamente disjuntos (inclusive o vazio, que vale
//! 1), de (−1)^|S| · Π_{L ∈ S} L. Δₖ é a mesma soma restrita aos
//! conjuntos que não tocam o caminho k.

use super::johnson::simple_cycles;
use super::paths::forward_paths;
use super::{Sfg, SolveError};
use crate::algebra::{CancelToken, Poly, RationalFn};

/// Máximo de laços aceitos (proteção contra grafos patológicos).
pub const MAX_LOOPS: usize = 2_000;
/// Máximo de conjuntos de laços disjuntos.
pub const MAX_COMBINATIONS: usize = 200_000;

/// Caminho direto ou laço, com seu ganho.
#[derive(Clone, Debug)]
pub struct GainPath {
    /// Nós do SFG na ordem percorrida (num laço, sem repetir o primeiro).
    pub nodes: Vec<usize>,
    pub gain: RationalFn,
}

impl GainPath {
    /// "X → Σ1 → G1 → Y". Num laço, `closed = true` repete o primeiro nó
    /// no fim: "Σ2 → G1 → H1 → Σ2".
    pub fn describe(&self, sfg: &Sfg, closed: bool) -> String {
        let mut labels: Vec<&str> = self
            .nodes
            .iter()
            .map(|&i| sfg.nodes[i].label.as_str())
            .collect();
        if closed {
            labels.push(&sfg.nodes[self.nodes[0]].label);
        }
        labels.join(" → ")
    }
}

/// Tudo o que a fórmula de Mason usou, para a lista didática.
#[derive(Clone, Debug)]
pub struct MasonReport {
    pub sfg: Sfg,
    /// Caminhos diretos Pₖ.
    pub paths: Vec<GainPath>,
    /// Laços Lᵢ.
    pub loops: Vec<GainPath>,
    /// Combinações de 2 ou mais laços que não se tocam (índices em
    /// `loops`), em ordem de tamanho.
    pub nontouching: Vec<Vec<usize>>,
    /// Δ.
    pub delta: RationalFn,
    /// Δₖ de cada caminho (mesma ordem de `paths`).
    pub cofactors: Vec<RationalFn>,
    /// T = Y/X.
    pub transfer: RationalFn,
}

/// Conjunto de nós como vetor de bits (testar "se tocam" vira um AND).
#[derive(Clone)]
struct NodeSet(Vec<u64>);

impl NodeSet {
    fn new(n: usize, nodes: &[usize]) -> Self {
        let mut bits = vec![0u64; n.div_ceil(64)];
        for &v in nodes {
            bits[v / 64] |= 1 << (v % 64);
        }
        NodeSet(bits)
    }

    fn touches(&self, other: &NodeSet) -> bool {
        self.0.iter().zip(&other.0).any(|(a, b)| a & b != 0)
    }

    fn union(&self, other: &NodeSet) -> NodeSet {
        NodeSet(self.0.iter().zip(&other.0).map(|(a, b)| a | b).collect())
    }
}

/// Um conjunto S de laços mutuamente disjuntos. Seu termo em Δ é
/// (−1)^|S| · Π dos ganhos dos ramos em `edges`.
struct Combo {
    loops: Vec<usize>,
    nodes: NodeSet,
    /// Ramos de todos os laços do conjunto (sem repetição: laços
    /// disjuntos não compartilham nó, logo nem ramo).
    edges: Vec<usize>,
}

/// Um termo de uma soma de Mason: ± produto dos ganhos de alguns ramos.
struct Term {
    negative: bool,
    edges: Vec<usize>,
}

/// Soma de termos ±Π gₑ sem nenhum MDC intermediário.
///
/// Cada ganho é uma fração gₑ = nₑ/dₑ. Tomando D = Π dₑ sobre os ramos
/// `involved` (que contém todos os ramos de todos os termos), cada termo
/// vira um polinômio quando multiplicado por D:
///
/// ```text
/// D · Π_{e∈termo} gₑ  =  Π_{e∈termo} nₑ · Π_{e∉termo} dₑ
/// ```
///
/// Isso só funciona porque um termo nunca usa o mesmo ramo duas vezes.
/// Devolve (N, D) com soma = N/D, ainda sem simplificar.
fn sum_of_products(
    sfg: &Sfg,
    terms: &[Term],
    involved: &[usize],
    cancel: &CancelToken,
) -> Result<(Poly, Poly), SolveError> {
    let mut num = Poly::zero();
    for t in terms {
        if cancel.is_cancelled() {
            return Err(SolveError::Cancelled);
        }
        let mut prod = Poly::one();
        for e in &t.edges {
            prod = &prod * sfg.edges[*e].gain.num();
        }
        for e in involved {
            if !t.edges.contains(e) {
                prod = &prod * sfg.edges[*e].gain.den();
            }
        }
        num = if t.negative {
            &num - &prod
        } else {
            &num + &prod
        };
    }
    let den = involved
        .iter()
        .fold(Poly::one(), |acc, e| &acc * sfg.edges[*e].gain.den());
    Ok((num, den))
}

/// Ramos com denominador ≠ 1 usados por algum termo (os outros não
/// contribuem para D).
fn involved_edges(sfg: &Sfg, terms: &[Term]) -> Vec<usize> {
    let mut v: Vec<usize> = terms
        .iter()
        .flat_map(|t| t.edges.iter().copied())
        .filter(|&e| *sfg.edges[e].gain.den() != Poly::one())
        .collect();
    v.sort_unstable();
    v.dedup();
    v
}

/// Índices dos ramos ao longo de `nodes`. Se `closed`, inclui o ramo do
/// último nó de volta ao primeiro.
fn chain_edges(sfg: &Sfg, nodes: &[usize], closed: bool) -> Vec<usize> {
    let mut pairs: Vec<(usize, usize)> = nodes.windows(2).map(|w| (w[0], w[1])).collect();
    if closed {
        pairs.push((nodes[nodes.len() - 1], nodes[0]));
    }
    pairs
        .iter()
        .map(|&(a, b)| {
            sfg.edges
                .iter()
                .position(|e| e.from == a && e.to == b)
                .expect("ramo do caminho existe")
        })
        .collect()
}

fn edges_gain(sfg: &Sfg, edges: &[usize]) -> RationalFn {
    RationalFn::product(edges.iter().map(|&e| &sfg.edges[e].gain))
}

/// Aplica Mason a um SFG. Requer que `source` não tenha ramos chegando.
pub fn mason(sfg: Sfg, cancel: &CancelToken) -> Result<MasonReport, SolveError> {
    let n = sfg.nodes.len();
    let succ = sfg.successors();
    let check = || {
        if cancel.is_cancelled() {
            Err(SolveError::Cancelled)
        } else {
            Ok(())
        }
    };

    // 1. Caminhos diretos e laços, com seus ramos e ganhos.
    let path_nodes = forward_paths(&succ, sfg.source, sfg.sink, cancel)?;
    let path_edges: Vec<Vec<usize>> = path_nodes
        .iter()
        .map(|p| chain_edges(&sfg, p, false))
        .collect();
    let paths: Vec<GainPath> = path_nodes
        .into_iter()
        .zip(&path_edges)
        .map(|(nodes, edges)| GainPath {
            gain: edges_gain(&sfg, edges),
            nodes,
        })
        .collect();
    check()?;
    let loop_nodes = simple_cycles(&succ, MAX_LOOPS, cancel)?;
    let loop_edges: Vec<Vec<usize>> = loop_nodes
        .iter()
        .map(|l| chain_edges(&sfg, l, true))
        .collect();
    let loops: Vec<GainPath> = loop_nodes
        .into_iter()
        .zip(&loop_edges)
        .map(|(nodes, edges)| GainPath {
            gain: edges_gain(&sfg, edges),
            nodes,
        })
        .collect();
    let loop_sets: Vec<NodeSet> = loops.iter().map(|l| NodeSet::new(n, &l.nodes)).collect();

    // 2. Todos os conjuntos de laços mutuamente disjuntos (tamanho ≥ 1),
    //    por backtracking: a partir de um conjunto, tenta acrescentar cada
    //    laço de índice maior que não toque nenhum dos já escolhidos.
    let mut combos: Vec<Combo> = Vec::new();
    let mut frontier: Vec<usize> = Vec::new(); // índices em `combos` a expandir
    for j in 0..loops.len() {
        combos.push(Combo {
            loops: vec![j],
            nodes: loop_sets[j].clone(),
            edges: loop_edges[j].clone(),
        });
        frontier.push(combos.len() - 1);
    }
    while let Some(c) = frontier.pop() {
        check()?;
        let last = *combos[c].loops.last().expect("conjunto não vazio");
        for j in last + 1..loops.len() {
            if combos[c].nodes.touches(&loop_sets[j]) {
                continue;
            }
            if combos.len() >= MAX_COMBINATIONS {
                return Err(SolveError::TooComplex {
                    what: "combinações de laços que não se tocam",
                    limit: MAX_COMBINATIONS,
                });
            }
            let parent = &combos[c];
            let mut ids = parent.loops.clone();
            ids.push(j);
            let mut edges = parent.edges.clone();
            edges.extend_from_slice(&loop_edges[j]);
            let child = Combo {
                loops: ids,
                nodes: parent.nodes.union(&loop_sets[j]),
                edges,
            };
            combos.push(child);
            frontier.push(combos.len() - 1);
        }
    }

    // 3. Termos de Δ (o vazio vale +1) e de cada Δₖ: só os conjuntos que
    //    não tocam o caminho k.
    let delta_terms = |keep: &dyn Fn(&Combo) -> bool, extra: &[usize]| -> Vec<Term> {
        std::iter::once(Term {
            negative: false,
            edges: extra.to_vec(),
        })
        .chain(combos.iter().filter(|c| keep(c)).map(|c| {
            let mut edges = extra.to_vec();
            edges.extend_from_slice(&c.edges);
            Term {
                negative: c.loops.len() % 2 == 1,
                edges,
            }
        }))
        .collect()
    };
    let touches_path: Vec<NodeSet> = paths.iter().map(|p| NodeSet::new(n, &p.nodes)).collect();
    let dterms = delta_terms(&|_| true, &[]);
    // Para a lista didática, Δ e cada Δₖ como frações simplificadas.
    let as_fraction = |terms: &[Term]| -> Result<RationalFn, SolveError> {
        let (num, den) = sum_of_products(&sfg, terms, &involved_edges(&sfg, terms), cancel)?;
        Ok(RationalFn::new(num, den).expect("D é produto de denominadores ≠ 0"))
    };
    let delta = as_fraction(&dterms)?;
    check()?;
    let mut cofactors = Vec::with_capacity(paths.len());
    for p_set in &touches_path {
        cofactors.push(as_fraction(&delta_terms(
            &|c| !c.nodes.touches(p_set),
            &[],
        ))?);
        check()?;
    }
    if delta.is_zero() {
        return Err(SolveError::DegenerateDelta);
    }

    // 4. T = Σ Pₖ Δₖ / Δ. Numerador e denominador usam o MESMO D (todos os
    //    ramos envolvidos em qualquer um dos dois), então D se cancela e
    //    T = N_T / N_Δ: um único MDC no fim.
    let mut tterms = Vec::new();
    for (k, p_set) in touches_path.iter().enumerate() {
        tterms.extend(delta_terms(&|c| !c.nodes.touches(p_set), &path_edges[k]));
    }
    let mut involved = involved_edges(&sfg, &tterms);
    involved.extend(involved_edges(&sfg, &dterms));
    involved.sort_unstable();
    involved.dedup();
    let (n_t, _) = sum_of_products(&sfg, &tterms, &involved, cancel)?;
    let (n_delta, _) = sum_of_products(&sfg, &dterms, &involved, cancel)?;
    let transfer = RationalFn::new(n_t, n_delta).map_err(|_| SolveError::DegenerateDelta)?;
    // O MDC pode ter abandonado simplificações se o cancelamento chegou
    // no meio; nesse caso o resultado não é canônico e é descartado.
    check()?;

    let mut nontouching: Vec<Vec<usize>> = combos
        .into_iter()
        .filter(|c| c.loops.len() >= 2)
        .map(|c| c.loops)
        .collect();
    nontouching.sort_by(|a, b| a.len().cmp(&b.len()).then_with(|| a.cmp(b)));

    Ok(MasonReport {
        sfg,
        paths,
        loops,
        nontouching,
        delta,
        cofactors,
        transfer,
    })
}
