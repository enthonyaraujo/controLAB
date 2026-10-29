//! Testes da camada de funções racionais (casos de controle clássicos
//! montados direto na álgebra, antes de existir parser ou diagrama).

use num_rational::BigRational;

use super::*;

fn s() -> RationalFn {
    RationalFn::var(Var::S)
}
fn sym(n: &str) -> RationalFn {
    RationalFn::var(Var::named(n))
}
fn k(n: i64) -> RationalFn {
    RationalFn::from_int(n)
}

/// Realimentação: G/(1 ∓ G·H) — `negativa = true` para o caso usual.
fn feedback(g: &RationalFn, h: &RationalFn, negativa: bool) -> RationalFn {
    let gh = g * h;
    let den = if negativa { k(1) + gh } else { k(1) - gh };
    g / den
}

#[test]
fn cancela_fator_comum_so_s() {
    // (s+1)/((s+1)(s+2)) → 1/(s+2)
    let f = (s() + k(1)) / ((s() + k(1)) * (s() + k(2)));
    assert_eq!(f, k(1) / (s() + k(2)));
    assert_eq!(f.num(), &Poly::one());
    assert_eq!(format!("{f}"), "(1)/(s + 2)");
}

#[test]
fn realimentacao_numerica() {
    // G = 10/(s(s+1)), H = 1 → 10/(s² + s + 10)
    let g = k(10) / (s() * (s() + k(1)));
    let t = feedback(&g, &k(1), true);
    assert_eq!(format!("{t}"), "(10)/(s^2 + s + 10)");
}

#[test]
fn realimentacao_simbolica() {
    let (g, h) = (sym("G"), sym("H"));
    assert_eq!(format!("{}", feedback(&g, &h, true)), "(G)/(G*H + 1)");
    assert_eq!(format!("{}", feedback(&g, &h, false)), "(G)/(-G*H + 1)");
    // Denominador com termo constante normalizado para 1, mesmo vindo
    // escalado: 2G/(2 + 2GH) = G/(1 + GH).
    let escalada = (k(2) * &g) / (k(2) + k(2) * &g * &h);
    assert_eq!(escalada, feedback(&g, &h, true));
}

#[test]
fn realimentacao_unitaria_com_simbolos_e_s() {
    // G1 = K/(s+a), H = 1 → K/(s + a + K): exige cancelar (s+a), que só o
    // MDC multivariável enxerga.
    let g1 = sym("K") / (s() + sym("a"));
    let t = feedback(&g1, &k(1), true);
    assert_eq!(t, sym("K") / (s() + sym("a") + sym("K")));
    assert_eq!(t.num(), &Poly::var(Var::named("K")));
}

#[test]
fn serie_e_paralelo() {
    let (g1, g2) = (sym("G1"), sym("G2"));
    assert_eq!(format!("{}", &g1 * &g2), "G1*G2");
    assert_eq!(format!("{}", &g1 + &g2), "G1 + G2");
}

#[test]
fn normalizacao_sem_termo_constante() {
    // (-2K)/(-2s - 2a) → K/(s + a): primitivo, líder positivo.
    let f = (k(-2) * sym("K")) / (k(-2) * s() - k(2) * sym("a"));
    assert_eq!(format!("{f}"), "(K)/(s + a)");
}

#[test]
fn potencias_e_inverso() {
    let f = (s() + k(1)) / (s() + k(2));
    assert_eq!(f.pow(-1).unwrap(), (s() + k(2)) / (s() + k(1)));
    assert_eq!(f.pow(2).unwrap(), &f * &f);
    assert_eq!(f.pow(0).unwrap(), k(1));
    assert_eq!(k(0).pow(-1), Err(AlgebraError::DivisionByZero));
    assert_eq!(k(1).checked_div(&k(0)), Err(AlgebraError::DivisionByZero));
}

#[test]
fn substituicao_simplifica_de_novo() {
    // K/(s + a + K) com a = 1, K = 2 → 2/(s + 3)
    let t = sym("K") / (s() + sym("a") + sym("K"));
    let mut p = Params::new();
    p.insert(Symbol::new("a"), BigRational::from_integer(1.into()));
    p.insert(Symbol::new("K"), BigRational::from_integer(2.into()));
    let n = t.substitute(&p).unwrap();
    assert_eq!(n, k(2) / (s() + k(3)));
    assert!(n.is_numeric());

    // (s + a)/(s + 1) com a = 1 cancela → 1.
    let f = (s() + sym("a")) / (s() + k(1));
    assert!(f.substitute(&p).unwrap().is_one());

    // Denominador que zera: 1/(a - 1) com a = 1.
    let g = k(1) / (sym("a") - k(1));
    assert_eq!(g.substitute(&p), Err(AlgebraError::DivisionByZero));
}

#[test]
fn simbolos_em_ordem_natural() {
    let f = (sym("H1") + sym("G10")) / (sym("G2") + s());
    let names: Vec<_> = f.symbols().iter().map(|x| x.name().to_string()).collect();
    assert_eq!(names, ["G2", "G10", "H1"]);
}

#[test]
fn soma_com_normalizacao_unica() {
    // 1/(s+1) + 1/(s+2) + 1/((s+1)(s+2)) = (2s+4)/((s+1)(s+2)) = 2/(s+1)
    let a = k(1) / (s() + k(1));
    let b = k(1) / (s() + k(2));
    let c = k(1) / ((s() + k(1)) * (s() + k(2)));
    assert_eq!(RationalFn::sum([&a, &b, &c]), k(2) / (s() + k(1)));
    assert_eq!(RationalFn::product([&a, &b]), c);
}

#[test]
fn monico_em_s_mesmo_com_simbolos() {
    // K/((s+1)(s+2)) com realimentação unitária: o denominador fica mônico
    // em s (s² + 3s + K + 2), não com constante 1 (½s² + 3/2·s + 1 + K/2).
    let g = sym("K") / ((s() + k(1)) * (s() + k(2)));
    let t = feedback(&g, &k(1), true);
    assert_eq!(format!("{t}"), "(K)/(s^2 + 3*s + K + 2)");
}

#[test]
fn constante_de_tempo_fica_com_constante_um() {
    // Coeficiente líder em s simbólico (T): vale a regra da constante = 1.
    let f = k(1) / (k(2) * sym("T") * s() + k(2));
    assert_eq!(format!("{f}"), "(1/2)/(s*T + 1)");
}
