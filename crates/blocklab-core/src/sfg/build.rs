//! Diagrama de blocos → grafo de fluxo de sinal.

use std::collections::HashMap;

use super::Sfg;
use crate::algebra::RationalFn;
use crate::diagram::{BlockGains, Diagram, NodeKind, PortRef, Sign};

/// Monta o SFG de um diagrama **já validado** (`gains` vem de
/// [`crate::diagram::validate`]).
///
/// Nós do SFG: a entrada, a saída de cada bloco, a saída de cada somador
/// e um nó final para a saída Y. Ramificações não geram nó: suas saídas
/// carregam o mesmo sinal que entra nelas.
///
/// # Panics
/// Se o diagrama não passou pela validação (porta solta, etc.).
pub fn build(d: &Diagram, gains: &BlockGains) -> Sfg {
    let mut g = Sfg::default();
    // Sinal (nó do SFG) gerado por cada porta de saída "de verdade".
    let mut signal: HashMap<PortRef, usize> = HashMap::new();
    let mut sums = 0;
    for n in &d.nodes {
        let label = match &n.kind {
            NodeKind::Input { name } => name.clone(),
            NodeKind::Block { expr } => expr.trim().to_string(),
            NodeKind::Sum { .. } => {
                sums += 1;
                format!("Σ{sums}")
            }
            NodeKind::Output { name } => {
                g.sink = g.add_node(name.clone(), Some(n.id));
                continue;
            }
            NodeKind::Branch { .. } => continue,
        };
        let idx = g.add_node(label, Some(n.id));
        signal.insert(PortRef::new(n.id, 0), idx);
        if matches!(n.kind, NodeKind::Input { .. }) {
            g.source = idx;
        }
    }

    // Sinal que chega numa porta de entrada: segue o fio para trás,
    // atravessando ramificações até achar quem realmente gera o sinal.
    // (Uma cadeia de ramificações sem fonte é barrada pela validação.)
    let source_of = |mut port: PortRef| -> usize {
        for _ in 0..=d.nodes.len() {
            let w = d
                .wire_into(port)
                .expect("diagrama validado: entrada ligada");
            match signal.get(&PortRef::new(w.from.node, 0)) {
                Some(&idx) => return idx,
                // Veio de uma ramificação: continua pela entrada dela.
                None => port = PortRef::new(w.from.node, 0),
            }
        }
        panic!("ciclo só de ramificações: o diagrama não foi validado")
    };

    let mut edges = Vec::new();
    for n in &d.nodes {
        match &n.kind {
            NodeKind::Block { .. } => {
                let gain = gains[&n.id].clone();
                edges.push((
                    source_of(PortRef::new(n.id, 0)),
                    signal[&PortRef::new(n.id, 0)],
                    gain,
                ));
            }
            NodeKind::Sum { inputs } => {
                let out = signal[&PortRef::new(n.id, 0)];
                for (i, input) in inputs.iter().enumerate() {
                    let gain = match input.sign {
                        Sign::Plus => RationalFn::one(),
                        Sign::Minus => -RationalFn::one(),
                    };
                    edges.push((source_of(PortRef::new(n.id, i as u8)), out, gain));
                }
            }
            NodeKind::Output { .. } => {
                edges.push((source_of(PortRef::new(n.id, 0)), g.sink, RationalFn::one()));
            }
            NodeKind::Input { .. } | NodeKind::Branch { .. } => {}
        }
    }
    for (from, to, gain) in edges {
        g.add_edge(from, to, gain);
    }
    g
}
