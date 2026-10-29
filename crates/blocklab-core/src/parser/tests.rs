//! Testes do parser: entradas válidas comparadas com a álgebra montada à
//! mão, e erros conferidos pelo tipo e pela posição.

use num_rational::BigRational;

use super::*;
use crate::algebra::{RationalFn, Var};

fn s() -> RationalFn {
    RationalFn::var(Var::S)
}
fn sym(n: &str) -> RationalFn {
    RationalFn::var(Var::named(n))
}
fn k(n: i64) -> RationalFn {
    RationalFn::from_int(n)
}
fn q(n: i64, d: i64) -> RationalFn {
    RationalFn::constant(BigRational::new(n.into(), d.into()))
}

fn ok(src: &str) -> RationalFn {
    parse_rational(src).unwrap_or_else(|e| panic!("{}", e.render(src)))
}

/// Confere tipo do erro e trecho sublinhado `[start, end)`.
fn err(src: &str, kind: ParseErrorKind, start: usize, end: usize) {
    let e = parse_rational(src).expect_err(src);
    assert_eq!(e.kind, kind, "{}", e.render(src));
    assert_eq!(e.span, Span::new(start, end), "{}", e.render(src));
}

#[test]
fn operadores_e_precedencia() {
    assert_eq!(ok("1 + 2*3"), k(7));
    assert_eq!(ok("(1 + 2)*3"), k(9));
    assert_eq!(ok("2^3"), k(8));
    assert_eq!(ok("-s^2"), -(s() * s())); // -(s²), não (-s)²
    assert_eq!(ok("8/4/2"), k(1)); // esquerda para a direita
    assert_eq!(ok("s - -1"), s() + k(1));
    assert_eq!(ok("+s"), s());
}

#[test]
fn multiplicacao_implicita() {
    assert_eq!(ok("2s"), k(2) * s());
    assert_eq!(ok("s(s+1)"), s() * (s() + k(1)));
    assert_eq!(ok("3(s+2)"), k(3) * (s() + k(2)));
    assert_eq!(ok("(s+1)(s+2)"), (s() + k(1)) * (s() + k(2)));
    assert_eq!(ok("2 s^2 + 3 s + 1"), k(2) * s() * s() + k(3) * s() + k(1));
    assert_eq!(ok("G1 G2"), sym("G1") * sym("G2"));
    assert_eq!(ok("s^2(s+1)"), s() * s() * (s() + k(1)));
}

#[test]
fn implicita_amarra_mais_que_divisao() {
    // Convenção de controle: 10/s(s+1) = 10/(s(s+1)).
    assert_eq!(ok("10/s(s+1)"), k(10) / (s() * (s() + k(1))));
    assert_eq!(ok("1/2s"), k(1) / (k(2) * s()));
    // Com '*' explícito, vale a esquerda-para-direita usual.
    assert_eq!(ok("1/2*s"), s() / k(2));
}

#[test]
fn decimais_exatos() {
    assert_eq!(ok("0.1 + 0.2"), q(3, 10));
    assert_eq!(ok("2.5s"), q(5, 2) * s());
    assert_eq!(ok(".5"), q(1, 2));
}

#[test]
fn simbolos() {
    assert_eq!(ok("G1"), sym("G1"));
    assert_eq!(ok("H_2"), sym("H_2"));
    assert_eq!(ok("Kp/(s+a)"), sym("Kp") / (s() + sym("a")));
    assert_eq!(ok("ω_n^2"), sym("ω_n") * sym("ω_n"));
    // "e" com expoente inteiro é só um símbolo ao quadrado.
    assert_eq!(ok("e^2"), sym("e") * sym("e"));
    // Notação tipográfica colada de PDF.
    assert_eq!(ok("2·s − 1"), k(2) * s() - k(1));
}

#[test]
fn expoentes_inteiros() {
    assert_eq!(ok("s^-1"), k(1) / s());
    assert_eq!(ok("s^(-2)"), k(1) / (s() * s()));
    assert_eq!(ok("(s+1)^0"), k(1));
    assert_eq!(
        ok("((s+1)/(s+2))^2"),
        (s() + k(1)) * (s() + k(1)) / ((s() + k(2)) * (s() + k(2)))
    );
}

#[test]
fn simplifica_ao_avaliar() {
    assert_eq!(ok("(s+1)/((s+1)(s+2))"), k(1) / (s() + k(2)));
    assert_eq!(
        ok("K/(s+a) / (1 + K/(s+a))"),
        sym("K") / (s() + sym("a") + sym("K"))
    );
}

#[test]
fn erros_de_expoente() {
    err("s^a", ParseErrorKind::SymbolicExponent("a".into()), 2, 3);
    err(
        "s^0.5",
        ParseErrorKind::NonIntegerExponent("um número decimal"),
        2,
        5,
    );
    err(
        "s^(1+1)",
        ParseErrorKind::NonIntegerExponent("uma expressão"),
        2,
        7,
    );
    // e^(...) sublinha a exponencial inteira.
    err("2 e^(-s*T)", ParseErrorKind::Exponential, 2, 10);
    err("e^0.5", ParseErrorKind::Exponential, 0, 5);
    err("e^-sT", ParseErrorKind::Exponential, 0, 5);
    err(
        "s^101",
        ParseErrorKind::ExponentTooLarge { max: MAX_EXPONENT },
        2,
        5,
    );
    err("s^2^3", ParseErrorKind::ChainedPower, 3, 4);
}

#[test]
fn erros_de_estrutura() {
    err("", ParseErrorKind::Empty, 0, 1);
    err("   ", ParseErrorKind::Empty, 0, 1);
    err("10/(s+1", ParseErrorKind::UnclosedParen, 3, 4);
    err("s+1)", ParseErrorKind::UnmatchedParen, 3, 4);
    err(")", ParseErrorKind::UnmatchedParen, 0, 1);
    err("s + $", ParseErrorKind::InvalidChar('$'), 4, 5);
    err("2.", ParseErrorKind::BadNumber, 0, 2);
    err(
        "s +",
        ParseErrorKind::Unexpected {
            expected: "um número, um símbolo ou '('",
            found: "o fim da expressão".into(),
        },
        3,
        4,
    );
    err(
        "()",
        ParseErrorKind::Unexpected {
            expected: "um número, um símbolo ou '('",
            found: "')'".into(),
        },
        1,
        2,
    );
    err(
        "2 3",
        ParseErrorKind::Unexpected {
            expected: "um operador entre os dois números",
            found: "o número 3".into(),
        },
        2,
        3,
    );
}

#[test]
fn divisao_por_zero_aponta_o_divisor() {
    err("1/(s - s)", ParseErrorKind::DivisionByZero, 2, 9);
    err("s/0", ParseErrorKind::DivisionByZero, 2, 3);
    err("0^-1", ParseErrorKind::DivisionByZero, 0, 1);
    // Zero só simbolicamente: 1/(a - 1) é válido (a é parâmetro).
    assert!(parse_rational("1/(a - 1)").is_ok());
}

#[test]
fn render_sublinha() {
    let e = parse_rational("10/(s+1").unwrap_err();
    assert_eq!(
        e.render("10/(s+1"),
        "10/(s+1\n   ^ posição 4: '(' sem o ')' correspondente"
    );
    assert_eq!(e.to_string(), "posição 4: '(' sem o ')' correspondente");
}
