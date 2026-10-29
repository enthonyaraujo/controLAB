//! Parser de expressões de blocos: texto → [`Expr`] → [`RationalFn`].
//!
//! Aceita `+ - * / ^`, parênteses, multiplicação implícita (`2s`,
//! `s(s+1)`, `3(s+2)`), decimais (convertidos para fração exata) e
//! símbolos com letras, dígitos e `_` (`G1`, `H_2`, `Kp`, `a`).
//! O nome `s` é a variável de Laplace; qualquer outro é símbolo.
//!
//! Veja `parse.rs` para a gramática e a precedência.

mod ast;
mod error;
mod lexer;
mod parse;

pub use ast::{BinOp, Expr, ExprKind};
pub use error::{ParseError, ParseErrorKind, Span};
pub use parse::{MAX_EXPONENT, parse};

use crate::algebra::RationalFn;

/// Atalho: texto direto para função racional simplificada.
pub fn parse_rational(src: &str) -> Result<RationalFn, ParseError> {
    parse(src)?.to_rational()
}

#[cfg(test)]
mod tests;
