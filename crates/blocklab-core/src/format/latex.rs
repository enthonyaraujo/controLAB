//! LaTeX: `\frac{G_{1} G_{4} \left(G_{2} + G_{3}\right)}{1 - ...}`.

use num_bigint::BigInt;
use num_rational::BigRational;
use num_traits::One;

use super::pretty::Pretty;
use crate::algebra::Var;

pub fn render(p: &Pretty) -> String {
    match p {
        Pretty::Num(q) if q.is_integer() => q.numer().to_string(),
        Pretty::Num(q) => format!("\\frac{{{}}}{{{}}}", q.numer(), q.denom()),
        Pretty::Var(v) => var(v),
        Pretty::Pow(v, e) => format!("{}^{{{e}}}", var(v)),
        Pretty::Product(fs) => {
            // Em modo matemático o espaço não aparece: `\mathrm{Kd} s` sairia
            // "Kds", parecendo um nome só. Ao lado de um nome de várias
            // letras, usamos um espaço fino visível (`\,`).
            let mut out = String::new();
            for (i, f) in fs.iter().enumerate() {
                if i > 0 {
                    out += if is_wide(&fs[i - 1]) || is_wide(f) {
                        "\\, "
                    } else {
                        " "
                    };
                }
                out += &paren_if_sum(f);
            }
            out
        }
        Pretty::Sum(ts) => {
            let mut out = String::new();
            for (i, (negative, t)) in ts.iter().enumerate() {
                out += match (i, negative) {
                    (0, false) => "",
                    (0, true) => "-",
                    (_, false) => " + ",
                    (_, true) => " - ",
                };
                out += &paren_if_sum(t);
            }
            out
        }
    }
}

fn paren_if_sum(p: &Pretty) -> String {
    if matches!(p, Pretty::Sum(_)) {
        format!("\\left({}\\right)", render(p))
    } else {
        render(p)
    }
}

/// `\frac{num}{den}`. Um coeficiente fracionário no numerador desce para
/// o denominador, para não aparecer fração dentro de fração:
/// (1/2)/(T·s + 1) → `\frac{1}{2 \left(T s + 1\right)}`.
pub fn fraction(num: &Pretty, den: Option<&Pretty>) -> String {
    let (negative, body) = match num {
        Pretty::Sum(ts) if ts.len() == 1 && ts[0].0 => (true, &ts[0].1),
        other => (false, other),
    };
    let sign = if negative { "-" } else { "" };
    if let Some((p, q, rest)) = fractional_coefficient(body) {
        let top = render(&product(Pretty::Num(BigRational::from_integer(p)), rest));
        let bottom = match den {
            None => q.to_string(),
            Some(d) => format!("{q} {}", paren_if_sum(d)),
        };
        return format!("{sign}\\frac{{{top}}}{{{bottom}}}");
    }
    // Sinal sempre fora da fração: -\frac{K}{s + 1}, como no caso acima.
    match den {
        None => render(num),
        Some(d) => format!("{sign}\\frac{{{}}}{{{}}}", render(body), render(d)),
    }
}

/// Se a expressão é `(p/q)·resto` com q > 1, devolve (p, q, resto).
fn fractional_coefficient(p: &Pretty) -> Option<(BigInt, BigInt, Vec<Pretty>)> {
    match p {
        Pretty::Num(q) if !q.is_integer() => Some((q.numer().clone(), q.denom().clone(), vec![])),
        Pretty::Product(fs) => match fs.first() {
            Some(Pretty::Num(q)) if !q.is_integer() => {
                Some((q.numer().clone(), q.denom().clone(), fs[1..].to_vec()))
            }
            _ => None,
        },
        _ => None,
    }
}

/// p·resto, sem escrever o 1.
fn product(coef: Pretty, rest: Vec<Pretty>) -> Pretty {
    let coef_is_one = matches!(&coef, Pretty::Num(q) if q.is_one());
    match (coef_is_one, rest.len()) {
        (_, 0) => coef,
        (true, 1) => rest.into_iter().next().expect("um fator"),
        (true, _) => Pretty::Product(rest),
        (false, _) => Pretty::Product(std::iter::once(coef).chain(rest).collect()),
    }
}

fn var(v: &Var) -> String {
    match v {
        Var::S => "s".to_string(),
        Var::Sym(sym) => symbol(sym.name()),
    }
}

/// Separa o nome em base e índice: `H_2` → (H, 2), `G10` → (G, 10),
/// `Kp` → (Kp, —).
fn split_name(name: &str) -> (&str, Option<&str>) {
    match name.split_once('_') {
        Some((b, s)) => (b, Some(s)),
        None => {
            // Dígitos no fim viram índice: G10 → G_{10}.
            let cut = name.trim_end_matches(|c: char| c.is_ascii_digit()).len();
            if cut > 0 && cut < name.len() {
                (&name[..cut], Some(&name[cut..]))
            } else {
                (name, None)
            }
        }
    }
}

/// Símbolo cuja base tem mais de uma letra (`Kp`, `ωc`): sai em
/// `\mathrm{}` e precisa de espaço fino ao lado de outros fatores.
fn is_wide(p: &Pretty) -> bool {
    match p {
        Pretty::Var(Var::Sym(sym)) | Pretty::Pow(Var::Sym(sym), _) => {
            split_name(sym.name()).0.chars().count() > 1
        }
        _ => false,
    }
}

/// Nome de símbolo em LaTeX. Em modo matemático só sai ASCII (o
/// `pdflatex` recusa grego cru); acentos ficam em `\mbox{}`:
/// `G1` → `G_{1}`, `H_2` → `H_{2}`, `Kp` → `\mathrm{Kp}`,
/// `ω_n` → `\omega_{n}`, `ωc` → `\mathrm{\omega c}`,
/// `Kç` → `\mathrm{K\mbox{ç}}`, `G_1_2` → `G_{1\_2}`.
pub fn symbol(name: &str) -> String {
    let (base, sub) = split_name(name);
    let base_tex = ascii_run(base);
    let base_tex = if base.chars().count() == 1 {
        base_tex
    } else {
        format!("\\mathrm{{{base_tex}}}")
    };
    match sub {
        Some(s) => format!("{base_tex}_{{{}}}", ascii_run(s)),
        None => base_tex,
    }
}

/// Converte um trecho do nome caractere a caractere: ASCII fica igual,
/// grego vira comando (`\omega`), `_` vira `\_` e o resto (ç, ã...) vai
/// em `\mbox{}` — modo texto, onde acentos funcionam.
fn ascii_run(s: &str) -> String {
    let mut out = String::new();
    // Um comando terminado em letra (`\omega`) seguido de letra/dígito
    // precisa de espaço: `\omega c`, não `\omegac`.
    let mut after_command = false;
    for c in s.chars() {
        let piece = char_tex(c);
        if after_command && piece.starts_with(|x: char| x.is_ascii_alphanumeric()) {
            out.push(' ');
        }
        after_command =
            piece.starts_with('\\') && piece.ends_with(|x: char| x.is_ascii_alphabetic());
        out += &piece;
    }
    out
}

fn char_tex(c: char) -> String {
    if c == '_' {
        return "\\_".to_string();
    }
    if c.is_ascii() {
        return c.to_string();
    }
    let name = match c {
        'α' => "alpha",
        'β' => "beta",
        'γ' => "gamma",
        'δ' => "delta",
        'ε' => "epsilon",
        'ζ' => "zeta",
        'η' => "eta",
        'θ' => "theta",
        'ι' => "iota",
        'κ' => "kappa",
        'λ' => "lambda",
        'μ' => "mu",
        'ν' => "nu",
        'ξ' => "xi",
        'π' => "pi",
        'ρ' => "rho",
        'σ' | 'ς' => "sigma",
        'τ' => "tau",
        'υ' => "upsilon",
        'φ' => "phi",
        'χ' => "chi",
        'ψ' => "psi",
        'ω' => "omega",
        'Γ' => "Gamma",
        'Δ' => "Delta",
        'Θ' => "Theta",
        'Λ' => "Lambda",
        'Ξ' => "Xi",
        'Π' => "Pi",
        'Σ' => "Sigma",
        'Υ' => "Upsilon",
        'Φ' => "Phi",
        'Ψ' => "Psi",
        'Ω' => "Omega",
        // Letras gregas sem comando próprio no LaTeX: iguais às latinas.
        'ο' => return "o".to_string(),
        'Α' => return "A".to_string(),
        'Β' => return "B".to_string(),
        'Ε' => return "E".to_string(),
        'Ζ' => return "Z".to_string(),
        'Η' => return "H".to_string(),
        'Ι' => return "I".to_string(),
        'Κ' => return "K".to_string(),
        'Μ' => return "M".to_string(),
        'Ν' => return "N".to_string(),
        'Ο' => return "O".to_string(),
        'Ρ' => return "P".to_string(),
        'Τ' => return "T".to_string(),
        'Χ' => return "X".to_string(),
        _ => return format!("\\mbox{{{c}}}"),
    };
    format!("\\{name}")
}
