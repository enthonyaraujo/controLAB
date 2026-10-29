//! Polinômio → árvore de apresentação.
//!
//! **Modo simbólico** — uma heurística de fatoração legível, aplicada
//! recursivamente:
//!
//! 1. *Fator comum*: se todos os termos têm um monômio em comum (ou um
//!    inteiro em comum), ele vai para fora:
//!    G1·G4·G2 + G1·G4·G3 → G1·G4·(G2+G3).
//! 2. *Produto de somas*: se um agrupamento Q (ver 3) divide o polinômio
//!    inteiro, mostramos Q·(P/Q):
//!    1 + G1·H1 + G2·H2 + G1·H1·G2·H2 → (1+G1·H1)·(1+G2·H2).
//! 3. *Agrupamento*: entre os pares de termos, acha o maior monômio K
//!    com símbolos que divide dois ou mais termos e põe K em evidência
//!    só neles; o resto segue para a mesma análise:
//!    1 − G1·G4·H1 + G1·G4·G2·H2 + G1·G4·G3·H2 →
//!    1 − G1·G4·H1 + G1·G4·(G2+G3)·H2.
//!
//! Escolher o K de **maior grau** (G1·G4·H2, não G1·G4) é o que preserva
//! os laços da fórmula de Mason visíveis no resultado.
//!
//! Se, depois do fator comum, ainda houver `s`, os passos 2 e 3 não
//! misturam potências diferentes de `s` (isso daria coisas como
//! (2·ζ·s + ω_n)·ω_n). O polinômio é mostrado por potências de `s`, e a
//! fatoração age só **dentro** de cada coeficiente:
//! s² + 2·ζ·ω_n·s + ω_n², G1·(G2+G3)·s + 1.
//!
//! **Modo numérico** — polinômio em `s` com potências decrescentes; se
//! sobrar símbolo, o coeficiente de cada potência vem entre parênteses:
//! s² + (K+a)·s + 10.
//!
//! Nada disso muda o valor: a forma canônica continua em `RationalFn`.

use num_rational::BigRational;
use num_traits::{One, Signed};

use super::pretty::{Pretty, Summand, TermKey, product, sum, term, var_pow};
use crate::algebra::{Monomial, Poly, Var};

type Terms = Vec<(BigRational, Monomial)>;

/// Acima disso a fatoração (quadrática no número de termos) não compensa:
/// mostra a soma expandida.
const MAX_FACTOR_TERMS: usize = 200;
/// Quantos candidatos testar como fator de um produto de somas.
const MAX_PRODUCT_TRIES: usize = 12;

pub fn symbolic(p: &Poly) -> Pretty {
    sum(factored(terms_of(p)))
}

pub fn numeric(p: &Poly) -> Pretty {
    let coeffs = p.coeffs_in(&Var::S);
    let mut parts = Vec::new();
    for (k, c) in coeffs.iter().enumerate().rev() {
        let k = k as u32;
        if c.is_zero() {
            continue;
        }
        if k == 0 {
            parts.extend(c.terms().map(|(m, q)| term(q, m)));
        } else if c.num_terms() == 1 {
            let (m, q) = c.leading().expect("não nulo");
            parts.push(term(q, &m.mul(&Monomial::var_pow(Var::S, k))));
        } else {
            // Coeficiente com vários termos: (K+a)·s. Se todos forem
            // negativos, o sinal sai para fora: −(K+a)·s.
            let negative = c.terms().all(|(_, q)| q.is_negative());
            let inner: Vec<Summand> = c
                .terms()
                .map(|(m, q)| term(&if negative { -q } else { q.clone() }, m))
                .collect();
            parts.push(Summand {
                negative,
                body: product(vec![sum(inner), var_pow(&Var::S, k)]),
                key: TermKey::of(&Monomial::var_pow(Var::S, k)),
            });
        }
    }
    sum(parts)
}

/// Modo simbólico com `s`: uma parcela por potência de `s` (o
/// coeficiente fatorado na frente), e o termo independente espalhado na
/// soma de fora.
fn by_powers_of_s(p: &Poly) -> Vec<Summand> {
    let mut out = Vec::new();
    for (k, c) in p.coeffs_in(&Var::S).iter().enumerate() {
        if c.is_zero() {
            continue;
        }
        let k = k as u32;
        let summands = factored(terms_of(c));
        if k == 0 {
            out.extend(summands);
            continue;
        }
        let (negative, coef) = split_sign(sum(summands));
        out.push(Summand {
            negative,
            body: product(vec![coef, var_pow(&Var::S, k)]),
            key: TermKey::of(&Monomial::var_pow(Var::S, k)),
        });
    }
    out
}

fn terms_of(p: &Poly) -> Terms {
    p.terms().map(|(m, c)| (c.clone(), m.clone())).collect()
}

fn to_poly(terms: &Terms) -> Poly {
    let mut p = Poly::zero();
    for (c, m) in terms {
        p.add_term(m.clone(), c.clone());
    }
    p
}

/// Chave de uma parcela agrupada: a do seu termo que viria primeiro.
fn min_key(terms: &Terms) -> TermKey {
    terms
        .iter()
        .map(|(_, m)| TermKey::of(m))
        .min()
        .expect("ao menos um termo")
}

/// Soma de uma parcela só com sinal negativo vira (true, corpo), para o
/// sinal subir até a soma de fora em vez de aparecer como (−X) num produto.
fn split_sign(p: Pretty) -> (bool, Pretty) {
    match p {
        Pretty::Sum(mut ts) if ts.len() == 1 && ts[0].0 => (true, ts.pop().expect("uma").1),
        other => (false, other),
    }
}

/// Conteúdo numérico positivo: mdc(numeradores)/mmc(denominadores).
fn content(terms: &Terms) -> BigRational {
    to_poly(terms).numeric_content()
}

fn factored(terms: Terms) -> Vec<Summand> {
    if terms.len() <= 1 || terms.len() > MAX_FACTOR_TERMS {
        return terms.iter().map(|(c, m)| term(c, m)).collect();
    }
    let key = min_key(&terms);

    // 1. Fator comum (monômio e/ou inteiro; o sinal vai junto se todos
    //    os termos forem negativos).
    let common = terms
        .iter()
        .skip(1)
        .fold(terms[0].1.clone(), |acc, (_, m)| acc.gcd(m));
    let c = content(&terms);
    let take_c = !c.is_one() && c.is_integer();
    if !common.is_one() || take_c {
        let all_negative = terms.iter().all(|(q, _)| q.is_negative());
        let mut k = if take_c { c } else { BigRational::one() };
        if all_negative {
            k = -k;
        }
        let rest: Terms = terms
            .iter()
            .map(|(q, m)| (q / &k, m.div(&common).expect("fator comum divide")))
            .collect();
        let (inner_negative, inner) = split_sign(sum(factored(rest)));
        let mut factors = vec![Pretty::Num(k.abs())];
        factors.extend(common.factors().map(|(v, e)| var_pow(v, e)));
        factors.push(inner);
        return vec![Summand {
            negative: all_negative != inner_negative,
            body: product(factors),
            key,
        }];
    }

    // Ainda tem s: organiza por potências de s, fatorando cada coeficiente.
    if terms.iter().any(|(_, m)| m.degree_in(&Var::S) > 0) {
        return by_powers_of_s(&to_poly(&terms));
    }

    // 2/3. Candidatos K: MDC de cada par de termos, se tiver símbolo.
    let mut candidates: Vec<(Monomial, Vec<usize>)> = Vec::new();
    for i in 0..terms.len() {
        for j in i + 1..terms.len() {
            let k = terms[i].1.gcd(&terms[j].1);
            let has_symbol = k.factors().any(|(v, _)| matches!(v, Var::Sym(_)));
            if !has_symbol || candidates.iter().any(|(c, _)| *c == k) {
                continue;
            }
            let members: Vec<usize> = (0..terms.len())
                .filter(|&t| terms[t].1.divisible_by(&k))
                .collect();
            candidates.push((k, members));
        }
    }
    // Maior grau primeiro; empate: mais termos; empate: ordem monomial.
    candidates.sort_by(|(ka, ma), (kb, mb)| {
        kb.total_degree()
            .cmp(&ka.total_degree())
            .then(mb.len().cmp(&ma.len()))
            .then(ka.cmp(kb))
    });
    let inside = |k: &Monomial, members: &[usize]| -> Terms {
        members
            .iter()
            .map(|&t| (terms[t].0.clone(), terms[t].1.div(k).expect("K divide")))
            .collect()
    };

    // 2. Produto de somas.
    let whole = to_poly(&terms);
    for (k, members) in candidates.iter().take(MAX_PRODUCT_TRIES) {
        let q = to_poly(&inside(k, members));
        if q.num_terms() < 2 {
            continue;
        }
        if let Some(s) = whole.exact_div(&q) {
            if s.num_terms() < 2 {
                continue;
            }
            let (na, a) = split_sign(symbolic(&q));
            let (nb, b) = split_sign(symbolic(&s));
            return vec![Summand {
                negative: na != nb,
                body: product(vec![a, b]),
                key,
            }];
        }
    }

    // 3. Agrupamento pelo melhor K.
    if let Some((k, members)) = candidates.first() {
        let group_terms: Terms = members.iter().map(|&t| terms[t].clone()).collect();
        let (negative, inner) = split_sign(sum(factored(inside(k, members))));
        let mut factors: Vec<Pretty> = k.factors().map(|(v, e)| var_pow(v, e)).collect();
        factors.push(inner);
        let group = Summand {
            negative,
            body: product(factors),
            key: min_key(&group_terms),
        };
        let rest: Terms = (0..terms.len())
            .filter(|t| !members.contains(t))
            .map(|t| terms[t].clone())
            .collect();
        let mut out = factored(rest);
        out.push(group);
        return out;
    }

    terms.iter().map(|(c, m)| term(c, m)).collect()
}
