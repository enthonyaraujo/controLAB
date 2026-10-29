//! Máximo divisor comum de polinômios.
//!
//! O MDC é o que permite simplificar frações: (s+1)/((s+1)(s+2)) só vira
//! 1/(s+2) se descobrirmos que s+1 é fator comum. Polinômios formam um
//! domínio de fatoração única (como os inteiros), então o MDC existe e é
//! único a menos de uma constante multiplicativa.
//!
//! Três algoritmos:
//!
//! * **Só `s`** (Q[s]): Q é um corpo, então vale o Euclides clássico,
//!   igualzinho ao dos inteiros: mdc(a, b) = mdc(b, a mod b).
//!
//! * **Com símbolos, caminho rápido: MDC heurístico** (GCDHEU, de Char,
//!   Geddes e Gonnet). Ideia: se trocarmos uma variável por um inteiro ξ
//!   bem grande, o polinômio vira "um número escrito na base ξ" — cada
//!   coeficiente é um dígito. Calculamos o MDC dessas imagens (com uma
//!   variável a menos, recursivamente, até sobrar MDC de inteiros) e
//!   reconstruímos o polinômio lendo os dígitos de volta. Como a
//!   reconstrução pode falhar, o candidato é sempre conferido por divisão
//!   exata; se não passar, tentamos outro ξ.
//!
//! * **Com símbolos, reserva: PRS primitiva.** Olhamos o polinômio como
//!   polinômio numa variável `v` com coeficientes nas outras e usamos o
//!   *lema de Gauss*:
//!
//!   ```text
//!   f = cont(f) · pp(f)
//!   mdc(f, g) = mdc(cont f, cont g) · mdc(pp f, pp g)
//!   ```
//!
//!   onde `cont` (conteúdo) é o MDC dos coeficientes em `v` e `pp` (parte
//!   primitiva) é f/cont(f). Nas partes primitivas roda o Euclides com
//!   pseudo-divisão. É sempre correto, mas com várias variáveis os
//!   coeficientes intermediários crescem exponencialmente; por isso só é
//!   usado se a heurística desistir.

use std::collections::BTreeSet;

use num_bigint::BigInt;
use num_integer::Integer;
use num_rational::BigRational;
use num_traits::{One, Signed, Zero};

use super::cancel::cancelled;
use super::monomial::Monomial;
use super::poly::Poly;
use super::symbol::Var;

/// MDC de `a` e `b`, normalizado com [`Poly::normalize_unit`]
/// (coeficientes inteiros primitivos, coeficiente líder positivo).
/// Convenção: mdc(0, b) = b e mdc(0, 0) = 0.
pub fn gcd(a: &Poly, b: &Poly) -> Poly {
    let g = if a.is_univariate_s() && b.is_univariate_s() {
        gcd_univariate(a, b)
    } else {
        gcd_multivariate(a, b)
    };
    g.normalize_unit()
}

// ─── Euclides em uma variável ─────────────────────────────────────────

/// Coeficientes densos de um polinômio que só tem a variável `v`:
/// `out[k]` é o coeficiente de v^k.
fn to_dense(p: &Poly, v: &Var) -> Vec<BigRational> {
    let mut out = vec![BigRational::zero(); p.degree_in(v) as usize + 1];
    for (m, c) in p.terms() {
        out[m.degree_in(v) as usize] = c.clone();
    }
    trim(&mut out);
    out
}

fn from_dense(coeffs: &[BigRational], v: &Var) -> Poly {
    let mut p = Poly::zero();
    for (k, c) in coeffs.iter().enumerate() {
        p.add_term(Monomial::var_pow(v.clone(), k as u32), c.clone());
    }
    p
}

/// Remove zeros do topo (vetor vazio representa o polinômio zero).
fn trim(v: &mut Vec<BigRational>) {
    while v.last().is_some_and(Zero::is_zero) {
        v.pop();
    }
}

/// Resto da divisão longa de `a` por `b` (b ≠ 0), como na escola:
/// elimina o termo de maior grau de `a` com um múltiplo de `b` até o grau
/// ficar menor que o de `b`.
fn rem_dense(mut a: Vec<BigRational>, b: &[BigRational]) -> Vec<BigRational> {
    let db = b.len() - 1;
    let lb = &b[db];
    while a.len() > db {
        let da = a.len() - 1;
        let factor = &a[da] / lb;
        let shift = da - db;
        for (k, bk) in b.iter().enumerate() {
            a[shift + k] -= &factor * bk;
        }
        // O termo de grau `da` zerou por construção.
        a.pop();
        trim(&mut a);
    }
    a
}

/// Euclides: mdc(x, y) = mdc(y, x mod y), até o resto zerar.
fn euclid_dense(mut x: Vec<BigRational>, mut y: Vec<BigRational>) -> Vec<BigRational> {
    while !y.is_empty() {
        let r = rem_dense(x, &y);
        x = y;
        y = r;
    }
    x
}

fn gcd_univariate(a: &Poly, b: &Poly) -> Poly {
    let g = euclid_dense(to_dense(a, &Var::S), to_dense(b, &Var::S));
    from_dense(&g, &Var::S)
}

// ─── Caso multivariável ───────────────────────────────────────────────

/// Despacho do caso com símbolos: separa o fator monomial comum, tenta a
/// heurística e, se ela desistir, usa a PRS.
fn gcd_multivariate(a: &Poly, b: &Poly) -> Poly {
    if a.is_zero() {
        return b.clone();
    }
    if b.is_zero() {
        return a.clone();
    }
    // Cálculo cancelado: "1" deixa a fração correta, só sem simplificar;
    // o resultado vai ser descartado (veja `cancel.rs`).
    if cancelled() {
        return Poly::one();
    }
    // Variáveis isoladas (s, G1...) são os únicos fatores monomiais
    // irredutíveis, então pela fatoração única:
    //   mdc(m₁·f', m₂·g') = mdc(m₁, m₂) · mdc(f', g')
    // quando f' e g' não têm mais fator monomial. Tirar isso antes deixa
    // os polinômios menores para o resto do algoritmo.
    let (ma, mb) = (a.monomial_content(), b.monomial_content());
    let m = Poly::term(ma.gcd(&mb), BigRational::one());
    let a = a
        .div_monomial(&ma)
        .expect("conteúdo monomial divide")
        .normalize_unit();
    let b = b
        .div_monomial(&mb)
        .expect("conteúdo monomial divide")
        .normalize_unit();
    if coprime_by_evaluation(&a, &b) {
        return m;
    }
    let g = match heu_gcd(&a, &b) {
        Some((g, _, _)) => g,
        None => gcd_prs(&a, &b),
    };
    &m * &g
}

// ─── Teste de coprimalidade por avaliação ─────────────────────────────

/// Valores usados nas avaliações: primos pequenos, para as imagens
/// ficarem pequenas e "genéricas".
const PRIMES: [i64; 16] = [2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47, 53];

/// Prova (rapidamente) que mdc(a, b) = 1, ou responde "não sei".
///
/// Para cada variável v, troca as **outras** por números e calcula o MDC
/// das imagens, que são polinômios só em v (Euclides, barato). Se g
/// divide a e b, então g(ponto) divide as imagens; e se os coeficientes
/// líderes em v não se anularam no ponto, g(ponto) tem o mesmo grau em v
/// que g. Logo: imagem com MDC de grau 0 ⇒ o MDC verdadeiro não contém v.
/// Se isso vale para toda variável, o MDC é constante.
///
/// O caso mais comum em diagramas (frações que já não têm o que
/// cancelar) sai daqui sem tocar na heurística.
fn coprime_by_evaluation(a: &Poly, b: &Poly) -> bool {
    let vars: BTreeSet<Var> = a.vars().into_iter().chain(b.vars()).collect();
    'var: for v in &vars {
        for attempt in 0..3 {
            if cancelled() {
                return false;
            }
            let (mut ea, mut eb) = (a.clone(), b.clone());
            for (i, w) in vars.iter().filter(|w| *w != v).enumerate() {
                let x = BigInt::from(PRIMES[(i + attempt * 5) % PRIMES.len()]);
                ea = eval_at(&ea, w, &x);
                eb = eval_at(&eb, w, &x);
            }
            // Coeficiente líder anulado (grau caiu) ou imagem nula: ponto
            // azarado, tenta outro.
            let degree_kept =
                ea.degree_in(v) == a.degree_in(v) && eb.degree_in(v) == b.degree_in(v);
            if !degree_kept || ea.is_zero() || eb.is_zero() {
                continue;
            }
            if euclid_dense(to_dense(&ea, v), to_dense(&eb, v)).len() == 1 {
                continue 'var;
            }
            // As imagens têm fator comum: pode ser verdadeiro ou azar.
            return false;
        }
        return false;
    }
    true
}

// ─── MDC heurístico (GCDHEU) ──────────────────────────────────────────

/// Acima disso (em bits) os inteiros da heurística ficam caros demais;
/// ela desiste e a PRS assume.
const HEU_MAX_BITS: u64 = 20_000;

/// Quantas escolhas de ξ testar antes de desistir.
const HEU_TRIES: usize = 6;

fn int(n: &BigInt) -> BigRational {
    BigRational::from_integer(n.clone())
}

/// MDC dos coeficientes (supostos inteiros).
fn int_content(p: &Poly) -> BigInt {
    p.terms().fold(BigInt::zero(), |g, (_, c)| g.gcd(c.numer()))
}

/// Maior coeficiente em valor absoluto.
fn max_norm(p: &Poly) -> BigInt {
    p.terms()
        .map(|(_, c)| c.numer().abs())
        .max()
        .unwrap_or_default()
}

/// Divisão exata em Z[...] (ver [`Poly::exact_div_integer`]).
fn exact_div_z(f: &Poly, d: &Poly) -> Option<Poly> {
    f.exact_div_integer(d)
}

/// Troca a variável `v` pelo inteiro `x`.
fn eval_at(p: &Poly, v: &Var, x: &BigInt) -> Poly {
    let mut out = Poly::zero();
    for (m, c) in p.terms() {
        let (e, rest) = m.split_off(v);
        out.add_term(rest, c * int(&num_traits::pow(x.clone(), e as usize)));
    }
    out
}

/// Resto simétrico: o representante de `c mod x` em (−x/2, x/2].
fn symmetric_mod(c: &BigInt, x: &BigInt) -> BigInt {
    let r = c.mod_floor(x);
    if &r * 2 > *x { r - x } else { r }
}

/// Reconstrói o polinômio em `v` a partir da imagem `h = H(ξ)`.
///
/// Escrevendo H = Σ hᵢ·vⁱ com coeficientes pequenos (|hᵢ| < ξ/2), a
/// imagem é h = Σ hᵢ·ξⁱ: os hᵢ são os "dígitos" de h na base ξ.
/// Igual a converter um número para base 10, com resto simétrico para
/// aceitar dígitos negativos:
///
/// ```text
/// h₀ = h mod ξ,  h ← (h − h₀)/ξ,  h₁ = h mod ξ, ...
/// ```
///
/// Aqui cada "dígito" é ele mesmo um polinômio nas outras variáveis:
/// aplicamos o resto simétrico coeficiente a coeficiente.
fn interpolate(mut h: Poly, v: &Var, x: &BigInt) -> Poly {
    let mut out = Poly::zero();
    let xr = int(x);
    let mut i = 0;
    while !h.is_zero() {
        let mut digit = Poly::zero();
        for (m, c) in h.terms() {
            digit.add_term(m.clone(), int(&symmetric_mod(c.numer(), x)));
        }
        h = (&h - &digit).scale(&xr.recip());
        let vi = Monomial::var_pow(v.clone(), i);
        for (m, c) in digit.terms() {
            out.add_term(m.mul(&vi), c.clone());
        }
        i += 1;
    }
    if out.leading().is_some_and(|(_, c)| c.is_negative()) {
        out = -out;
    }
    out
}

/// GCDHEU para polinômios com coeficientes inteiros, ambos não nulos.
/// Devolve `(h, f/h, g/h)` ou `None` se não teve sorte com nenhum ξ.
fn heu_gcd(f: &Poly, g: &Poly) -> Option<(Poly, Poly, Poly)> {
    // Tira o MDC inteiro dos coeficientes: mdc(c·f', c·g') = c·mdc(f', g').
    let c = int_content(f).gcd(&int_content(g));
    let cr = int(&c);
    let f = f.scale(&cr.recip());
    let g = g.scale(&cr.recip());

    let vars: BTreeSet<Var> = f.vars().into_iter().chain(g.vars()).collect();
    let Some(v) = vars.last().cloned() else {
        // Caso base: dois inteiros.
        let (a, b) = (f.constant_term().cloned(), g.constant_term().cloned());
        let (a, b) = (a.unwrap_or_default(), b.unwrap_or_default());
        let h = a.numer().gcd(b.numer());
        let hr = int(&h);
        return Some((
            Poly::constant(&hr * &cr),
            Poly::constant(a / &hr),
            Poly::constant(b / &hr),
        ));
    };
    if f.is_constant() || g.is_constant() {
        // Um número só tem divisores numéricos: o MDC é o MDC inteiro
        // dele com o conteúdo do outro.
        let h = int(&int_content(&f).gcd(&int_content(&g)));
        return Some((
            Poly::constant(&h * &cr),
            f.scale(&h.recip()),
            g.scale(&h.recip()),
        ));
    }

    // Escolha de ξ (mesmas constantes do SymPy): grande o bastante para
    // que os "dígitos" não se misturem, mas sem exagero.
    let (fn_, gn) = (max_norm(&f), max_norm(&g));
    let lc = |p: &Poly| {
        p.leading()
            .map(|(_, c)| c.numer().abs())
            .unwrap_or_else(BigInt::one)
    };
    let b: BigInt = 2 * fn_.clone().min(gn.clone()) + 29;
    let mut x = b
        .clone()
        .min(99 * b.sqrt())
        .max(2 * (&fn_ / lc(&f)).min(&gn / lc(&g)) + 4);

    for _ in 0..HEU_TRIES {
        if cancelled() {
            return None;
        }
        let (ff, gg) = (eval_at(&f, &v, &x), eval_at(&g, &v, &x));
        // Cada variável avaliada multiplica o tamanho dos inteiros pelo
        // grau; com muitas variáveis isso explode.
        if max_norm(&ff).bits().max(max_norm(&gg).bits()) > HEU_MAX_BITS {
            return None;
        }
        if !ff.is_zero()
            && !gg.is_zero()
            && let Some((h, cff, cfg)) = heu_gcd(&ff, &gg)
        {
            // Três chances de reconstrução: pelo MDC ou por um dos
            // cofatores. Qualquer uma que divida exatamente serve.
            let h = interpolate(h, &v, &x);
            let h = h.scale(&int(&int_content(&h)).recip());
            if let (Some(a), Some(b)) = (exact_div_z(&f, &h), exact_div_z(&g, &h)) {
                return Some((h.scale(&cr), a, b));
            }
            let cff = interpolate(cff, &v, &x);
            if let Some(h) = exact_div_z(&f, &cff)
                && let Some(b) = exact_div_z(&g, &h)
            {
                return Some((h.scale(&cr), cff, b));
            }
            let cfg = interpolate(cfg, &v, &x);
            if let Some(h) = exact_div_z(&g, &cfg)
                && let Some(a) = exact_div_z(&f, &h)
            {
                return Some((h.scale(&cr), a, cfg));
            }
        }
        // Próximo ξ: cresce por um fator "irregular" para não cair de novo
        // num valor azarado.
        x = 73794 * &x * x.sqrt().sqrt() / 27011;
    }
    None
}

// ─── PRS primitiva (reserva) ──────────────────────────────────────────

fn gcd_prs(a: &Poly, b: &Poly) -> Poly {
    if a.is_zero() {
        return b.clone();
    }
    if b.is_zero() {
        return a.clone();
    }
    if a.is_constant() || b.is_constant() {
        return Poly::one();
    }
    // Se um deles é um monômio (c·G1·H2...), seus divisores também são
    // monômios: basta o MDC com o "monômio comum" do outro.
    if a.num_terms() == 1 || b.num_terms() == 1 {
        let g = a.monomial_content().gcd(&b.monomial_content());
        return Poly::term(g, BigRational::from_integer(1.into()));
    }
    // Um fator comum não constante precisa ter alguma variável presente
    // nos dois. Sem variável em comum, o MDC é 1.
    let va = a.vars();
    let Some(v) = b.vars().into_iter().find(|v| va.contains(v)) else {
        return Poly::one();
    };

    // Lema de Gauss: separa conteúdo (em v) e parte primitiva.
    let ca = content(a, &v);
    let cb = content(b, &v);
    let c = gcd_multivariate(&ca, &cb);
    let mut p = a.exact_div(&ca).expect("o conteúdo divide o polinômio");
    let mut q = b.exact_div(&cb).expect("o conteúdo divide o polinômio");
    if p.degree_in(&v) < q.degree_in(&v) {
        std::mem::swap(&mut p, &mut q);
    }

    // Sequência de restos primitiva.
    let g = loop {
        if cancelled() {
            break Poly::one();
        }
        let r = pseudo_rem(&p, &q, &v);
        if r.is_zero() {
            break q;
        }
        if r.degree_in(&v) == 0 {
            // Resto sem v, não nulo: as partes primitivas são coprimas.
            break Poly::one();
        }
        p = q;
        q = primitive_part(&r, &v);
    };
    &c * &primitive_part(&g, &v)
}

/// Conteúdo de `p` em `v`: MDC de todos os coeficientes de `p` vistos
/// como polinômio em `v`. Ex.: (a·s + a·b) em s → mdc(a, a·b) = a.
fn content(p: &Poly, v: &Var) -> Poly {
    let mut g = Poly::zero();
    for coef in p.coeffs_in(v) {
        if coef.is_zero() {
            continue;
        }
        g = gcd_multivariate(&g, &coef);
        if g.is_constant() {
            return Poly::one();
        }
    }
    g.normalize_unit()
}

/// Parte primitiva em `v`, com a constante numérica também normalizada.
fn primitive_part(p: &Poly, v: &Var) -> Poly {
    p.exact_div(&content(p, v))
        .expect("o conteúdo divide o polinômio")
        .normalize_unit()
}

/// Pseudo-resto de `p` por `q` em `v`.
///
/// Na divisão comum precisaríamos dividir pelo coeficiente líder de q,
/// lc(q), que aqui é um polinômio (pode ser `a`, por exemplo). Para fugir
/// disso, a cada passo multiplicamos o resto por lc(q) antes de subtrair:
///
/// ```text
/// r ← lc(q)·r − lc(r)·v^(deg r − deg q)·q
/// ```
///
/// O termo de maior grau em v se cancela, o grau cai, e só usamos
/// produtos e subtrações. O resultado difere do resto "verdadeiro" por um
/// fator que não afeta o MDC das partes primitivas.
fn pseudo_rem(p: &Poly, q: &Poly, v: &Var) -> Poly {
    let dq = q.degree_in(v);
    let lcq = q.leading_coeff_in(v);
    let mut r = p.clone();
    while !r.is_zero() && r.degree_in(v) >= dq {
        let dr = r.degree_in(v);
        let lcr = r.leading_coeff_in(v);
        let shift = Poly::var_pow(v.clone(), dr - dq);
        r = &(&r * &lcq) - &(&(&lcr * &shift) * q);
    }
    r
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

    #[test]
    fn euclides_so_s() {
        // (s+1)(s+2) e (s+1)(s+3) → s+1
        let a = &(&s() + &k(1)) * &(&s() + &k(2));
        let b = &(&s() + &k(1)) * &(&s() + &k(3));
        assert_eq!(gcd(&a, &b), &s() + &k(1));
        // coprimos
        assert_eq!(gcd(&(&s() + &k(1)), &(&s() + &k(2))), k(1));
        // escala: mdc(2s+2, 4) = 1 (constantes são unidades em Q[s])
        assert_eq!(gcd(&(&k(2) * &(&s() + &k(1))), &k(4)), k(1));
    }

    #[test]
    fn multivariavel_fator_comum() {
        let sa = &s() + &sym("a");
        let a = &sa * &(&s() + &sym("b"));
        let b = &sa * &(&(&s() * &sym("K")) + &k(1));
        assert_eq!(gcd(&a, &b), sa);
    }

    #[test]
    fn multivariavel_simbolos_puros() {
        // G1·G4·(G2+G3) e (G2+G3)·H2 → G2+G3
        let g23 = &sym("G2") + &sym("G3");
        let a = &(&sym("G1") * &sym("G4")) * &g23;
        let b = &g23 * &sym("H2");
        assert_eq!(gcd(&a, &b), g23);
        // Monômio contra polinômio: mdc(G1·H1, G1·H1 + G1) = G1
        let m = &sym("G1") * &sym("H1");
        assert_eq!(gcd(&m, &(&m + &sym("G1"))), sym("G1"));
    }

    #[test]
    fn multivariavel_com_conteudo() {
        // a·(s+1)·(s+b) e a·b·(s+1) → a·(s+1)
        let a = &(&sym("a") * &(&s() + &k(1))) * &(&s() + &sym("b"));
        let b = &(&sym("a") * &sym("b")) * &(&s() + &k(1));
        assert_eq!(gcd(&a, &b), &(&sym("a") * &s()) + &sym("a"));
    }

    #[test]
    fn coprimos_multivariaveis() {
        let a = &(&s() * &s()) + &sym("a");
        let b = &s() + &sym("a");
        assert_eq!(gcd(&a, &b), k(1));
    }

    /// Pares (a, b, mdc esperado) usados para conferir os dois algoritmos.
    fn casos() -> Vec<(Poly, Poly, Poly)> {
        let sa = &s() + &sym("a");
        let g23 = &sym("G2") + &sym("G3");
        let fator = &(&sym("G1") * &sym("H1")) - &(&k(2) * &s());
        vec![
            (
                &sa * &(&s() + &sym("b")),
                &sa * &(&s() * &sym("K") + &k(1)),
                sa.clone(),
            ),
            (&g23 * &sym("G1"), &g23 * &(&sym("H2") + &k(3)), g23.clone()),
            (
                &fator * &fator,
                &fator * &(&s() + &sym("G1")),
                fator.clone(),
            ),
            (&(&s() * &s()) + &sym("a"), sa, k(1)),
        ]
    }

    #[test]
    fn heuristica_acha_o_mdc_sem_desistir() {
        for (a, b, esperado) in casos() {
            let (g, ca, cb) = heu_gcd(&a, &b).expect("heurística não deveria falhar aqui");
            assert_eq!(g.normalize_unit(), esperado.normalize_unit());
            // Os cofatores devolvidos fecham a conta: a = g·ca, b = g·cb.
            assert_eq!(&g * &ca, a);
            assert_eq!(&g * &cb, b);
        }
    }

    #[test]
    fn prs_de_reserva_concorda() {
        for (a, b, esperado) in casos() {
            assert_eq!(gcd_prs(&a, &b).normalize_unit(), esperado.normalize_unit());
        }
    }

    #[test]
    fn regressao_prs_explosiva() {
        // Este par fazia a PRS pura levar minutos (coeficientes de 10¹³
        // em recursões aninhadas). Com a heurística sai na hora.
        let a = &s() - &sym("G1");
        let b = &(&(&s().pow(2) * &sym("G1").pow(2)) * &sym("a")) + &(&sym("G1") * &sym("H1"));
        let b = &b + &k(1);
        let num = &(-&b) + &(&(&(&s() * &sym("H1")) - &sym("G1")) * &a);
        let den = &a * &b;
        assert_eq!(gcd(&num, &den), k(1));
    }
}
