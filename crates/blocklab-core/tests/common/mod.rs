//! Geradores (strategies) do proptest compartilhados entre os testes.

#![allow(dead_code)]

use blocklab_core::algebra::{Monomial, Poly, RationalFn, Var};
use num_rational::BigRational;
use proptest::prelude::*;

pub const VARS: [&str; 4] = ["s", "G1", "H1", "a"];

/// Um termo c·s^e0·G1^e1·H1^e2·a^e3 com coeficiente pequeno não nulo.
pub fn term() -> impl Strategy<Value = Poly> {
    (
        (-3i64..=3).prop_filter("coeficiente não nulo", |c| *c != 0),
        proptest::collection::vec(0u32..=2, VARS.len()),
    )
        .prop_map(|(c, exps)| {
            let m = VARS.iter().zip(exps).fold(Monomial::one(), |acc, (n, e)| {
                acc.mul(&Monomial::var_pow(Var::named(n), e))
            });
            Poly::term(m, BigRational::from_integer(c.into()))
        })
}

pub fn poly() -> impl Strategy<Value = Poly> {
    proptest::collection::vec(term(), 1..=3)
        .prop_map(|ts| ts.iter().fold(Poly::zero(), |acc, t| &acc + t))
}

pub fn nonzero_poly() -> impl Strategy<Value = Poly> {
    poly().prop_filter("não nulo", |p| !p.is_zero())
}

pub fn rational() -> impl Strategy<Value = RationalFn> {
    (poly(), nonzero_poly()).prop_map(|(n, d)| RationalFn::new(n, d).unwrap())
}

pub fn nonzero_rational() -> impl Strategy<Value = RationalFn> {
    rational().prop_filter("não nula", |f| !f.is_zero())
}
