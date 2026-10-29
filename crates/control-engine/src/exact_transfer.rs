//! Exact univariate cancellation for expression inputs (never numerical minreal).
use crate::{
    Result,
    symbolic::{self, Rational, SPolynomial},
};
use num_bigint::BigInt;
use num_traits::{One, Signed, ToPrimitive, Zero};

fn coefficients(p: SPolynomial) -> Result<Vec<Rational>> {
    p.0.into_iter()
        .rev()
        .map(|c| {
            if c.has_k() || c.has_epsilon() {
                return Err("A função de transferência deve ter coeficientes reais.".into());
            }
            Ok(c.num.0.get(&(0, 0)).cloned().unwrap_or_else(Rational::zero)
                / c.den
                    .0
                    .get(&(0, 0))
                    .ok_or("Denominador simbólico inválido.")?)
        })
        .collect()
}
fn trim(mut p: Vec<Rational>) -> Vec<Rational> {
    let start = p.iter().position(|v| !v.is_zero()).unwrap_or(p.len() - 1);
    p.drain(..start);
    p
}
fn divide(a: &[Rational], b: &[Rational]) -> (Vec<Rational>, Vec<Rational>) {
    if a.len() < b.len() {
        return (vec![Rational::zero()], a.to_vec());
    }
    let mut rest = a.to_vec();
    let mut q = vec![Rational::zero(); a.len() - b.len() + 1];
    for i in 0..q.len() {
        q[i] = &rest[i] / &b[0];
        for j in 0..b.len() {
            let sub = &q[i] * &b[j];
            rest[i + j] -= sub;
        }
    }
    (trim(q), trim(rest))
}
fn gcd_integer(mut a: BigInt, mut b: BigInt) -> BigInt {
    while !b.is_zero() {
        let r = &a % &b;
        a = b;
        b = r;
    }
    a.abs()
}

pub fn parse(source: &str) -> Result<(Vec<f64>, Vec<f64>)> {
    let (n, d) = symbolic::parse_transfer(source)?;
    let n = coefficients(n)?;
    let d = coefficients(d)?;
    if d.iter().all(Rational::is_zero) {
        return Err("O denominador não pode ser zero.".into());
    }
    if n.iter().all(Rational::is_zero) {
        return Ok((vec![0.], vec![1.]));
    }
    let mut a = n.clone();
    let mut b = d.clone();
    while b.iter().any(|v| !v.is_zero()) {
        let (_, r) = divide(&a, &b);
        a = b;
        b = r;
    }
    let lead = a[0].clone();
    for c in &mut a {
        *c /= &lead;
    }
    let (mut n, _) = divide(&n, &a);
    let (mut d, _) = divide(&d, &a);
    let approximate =
        source.contains('.') || source.as_bytes().iter().any(|c| matches!(c, b'e' | b'E'));
    if !approximate {
        let mut lcm = BigInt::one();
        for c in n.iter().chain(&d) {
            lcm = &lcm / gcd_integer(lcm.clone(), c.denom().clone()) * c.denom();
        }
        let multiplier = Rational::from_integer(lcm);
        for c in n.iter_mut().chain(&mut d) {
            *c *= &multiplier;
        }
    }
    if approximate && d.len() == 1 {
        let divisor = d[0].clone();
        for c in &mut n {
            *c /= &divisor;
        }
        d[0] = Rational::one();
    } else if n.iter().chain(&d).all(Rational::is_integer) {
        let mut content = BigInt::zero();
        for c in n.iter().chain(&d) {
            content = gcd_integer(content, c.numer().clone());
        }
        let divisor = Rational::from_integer(content);
        for c in n.iter_mut().chain(&mut d) {
            *c /= &divisor;
        }
    }
    if d[0].is_negative() {
        for c in n.iter_mut().chain(&mut d) {
            *c = -c.clone();
        }
    }
    let convert = |p: Vec<Rational>| -> Result<Vec<f64>> {
        p.iter()
            .map(|c| {
                c.to_f64()
                    .filter(|v| v.is_finite())
                    .ok_or_else(|| "Coeficientes excedem a precisão numérica.".into())
            })
            .collect()
    };
    Ok((convert(n)?, convert(d)?))
}
