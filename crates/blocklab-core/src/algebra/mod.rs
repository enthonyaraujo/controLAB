//! Álgebra exata: polinômios esparsos em Q[s, símbolos] e funções
//! racionais (frações de polinômios) sempre simplificadas.
//!
//! Camadas, da mais simples para a mais completa:
//! [`Symbol`]/[`Var`] → [`Monomial`] → [`Poly`] → [`RationalFn`].

/// Gera as variantes "por valor" de um operador binário a partir da
/// implementação por referência (`&T op &T`), para podermos escrever
/// `a + b`, `a + &b` e `&a + b` sem repetir código.
macro_rules! forward_binop {
    ($Trait:ident, $method:ident, $T:ty) => {
        impl std::ops::$Trait<$T> for $T {
            type Output = $T;
            fn $method(self, rhs: $T) -> $T {
                std::ops::$Trait::$method(&self, &rhs)
            }
        }
        impl std::ops::$Trait<&$T> for $T {
            type Output = $T;
            fn $method(self, rhs: &$T) -> $T {
                std::ops::$Trait::$method(&self, rhs)
            }
        }
        impl std::ops::$Trait<$T> for &$T {
            type Output = $T;
            fn $method(self, rhs: $T) -> $T {
                std::ops::$Trait::$method(self, &rhs)
            }
        }
    };
}

/// `+`, `-`, `*` e negação por valor.
macro_rules! forward_ops {
    ($T:ty) => {
        $crate::algebra::forward_binop!(Add, add, $T);
        $crate::algebra::forward_binop!(Sub, sub, $T);
        $crate::algebra::forward_binop!(Mul, mul, $T);
        impl std::ops::Neg for $T {
            type Output = $T;
            fn neg(self) -> $T {
                -&self
            }
        }
    };
}

pub(crate) use forward_binop;
pub(crate) use forward_ops;

pub mod cancel;
mod gcd;
mod monomial;
mod poly;
mod rational;
mod symbol;

pub use cancel::{CancelToken, with_cancel};
pub use gcd::gcd;
pub use monomial::Monomial;
pub use poly::{Params, Poly};
pub use rational::{AlgebraError, RationalFn};
pub use symbol::{Symbol, Var};

#[cfg(test)]
mod tests;
