//! Exact rational functions in the real gain K and positive infinitesimal epsilon.
//!
//! Routh arithmetic uses Q[K, epsilon], including polynomial GCD cancellation.
//! Epsilon limits use the lowest nonzero power, never an arbitrary small value.
use crate::Result;
use num_bigint::BigInt;
use num_rational::BigRational;
use num_traits::{One, Signed, ToPrimitive, Zero};
use std::collections::BTreeMap;

pub type Rational = BigRational;
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Polynomial(pub BTreeMap<(usize, usize), Rational>);
impl Polynomial {
    pub fn zero() -> Self {
        Self(BTreeMap::new())
    }
    pub fn constant(c: Rational) -> Self {
        if c.is_zero() {
            Self::zero()
        } else {
            Self(BTreeMap::from([((0, 0), c)]))
        }
    }
    pub fn integer(c: i64) -> Self {
        Self::constant(Rational::from_integer(c.into()))
    }
    pub fn variable(epsilon: bool) -> Self {
        Self(BTreeMap::from([(
            (usize::from(!epsilon), usize::from(epsilon)),
            Rational::one(),
        )]))
    }
    pub fn is_zero(&self) -> bool {
        self.0.is_empty()
    }
    pub fn is_one(&self) -> bool {
        self == &Self::integer(1)
    }
    pub fn has_k(&self) -> bool {
        self.0.keys().any(|(k, _)| *k > 0)
    }
    pub fn has_epsilon(&self) -> bool {
        self.0.keys().any(|(_, e)| *e > 0)
    }
    pub fn add(&self, b: &Self) -> Self {
        let mut out = self.0.clone();
        for (k, v) in &b.0 {
            let slot = out.entry(*k).or_insert_with(Rational::zero);
            *slot += v;
            if slot.is_zero() {
                out.remove(k);
            }
        }
        Self(out)
    }
    pub fn neg(&self) -> Self {
        self.scale(&-Rational::one())
    }
    pub fn sub(&self, b: &Self) -> Self {
        self.add(&b.neg())
    }
    pub fn scale(&self, c: &Rational) -> Self {
        if c.is_zero() {
            return Self::zero();
        }
        Self(self.0.iter().map(|(k, v)| (*k, v * c)).collect())
    }
    pub fn mul(&self, b: &Self) -> Self {
        let mut out = Self::zero();
        for (&(k, e), v) in &self.0 {
            for (&(bk, be), bv) in &b.0 {
                let key = (k + bk, e + be);
                let slot = out.0.entry(key).or_insert_with(Rational::zero);
                *slot += v * bv;
                if slot.is_zero() {
                    out.0.remove(&key);
                }
            }
        }
        out
    }
    pub fn shift(&self, k: usize, e: usize) -> Self {
        Self(
            self.0
                .iter()
                .map(|(&(pk, pe), v)| ((pk + k, pe + e), v.clone()))
                .collect(),
        )
    }
    fn monic(&self) -> Self {
        match self.0.last_key_value() {
            Some((_, v)) => self.scale(&v.recip()),
            None => self.clone(),
        }
    }
    /// Exact multivariate division in a fixed monomial order.
    pub fn exact_div(&self, b: &Self) -> Option<Self> {
        let (&(bk, be), bc) = b.0.last_key_value()?;
        let mut rem = self.clone();
        let mut q = Self::zero();
        while let Some((&(rk, re), rc)) = rem.0.last_key_value() {
            if rk < bk || re < be {
                return None;
            }
            let term = Self(BTreeMap::from([((rk - bk, re - be), rc / bc)]));
            q = q.add(&term);
            rem = rem.sub(&term.mul(b));
        }
        Some(q)
    }
    fn degree_e(&self) -> usize {
        self.0.keys().map(|(_, e)| *e).max().unwrap_or(0)
    }
    fn coeff_e(&self, e: usize) -> Self {
        Self(
            self.0
                .iter()
                .filter_map(|(&(k, pe), v)| (e == pe).then_some(((k, 0), v.clone())))
                .collect(),
        )
    }
    fn content_e(&self) -> Self {
        let mut out = Self::zero();
        for e in 0..=self.degree_e() {
            let c = self.coeff_e(e);
            if !c.is_zero() {
                out = Self::gcd_univariate(out, c);
                if out.is_one() {
                    break;
                }
            }
        }
        out
    }
    fn primitive_e(&self) -> Self {
        if self.is_zero() {
            self.clone()
        } else {
            self.exact_div(&self.content_e())
                .expect("polynomial content divides")
                .monic()
        }
    }
    fn rem_univariate(mut a: Self, b: &Self) -> Self {
        let (&(bk, be), bc) = b.0.last_key_value().unwrap();
        while let Some((&(ak, ae), ac)) = a.0.last_key_value() {
            if ak < bk || ae < be {
                break;
            }
            let term = Self(BTreeMap::from([((ak - bk, ae - be), ac / bc)]));
            a = a.sub(&term.mul(b));
        }
        a
    }
    fn gcd_univariate(mut a: Self, mut b: Self) -> Self {
        while !b.is_zero() {
            let r = Self::rem_univariate(a, &b);
            a = b;
            b = r;
        }
        a.monic()
    }
    pub fn gcd(a: &Self, b: &Self) -> Self {
        if a.is_zero() {
            return b.monic();
        }
        if b.is_zero() {
            return a.monic();
        }
        if !a.has_epsilon() && !b.has_epsilon() || !a.has_k() && !b.has_k() {
            return Self::gcd_univariate(a.clone(), b.clone());
        }
        let content = Self::gcd_univariate(a.content_e(), b.content_e());
        let mut x = a.primitive_e();
        let mut y = b.primitive_e();
        if x.degree_e() < y.degree_e() {
            std::mem::swap(&mut x, &mut y);
        }
        while !y.is_zero() {
            let yd = y.degree_e();
            let yl = y.coeff_e(yd);
            let mut r = x;
            while !r.is_zero() && r.degree_e() >= yd {
                let rd = r.degree_e();
                let rl = r.coeff_e(rd);
                r = yl.mul(&r).sub(&rl.mul(&y).shift(0, rd - yd));
                if !r.is_zero() {
                    r = r.primitive_e();
                }
            }
            x = y;
            y = r;
        }
        content.mul(&x.primitive_e()).monic()
    }
    pub fn evaluate(&self, k: f64, e: f64) -> f64 {
        self.0
            .iter()
            .map(|(&(kp, ep), c)| {
                c.to_f64().unwrap_or(f64::NAN) * k.powi(kp as i32) * e.powi(ep as i32)
            })
            .sum()
    }
    pub fn substitute_k(&self, k: &Rational) -> Self {
        let mut out = Self::zero();
        for (&(kp, ep), c) in &self.0 {
            let v = c * k.pow(kp as i32);
            out = out.add(&Self(BTreeMap::from([((0, ep), v)])));
        }
        out
    }
    pub fn first_e_coeff(&self) -> (usize, Self) {
        let e = self.0.keys().map(|(_, e)| *e).min().unwrap_or(0);
        (e, self.coeff_e(e))
    }
    pub fn k_coefficients(&self) -> Vec<Rational> {
        let n = self.0.keys().map(|(k, _)| *k).max().unwrap_or(0);
        (0..=n)
            .rev()
            .map(|k| self.0.get(&(k, 0)).cloned().unwrap_or_else(Rational::zero))
            .collect()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Expr {
    pub num: Polynomial,
    pub den: Polynomial,
    pub approximate: bool,
}
impl Expr {
    pub fn new(num: Polynomial, den: Polynomial) -> Self {
        assert!(!den.is_zero(), "symbolic division by zero");
        if num.is_zero() {
            return Self::integer(0);
        }
        let g = Polynomial::gcd(&num, &den);
        let num = num.exact_div(&g).unwrap();
        let den = den.exact_div(&g).unwrap();
        let lead = den.0.last_key_value().unwrap().1.clone();
        Self {
            num: num.scale(&lead.recip()),
            den: den.scale(&lead.recip()),
            approximate: false,
        }
    }
    pub fn integer(n: i64) -> Self {
        Self {
            num: Polynomial::integer(n),
            den: Polynomial::integer(1),
            approximate: false,
        }
    }
    pub fn rational(n: Rational) -> Self {
        Self {
            num: Polynomial::constant(n),
            den: Polynomial::integer(1),
            approximate: false,
        }
    }
    pub fn k() -> Self {
        Self {
            num: Polynomial::variable(false),
            den: Polynomial::integer(1),
            approximate: false,
        }
    }
    pub fn epsilon() -> Self {
        Self {
            num: Polynomial::variable(true),
            den: Polynomial::integer(1),
            approximate: false,
        }
    }
    pub fn is_zero(&self) -> bool {
        self.num.is_zero()
    }
    pub fn is_one(&self) -> bool {
        self.num == self.den
    }
    pub fn has_k(&self) -> bool {
        self.num.has_k() || self.den.has_k()
    }
    pub fn has_epsilon(&self) -> bool {
        self.num.has_epsilon() || self.den.has_epsilon()
    }
    pub fn add(&self, b: &Self) -> Self {
        Self::new(
            self.num.mul(&b.den).add(&b.num.mul(&self.den)),
            self.den.mul(&b.den),
        )
        .with_approximate(self.approximate || b.approximate)
    }
    pub fn neg(&self) -> Self {
        Self {
            num: self.num.neg(),
            den: self.den.clone(),
            approximate: self.approximate,
        }
    }
    pub fn sub(&self, b: &Self) -> Self {
        self.add(&b.neg())
    }
    pub fn mul(&self, b: &Self) -> Self {
        Self::new(self.num.mul(&b.num), self.den.mul(&b.den))
            .with_approximate(self.approximate || b.approximate)
    }
    pub fn div(&self, b: &Self) -> Result<Self> {
        if b.is_zero() {
            Err("Divisão por zero na expressão simbólica.".into())
        } else {
            Ok(Self::new(self.num.mul(&b.den), self.den.mul(&b.num))
                .with_approximate(self.approximate || b.approximate))
        }
    }
    pub fn evaluate(&self, k: f64, e: f64) -> f64 {
        self.num.evaluate(k, e) / self.den.evaluate(k, e)
    }
    pub fn substitute_k(&self, k: &Rational) -> Result<Self> {
        let d = self.den.substitute_k(k);
        if d.is_zero() {
            Err("Ganho singular na expressão simbólica.".into())
        } else {
            Ok(Self::new(self.num.substitute_k(k), d))
        }
    }
    /// Limit epsilon -> 0+ as a symbolic coefficient ratio and an epsilon power.
    pub fn epsilon_leading(&self) -> (isize, Self) {
        let (ne, n) = self.num.first_e_coeff();
        let (de, d) = self.den.first_e_coeff();
        (ne as isize - de as isize, Self::new(n, d))
    }
    pub fn limit_sign(&self, k: f64) -> i8 {
        if self.is_zero() {
            return 0;
        }
        let (order, coefficient) = self.epsilon_leading();
        if order > 0 {
            return 0;
        }
        let val = coefficient.evaluate(k, 0.);
        if val == 0. || (order == 0 && val.abs() < 1e-12) {
            0
        } else if val > 0. {
            1
        } else {
            -1
        }
    }
    pub fn with_approximate(mut self, approximate: bool) -> Self {
        self.approximate = approximate && !self.is_zero();
        self
    }
    pub fn latex(&self) -> String {
        if self.approximate {
            approximate_latex(&self.num, &self.den)
        } else {
            latex_ratio(&self.num, &self.den)
        }
    }
}

/// SymPy Float uses fifteen significant decimal digits in its default LaTeX.
pub fn float_latex(r: &Rational) -> String {
    let v = r.to_f64().unwrap_or(f64::NAN);
    if v == 0. {
        return "0.0".into();
    }
    let exponent = v.abs().log10().floor() as i32;
    if !(-4..15).contains(&exponent) {
        let t = format!("{v:.14e}");
        let (m, e) = t.split_once('e').unwrap();
        let mut m = m.trim_end_matches('0').to_string();
        if m.ends_with('.') {
            m.push('0');
        }
        return format!("{m} \\cdot 10^{{{}}}", e.parse::<i32>().unwrap());
    }
    let decimals = (14 - exponent).max(0) as usize;
    let text = format!("{v:.decimals$}");
    let mut text = if text.contains('.') {
        text.trim_end_matches('0').to_string()
    } else {
        text
    };
    if text.ends_with('.') {
        text.push('0');
    } else if !text.contains('.') {
        text.push_str(".0");
    }
    text
}
fn approximate_poly_latex(p: &Polynomial) -> String {
    if p.is_zero() {
        return "0.0".into();
    }
    let mut entries: Vec<_> = p.0.iter().rev().collect();
    if entries.len() == 2
        && entries[0].1.is_negative()
        && entries[1].0 == &(0, 0)
        && entries[1].1.is_positive()
    {
        entries.swap(0, 1);
    }
    join_terms(
        entries
            .into_iter()
            .map(|(&(k, e), v)| {
                let c = float_latex(&v.abs());
                let m = monomial(k, e);
                (
                    if m.is_empty() { c } else { format!("{c} {m}") },
                    v.is_negative(),
                )
            })
            .collect(),
    )
}
fn approximate_latex(n: &Polynomial, d: &Polynomial) -> String {
    if d.is_one() {
        return approximate_poly_latex(n);
    }
    if d.0.len() == 1 {
        let (&(dk, de), dc) = d.0.first_key_value().unwrap();
        let mut terms = vec![];
        for (&(k, e), c) in n.0.iter().rev() {
            let c = c / dc;
            let top_m = monomial(k.saturating_sub(dk), e.saturating_sub(de));
            let bottom = monomial(dk.saturating_sub(k), de.saturating_sub(e));
            let mut top = float_latex(&c.abs());
            if !top_m.is_empty() {
                top.push(' ');
                top.push_str(&top_m);
            }
            terms.push((
                if bottom.is_empty() {
                    top
                } else {
                    format!("\\frac{{{top}}}{{{bottom}}}")
                },
                c.is_negative(),
            ));
        }
        if terms.len() == 2 && terms[0].1 && !terms[1].1 {
            terms.swap(0, 1);
        }
        return join_terms(terms);
    }
    format!(
        "\\frac{{{}}}{{{}}}",
        approximate_poly_latex(n),
        approximate_poly_latex(d)
    )
}
pub fn rational_latex(r: &Rational) -> String {
    if r.is_integer() {
        r.to_integer().to_string()
    } else if r.is_negative() {
        format!("- \\frac{{{}}}{{{}}}", -r.numer(), r.denom())
    } else {
        format!("\\frac{{{}}}{{{}}}", r.numer(), r.denom())
    }
}
fn monomial(k: usize, e: usize) -> String {
    let mut p = vec![];
    for (deg, sym) in [(k, "K"), (e, "\\varepsilon")] {
        if deg == 1 {
            p.push(sym.to_string());
        } else if deg > 1 {
            p.push(format!("{sym}^{{{deg}}}"));
        }
    }
    p.join(" ")
}
fn join_terms(mut terms: Vec<(String, bool)>) -> String {
    if terms.is_empty() {
        return "0".into();
    }
    let mut out = String::new();
    for (t, negative) in terms.drain(..) {
        if out.is_empty() {
            if negative {
                out.push_str("- ");
            }
        } else {
            out.push_str(if negative { " - " } else { " + " });
        }
        out.push_str(&t);
    }
    out
}
pub fn polynomial_latex(p: &Polynomial) -> String {
    if p.0.len() == 1 {
        if let Some(c) = p.0.get(&(0, 0)) {
            return rational_latex(c);
        }
    }
    let mut entries: Vec<_> = p.0.iter().rev().collect();
    // SymPy presents a two-term subtraction with its positive constant first.
    if entries.len() == 2
        && entries[0].1.is_negative()
        && entries[1].0 == &(0, 0)
        && entries[1].1.is_positive()
    {
        entries.swap(0, 1);
    }
    join_terms(
        entries
            .into_iter()
            .map(|(&(k, e), v)| {
                let c = v.abs();
                let m = monomial(k, e);
                let t = if m.is_empty() {
                    rational_latex(&c)
                } else if c.is_one() {
                    m
                } else if c.is_integer() {
                    format!("{} {m}", c.to_integer())
                } else {
                    let n = if c.numer().is_one() {
                        m
                    } else {
                        format!("{} {m}", c.numer())
                    };
                    format!("\\frac{{{n}}}{{{}}}", c.denom())
                };
                (t, v.is_negative())
            })
            .collect(),
    )
}
fn latex_ratio(n: &Polynomial, d: &Polynomial) -> String {
    if d.is_one() {
        return polynomial_latex(n);
    }
    // Split monomial denominators into Laurent terms, as sympy.simplify does.
    if d.0.len() == 1 {
        let (&(dk, de), dc) = d.0.first_key_value().unwrap();
        let mut entries: Vec<_> = n.0.iter().rev().collect();
        if entries.len() == 2 && entries[0].1.is_negative() && entries[1].1.is_positive() {
            entries.swap(0, 1);
        }
        return join_terms(
            entries
                .into_iter()
                .map(|(&(k, e), c)| {
                    let c = c / dc;
                    let cabs = c.abs();
                    let nk = k.saturating_sub(dk);
                    let ne = e.saturating_sub(de);
                    let rk = dk.saturating_sub(k);
                    let re = de.saturating_sub(e);
                    let m = monomial(nk, ne);
                    let top = if m.is_empty() {
                        cabs.numer().to_string()
                    } else if cabs.numer().is_one() {
                        m
                    } else {
                        format!("{} {m}", cabs.numer())
                    };
                    let m = monomial(rk, re);
                    let bottom = if m.is_empty() {
                        cabs.denom().to_string()
                    } else if cabs.denom().is_one() {
                        m
                    } else {
                        format!("{} {m}", cabs.denom())
                    };
                    let t = if bottom == "1" {
                        top
                    } else {
                        format!("\\frac{{{top}}}{{{bottom}}}")
                    };
                    (t, c.is_negative())
                })
                .collect(),
        );
    }
    // Clear rational scalar content for a conventional integer coefficient fraction.
    let mut lcm = BigInt::one();
    for c in n.0.values().chain(d.0.values()) {
        lcm = num_integer_lcm(lcm, c.denom().clone());
    }
    let scale = Rational::from_integer(lcm);
    let nn = n.scale(&scale);
    let dd = d.scale(&scale);
    let numerator = if (!nn.has_k() && nn.degree_e() == 2 && dd.degree_e() == 1
        || !nn.has_epsilon() && nn.k_coefficients().len() == 3 && dd.k_coefficients().len() == 2)
        && nn.0.len() == 3
    {
        let epsilon = nn.has_epsilon();
        let key = |degree| if epsilon { (0, degree) } else { (degree, 0) };
        let leading = nn.0.get(&key(2)).unwrap();
        let negative = leading.is_negative();
        let mut inner = Polynomial::zero();
        for degree in [1, 2] {
            if let Some(c) = nn.0.get(&key(degree)) {
                inner
                    .0
                    .insert(key(degree - 1), if negative { -c } else { c.clone() });
            }
        }
        let mut head = format!(
            "{}{} \\left({}\\right)",
            if negative { "- " } else { "" },
            if epsilon { "\\varepsilon" } else { "K" },
            polynomial_latex(&inner)
        );
        let constant = nn.0.get(&(0, 0)).unwrap();
        head.push_str(if constant.is_negative() { " - " } else { " + " });
        head.push_str(&rational_latex(&constant.abs()));
        head
    } else {
        polynomial_latex(&nn)
    };
    format!("\\frac{{{numerator}}}{{{}}}", content_factored_latex(&dd))
}
fn content_factored_latex(p: &Polynomial) -> String {
    let mut content = BigInt::zero();
    for c in p.0.values() {
        if !c.is_integer() {
            return polynomial_latex(p);
        }
        let mut a = content;
        let mut b = c.to_integer().abs();
        while !b.is_zero() {
            let r = &a % &b;
            a = b;
            b = r;
        }
        content = a;
    }
    if content <= BigInt::one() {
        return polynomial_latex(p);
    }
    let primitive = p.scale(&Rational::new(BigInt::one(), content.clone()));
    if primitive.0.len() > 1 {
        format!("{content} \\left({}\\right)", polynomial_latex(&primitive))
    } else {
        polynomial_latex(p)
    }
}
fn num_integer_lcm(a: BigInt, b: BigInt) -> BigInt {
    let mut x = a.clone();
    let mut y = b.clone();
    while !y.is_zero() {
        let r = &x % &y;
        x = y;
        y = r;
    }
    (a / x) * b
}
pub fn parse_rational(s: &str) -> Result<Rational> {
    let (mantissa, exponent) = if let Some((m, e)) = s.split_once(['e', 'E']) {
        (
            m,
            e.parse::<i32>().map_err(|_| "Expoente decimal inválido.")?,
        )
    } else {
        (s, 0)
    };
    if exponent.abs() > 10000 {
        return Err("Expoente decimal excede o limite de recursos.".into());
    }
    let decimals = mantissa.split_once('.').map(|(_, f)| f.len()).unwrap_or(0) as i32;
    let digits = mantissa.replace('.', "");
    let n = digits.parse::<BigInt>().map_err(|_| "Número inválido.")?;
    let power = exponent - decimals;
    if power >= 0 {
        Ok(Rational::from_integer(
            n * BigInt::from(10).pow(power as u32),
        ))
    } else {
        Ok(Rational::new(n, BigInt::from(10).pow((-power) as u32)))
    }
}

/// Polynomial in s with exact rational-function coefficients, ascending powers.
#[derive(Clone, Debug)]
pub struct SPolynomial(pub Vec<Expr>);
impl SPolynomial {
    pub fn new(mut c: Vec<Expr>) -> Self {
        while c.len() > 1 && c.last().unwrap().is_zero() {
            c.pop();
        }
        if c.is_empty() {
            c.push(Expr::integer(0));
        }
        Self(c)
    }
    pub fn scalar(v: Expr) -> Self {
        Self(vec![v])
    }
    pub fn add(&self, b: &Self) -> Self {
        let mut c = vec![Expr::integer(0); self.0.len().max(b.0.len())];
        for (i, x) in self.0.iter().enumerate() {
            c[i] = c[i].add(x);
        }
        for (i, x) in b.0.iter().enumerate() {
            c[i] = c[i].add(x);
        }
        Self::new(c)
    }
    pub fn neg(&self) -> Self {
        Self(self.0.iter().map(Expr::neg).collect())
    }
    pub fn mul(&self, b: &Self) -> Self {
        let mut c = vec![Expr::integer(0); self.0.len() + b.0.len() - 1];
        for (i, x) in self.0.iter().enumerate() {
            for (j, y) in b.0.iter().enumerate() {
                c[i + j] = c[i + j].add(&x.mul(y));
            }
        }
        Self::new(c)
    }
    pub fn scale(&self, v: &Expr) -> Self {
        Self::new(self.0.iter().map(|x| x.mul(v)).collect())
    }
    pub fn numeric(&self, k: f64) -> Vec<f64> {
        self.0.iter().rev().map(|v| v.evaluate(k, 0.)).collect()
    }
    pub fn degree(&self) -> usize {
        self.0.len() - 1
    }
    pub fn has_k(&self) -> bool {
        self.0.iter().any(Expr::has_k)
    }
    pub fn latex(&self) -> String {
        let mut out = String::new();
        for (power, c) in self.0.iter().enumerate().rev() {
            if c.is_zero() {
                continue;
            }
            let l = c.latex();
            let term = if power == 0 {
                l
            } else {
                let variable = if power == 1 {
                    "s".into()
                } else {
                    format!("s^{{{power}}}")
                };
                if c.is_one() && !c.approximate {
                    variable
                } else if c.neg().is_one() && !c.approximate {
                    format!("-{variable}")
                } else {
                    let additive = c.den.is_one() && c.num.0.len() > 1;
                    format!("{} {variable}", if additive { format!("({l})") } else { l })
                }
            };
            if out.is_empty() {
                out = term;
            } else if let Some(t) = term.strip_prefix('-') {
                out.push_str(" - ");
                out.push_str(t.trim());
            } else {
                out.push_str(" + ");
                out.push_str(&term);
            }
        }
        if out.is_empty() {
            "0".into()
        } else {
            out
        }
    }
}
#[derive(Clone, Debug)]
struct SFraction {
    num: SPolynomial,
    den: SPolynomial,
}
impl SFraction {
    fn scalar(v: Expr) -> Self {
        Self {
            num: SPolynomial::scalar(v),
            den: SPolynomial::scalar(Expr::integer(1)),
        }
    }
    fn add(&self, b: &Self) -> Self {
        Self {
            num: self.num.mul(&b.den).add(&b.num.mul(&self.den)),
            den: self.den.mul(&b.den),
        }
    }
    fn mul(&self, b: &Self) -> Self {
        Self {
            num: self.num.mul(&b.num),
            den: self.den.mul(&b.den),
        }
    }
    fn div(&self, b: &Self) -> Result<Self> {
        if b.num.0.iter().all(Expr::is_zero) {
            return Err("O denominador não pode ser zero.".into());
        }
        Ok(Self {
            num: self.num.mul(&b.den),
            den: self.den.mul(&b.num),
        })
    }
}
#[derive(Clone, Debug, PartialEq)]
enum Token {
    Number(String),
    S,
    K,
    Plus,
    Minus,
    Mul,
    Div,
    Pow,
    Left,
    Right,
}
fn lex(s: &str) -> Result<Vec<Token>> {
    let b = s.as_bytes();
    let mut p = 0;
    let mut tokens = vec![];
    while p < b.len() {
        let c = b[p];
        if c.is_ascii_whitespace() {
            p += 1;
            continue;
        }
        if c.is_ascii_digit() || c == b'.' {
            let start = p;
            p += 1;
            while p < b.len() && (b[p].is_ascii_digit() || b[p] == b'.') {
                p += 1;
            }
            if p < b.len() && (b[p] == b'e' || b[p] == b'E') {
                p += 1;
                if p < b.len() && (b[p] == b'+' || b[p] == b'-') {
                    p += 1;
                }
                while p < b.len() && b[p].is_ascii_digit() {
                    p += 1;
                }
            }
            tokens.push(Token::Number(s[start..p].into()));
            continue;
        }
        tokens.push(match c {
            b's' | b'S' => Token::S,
            b'K' | b'k' => Token::K,
            b'+' => Token::Plus,
            b'-' => Token::Minus,
            b'*' => {
                if p + 1 < b.len() && b[p + 1] == b'*' {
                    p += 1;
                    Token::Pow
                } else {
                    Token::Mul
                }
            }
            b'/' => Token::Div,
            b'^' => Token::Pow,
            b'(' => Token::Left,
            b')' => Token::Right,
            _ => {
                return Err(
                    "Erro ao analisar a expressão matemática: símbolo não suportado.".into(),
                )
            }
        });
        p += 1;
    }
    if tokens.len() > 4096 {
        return Err("Expressão excede o limite de recursos.".into());
    }
    Ok(tokens)
}
struct Parser {
    tokens: Vec<Token>,
    p: usize,
    depth: usize,
}
impl Parser {
    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.p)
    }
    fn take(&mut self) -> Option<Token> {
        let v = self.peek().cloned();
        self.p += usize::from(v.is_some());
        v
    }
    fn expr(&mut self) -> Result<SFraction> {
        let mut a = self.product()?;
        loop {
            match self.peek() {
                Some(Token::Plus) => {
                    self.take();
                    a = a.add(&self.product()?);
                }
                Some(Token::Minus) => {
                    self.take();
                    let mut b = self.product()?;
                    b.num = b.num.neg();
                    a = a.add(&b);
                }
                _ => break,
            }
        }
        Ok(a)
    }
    fn product(&mut self) -> Result<SFraction> {
        let mut a = self.unary()?;
        loop {
            match self.peek() {
                Some(Token::Mul) => {
                    self.take();
                    a = a.mul(&self.unary()?);
                }
                Some(Token::Div) => {
                    self.take();
                    a = a.div(&self.unary()?)?;
                }
                Some(Token::Number(_) | Token::S | Token::K | Token::Left) => {
                    a = a.mul(&self.unary()?)
                }
                _ => break,
            }
            if a.num.degree() > 128 || a.den.degree() > 128 {
                return Err("O grau excede o limite de recursos (128).".into());
            }
        }
        Ok(a)
    }
    fn unary(&mut self) -> Result<SFraction> {
        match self.peek() {
            Some(Token::Plus) => {
                self.take();
                self.unary()
            }
            Some(Token::Minus) => {
                self.take();
                let mut a = self.unary()?;
                a.num = a.num.neg();
                Ok(a)
            }
            _ => self.power(),
        }
    }
    fn power(&mut self) -> Result<SFraction> {
        let mut a = self.atom()?;
        if self.peek() == Some(&Token::Pow) {
            self.take();
            let exp = self.unary()?;
            if exp.num.degree() > 0 || exp.den.degree() > 0 || exp.num.has_k() || exp.den.has_k() {
                return Err("O expoente deve ser inteiro.".into());
            }
            let v = exp.num.0[0].div(&exp.den.0[0])?.evaluate(0., 0.);
            if !v.is_finite() || v.fract() != 0. || v.abs() > 128. {
                return Err("O expoente deve ser inteiro e ter módulo até 128.".into());
            }
            let base = a;
            a = SFraction::scalar(Expr::integer(1));
            for _ in 0..v.abs() as usize {
                a = a.mul(&base);
            }
            if v < 0. {
                a = SFraction::scalar(Expr::integer(1)).div(&a)?;
            }
        }
        Ok(a)
    }
    fn atom(&mut self) -> Result<SFraction> {
        match self.take() {
            Some(Token::Number(n)) => Ok(SFraction::scalar(Expr::rational(parse_rational(&n)?))),
            Some(Token::K) => Ok(SFraction::scalar(Expr::k())),
            Some(Token::S) => Ok(SFraction {
                num: SPolynomial(vec![Expr::integer(0), Expr::integer(1)]),
                den: SPolynomial::scalar(Expr::integer(1)),
            }),
            Some(Token::Left) => {
                self.depth += 1;
                if self.depth > 128 {
                    return Err("Expressão excede o limite de aninhamento.".into());
                }
                let a = self.expr()?;
                if self.take() != Some(Token::Right) {
                    return Err("Parênteses não balanceados.".into());
                }
                self.depth -= 1;
                Ok(a)
            }
            _ => Err("Erro ao analisar a expressão matemática: expressão inválida.".into()),
        }
    }
}
/// Parse the transfer numerator/denominator without forming the characteristic equation.
/// Coefficients remain exact; no polynomial factor cancellation is performed here.
pub fn parse_transfer(source: &str) -> Result<(SPolynomial, SPolynomial)> {
    let normalized = crate::transfer::normalize(source)?.replace('—', "-");
    let mut parser = Parser {
        tokens: lex(&normalized)?,
        p: 0,
        depth: 0,
    };
    let f = parser.expr()?;
    if parser.p != parser.tokens.len() {
        return Err("Erro ao analisar a expressão matemática: expressão inválida.".into());
    }
    Ok((f.num, f.den))
}
pub fn parse_characteristic(source: &str, has_k_loop: bool) -> Result<(SPolynomial, bool)> {
    let mut source = source.trim();
    if source.is_empty() {
        return Err("A expressão não pode estar vazia.".into());
    }
    if let Some(s) = source
        .strip_suffix("= 0")
        .or_else(|| source.strip_suffix("=0"))
    {
        source = s.trim();
    }
    if let Some((label, s)) = source.split_once('=') {
        if label
            .chars()
            .all(|c| c.is_alphanumeric() || " ()_,".contains(c))
        {
            source = s.trim();
        } else {
            return Err("Prefixo de expressão inválido.".into());
        }
    }
    let normalized = crate::transfer::normalize(source)?.replace('—', "-");
    let mut parser = Parser {
        tokens: lex(&normalized)?,
        p: 0,
        depth: 0,
    };
    let f = parser.expr()?;
    if parser.p != parser.tokens.len() {
        return Err("Erro ao analisar a expressão matemática: expressão inválida.".into());
    }
    let fraction = f.den.degree() > 0 || !f.den.0[0].is_one();
    let mut characteristic = if fraction {
        let contains_k = f.num.has_k() || f.den.has_k();
        f.den.add(&f.num.scale(&if has_k_loop && !contains_k {
            Expr::k()
        } else {
            Expr::integer(1)
        }))
    } else {
        f.num
    };
    if parser
        .tokens
        .iter()
        .any(|t| matches!(t,Token::Number(n) if n.contains(['.','e','E'])))
    {
        for coefficient in &mut characteristic.0 {
            coefficient.approximate = true;
        }
    }
    if characteristic.degree() < 1 {
        return Err("O grau do polinômio característico deve ser pelo menos 1.".into());
    }
    Ok((characteristic, fraction))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exact_cancellation() {
        let k = Expr::k();
        let a = k.add(&Expr::integer(1));
        let e = Expr::epsilon();
        let b = k.mul(&e).add(&Expr::integer(2));
        assert_eq!(a.mul(&b).div(&b).unwrap(), a);
        assert_eq!(a.mul(&b).div(&a).unwrap(), b);
    }
    #[test]
    fn infinitesimal_leading_order() {
        let e = Expr::epsilon();
        let a = Expr::integer(2).sub(&Expr::integer(3).div(&e).unwrap());
        assert_eq!(a.limit_sign(0.), -1);
        assert_eq!(e.limit_sign(0.), 0);
        assert_eq!(a.latex(), "2 - \\frac{3}{\\varepsilon}");
    }
    #[test]
    fn parses_exact_polynomial() {
        let (p, f) = parse_characteristic("1/(s*(s+1)*(s+2))", true).unwrap();
        assert!(f);
        assert_eq!(p.latex(), "s^{3} + 3 s^{2} + 2 s + K");
    }
}

/// A real algebraic boundary certified by a rational isolating interval.
#[derive(Clone, Debug)]
pub struct RealRoot {
    pub low: Rational,
    pub high: Rational,
    pub exact: Option<Rational>,
}
impl RealRoot {
    pub fn value(&self) -> f64 {
        ((&self.low + &self.high) / Rational::from_integer(2.into()))
            .to_f64()
            .unwrap()
    }
}
fn q_trim(mut p: Vec<Rational>) -> Vec<Rational> {
    while p.len() > 1 && p[0].is_zero() {
        p.remove(0);
    }
    p
}
fn q_eval(p: &[Rational], x: &Rational) -> Rational {
    p.iter().fold(Rational::zero(), |a, c| a * x + c)
}
fn q_derivative(p: &[Rational]) -> Vec<Rational> {
    if p.len() < 2 {
        return vec![Rational::zero()];
    }
    p[..p.len() - 1]
        .iter()
        .enumerate()
        .map(|(i, c)| c * Rational::from_integer((p.len() - 1 - i).into()))
        .collect()
}
fn q_remainder(a: &[Rational], b: &[Rational]) -> Vec<Rational> {
    let mut a = q_trim(a.to_vec());
    while a.len() >= b.len() && !a.iter().all(Zero::is_zero) {
        let c = &a[0] / &b[0];
        for (i, v) in b.iter().enumerate() {
            a[i] -= &c * v;
        }
        a = q_trim(a);
    }
    a
}
fn q_gcd(mut a: Vec<Rational>, mut b: Vec<Rational>) -> Vec<Rational> {
    while !b.iter().all(Zero::is_zero) {
        let r = q_remainder(&a, &b);
        a = b;
        b = r;
    }
    let lead = a[0].clone();
    a.iter().map(|v| v / &lead).collect()
}
fn q_divide(mut a: Vec<Rational>, b: &[Rational]) -> Vec<Rational> {
    let mut q = vec![Rational::zero(); a.len() - b.len() + 1];
    while a.len() >= b.len() && !a.iter().all(Zero::is_zero) {
        let deg = a.len() - b.len();
        let c = &a[0] / &b[0];
        let idx = q.len() - 1 - deg;
        q[idx] = c.clone();
        for (i, v) in b.iter().enumerate() {
            a[i] -= &c * v;
        }
        a = q_trim(a);
    }
    q_trim(q)
}
fn variations(sturm: &[Vec<Rational>], x: &Rational) -> usize {
    let signs: Vec<_> = sturm
        .iter()
        .map(|p| q_eval(p, x))
        .filter(|v| !v.is_zero())
        .map(|v| v.is_positive())
        .collect();
    signs.windows(2).filter(|w| w[0] != w[1]).count()
}
/// Sturm isolation gives every distinct real root, including even multiplicities.
/// It never guesses gain boundaries by scanning a grid.
pub fn real_roots(p: &Polynomial) -> Result<Vec<RealRoot>> {
    if p.has_epsilon() {
        return Err("A fronteira de ganho ainda contém epsilon.".into());
    }
    let mut q = q_trim(p.k_coefficients());
    if q.len() < 2 {
        return Ok(vec![]);
    }
    let gcd = q_gcd(q.clone(), q_derivative(&q));
    if gcd.len() > 1 {
        q = q_divide(q, &gcd);
    }
    if q.len() == 2 {
        let x = -&q[1] / &q[0];
        return Ok(vec![RealRoot {
            low: x.clone(),
            high: x.clone(),
            exact: Some(x),
        }]);
    }
    let mut sturm = vec![q.clone(), q_derivative(&q)];
    loop {
        let n = sturm.len();
        let rem = q_remainder(&sturm[n - 2], &sturm[n - 1]);
        if rem.iter().all(Zero::is_zero) {
            break;
        }
        let max = rem.iter().map(Signed::abs).max().unwrap();
        sturm.push(rem.iter().map(|c| -c / &max).collect());
    }
    let max = q[1..]
        .iter()
        .map(|v| (v / &q[0]).abs())
        .max()
        .unwrap_or_else(Rational::zero);
    let bound = max.to_integer() + BigInt::from(2);
    let low = Rational::from_integer(-bound.clone());
    let high = Rational::from_integer(bound);
    let count = variations(&sturm, &low) - variations(&sturm, &high);
    let mut pending = vec![(low, high, count, 0usize)];
    let mut roots = vec![];
    while let Some((mut lo, mut hi, count, depth)) = pending.pop() {
        if count == 0 {
            continue;
        }
        if depth > 4096 {
            return Err("Não foi possível isolar as raízes de K no limite de recursos.".into());
        }
        if count == 1 {
            let mut exact = None;
            if q_eval(&q, &hi).is_zero() {
                exact = Some(hi.clone());
                lo = hi.clone();
            } else {
                for _ in 0..100 {
                    let mid = (&lo + &hi) / Rational::from_integer(2.into());
                    if q_eval(&q, &mid).is_zero() {
                        lo = mid.clone();
                        hi = mid.clone();
                        exact = Some(mid);
                        break;
                    }
                    let left = variations(&sturm, &lo) - variations(&sturm, &mid);
                    if left == 1 {
                        hi = mid;
                    } else {
                        lo = mid;
                    }
                }
            }
            if exact.is_none() {
                let midpoint = (&lo + &hi) / Rational::from_integer(2.into());
                let mut value = midpoint.clone();
                let (mut p0, mut p1) = (BigInt::zero(), BigInt::one());
                let (mut q0, mut q1) = (BigInt::one(), BigInt::zero());
                for _ in 0..128 {
                    let integer = value.floor().to_integer();
                    let p2 = &integer * &p1 + &p0;
                    let q2 = &integer * &q1 + &q0;
                    if q2 > BigInt::from(1_000_000_000_000i64) {
                        break;
                    }
                    let candidate = Rational::new(p2.clone(), q2.clone());
                    if candidate >= lo && candidate <= hi && q_eval(&q, &candidate).is_zero() {
                        lo = candidate.clone();
                        hi = candidate.clone();
                        exact = Some(candidate);
                        break;
                    }
                    let remainder = &value - Rational::from_integer(integer);
                    if remainder.is_zero() {
                        break;
                    }
                    value = remainder.recip();
                    p0 = p1;
                    p1 = p2;
                    q0 = q1;
                    q1 = q2;
                }
            }
            roots.push(RealRoot {
                low: lo,
                high: hi,
                exact,
            });
        } else {
            let mid = (&lo + &hi) / Rational::from_integer(2.into());
            let left = variations(&sturm, &lo) - variations(&sturm, &mid);
            pending.push((mid.clone(), hi, count - left, depth + 1));
            pending.push((lo, mid, left, depth + 1));
        }
    }
    roots.sort_by(|a, b| a.low.cmp(&b.low));
    Ok(roots)
}

#[cfg(test)]
mod isolation_tests {
    use super::*;
    #[test]
    fn sturm_finds_even_multiplicity_and_close_rational_roots() {
        let k = Expr::k();
        let a = k.sub(&Expr::integer(1));
        let b = k.sub(&Expr::rational(parse_rational("1.000001").unwrap()));
        let p = a.mul(&a).mul(&b);
        let roots = real_roots(&p.num).unwrap();
        assert_eq!(roots.len(), 2);
        assert_eq!(roots[0].exact, Some(Rational::one()));
        assert_eq!(roots[1].exact, Some(parse_rational("1.000001").unwrap()));
    }
    #[test]
    fn sturm_certifies_irrational_real_roots_without_grid() {
        let p = Expr::k().mul(&Expr::k()).sub(&Expr::integer(2));
        let roots = real_roots(&p.num).unwrap();
        assert_eq!(roots.len(), 2);
        assert!((roots[0].value() + 2f64.sqrt()).abs() < 1e-14);
        assert!((roots[1].value() - 2f64.sqrt()).abs() < 1e-14);
        assert!(roots.iter().all(|root| root.exact.is_none()));
    }
}
