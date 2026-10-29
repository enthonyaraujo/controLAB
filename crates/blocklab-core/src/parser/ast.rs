//! Árvore sintática (AST) e sua avaliação para [`RationalFn`].

use num_rational::BigRational;

use super::error::{ParseError, ParseErrorKind, Span};
use crate::algebra::{RationalFn, Var};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExprKind {
    Num(BigRational),
    Var(Var),
    Neg(Box<Expr>),
    Bin(BinOp, Box<Expr>, Box<Expr>),
    /// Base elevada a um inteiro literal (pode ser negativo: s^-1 = 1/s).
    Pow(Box<Expr>, i32),
}

/// Nó da AST com o trecho do texto que o originou (para apontar erros
/// que só aparecem na avaliação, como divisão por zero).
#[derive(Debug, Clone, PartialEq)]
pub struct Expr {
    pub kind: ExprKind,
    pub span: Span,
}

impl Expr {
    pub fn new(kind: ExprKind, span: Span) -> Self {
        Expr { kind, span }
    }

    /// Avalia a árvore de baixo para cima, gerando uma função racional
    /// já simplificada. Erro se algum divisor valer zero — e "valer zero"
    /// é decidido simbolicamente: 1/(s - s) é erro, mas 1/(a - 1) não.
    pub fn to_rational(&self) -> Result<RationalFn, ParseError> {
        Ok(match &self.kind {
            ExprKind::Num(q) => RationalFn::constant(q.clone()),
            ExprKind::Var(v) => RationalFn::var(v.clone()),
            ExprKind::Neg(e) => -e.to_rational()?,
            ExprKind::Bin(op, a, b) => {
                let (x, y) = (a.to_rational()?, b.to_rational()?);
                match op {
                    BinOp::Add => x + y,
                    BinOp::Sub => x - y,
                    BinOp::Mul => x * y,
                    BinOp::Div => x
                        .checked_div(&y)
                        .map_err(|_| ParseError::new(ParseErrorKind::DivisionByZero, b.span))?,
                }
            }
            ExprKind::Pow(base, exp) => base
                .to_rational()?
                .pow(*exp)
                // Só falha para 0 elevado a expoente negativo.
                .map_err(|_| ParseError::new(ParseErrorKind::DivisionByZero, base.span))?,
        })
    }
}
