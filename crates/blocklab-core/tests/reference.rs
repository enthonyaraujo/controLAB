//! Diagrama de referência do enunciado, com H1 entrando com + e com −,
//! incluindo a lista didática de caminhos e laços.

use blocklab_core::diagram::{Sign, examples};
use blocklab_core::parser::parse_rational;
use blocklab_core::solve;

#[test]
fn h1_com_mais() {
    let r = solve(&examples::reference(Sign::Plus)).unwrap();
    assert_eq!(
        r.transfer,
        parse_rational("G1 G4 (G2+G3) / (1 - G1 G4 H1 + G1 G4 (G2+G3) H2)").unwrap()
    );
}

#[test]
fn h1_com_menos() {
    let r = solve(&examples::reference(Sign::Minus)).unwrap();
    assert_eq!(
        r.transfer,
        parse_rational("G1 G4 (G2+G3) / (1 + G1 G4 H1 + G1 G4 (G2+G3) H2)").unwrap()
    );
}

#[test]
fn caminhos_e_lacos_para_a_lista_didatica() {
    let r = solve(&examples::reference(Sign::Plus)).unwrap();
    let paths: Vec<String> = r.paths.iter().map(|p| p.describe(&r.sfg, false)).collect();
    assert_eq!(
        paths,
        [
            "X → Σ1 → Σ2 → G1 → G4 → G2 → Σ3 → Y",
            "X → Σ1 → Σ2 → G1 → G4 → G3 → Σ3 → Y",
        ]
    );
    let gains: Vec<String> = r.paths.iter().map(|p| p.gain.to_string()).collect();
    assert_eq!(gains, ["G1*G2*G4", "G1*G3*G4"]);

    let loops: Vec<String> = r.loops.iter().map(|l| l.describe(&r.sfg, true)).collect();
    assert_eq!(
        loops,
        [
            "Σ1 → Σ2 → G1 → G4 → G2 → Σ3 → H2 → Σ1",
            "Σ1 → Σ2 → G1 → G4 → G3 → Σ3 → H2 → Σ1",
            "Σ2 → G1 → G4 → H1 → Σ2",
        ]
    );
    let loop_gains: Vec<String> = r.loops.iter().map(|l| l.gain.to_string()).collect();
    assert_eq!(loop_gains, ["-G1*G2*G4*H2", "-G1*G3*G4*H2", "G1*G4*H1"]);

    // Todos os laços passam por G1: nenhum par deixa de se tocar, e cada
    // caminho toca todos os laços, então Δ₁ = Δ₂ = 1.
    assert!(r.nontouching.is_empty());
    assert!(r.cofactors.iter().all(|dk| dk.is_one()));
}

#[test]
fn arquivo_de_exemplo_em_dia_com_o_codigo() {
    // examples/referencia.json é gerado por
    // `cargo run --example solve -- --json 6 > examples/referencia.json`.
    let text = include_str!("../../../examples/referencia.json");
    let d = blocklab_core::diagram::Diagram::from_json(text).unwrap();
    assert_eq!(d, examples::reference(Sign::Plus));
}
