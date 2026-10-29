//! Validação do diagrama antes do cálculo. Todo erro aponta o(s) nó(s)
//! responsável(is), para a interface destacar.

use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};

use thiserror::Error;

use super::model::{Diagram, NodeId, NodeKind, PortDir, PortRef, WireId};
use crate::algebra::{Params, RationalFn, Var};
use crate::parser::{ExprKind, ParseError, parse, parse_rational};

/// Ganho já convertido de cada bloco.
pub type BlockGains = BTreeMap<NodeId, RationalFn>;

#[derive(Debug, Clone, PartialEq, Error)]
pub enum DiagramError {
    #[error("o diagrama não tem entrada")]
    NoInput,
    #[error("o diagrama não tem saída")]
    NoOutput,
    #[error("há mais de uma entrada; nesta versão o diagrama tem só uma")]
    MultipleInputs(Vec<NodeId>),
    #[error("há mais de uma saída; nesta versão o diagrama tem só uma")]
    MultipleOutputs(Vec<NodeId>),
    #[error("o fio {0:?} liga portas que não existem")]
    InvalidWire(WireId),
    #[error("porta de {dir} {port} solta (sem fio)")]
    LoosePort {
        node: NodeId,
        port: u8,
        dir: PortDir,
    },
    #[error("a porta de entrada {port} recebe mais de um fio")]
    MultipleWiresIntoPort { node: NodeId, port: u8 },
    #[error("expressão inválida: {err}")]
    BadExpr { node: NodeId, err: ParseError },
    #[error("não há caminho deste nó até a saída")]
    NoPathToOutput(NodeId),
    #[error("este nó não recebe sinal da entrada")]
    UnreachableFromInput(NodeId),
}

impl DiagramError {
    /// Nós envolvidos (para destacar na interface).
    pub fn nodes(&self) -> Vec<NodeId> {
        match self {
            DiagramError::NoInput | DiagramError::NoOutput | DiagramError::InvalidWire(_) => {
                vec![]
            }
            DiagramError::MultipleInputs(ids) | DiagramError::MultipleOutputs(ids) => ids.clone(),
            DiagramError::LoosePort { node, .. }
            | DiagramError::MultipleWiresIntoPort { node, .. }
            | DiagramError::BadExpr { node, .. }
            | DiagramError::NoPathToOutput(node)
            | DiagramError::UnreachableFromInput(node) => vec![*node],
        }
    }

    /// Mensagem completa, nomeando o nó: "bloco 'G1' (#7): expressão
    /// inválida: posição 3: ...".
    pub fn describe(&self, d: &Diagram) -> String {
        match self.nodes().as_slice() {
            [id] => match d.node(*id) {
                Some(n) => format!("{}: {self}", n.describe()),
                None => self.to_string(),
            },
            _ => self.to_string(),
        }
    }
}

/// Confere o diagrama inteiro e devolve os ganhos dos blocos, ou a lista
/// de **todos** os problemas encontrados (não só o primeiro).
pub fn validate(d: &Diagram) -> Result<BlockGains, Vec<DiagramError>> {
    let mut errors = Vec::new();

    // 1. Exatamente uma entrada e uma saída.
    let ids_of = |pred: fn(&NodeKind) -> bool| -> Vec<NodeId> {
        d.nodes
            .iter()
            .filter(|n| pred(&n.kind))
            .map(|n| n.id)
            .collect()
    };
    let inputs = ids_of(|k| matches!(k, NodeKind::Input { .. }));
    let outputs = ids_of(|k| matches!(k, NodeKind::Output { .. }));
    match inputs.len() {
        0 => errors.push(DiagramError::NoInput),
        1 => {}
        _ => errors.push(DiagramError::MultipleInputs(inputs.clone())),
    }
    match outputs.len() {
        0 => errors.push(DiagramError::NoOutput),
        1 => {}
        _ => errors.push(DiagramError::MultipleOutputs(outputs.clone())),
    }

    // 2. Fios apontando para portas que existem (arquivo corrompido ou
    //    editado à mão). Os inválidos são ignorados no resto da análise.
    let port_exists = |p: PortRef, dir: PortDir| {
        d.node(p.node).is_some_and(|n| {
            let count = match dir {
                PortDir::In => n.n_inputs(),
                PortDir::Out => n.n_outputs(),
            };
            (p.port as usize) < count
        })
    };
    let mut wires = Vec::new();
    for w in &d.wires {
        if port_exists(w.from, PortDir::Out) && port_exists(w.to, PortDir::In) {
            wires.push(w);
        } else {
            errors.push(DiagramError::InvalidWire(w.id));
        }
    }

    // 3. Portas soltas: toda entrada com exatamente um fio, toda saída
    //    com pelo menos um.
    let mut into: HashMap<PortRef, usize> = HashMap::new();
    let mut used_out: HashSet<PortRef> = HashSet::new();
    for w in &wires {
        *into.entry(w.to).or_default() += 1;
        used_out.insert(w.from);
    }
    for n in &d.nodes {
        for port in 0..n.n_inputs() as u8 {
            match into.get(&PortRef::new(n.id, port)).copied().unwrap_or(0) {
                0 => errors.push(DiagramError::LoosePort {
                    node: n.id,
                    port,
                    dir: PortDir::In,
                }),
                1 => {}
                _ => errors.push(DiagramError::MultipleWiresIntoPort { node: n.id, port }),
            }
        }
        for port in 0..n.n_outputs() as u8 {
            if !used_out.contains(&PortRef::new(n.id, port)) {
                errors.push(DiagramError::LoosePort {
                    node: n.id,
                    port,
                    dir: PortDir::Out,
                });
            }
        }
    }

    // 4. Expressões dos blocos.
    let mut gains = BlockGains::new();
    for n in &d.nodes {
        if let NodeKind::Block { expr } = &n.kind {
            match parse_rational(expr) {
                Ok(g) => {
                    gains.insert(n.id, g);
                }
                Err(err) => errors.push(DiagramError::BadExpr { node: n.id, err }),
            }
        }
    }

    // 5. Alcançabilidade: busca em largura a partir da entrada (seguindo
    //    os fios) e a partir da saída (seguindo os fios ao contrário).
    if let ([input], [output]) = (inputs.as_slice(), outputs.as_slice()) {
        let forward = reachable(*input, &wires, |w| (w.from.node, w.to.node));
        let backward = reachable(*output, &wires, |w| (w.to.node, w.from.node));
        for n in &d.nodes {
            if !forward.contains(&n.id) {
                errors.push(DiagramError::UnreachableFromInput(n.id));
            }
            if !backward.contains(&n.id) {
                errors.push(DiagramError::NoPathToOutput(n.id));
            }
        }
    }

    if errors.is_empty() {
        Ok(gains)
    } else {
        Err(errors)
    }
}

/// Nós alcançáveis a partir de `start`; `edge` diz a direção de cada fio.
fn reachable(
    start: NodeId,
    wires: &[&super::model::Wire],
    edge: impl Fn(&super::model::Wire) -> (NodeId, NodeId),
) -> HashSet<NodeId> {
    let mut adj: HashMap<NodeId, Vec<NodeId>> = HashMap::new();
    for w in wires {
        let (a, b) = edge(w);
        adj.entry(a).or_default().push(b);
    }
    let mut seen = HashSet::from([start]);
    let mut queue = VecDeque::from([start]);
    while let Some(v) = queue.pop_front() {
        for &w in adj.get(&v).into_iter().flatten() {
            if seen.insert(w) {
                queue.push_back(w);
            }
        }
    }
    seen
}

#[derive(Debug, Clone, PartialEq, Error)]
pub enum ParamError {
    #[error("'{0}' não é um nome de parâmetro válido")]
    BadName(String),
    #[error("parâmetro '{name}': {err}")]
    Parse { name: String, err: ParseError },
    #[error("parâmetro '{name}' precisa ser um número, mas vale '{value}'")]
    NotNumeric { name: String, value: String },
}

impl Diagram {
    /// Converte a tabela de parâmetros (texto) em valores exatos.
    /// Valores vazios são ignorados (o símbolo continua simbólico).
    pub fn numeric_params(&self) -> Result<Params, ParamError> {
        let mut out = Params::new();
        for (name, value) in &self.params {
            if value.trim().is_empty() {
                continue;
            }
            // O nome tem que ser lido pelo parser como um único símbolo.
            let symbol = match parse(name).map(|e| e.kind) {
                Ok(ExprKind::Var(Var::Sym(sym))) => sym,
                _ => return Err(ParamError::BadName(name.clone())),
            };
            let v = parse_rational(value).map_err(|err| ParamError::Parse {
                name: name.clone(),
                err,
            })?;
            let c = v.as_constant().ok_or_else(|| ParamError::NotNumeric {
                name: name.clone(),
                value: value.clone(),
            })?;
            out.insert(symbol, c);
        }
        Ok(out)
    }
}
