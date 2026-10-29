//! Funções racionais: numerador/denominador, sempre em forma canônica.
//!
//! "Canônica" quer dizer que duas frações matematicamente iguais têm
//! exatamente a mesma representação, então `==` funciona. Para isso,
//! toda fração criada passa por [`RationalFn::normalized`]:
//!
//! 1. cancela o MDC entre numerador e denominador;
//! 2. ainda sobra uma escolha livre: multiplicar em cima e embaixo pela
//!    mesma constante c ≠ 0 não muda a fração. Fixamos c pela regra:
//!    * denominador com `s` cujo coeficiente da maior potência de `s` é
//!      um **número** → denominador **mônico em s**: 10/(s²+s+10),
//!      K/(s² + 3s + K + 2);
//!    * senão, se o denominador tem termo constante → constante = 1,
//!      a forma usual de controle: G/(1 + G·H), 1/(T·s + 1);
//!    * senão → coeficientes inteiros primitivos e termo líder positivo.
//!
//!    A regra depende só de propriedades que não mudam ao multiplicar o
//!    denominador por uma constante (ter `s`, o coeficiente líder em `s`
//!    ser número, ter termo constante), então a escolha é única.

use std::collections::BTreeSet;
use std::fmt;
use std::ops::{Add, Div, Mul, Neg, Sub};

use num_rational::BigRational;
use num_traits::{One, Signed};
use thiserror::Error;

use super::gcd::gcd;
use super::poly::{Params, Poly};
use super::symbol::{Symbol, Var};

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum AlgebraError {
    #[error("divisão por zero")]
    DivisionByZero,
}

/// Função racional `num/den`. Invariante: `den ≠ 0` e forma canônica.
#[derive(Clone, PartialEq, Eq)]
pub struct RationalFn {
    num: Poly,
    den: Poly,
}

impl RationalFn {
    /// Cria `num/den` já normalizada. Erro se `den = 0`.
    pub fn new(num: Poly, den: Poly) -> Result<Self, AlgebraError> {
        if den.is_zero() {
            return Err(AlgebraError::DivisionByZero);
        }
        Ok(RationalFn::normalized(num, den))
    }

    pub fn zero() -> Self {
        RationalFn::from_poly(Poly::zero())
    }

    pub fn one() -> Self {
        RationalFn::from_poly(Poly::one())
    }

    pub fn from_int(n: i64) -> Self {
        RationalFn::from_poly(Poly::from_int(n))
    }

    pub fn constant(c: BigRational) -> Self {
        RationalFn::from_poly(Poly::constant(c))
    }

    pub fn var(v: Var) -> Self {
        RationalFn::from_poly(Poly::var(v))
    }

    /// Um polinômio é uma fração com denominador 1 (já canônica).
    pub fn from_poly(p: Poly) -> Self {
        RationalFn {
            num: p,
            den: Poly::one(),
        }
    }

    pub fn num(&self) -> &Poly {
        &self.num
    }

    pub fn den(&self) -> &Poly {
        &self.den
    }

    pub fn is_zero(&self) -> bool {
        self.num.is_zero()
    }

    pub fn is_one(&self) -> bool {
        self.num == Poly::one() && self.den == Poly::one()
    }

    /// Símbolos que aparecem (sem `s`), em ordem natural.
    pub fn symbols(&self) -> BTreeSet<Symbol> {
        self.num
            .vars()
            .into_iter()
            .chain(self.den.vars())
            .filter_map(|v| match v {
                Var::Sym(sym) => Some(sym),
                Var::S => None,
            })
            .collect()
    }

    /// O número, se a função for constante (sem `s` nem símbolos).
    pub fn as_constant(&self) -> Option<BigRational> {
        if self.den != Poly::one() || !self.num.is_constant() {
            return None;
        }
        Some(self.num.constant_term().cloned().unwrap_or_default())
    }

    /// `true` se não há símbolos (só números e `s`).
    pub fn is_numeric(&self) -> bool {
        self.num.is_univariate_s() && self.den.is_univariate_s()
    }

    /// Inverso multiplicativo 1/f.
    pub fn inv(&self) -> Result<Self, AlgebraError> {
        RationalFn::new(self.den.clone(), self.num.clone())
    }

    /// Divisão que devolve erro em vez de entrar em pânico.
    pub fn checked_div(&self, rhs: &Self) -> Result<Self, AlgebraError> {
        if rhs.is_zero() {
            return Err(AlgebraError::DivisionByZero);
        }
        Ok(RationalFn::normalized(
            &self.num * &rhs.den,
            &self.den * &rhs.num,
        ))
    }

    /// Potência inteira; expoente negativo inverte (erro se a base é 0).
    pub fn pow(&self, exp: i32) -> Result<Self, AlgebraError> {
        let base = if exp < 0 { self.inv()? } else { self.clone() };
        let e = exp.unsigned_abs();
        // (n/d)^e = n^e/d^e já sai canônica, sem precisar de MDC:
        // * se n e d não têm fator comum, n^e e d^e também não;
        // * a escala se preserva: d mônico ⇒ d^e mônico; termo constante
        //   1 ⇒ 1^e = 1; e produto de polinômios primitivos é primitivo
        //   (lema de Gauss), com coeficiente líder lc^e > 0.
        Ok(RationalFn {
            num: base.num.pow(e),
            den: base.den.pow(e),
        })
    }

    /// Substitui símbolos por números e simplifica de novo.
    /// Erro se o denominador zerar com esses valores.
    pub fn substitute(&self, params: &Params) -> Result<Self, AlgebraError> {
        RationalFn::new(self.num.substitute(params), self.den.substitute(params))
    }

    /// Soma de várias frações com **uma única** normalização no final.
    ///
    /// Somar uma a uma com `+` calcularia um MDC a cada passo. Aqui
    /// acumulamos n/d sem simplificar (a/b + c/e = (a·e + c·b)/(b·e), com
    /// atalho quando os denominadores são iguais) e só cancelamos no fim.
    pub fn sum<'a>(items: impl IntoIterator<Item = &'a RationalFn>) -> Self {
        let mut num = Poly::zero();
        let mut den = Poly::one();
        for f in items {
            if f.den == den {
                num = &num + &f.num;
            } else {
                num = &(&num * &f.den) + &(&f.num * &den);
                den = &den * &f.den;
            }
        }
        RationalFn::normalized(num, den)
    }

    /// Produto de várias frações com uma única normalização no final.
    pub fn product<'a>(items: impl IntoIterator<Item = &'a RationalFn>) -> Self {
        let mut num = Poly::one();
        let mut den = Poly::one();
        for f in items {
            num = &num * &f.num;
            den = &den * &f.den;
        }
        RationalFn::normalized(num, den)
    }

    /// Coloca `num/den` na forma canônica (ver doc do módulo).
    /// Pré-condição: `den ≠ 0`.
    fn normalized(mut num: Poly, mut den: Poly) -> Self {
        debug_assert!(!den.is_zero());
        if num.is_zero() {
            return RationalFn::zero();
        }
        // Denominador numérico: basta dividir o numerador por ele.
        if den.is_constant() {
            let c = den.constant_term().expect("den ≠ 0");
            return RationalFn::from_poly(num.scale(&c.recip()));
        }

        // 1. Cancela fatores comuns.
        let g = gcd(&num, &den);
        if !g.is_constant() {
            num = num.exact_div(&g).expect("o MDC divide o numerador");
            den = den.exact_div(&g).expect("o MDC divide o denominador");
        }

        // 2. Fixa a escala pela regra do denominador.
        let lc_s = den.leading_coeff_in(&Var::S);
        let k: BigRational = if den.degree_in(&Var::S) > 0 && lc_s.is_constant() {
            lc_s.constant_term().expect("coeficiente líder ≠ 0").clone()
        } else if let Some(c) = den.constant_term() {
            c.clone()
        } else {
            let lc = den.leading().expect("den ≠ 0").1;
            let content = den.numeric_content();
            if lc.is_negative() { -content } else { content }
        };
        if !k.is_one() {
            let inv = k.recip();
            num = num.scale(&inv);
            den = den.scale(&inv);
        }
        RationalFn { num, den }
    }
}

impl From<Poly> for RationalFn {
    fn from(p: Poly) -> Self {
        RationalFn::from_poly(p)
    }
}

impl From<Var> for RationalFn {
    fn from(v: Var) -> Self {
        RationalFn::var(v)
    }
}

impl Add<&RationalFn> for &RationalFn {
    type Output = RationalFn;
    fn add(self, rhs: &RationalFn) -> RationalFn {
        RationalFn::sum([self, rhs])
    }
}

impl Sub<&RationalFn> for &RationalFn {
    type Output = RationalFn;
    fn sub(self, rhs: &RationalFn) -> RationalFn {
        RationalFn::sum([self, &-rhs])
    }
}

impl Mul<&RationalFn> for &RationalFn {
    type Output = RationalFn;
    fn mul(self, rhs: &RationalFn) -> RationalFn {
        RationalFn::product([self, rhs])
    }
}

impl Div<&RationalFn> for &RationalFn {
    type Output = RationalFn;
    /// # Panics
    /// Se `rhs` é zero. Use [`RationalFn::checked_div`] para tratar o erro.
    fn div(self, rhs: &RationalFn) -> RationalFn {
        self.checked_div(rhs)
            .expect("divisão de função racional por zero")
    }
}

impl Neg for &RationalFn {
    type Output = RationalFn;
    /// Trocar o sinal do numerador não altera a forma canônica.
    fn neg(self) -> RationalFn {
        RationalFn {
            num: -&self.num,
            den: self.den.clone(),
        }
    }
}

super::forward_ops!(RationalFn);
super::forward_binop!(Div, div, RationalFn);

/// Formato de depuração: `(num)/(den)`, expandido.
impl fmt::Display for RationalFn {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.den == Poly::one() {
            write!(f, "{}", self.num)
        } else {
            write!(f, "({})/({})", self.num, self.den)
        }
    }
}

impl fmt::Debug for RationalFn {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}
