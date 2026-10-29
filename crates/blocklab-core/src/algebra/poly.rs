//! Polinômios esparsos em Q[s, símbolos].
//!
//! Um polinômio é um mapa monômio → coeficiente racional exato
//! (`BigRational`, sem ponto flutuante). Só os termos não nulos são
//! guardados, por isso "esparso": `s^100 + 1` ocupa dois termos.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::ops::{Add, Mul, Neg, Sub};

use num_bigint::BigInt;
use num_integer::Integer;
use num_rational::BigRational;
use num_traits::{One, Signed, Zero};

use super::monomial::Monomial;
use super::symbol::{Symbol, Var};

/// Valores numéricos para os símbolos (tabela de parâmetros).
pub type Params = BTreeMap<Symbol, BigRational>;

/// Polinômio esparso. Invariante: nenhum coeficiente guardado é zero.
/// Assim a igualdade estrutural (`==`) é a igualdade matemática.
#[derive(Clone, PartialEq, Eq, Default)]
pub struct Poly {
    terms: BTreeMap<Monomial, BigRational>,
}

impl Poly {
    pub fn zero() -> Self {
        Poly::default()
    }

    pub fn one() -> Self {
        Poly::constant(BigRational::one())
    }

    pub fn constant(c: BigRational) -> Self {
        Poly::term(Monomial::one(), c)
    }

    pub fn from_int(n: i64) -> Self {
        Poly::constant(BigRational::from_integer(n.into()))
    }

    /// O polinômio formado só pela variável `v`.
    pub fn var(v: Var) -> Self {
        Poly::var_pow(v, 1)
    }

    pub fn var_pow(v: Var, exp: u32) -> Self {
        Poly::term(Monomial::var_pow(v, exp), BigRational::one())
    }

    /// Um único termo `c·m`.
    pub fn term(m: Monomial, c: BigRational) -> Self {
        let mut p = Poly::zero();
        p.add_term(m, c);
        p
    }

    pub fn is_zero(&self) -> bool {
        self.terms.is_empty()
    }

    /// `true` se é um número (inclusive zero).
    pub fn is_constant(&self) -> bool {
        self.terms.keys().all(Monomial::is_one)
    }

    /// Coeficiente do termo independente, se não for zero.
    pub fn constant_term(&self) -> Option<&BigRational> {
        self.terms.get(&Monomial::one())
    }

    /// Termos em ordem crescente da ordem monomial (o último é o líder).
    pub fn terms(&self) -> impl DoubleEndedIterator<Item = (&Monomial, &BigRational)> {
        self.terms.iter()
    }

    pub fn num_terms(&self) -> usize {
        self.terms.len()
    }

    /// Termo líder: o maior monômio na ordem definida em `monomial.rs`
    /// (primeiro grau em s, depois grau total). `None` para o polinômio zero.
    pub fn leading(&self) -> Option<(&Monomial, &BigRational)> {
        self.terms.iter().next_back()
    }

    /// Todas as variáveis que aparecem.
    pub fn vars(&self) -> BTreeSet<Var> {
        self.terms
            .keys()
            .flat_map(|m| m.factors().map(|(v, _)| v.clone()))
            .collect()
    }

    /// `true` se a única variável presente (se houver) é `s`.
    pub fn is_univariate_s(&self) -> bool {
        self.terms
            .keys()
            .all(|m| m.factors().all(|(v, _)| *v == Var::S))
    }

    /// Maior expoente de `v` entre os termos.
    pub fn degree_in(&self, v: &Var) -> u32 {
        self.terms.keys().map(|m| m.degree_in(v)).max().unwrap_or(0)
    }

    /// Soma `c·m` ao polinômio, mantendo a invariante (remove se zerar).
    pub fn add_term(&mut self, m: Monomial, c: BigRational) {
        if c.is_zero() {
            return;
        }
        match self.terms.entry(m) {
            std::collections::btree_map::Entry::Vacant(e) => {
                e.insert(c);
            }
            std::collections::btree_map::Entry::Occupied(mut e) => {
                *e.get_mut() += c;
                if e.get().is_zero() {
                    e.remove();
                }
            }
        }
    }

    /// `self += c · m · other`, sem criar polinômios intermediários.
    /// É o passo básico da divisão: subtrair um múltiplo do divisor.
    fn add_scaled(&mut self, other: &Poly, m: &Monomial, c: &BigRational) {
        for (om, oc) in &other.terms {
            self.add_term(om.mul(m), oc * c);
        }
    }

    /// Multiplica todos os coeficientes por `c`.
    pub fn scale(&self, c: &BigRational) -> Poly {
        if c.is_zero() {
            return Poly::zero();
        }
        Poly {
            terms: self.terms.iter().map(|(m, k)| (m.clone(), k * c)).collect(),
        }
    }

    /// Potência por quadrados sucessivos: p^e com O(log e) produtos.
    pub fn pow(&self, mut exp: u32) -> Poly {
        let mut base = self.clone();
        let mut acc = Poly::one();
        while exp > 0 {
            if exp & 1 == 1 {
                acc = &acc * &base;
            }
            exp >>= 1;
            if exp > 0 {
                base = &base * &base;
            }
        }
        acc
    }

    /// Conteúdo numérico: o racional positivo c tal que `self / c` tem
    /// coeficientes inteiros sem fator comum.
    ///
    /// c = mdc(numeradores) / mmc(denominadores).
    /// Ex.: 4/3·s + 2 → c = mdc(4, 2)/mmc(3, 1) = 2/3, e p/c = 2s + 3.
    pub fn numeric_content(&self) -> BigRational {
        let mut g = BigInt::zero();
        let mut l = BigInt::one();
        for c in self.terms.values() {
            g = g.gcd(c.numer());
            l = l.lcm(c.denom());
        }
        if g.is_zero() {
            BigRational::one()
        } else {
            BigRational::new(g, l)
        }
    }

    /// MDC de todos os monômios (maior monômio que divide todos os termos).
    pub fn monomial_content(&self) -> Monomial {
        let mut it = self.terms.keys();
        match it.next() {
            None => Monomial::one(),
            Some(first) => it.fold(first.clone(), |acc, m| acc.gcd(m)),
        }
    }

    /// Divide por um monômio que divide todos os termos.
    pub fn div_monomial(&self, d: &Monomial) -> Option<Poly> {
        let mut out = BTreeMap::new();
        for (m, c) in &self.terms {
            out.insert(m.div(d)?, c.clone());
        }
        Some(Poly { terms: out })
    }

    /// Divisão exata: devolve q com `self = q · d`, ou `None` se `d` não
    /// divide `self` (ou se `d = 0`).
    ///
    /// Algoritmo: como a ordem monomial é compatível com o produto, o
    /// termo líder de q·d é LT(q)·LT(d). Então, enquanto sobrar resto r,
    /// LT(r) precisa ser divisível por LT(d); o quociente desse passo é
    /// LT(r)/LT(d), e subtraímos (LT(r)/LT(d))·d de r. O termo líder de r
    /// diminui a cada passo, então o laço termina. Se em algum passo
    /// LT(d) não dividir LT(r), a divisão não é exata.
    ///
    /// **Desistir cedo importa.** Quando `d` não divide, o laço pode dar
    /// muitos passos com frações cada vez maiores antes de falhar (isso
    /// travou o MDC heurístico, que testa candidatos por divisão). Então
    /// também paramos se um termo do quociente passar do grau possível:
    /// se f = q·d, então grau_v(q) = grau_v(f) − grau_v(d) para cada
    /// variável v, e nenhum termo de q pode exceder isso.
    pub fn exact_div(&self, d: &Poly) -> Option<Poly> {
        self.exact_div_impl(d, false)
    }

    /// Divisão exata em Z[...]: como [`Poly::exact_div`], mas exige
    /// quociente com coeficientes inteiros. Se a divisão é exata em Z,
    /// cada coeficiente calculado no caminho já é um coeficiente de q,
    /// logo inteiro; o primeiro coeficiente fracionário encerra a busca.
    pub fn exact_div_integer(&self, d: &Poly) -> Option<Poly> {
        self.exact_div_impl(d, true)
    }

    fn exact_div_impl(&self, d: &Poly, integer: bool) -> Option<Poly> {
        let (lm_d, lc_d) = d.leading()?;
        if d.num_terms() == 1 {
            // Caso comum e barato: divisor monomial.
            let q = self.div_monomial(lm_d)?.scale(&lc_d.recip());
            return (!integer || q.terms.values().all(BigRational::is_integer)).then_some(q);
        }
        // Grau máximo de cada variável no quociente.
        let mut bound: BTreeMap<Var, u32> = BTreeMap::new();
        for v in self.vars().into_iter().chain(d.vars()) {
            let (df, dd) = (self.degree_in(&v), d.degree_in(&v));
            if dd > df {
                return None;
            }
            bound.insert(v, df - dd);
        }
        let mut r = self.clone();
        let mut q = Poly::zero();
        while let Some((lm_r, lc_r)) = r.leading() {
            let m = lm_r.div(lm_d)?;
            if m.factors().any(|(v, e)| e > bound[v]) {
                return None;
            }
            let c = lc_r / lc_d;
            if integer && !c.is_integer() {
                return None;
            }
            r.add_scaled(d, &m, &-c.clone());
            q.add_term(m, c);
        }
        Some(q)
    }

    /// Coeficiente de `v^k` vendo o polinômio como polinômio em `v`
    /// (o resultado não contém `v`).
    pub fn coeff_in(&self, v: &Var, k: u32) -> Poly {
        let mut out = Poly::zero();
        for (m, c) in &self.terms {
            let (e, rest) = m.split_off(v);
            if e == k {
                out.add_term(rest, c.clone());
            }
        }
        out
    }

    /// Todos os coeficientes em `v`, do grau 0 ao grau máximo.
    pub fn coeffs_in(&self, v: &Var) -> Vec<Poly> {
        let mut out = vec![Poly::zero(); self.degree_in(v) as usize + 1];
        for (m, c) in &self.terms {
            let (e, rest) = m.split_off(v);
            out[e as usize].add_term(rest, c.clone());
        }
        out
    }

    /// Coeficiente líder em `v` (coeficiente da maior potência de `v`).
    pub fn leading_coeff_in(&self, v: &Var) -> Poly {
        self.coeff_in(v, self.degree_in(v))
    }

    /// Troca cada símbolo que tem valor em `params` pelo número.
    /// Símbolos sem valor continuam simbólicos.
    pub fn substitute(&self, params: &Params) -> Poly {
        let mut out = Poly::zero();
        for (m, c) in &self.terms {
            let mut coef = c.clone();
            let mut rest = Monomial::one();
            for (v, e) in m.factors() {
                let value = match v {
                    Var::Sym(sym) => params.get(sym),
                    Var::S => None,
                };
                match value {
                    Some(x) => coef *= num_traits::pow(x.clone(), e as usize),
                    None => rest = rest.mul(&Monomial::var_pow(v.clone(), e)),
                }
            }
            out.add_term(rest, coef);
        }
        out
    }

    /// Divide pelo conteúdo numérico e acerta o sinal para que o
    /// coeficiente líder seja positivo. Serve como "representante
    /// canônico" de um polinômio definido a menos de constante (como o MDC).
    pub fn normalize_unit(&self) -> Poly {
        let Some((_, lc)) = self.leading() else {
            return Poly::zero();
        };
        let mut k = self.numeric_content();
        if lc.is_negative() {
            k = -k;
        }
        self.scale(&k.recip())
    }
}

impl From<BigRational> for Poly {
    fn from(c: BigRational) -> Self {
        Poly::constant(c)
    }
}

impl From<Var> for Poly {
    fn from(v: Var) -> Self {
        Poly::var(v)
    }
}

impl Add<&Poly> for &Poly {
    type Output = Poly;
    fn add(self, rhs: &Poly) -> Poly {
        let mut out = self.clone();
        for (m, c) in &rhs.terms {
            out.add_term(m.clone(), c.clone());
        }
        out
    }
}

impl Sub<&Poly> for &Poly {
    type Output = Poly;
    fn sub(self, rhs: &Poly) -> Poly {
        let mut out = self.clone();
        for (m, c) in &rhs.terms {
            out.add_term(m.clone(), -c.clone());
        }
        out
    }
}

impl Mul<&Poly> for &Poly {
    type Output = Poly;
    /// Produto termo a termo (distributiva): (Σ aᵢmᵢ)(Σ bⱼnⱼ) = Σ aᵢbⱼ·mᵢnⱼ.
    fn mul(self, rhs: &Poly) -> Poly {
        let mut out = Poly::zero();
        for (m, c) in &self.terms {
            out.add_scaled(rhs, m, c);
        }
        out
    }
}

impl Neg for &Poly {
    type Output = Poly;
    fn neg(self) -> Poly {
        self.scale(&-BigRational::one())
    }
}

super::forward_ops!(Poly);

/// Formato de depuração, expandido, termos do maior para o menor.
/// A formatação "bonita" (fatorada, LaTeX) fica no módulo `format`.
impl fmt::Display for Poly {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_zero() {
            return f.write_str("0");
        }
        for (k, (m, c)) in self.terms.iter().rev().enumerate() {
            let negative = c.is_negative();
            let abs = c.abs();
            match (k, negative) {
                (0, true) => f.write_str("-")?,
                (0, false) => {}
                (_, true) => f.write_str(" - ")?,
                (_, false) => f.write_str(" + ")?,
            }
            if m.is_one() {
                write!(f, "{abs}")?;
            } else if abs.is_one() {
                write!(f, "{m}")?;
            } else {
                write!(f, "{abs}*{m}")?;
            }
        }
        Ok(())
    }
}

impl fmt::Debug for Poly {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s() -> Poly {
        Poly::var(Var::S)
    }
    fn sym(n: &str) -> Poly {
        Poly::var(Var::named(n))
    }
    fn k(n: i64) -> Poly {
        Poly::from_int(n)
    }
    fn q(n: i64, d: i64) -> BigRational {
        BigRational::new(n.into(), d.into())
    }

    #[test]
    fn operacoes_basicas() {
        let p = &s() + &k(1); // s + 1
        let quad = &p * &p;
        assert_eq!(quad, &(&s().pow(2) + &(&k(2) * &s())) + &k(1));
        assert_eq!(&quad - &quad, Poly::zero());
        assert_eq!(p.pow(3).degree_in(&Var::S), 3);
        assert_eq!(format!("{quad}"), "s^2 + 2*s + 1");
    }

    #[test]
    fn divisao_exata() {
        let a = &s() + &sym("a");
        let b = &(&s() * &sym("K")) - &k(2);
        let prod = &a * &b;
        assert_eq!(prod.exact_div(&a), Some(b.clone()));
        assert_eq!(prod.exact_div(&b), Some(a.clone()));
        // s + 2 não divide s² + 1.
        assert_eq!((&s().pow(2) + &k(1)).exact_div(&(&s() + &k(2))), None);
        assert_eq!(prod.exact_div(&Poly::zero()), None);
    }

    #[test]
    fn conteudos() {
        // 4/3·s·G1 + 2·G1 → conteúdo numérico 2/3, monomial G1.
        let p = &(&s() * &sym("G1")).scale(&q(4, 3)) + &(&k(2) * &sym("G1"));
        assert_eq!(p.numeric_content(), q(2, 3));
        assert_eq!(p.monomial_content(), Monomial::var_pow(Var::named("G1"), 1));
        // Normalização: divide por 2/3 → 2s·G1 + 3·G1.
        assert_eq!(format!("{}", p.normalize_unit()), "2*s*G1 + 3*G1");
        assert_eq!(format!("{}", (-&p).normalize_unit()), "2*s*G1 + 3*G1");
    }

    #[test]
    fn coeficientes_em_variavel() {
        // (a + 1)·s² + 3: coeficiente de s² é a + 1.
        let p = &(&(&sym("a") + &k(1)) * &s().pow(2)) + &k(3);
        assert_eq!(p.leading_coeff_in(&Var::S), &sym("a") + &k(1));
        assert_eq!(
            p.coeffs_in(&Var::S),
            vec![k(3), Poly::zero(), &sym("a") + &k(1)]
        );
    }

    #[test]
    fn substituicao_parcial() {
        let p = &(&sym("K") * &s()) + &sym("a");
        let mut params = Params::new();
        params.insert(Symbol::new("K"), q(1, 2));
        assert_eq!(format!("{}", p.substitute(&params)), "1/2*s + a");
    }
}
