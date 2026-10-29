//! Monômios: produtos de variáveis elevadas a expoentes naturais,
//! por exemplo `s²·G1·H2`. O coeficiente numérico fica no polinômio.

use std::cmp::Ordering;
use std::fmt;

use super::symbol::Var;

/// Produto de potências de variáveis. Invariante: a lista está ordenada
/// por variável, sem repetição, e todos os expoentes são > 0.
/// A lista vazia representa o monômio 1.
#[derive(Clone, PartialEq, Eq, Hash, Default)]
pub struct Monomial(Vec<(Var, u32)>);

impl Monomial {
    /// O monômio 1.
    pub fn one() -> Self {
        Monomial(Vec::new())
    }

    /// `v^exp` (com `exp = 0` dá 1).
    pub fn var_pow(v: Var, exp: u32) -> Self {
        if exp == 0 {
            Monomial::one()
        } else {
            Monomial(vec![(v, exp)])
        }
    }

    pub fn is_one(&self) -> bool {
        self.0.is_empty()
    }

    /// Pares (variável, expoente) em ordem crescente de variável.
    pub fn factors(&self) -> impl Iterator<Item = (&Var, u32)> {
        self.0.iter().map(|(v, e)| (v, *e))
    }

    /// Expoente de `v` (0 se não aparece).
    pub fn degree_in(&self, v: &Var) -> u32 {
        self.0.iter().find(|(w, _)| w == v).map_or(0, |(_, e)| *e)
    }

    /// Soma de todos os expoentes.
    pub fn total_degree(&self) -> u32 {
        self.0.iter().map(|(_, e)| e).sum()
    }

    /// Junta duas listas ordenadas aplicando `op` aos expoentes da mesma
    /// variável (ausente conta como 0). Termos com expoente 0 somem.
    fn merge(&self, other: &Self, op: impl Fn(u32, u32) -> u32) -> Self {
        let (a, b) = (&self.0, &other.0);
        let (mut i, mut j) = (0, 0);
        let mut out = Vec::with_capacity(a.len() + b.len());
        while i < a.len() || j < b.len() {
            let ord = match (a.get(i), b.get(j)) {
                (Some((va, _)), Some((vb, _))) => va.cmp(vb),
                (Some(_), None) => Ordering::Less,
                _ => Ordering::Greater,
            };
            let (v, e) = match ord {
                Ordering::Less => {
                    i += 1;
                    (a[i - 1].0.clone(), op(a[i - 1].1, 0))
                }
                Ordering::Greater => {
                    j += 1;
                    (b[j - 1].0.clone(), op(0, b[j - 1].1))
                }
                Ordering::Equal => {
                    i += 1;
                    j += 1;
                    (a[i - 1].0.clone(), op(a[i - 1].1, b[j - 1].1))
                }
            };
            if e > 0 {
                out.push((v, e));
            }
        }
        Monomial(out)
    }

    /// Produto: soma os expoentes.
    pub fn mul(&self, other: &Self) -> Self {
        self.merge(other, |x, y| x + y)
    }

    /// `true` se `other` divide `self` (todo expoente de `other` ≤ o de `self`).
    pub fn divisible_by(&self, other: &Self) -> bool {
        other.0.iter().all(|(v, e)| self.degree_in(v) >= *e)
    }

    /// Divisão exata; `None` se `other` não divide `self`.
    pub fn div(&self, other: &Self) -> Option<Self> {
        if self.divisible_by(other) {
            Some(self.merge(other, |x, y| x - y))
        } else {
            None
        }
    }

    /// MDC de monômios: menor expoente de cada variável.
    /// Ex.: mdc(s²·G1, s·G1·H1) = s·G1.
    pub fn gcd(&self, other: &Self) -> Self {
        self.merge(other, u32::min)
    }

    /// Substitui `v` por nada (remove a variável), devolvendo o expoente que
    /// ela tinha. Útil para enxergar o polinômio como polinômio em `v`.
    pub fn split_off(&self, v: &Var) -> (u32, Self) {
        let mut rest = self.0.clone();
        let exp = match rest.iter().position(|(w, _)| w == v) {
            Some(idx) => rest.remove(idx).1,
            None => 0,
        };
        (exp, Monomial(rest))
    }
}

/// Ordem monomial usada em todo o crate para escolher o "termo líder".
///
/// Compara, nesta ordem:
/// 1. grau em `s`;
/// 2. grau total;
/// 3. lexicográfica: na primeira variável (em ordem crescente) em que os
///    expoentes diferem, ganha quem tem o expoente maior.
///
/// Isso é uma *ordem monomial* de verdade (compatível com o produto:
/// a < b ⇒ a·c < b·c, e todo conjunto tem mínimo). Essa propriedade
/// garante que o termo líder de f·g é o produto dos termos líderes, e é
/// nela que se apoia a divisão exata de polinômios em `poly.rs`.
impl Ord for Monomial {
    fn cmp(&self, other: &Self) -> Ordering {
        let deg_s = self.degree_in(&Var::S).cmp(&other.degree_in(&Var::S));
        let total = self.total_degree().cmp(&other.total_degree());
        deg_s.then(total).then_with(|| {
            let (a, b) = (&self.0, &other.0);
            let (mut i, mut j) = (0, 0);
            loop {
                match (a.get(i), b.get(j)) {
                    (None, None) => return Ordering::Equal,
                    // Só `self` ainda tem variáveis: na primeira delas o
                    // expoente de `self` é > 0 e o de `other` é 0.
                    (Some(_), None) => return Ordering::Greater,
                    (None, Some(_)) => return Ordering::Less,
                    (Some((va, ea)), Some((vb, eb))) => match va.cmp(vb) {
                        Ordering::Less => return Ordering::Greater,
                        Ordering::Greater => return Ordering::Less,
                        Ordering::Equal => {
                            if ea != eb {
                                return ea.cmp(eb);
                            }
                            i += 1;
                            j += 1;
                        }
                    },
                }
            }
        })
    }
}

impl PartialOrd for Monomial {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl fmt::Display for Monomial {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_one() {
            return f.write_str("1");
        }
        for (k, (v, e)) in self.0.iter().enumerate() {
            if k > 0 {
                f.write_str("*")?;
            }
            if *e == 1 {
                write!(f, "{v}")?;
            } else {
                write!(f, "{v}^{e}")?;
            }
        }
        Ok(())
    }
}

impl fmt::Debug for Monomial {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn m(pairs: &[(&str, u32)]) -> Monomial {
        pairs.iter().fold(Monomial::one(), |acc, (n, e)| {
            acc.mul(&Monomial::var_pow(Var::named(n), *e))
        })
    }

    #[test]
    fn produto_divisao_mdc() {
        let a = m(&[("s", 2), ("G1", 1)]);
        let b = m(&[("s", 1), ("G1", 1), ("H1", 1)]);
        assert_eq!(a.mul(&b), m(&[("s", 3), ("G1", 2), ("H1", 1)]));
        assert_eq!(a.gcd(&b), m(&[("s", 1), ("G1", 1)]));
        assert_eq!(a.div(&m(&[("s", 1)])), Some(m(&[("s", 1), ("G1", 1)])));
        assert_eq!(a.div(&b), None);
    }

    #[test]
    fn ordem_prioriza_grau_em_s() {
        // s vence G1·G2·H3 apesar do grau total menor.
        assert!(m(&[("s", 1)]) > m(&[("G1", 1), ("G2", 1), ("H3", 1)]));
        // Mesmo grau em s: grau total decide.
        assert!(m(&[("G1", 1), ("H1", 1)]) > m(&[("K", 1)]));
        // Mesmo grau total: G1 vem antes, então G1 > G2 na lexicográfica.
        assert!(m(&[("G1", 1)]) > m(&[("G2", 1)]));
        assert!(Monomial::one() < m(&[("a", 1)]));
    }

    #[test]
    fn ordem_compativel_com_produto() {
        let a = m(&[("G1", 1)]);
        let b = m(&[("G2", 1)]);
        let c = m(&[("s", 1), ("G2", 3)]);
        assert_eq!(a.cmp(&b), a.mul(&c).cmp(&b.mul(&c)));
    }
}
