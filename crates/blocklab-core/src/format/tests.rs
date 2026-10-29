use super::*;
use crate::diagram::{Sign, examples};
use crate::parser::parse_rational;
use crate::solve;

fn f(src: &str) -> RationalFn {
    parse_rational(src).unwrap()
}

fn sym(src: &str) -> String {
    to_text(&f(src), Mode::Symbolic)
}

fn num(src: &str) -> String {
    to_text(&f(src), Mode::Numeric)
}

#[test]
fn referencia_fatorada() {
    let t = solve(&examples::reference(Sign::Plus)).unwrap().transfer;
    assert_eq!(
        to_text(&t, Mode::Symbolic),
        "G1·G4·(G2+G3) / (1 − G1·G4·H1 + G1·G4·(G2+G3)·H2)"
    );
    assert_eq!(
        to_ascii(&t, Mode::Symbolic),
        "G1*G4*(G2+G3) / (1 - G1*G4*H1 + G1*G4*(G2+G3)*H2)"
    );
    assert_eq!(
        to_latex(&t, Mode::Symbolic),
        r"\frac{G_{1} G_{4} \left(G_{2} + G_{3}\right)}{1 - G_{1} G_{4} H_{1} + G_{1} G_{4} \left(G_{2} + G_{3}\right) H_{2}}"
    );
    let t = solve(&examples::reference(Sign::Minus)).unwrap().transfer;
    assert_eq!(
        to_text(&t, Mode::Symbolic),
        "G1·G4·(G2+G3) / (1 + G1·G4·H1 + G1·G4·(G2+G3)·H2)"
    );
}

#[test]
fn produto_de_somas() {
    let t = solve(&examples::two_loops()).unwrap().transfer;
    assert_eq!(to_text(&t, Mode::Symbolic), "G1·G2 / ((1+G1·H1)·(1+G2·H2))");
    assert_eq!(sym("G1 G2 + G1 G3 + H1 G2 + H1 G3"), "(G1+H1)·(G2+G3)");
}

#[test]
fn realimentacao_classica() {
    assert_eq!(sym("G/(1 + G H)"), "G / (1 + G·H)");
    assert_eq!(sym("G/(1 - G H)"), "G / (1 − G·H)");
    assert_eq!(
        to_latex(&f("G/(1 + G H)"), Mode::Symbolic),
        r"\frac{G}{1 + G H}"
    );
}

#[test]
fn ordem_estavel() {
    // G antes de H, índices crescentes (G2 antes de G10), s por último.
    assert_eq!(sym("H1 G2"), "G2·H1");
    assert_eq!(sym("G10 G2"), "G2·G10");
    assert_eq!(sym("s K"), "K·s");
    assert_eq!(sym("H2 + G10 + G2 + 1"), "1 + G2 + G10 + H2");
}

#[test]
fn polinomios_em_s() {
    assert_eq!(num("10/(s(s+1) + 10)"), "10 / (s^2 + s + 10)");
    assert_eq!(sym("10/(s(s+1) + 10)"), "10 / (s^2 + s + 10)");
    // Fator comum s aparece no modo simbólico, não no numérico.
    assert_eq!(sym("1/(s^2 + 2s)"), "1 / (s·(s+2))");
    assert_eq!(num("1/(s^2 + 2s)"), "1 / (s^2 + 2·s)");
    assert_eq!(num("1/(s+2)"), "1 / (s + 2)");
    assert_eq!(
        to_latex(&f("10/(s^2 + s + 10)"), Mode::Numeric),
        r"\frac{10}{s^{2} + s + 10}"
    );
}

#[test]
fn numerico_com_simbolos_restantes() {
    assert_eq!(num("K/(s + a + K)"), "K / (s + K + a)");
    assert_eq!(num("1/(s^2 + (K + a) s + 10)"), "1 / (s^2 + (K+a)·s + 10)");
    assert_eq!(num("1/(s^2 - (K + a) s + 10)"), "1 / (s^2 − (K+a)·s + 10)");
}

#[test]
fn coeficientes_racionais_e_sinais() {
    assert_eq!(num("0.5 s + 1"), "1/2·s + 1");
    assert_eq!(
        to_latex(&f("0.5 s + 1"), Mode::Numeric),
        r"\frac{1}{2} s + 1"
    );
    assert_eq!(sym("-G/(1 + G H)"), "−G / (1 + G·H)");
    assert_eq!(sym("-2 G1 - 2 G2"), "−2·(G1+G2)");
    assert_eq!(sym("2 G1 G2 + 2 G1 G3"), "2·G1·(G2+G3)");
    assert_eq!(sym("0"), "0");
    assert_eq!(sym("1"), "1");
    assert_eq!(sym("3/4"), "3/4");
}

#[test]
fn simbolico_com_s_organiza_por_potencias() {
    // Sistema de 2ª ordem: nada de (2·ζ·s+ω_n)·ω_n.
    assert_eq!(
        sym("ω_n^2/(s^2 + 2ζ ω_n s + ω_n^2)"),
        "ω_n^2 / (s^2 + 2·ζ·ω_n·s + ω_n^2)"
    );
    // A fatoração continua valendo dentro de cada coeficiente.
    assert_eq!(sym("(G1 G2 + G1 G3) s + 1"), "G1·(G2+G3)·s + 1");
    assert_eq!(
        sym("1/(s^2 + (K + a) s + K a)"),
        "1 / (s^2 + (K+a)·s + K·a)"
    );
    // s em evidência vem antes da soma que contém s.
    assert_eq!(sym("K/(s^3 + a s^2 + s)"), "K / (s·(s^2+a·s+1))");
    assert_eq!(sym("K s^2 + K a s"), "K·s·(s+a)");
}

#[test]
fn latex_sem_fracao_dentro_de_fracao() {
    assert_eq!(
        to_latex(&f("1/(2 T s + 2)"), Mode::Symbolic),
        r"\frac{1}{2 \left(T s + 1\right)}"
    );
    assert_eq!(to_latex(&f("3/(4 s)"), Mode::Symbolic), r"\frac{3}{4 s}");
    assert_eq!(to_latex(&f("s/2"), Mode::Symbolic), r"\frac{s}{2}");
    assert_eq!(
        to_latex(&f("-s/(2 s + 2)"), Mode::Symbolic),
        r"-\frac{s}{2 \left(s + 1\right)}"
    );
    // O texto em linha continua legível pelo parser.
    assert_eq!(sym("1/(2 T s + 2)"), "1/2 / (T·s + 1)");
}

#[test]
fn denominador_de_um_termo_so() {
    assert_eq!(sym("1/s"), "1 / s");
    assert_eq!(sym("1/s^2"), "1 / s^2");
    assert_eq!(sym("1/(2 s)"), "1/2 / s");
    assert_eq!(sym("G/(K s)"), "G / (K·s)");
}

#[test]
fn simbolos_em_latex() {
    assert_eq!(latex_symbol("G1"), "G_{1}");
    assert_eq!(latex_symbol("G10"), "G_{10}");
    assert_eq!(latex_symbol("H_2"), "H_{2}");
    assert_eq!(latex_symbol("Kp"), r"\mathrm{Kp}");
    assert_eq!(latex_symbol("a"), "a");
    assert_eq!(latex_symbol("ω_n"), r"\omega_{n}");
    assert_eq!(latex_symbol("τ"), r"\tau");
    assert_eq!(
        to_latex(&f("ω_n^2 / (s^2 + 2 ζ ω_n s + ω_n^2)"), Mode::Numeric),
        r"\frac{\omega_{n}^{2}}{s^{2} + 2 \zeta \omega_{n} s + \omega_{n}^{2}}"
    );
}

#[test]
fn partes_separadas_para_a_interface() {
    let p = pretty(&f("G1 G4 (G2 + G3) / (1 + G1 H1)"), Mode::Symbolic);
    assert_eq!(p.num.text(), "G1·G4·(G2+G3)");
    assert_eq!(p.den.as_ref().unwrap().text(), "1 + G1·H1");
    assert_eq!(pretty(&f("G1 G2"), Mode::Symbolic).den, None);
}

#[test]
fn latex_so_ascii_para_qualquer_nome() {
    // O pdflatex recusa grego/acentos crus em modo matemático. Fora de
    // \mbox{} (modo texto, onde UTF-8 funciona) a saída tem que ser ASCII
    // para qualquer nome que o parser aceite.
    for src in [
        "ωc/(s + ωc)",
        "ζω/(s+1)",
        "Kç s",
        "ω_n^2/(s^2 + 2ζ ω_n s + ω_n^2)",
        "α_1 β2 + Ωx",
        "τ s + 1",
        "ãé_ü/(s+1)",
    ] {
        for mode in [Mode::Symbolic, Mode::Numeric] {
            let tex = to_latex(&f(src), mode);
            let mut math = String::new();
            let mut rest = tex.as_str();
            while let Some(i) = rest.find("\\mbox{") {
                math += &rest[..i];
                let after = &rest[i..];
                rest = &after[after.find('}').unwrap() + 1..];
            }
            math += rest;
            assert!(math.is_ascii(), "{src}: {tex}");
        }
    }
}

#[test]
fn nomes_de_varias_letras_e_caracteres_especiais() {
    assert_eq!(latex_symbol("ωc"), r"\mathrm{\omega c}");
    assert_eq!(latex_symbol("ζω"), r"\mathrm{\zeta\omega}");
    assert_eq!(latex_symbol("Kç"), r"\mathrm{K\mbox{ç}}");
    assert_eq!(latex_symbol("K_ωc"), r"K_{\omega c}");
    assert_eq!(latex_symbol("G_1_2"), r"G_{1\_2}");
    assert_eq!(latex_symbol("Kp1"), r"\mathrm{Kp}_{1}");
}

#[test]
fn espaco_fino_ao_lado_de_nome_de_varias_letras() {
    // Sem o \, o LaTeX mostraria "Kds² + Kps + Ki".
    assert_eq!(
        to_latex(&f("Kp + Ki/s + Kd s"), Mode::Symbolic),
        r"\frac{\mathrm{Kd}\, s^{2} + \mathrm{Kp}\, s + \mathrm{Ki}}{s}"
    );
    // Nomes de uma letra continuam só justapostos.
    assert_eq!(to_latex(&f("K T s"), Mode::Symbolic), r"K T s");
}

#[test]
fn sinal_negativo_sempre_fora_da_fracao() {
    assert_eq!(
        to_latex(&f("-K/(s+1)"), Mode::Symbolic),
        r"-\frac{K}{s + 1}"
    );
    assert_eq!(
        to_latex(&f("-s/(2s+2)"), Mode::Symbolic),
        r"-\frac{s}{2 \left(s + 1\right)}"
    );
    assert_eq!(to_latex(&f("-K"), Mode::Symbolic), "-K");
}
