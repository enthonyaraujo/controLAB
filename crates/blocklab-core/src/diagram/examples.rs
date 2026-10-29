//! Diagramas prontos: usados nos testes e no menu "Exemplos" da interface.
//!
//! As posições seguem uma grade de 20 px, com o fluxo principal da
//! esquerda para a direita e as realimentações por baixo, "viradas"
//! (`flipped`) da direita para a esquerda.

use super::model::{Diagram, NodeId, Pos, Side, Sign};

/// Todos os exemplos, com nome para o menu.
pub fn all() -> Vec<(&'static str, Diagram)> {
    vec![
        ("Série: G1·G2", series()),
        ("Paralelo: G1 + G2", parallel()),
        ("Realimentação negativa", feedback(Sign::Minus)),
        ("Realimentação positiva", feedback(Sign::Plus)),
        ("Dois laços que não se tocam", two_loops()),
        ("Referência (H1 com +)", reference(Sign::Plus)),
        ("Referência (H1 com −)", reference(Sign::Minus)),
        ("Laços aninhados", nested_loops()),
    ]
}

fn p(x: f32, y: f32) -> Pos {
    Pos::new(x, y)
}

/// Liga e entra em pânico se falhar (os exemplos são fixos).
fn link(d: &mut Diagram, a: NodeId, b: NodeId, port: u8) {
    d.link(a, b, port).expect("exemplo bem formado");
}

fn link_out(d: &mut Diagram, a: NodeId, out: u8, b: NodeId, port: u8) {
    use super::model::PortRef;
    d.connect(PortRef::new(a, out), PortRef::new(b, port))
        .expect("exemplo bem formado");
}

fn flip(d: &mut Diagram, id: NodeId) {
    d.node_mut(id).expect("nó existe").flipped = true;
}

/// X → G1 → G2 → Y
pub fn series() -> Diagram {
    let mut d = Diagram::new();
    let x = d.add_input("X", p(40.0, 200.0));
    let g1 = d.add_block("G1", p(160.0, 200.0));
    let g2 = d.add_block("G2", p(300.0, 200.0));
    let y = d.add_output("Y", p(420.0, 200.0));
    link(&mut d, x, g1, 0);
    link(&mut d, g1, g2, 0);
    link(&mut d, g2, y, 0);
    d
}

/// X ramifica em G1 e G2, somados com + e +.
pub fn parallel() -> Diagram {
    let mut d = Diagram::new();
    let x = d.add_input("X", p(40.0, 200.0));
    let br = d.add_branch(2, p(120.0, 200.0));
    let g1 = d.add_block("G1", p(220.0, 140.0));
    let g2 = d.add_block("G2", p(220.0, 260.0));
    let sum = d.add_sum(
        &[(Sign::Plus, Side::Top), (Sign::Plus, Side::Bottom)],
        p(340.0, 200.0),
    );
    let y = d.add_output("Y", p(440.0, 200.0));
    link(&mut d, x, br, 0);
    link_out(&mut d, br, 0, g1, 0);
    link_out(&mut d, br, 1, g2, 0);
    link(&mut d, g1, sum, 0);
    link(&mut d, g2, sum, 1);
    link(&mut d, sum, y, 0);
    d
}

/// Realimentação clássica: X → Σ(+, sinal) → G → Y, com H de Y de volta.
/// `Sign::Minus` dá G/(1+GH); `Sign::Plus`, G/(1−GH).
pub fn feedback(sign: Sign) -> Diagram {
    feedback_with("G", "H", sign)
}

/// Realimentação com expressões quaisquer em G e H.
pub fn feedback_with(g: &str, h: &str, sign: Sign) -> Diagram {
    let mut d = Diagram::new();
    let x = d.add_input("X", p(40.0, 200.0));
    let sum = d.add_sum(
        &[(Sign::Plus, Side::Left), (sign, Side::Bottom)],
        p(140.0, 200.0),
    );
    let gb = d.add_block(g, p(260.0, 200.0));
    let br = d.add_branch(2, p(380.0, 200.0));
    let y = d.add_output("Y", p(480.0, 200.0));
    let hb = d.add_block(h, p(260.0, 300.0));
    flip(&mut d, hb);
    link(&mut d, x, sum, 0);
    link(&mut d, sum, gb, 0);
    link(&mut d, gb, br, 0);
    link_out(&mut d, br, 0, y, 0);
    link_out(&mut d, br, 1, hb, 0);
    link(&mut d, hb, sum, 1);
    d
}

/// Dois laços locais que não se tocam:
/// X → Σ1(+, −H1·a) → G1 → a → Σ2(+, −H2·Y) → G2 → Y.
/// T = G1·G2 / ((1 + G1·H1)(1 + G2·H2)); Δ tem o termo L1·L2.
pub fn two_loops() -> Diagram {
    let mut d = Diagram::new();
    let x = d.add_input("X", p(40.0, 200.0));
    let s1 = d.add_sum(
        &[(Sign::Plus, Side::Left), (Sign::Minus, Side::Bottom)],
        p(120.0, 200.0),
    );
    let g1 = d.add_block("G1", p(220.0, 200.0));
    let b1 = d.add_branch(2, p(320.0, 200.0));
    let h1 = d.add_block("H1", p(220.0, 300.0));
    let s2 = d.add_sum(
        &[(Sign::Plus, Side::Left), (Sign::Minus, Side::Bottom)],
        p(400.0, 200.0),
    );
    let g2 = d.add_block("G2", p(500.0, 200.0));
    let b2 = d.add_branch(2, p(600.0, 200.0));
    let h2 = d.add_block("H2", p(500.0, 300.0));
    let y = d.add_output("Y", p(700.0, 200.0));
    flip(&mut d, h1);
    flip(&mut d, h2);
    link(&mut d, x, s1, 0);
    link(&mut d, s1, g1, 0);
    link(&mut d, g1, b1, 0);
    link_out(&mut d, b1, 0, s2, 0);
    link_out(&mut d, b1, 1, h1, 0);
    link(&mut d, h1, s1, 1);
    link(&mut d, s2, g2, 0);
    link(&mut d, g2, b2, 0);
    link_out(&mut d, b2, 0, y, 0);
    link_out(&mut d, b2, 1, h2, 0);
    link(&mut d, h2, s2, 1);
    d
}

/// Laços aninhados: o interno (H1) envolve só G2; o externo (H2), G1 e G2.
/// X → Σ1(+, −H2·Y) → G1 → Σ2(+, −H1·a) → G2 → a → b → Y.
/// T = G1·G2 / (1 + G2·H1 + G1·G2·H2).
pub fn nested_loops() -> Diagram {
    let mut d = Diagram::new();
    let x = d.add_input("X", p(40.0, 200.0));
    let s1 = d.add_sum(
        &[(Sign::Plus, Side::Left), (Sign::Minus, Side::Bottom)],
        p(120.0, 200.0),
    );
    let g1 = d.add_block("G1", p(220.0, 200.0));
    let s2 = d.add_sum(
        &[(Sign::Plus, Side::Left), (Sign::Minus, Side::Bottom)],
        p(320.0, 200.0),
    );
    let g2 = d.add_block("G2", p(420.0, 200.0));
    let a = d.add_branch(2, p(520.0, 200.0));
    let b = d.add_branch(2, p(600.0, 200.0));
    let y = d.add_output("Y", p(700.0, 200.0));
    let h1 = d.add_block("H1", p(420.0, 280.0));
    let h2 = d.add_block("H2", p(360.0, 340.0));
    flip(&mut d, h1);
    flip(&mut d, h2);
    link(&mut d, x, s1, 0);
    link(&mut d, s1, g1, 0);
    link(&mut d, g1, s2, 0);
    link(&mut d, s2, g2, 0);
    link(&mut d, g2, a, 0);
    link_out(&mut d, a, 0, b, 0);
    link_out(&mut d, a, 1, h1, 0);
    link(&mut d, h1, s2, 1);
    link_out(&mut d, b, 0, y, 0);
    link_out(&mut d, b, 1, h2, 0);
    link(&mut d, h2, s1, 1);
    d
}

/// Diagrama de referência:
///
/// ```text
/// X → Σ1(+X, −H2·Y) → Σ2(+, h1·H1·A) → G1 → G4 → A
/// A → H1 → Σ2;   A → G2 ┐
///                A → G3 ┴→ Σ3(+, +) → Y;   Y → H2 → Σ1
/// ```
///
/// Com H1 entrando com `+`: G1G4(G2+G3) / (1 − G1G4H1 + G1G4(G2+G3)H2).
/// Com `−`:                 G1G4(G2+G3) / (1 + G1G4H1 + G1G4(G2+G3)H2).
pub fn reference(h1_sign: Sign) -> Diagram {
    let mut d = Diagram::new();
    let x = d.add_input("X", p(40.0, 200.0));
    let s1 = d.add_sum(
        &[(Sign::Plus, Side::Left), (Sign::Minus, Side::Bottom)],
        p(120.0, 200.0),
    );
    let s2 = d.add_sum(
        &[(Sign::Plus, Side::Left), (h1_sign, Side::Bottom)],
        p(200.0, 200.0),
    );
    let g1 = d.add_block("G1", p(300.0, 200.0));
    let g4 = d.add_block("G4", p(420.0, 200.0));
    let a = d.add_branch(3, p(520.0, 200.0));
    let g2 = d.add_block("G2", p(620.0, 140.0));
    let g3 = d.add_block("G3", p(620.0, 260.0));
    let h1 = d.add_block("H1", p(360.0, 300.0));
    let s3 = d.add_sum(
        &[(Sign::Plus, Side::Top), (Sign::Plus, Side::Bottom)],
        p(720.0, 200.0),
    );
    let yb = d.add_branch(2, p(800.0, 200.0));
    let y = d.add_output("Y", p(880.0, 200.0));
    let h2 = d.add_block("H2", p(460.0, 380.0));
    flip(&mut d, h1);
    flip(&mut d, h2);
    link(&mut d, x, s1, 0);
    link(&mut d, s1, s2, 0);
    link(&mut d, s2, g1, 0);
    link(&mut d, g1, g4, 0);
    link(&mut d, g4, a, 0);
    link_out(&mut d, a, 0, g2, 0);
    link_out(&mut d, a, 1, g3, 0);
    link_out(&mut d, a, 2, h1, 0);
    link(&mut d, h1, s2, 1);
    link(&mut d, g2, s3, 0);
    link(&mut d, g3, s3, 1);
    link(&mut d, s3, yb, 0);
    link_out(&mut d, yb, 0, y, 0);
    link_out(&mut d, yb, 1, h2, 0);
    link(&mut d, h2, s1, 1);
    d
}
