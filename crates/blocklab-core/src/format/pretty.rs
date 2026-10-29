//! Árvore de apresentação: como a expressão vai *aparecer*, já com
//! fatores postos em evidência e termos ordenados. É independente do
//! formato final (texto, LaTeX ou o desenho na interface).

use std::cmp::Reverse;

use num_rational::BigRational;
use num_traits::{One, Signed, Zero};

use crate::algebra::{Monomial, Symbol, Var};

#[derive(Clone, Debug, PartialEq)]
pub enum Pretty {
    /// Número não negativo (o sinal fica na soma que o contém).
    Num(BigRational),
    Var(Var),
    /// Variável elevada a um expoente ≥ 2.
    Pow(Var, u32),
    /// Produto; um eventual coeficiente numérico vem primeiro.
    Product(Vec<Pretty>),
    /// Soma de parcelas com sinal (`true` = negativa).
    Sum(Vec<(bool, Pretty)>),
}

impl Pretty {
    /// `true` se é uma soma de duas ou mais parcelas (precisa de
    /// parênteses dentro de um produto).
    pub fn is_compound_sum(&self) -> bool {
        matches!(self, Pretty::Sum(ts) if ts.len() > 1)
    }

    /// `true` se é um "átomo": número inteiro, variável ou potência.
    /// Só átomos dispensam parênteses num denominador escrito em linha.
    pub fn is_atom(&self) -> bool {
        match self {
            Pretty::Num(q) => q.is_integer(),
            Pretty::Var(_) | Pretty::Pow(..) => true,
            _ => false,
        }
    }
}

/// Uma parcela de soma durante a montagem, com a chave que decide sua
/// posição.
#[derive(Clone, Debug)]
pub struct Summand {
    pub negative: bool,
    pub body: Pretty,
    pub key: TermKey,
}

/// Ordem das parcelas: maior grau em `s` primeiro (s² + s + 10); depois
/// menor grau total (1 − G·H, a constante na frente); depois ordem
/// natural das variáveis (G1·H1 antes de G2·H2).
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct TermKey(Reverse<u32>, u32, Vec<(VarKey, u32)>);

impl TermKey {
    pub fn of(m: &Monomial) -> Self {
        TermKey(
            Reverse(m.degree_in(&Var::S)),
            m.total_degree(),
            m.factors().map(|(v, e)| (VarKey::of(v), e)).collect(),
        )
    }
}

/// Variáveis com `s` por último, para escrever `K·s` e não `s·K`.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum VarKey {
    Sym(Symbol),
    S,
}

impl VarKey {
    fn of(v: &Var) -> Self {
        match v {
            Var::S => VarKey::S,
            Var::Sym(sym) => VarKey::Sym(sym.clone()),
        }
    }
}

/// Ordem dos fatores num produto:
/// 1. o coeficiente numérico;
/// 2. símbolos e somas sem `s`, por "família" de letra (G, H, K...),
///    dentro dela os símbolos soltos antes das somas: G1·G4·(G2+G3)·H2;
/// 3. a variável `s` (K·s, não s·K);
/// 4. somas que contêm `s`: s·(s+2), K·s·(s^2+a·s+1).
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum FactorKey {
    Number,
    Symbolic {
        prefix: String,
        is_sum: bool,
        first: Symbol,
    },
    S,
    SumWithS {
        first: Option<Symbol>,
    },
}

/// Letras do começo do nome: "G1" → "G", "H_2" → "H_", "Kp" → "Kp".
fn prefix(sym: &Symbol) -> String {
    sym.name()
        .chars()
        .take_while(|c| !c.is_ascii_digit())
        .collect()
}

/// Menor símbolo que aparece na expressão (se houver).
fn first_symbol(p: &Pretty) -> Option<Symbol> {
    match p {
        Pretty::Num(_) | Pretty::Var(Var::S) | Pretty::Pow(Var::S, _) => None,
        Pretty::Var(Var::Sym(s)) | Pretty::Pow(Var::Sym(s), _) => Some(s.clone()),
        Pretty::Product(fs) => fs.iter().filter_map(first_symbol).min(),
        Pretty::Sum(ts) => ts.iter().filter_map(|(_, t)| first_symbol(t)).min(),
    }
}

fn contains_s(p: &Pretty) -> bool {
    match p {
        Pretty::Num(_) => false,
        Pretty::Var(v) | Pretty::Pow(v, _) => *v == Var::S,
        Pretty::Product(fs) => fs.iter().any(contains_s),
        Pretty::Sum(ts) => ts.iter().any(|(_, t)| contains_s(t)),
    }
}

fn factor_key(p: &Pretty) -> FactorKey {
    match p {
        Pretty::Num(_) => FactorKey::Number,
        Pretty::Var(Var::S) | Pretty::Pow(Var::S, _) => FactorKey::S,
        _ if contains_s(p) => FactorKey::SumWithS {
            first: first_symbol(p),
        },
        _ => {
            let first = first_symbol(p).expect("sem s e não numérico: tem símbolo");
            FactorKey::Symbolic {
                prefix: prefix(&first),
                is_sum: p.is_compound_sum(),
                first,
            }
        }
    }
}

/// `v^e` como átomo.
pub fn var_pow(v: &Var, e: u32) -> Pretty {
    if e == 1 {
        Pretty::Var(v.clone())
    } else {
        Pretty::Pow(v.clone(), e)
    }
}

/// Produto de fatores: achata produtos internos, descarta 1, ordena.
/// Um só fator vira ele mesmo; nenhum vira 1.
pub fn product(factors: Vec<Pretty>) -> Pretty {
    let mut flat = Vec::new();
    for f in factors {
        match f {
            Pretty::Product(inner) => flat.extend(inner),
            Pretty::Num(q) if q.is_one() => {}
            other => flat.push(other),
        }
    }
    flat.sort_by_key(factor_key);
    match flat.len() {
        0 => Pretty::Num(BigRational::one()),
        1 => flat.pop().expect("um fator"),
        _ => Pretty::Product(flat),
    }
}

/// Termo c·m (c ≠ 0) como parcela: |c| e as variáveis, com o sinal à parte.
pub fn term(c: &BigRational, m: &Monomial) -> Summand {
    let mut factors = vec![Pretty::Num(c.abs())];
    factors.extend(m.factors().map(|(v, e)| var_pow(v, e)));
    Summand {
        negative: c.is_negative(),
        body: product(factors),
        key: TermKey::of(m),
    }
}

/// Junta parcelas numa expressão: ordena pela chave; uma parcela
/// positiva sozinha vira ela mesma.
pub fn sum(mut parts: Vec<Summand>) -> Pretty {
    parts.sort_by(|a, b| a.key.cmp(&b.key));
    match parts.as_slice() {
        [] => Pretty::Num(BigRational::zero()),
        [one] if !one.negative => one.body.clone(),
        _ => Pretty::Sum(parts.into_iter().map(|p| (p.negative, p.body)).collect()),
    }
}
