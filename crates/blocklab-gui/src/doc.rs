//! Documento aberto: o diagrama mais o histórico de desfazer/refazer.
//!
//! O histórico guarda *cópias inteiras* do diagrama (snapshots). Para
//! diagramas de dezenas de nós isso é barato e muito mais simples (e
//! robusto) do que registrar cada operação e seu inverso.

use blocklab_core::diagram::{Diagram, NodeKind};

const MAX_HISTORY: usize = 200;

pub struct Document {
    pub diagram: Diagram,
    undo: Vec<Diagram>,
    redo: Vec<Diagram>,
    /// Snapshot tirado no começo de um gesto (arrastar), confirmado no fim.
    gesture: Option<Diagram>,
    /// Muda a cada alteração; quem observa o documento compara com o
    /// último valor visto.
    pub revision: u64,
}

impl Document {
    pub fn new(diagram: Diagram) -> Self {
        Document {
            diagram,
            undo: Vec::new(),
            redo: Vec::new(),
            gesture: None,
            revision: 0,
        }
    }

    fn push_undo(&mut self, snapshot: Diagram) {
        self.undo.push(snapshot);
        if self.undo.len() > MAX_HISTORY {
            self.undo.remove(0);
        }
        self.redo.clear();
    }

    /// Alteração "de uma vez" (apagar, ligar, trocar sinal...): entra no
    /// histórico como um passo.
    pub fn edit(&mut self, f: impl FnOnce(&mut Diagram)) {
        let before = self.diagram.clone();
        f(&mut self.diagram);
        compact_branches(&mut self.diagram);
        if self.diagram != before {
            self.push_undo(before);
            self.revision += 1;
        }
    }

    /// Começa um gesto (ex.: arrastar nós): as mudanças até
    /// [`Self::end_gesture`] viram um único passo no histórico.
    pub fn begin_gesture(&mut self) {
        self.gesture = Some(self.diagram.clone());
    }

    /// Mudança dentro de um gesto (não cria passo de histórico sozinha).
    pub fn gesture_edit(&mut self, f: impl FnOnce(&mut Diagram)) {
        f(&mut self.diagram);
        self.revision += 1;
    }

    pub fn end_gesture(&mut self) {
        if let Some(before) = self.gesture.take() {
            compact_branches(&mut self.diagram);
            if self.diagram != before {
                self.push_undo(before);
                self.revision += 1;
            }
        }
    }

    /// Desiste do gesto: o diagrama volta a como estava no começo.
    pub fn cancel_gesture(&mut self) {
        if let Some(before) = self.gesture.take() {
            self.diagram = before;
            self.revision += 1;
        }
    }

    pub fn in_gesture(&self) -> bool {
        self.gesture.is_some()
    }

    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    pub fn undo(&mut self) {
        if let Some(prev) = self.undo.pop() {
            self.redo.push(std::mem::replace(&mut self.diagram, prev));
            self.revision += 1;
        }
    }

    pub fn redo(&mut self) {
        if let Some(next) = self.redo.pop() {
            self.undo.push(std::mem::replace(&mut self.diagram, next));
            self.revision += 1;
        }
    }
}

/// As saídas de uma ramificação são geridas automaticamente: uma por fio
/// que sai dela, numeradas 0, 1, 2... sem buracos. Chamado depois de toda
/// edição, então apagar um fio nunca deixa uma saída "solta".
pub fn compact_branches(d: &mut Diagram) {
    let branches: Vec<_> = d
        .nodes
        .iter()
        .filter(|n| matches!(n.kind, NodeKind::Branch { .. }))
        .map(|n| n.id)
        .collect();
    for id in branches {
        let mut outs: Vec<usize> = (0..d.wires.len())
            .filter(|&i| d.wires[i].from.node == id)
            .collect();
        outs.sort_by_key(|&i| (d.wires[i].from.port, d.wires[i].id));
        for (k, &i) in outs.iter().enumerate() {
            d.wires[i].from.port = k as u8;
        }
        if let Some(n) = d.node_mut(id) {
            n.kind = NodeKind::Branch {
                outputs: outs.len() as u8,
            };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use blocklab_core::diagram::{PortRef, Pos, Sign, examples};

    #[test]
    fn desfazer_e_refazer() {
        let mut doc = Document::new(examples::series());
        let n0 = doc.diagram.nodes.len();
        doc.edit(|d| {
            d.add_block("G9", Pos::new(0.0, 0.0));
        });
        assert_eq!(doc.diagram.nodes.len(), n0 + 1);
        doc.undo();
        assert_eq!(doc.diagram.nodes.len(), n0);
        doc.redo();
        assert_eq!(doc.diagram.nodes.len(), n0 + 1);
        // Edição nova apaga o "refazer".
        doc.undo();
        doc.edit(|d| d.nodes[0].pos.x += 20.0);
        assert!(!doc.can_redo());
    }

    #[test]
    fn edicao_sem_efeito_nao_entra_no_historico() {
        let mut doc = Document::new(examples::series());
        doc.edit(|_| {});
        assert!(!doc.can_undo());
    }

    #[test]
    fn gesto_vira_um_passo_so() {
        let mut doc = Document::new(examples::series());
        doc.begin_gesture();
        for _ in 0..10 {
            doc.gesture_edit(|d| d.nodes[0].pos.x += 1.0);
        }
        doc.end_gesture();
        doc.undo();
        assert_eq!(doc.diagram, examples::series());
        assert!(!doc.can_undo());
    }

    #[test]
    fn saidas_de_ramificacao_se_renumeram() {
        // Ramificação A do exemplo de referência tem 3 saídas; apagar o
        // fio da saída 1 deixa 2 saídas, numeradas 0 e 1.
        let mut doc = Document::new(examples::reference(Sign::Plus));
        let a = doc
            .diagram
            .nodes
            .iter()
            .find(|n| n.kind == NodeKind::Branch { outputs: 3 })
            .unwrap()
            .id;
        let w = doc
            .diagram
            .wires
            .iter()
            .find(|w| w.from == PortRef::new(a, 1))
            .unwrap()
            .id;
        doc.edit(|d| d.remove_wire(w));
        let ports: Vec<u8> = doc
            .diagram
            .wires
            .iter()
            .filter(|w| w.from.node == a)
            .map(|w| w.from.port)
            .collect();
        assert_eq!(
            doc.diagram.node(a).unwrap().kind,
            NodeKind::Branch { outputs: 2 }
        );
        let mut sorted = ports.clone();
        sorted.sort();
        assert_eq!(sorted, vec![0, 1]);
    }
}
