//! Texto em uma linha. Duas variantes:
//!
//! * Unicode, para a tela: `G1·G4·(G2+G3) / (1 − G1·G4·H1)`;
//! * ASCII, para colar em MATLAB/Python: `G1*G4*(G2+G3) / (1 - G1*G4*H1)`.
//!
//! As duas são lidas de volta pelo parser do BlockLab e dão exatamente a
//! mesma função (garantido por teste de propriedade).

use super::pretty::Pretty;

#[derive(Clone, Copy)]
pub struct Style {
    pub times: &'static str,
    pub minus: &'static str,
}

pub const UNICODE: Style = Style {
    times: "·",
    minus: "−",
};

pub const ASCII: Style = Style {
    times: "*",
    minus: "-",
};

/// `nested`: dentro de parênteses, as somas ficam sem espaços (G2+G3),
/// para o nível de fora se destacar.
pub fn render(p: &Pretty, style: Style, nested: bool) -> String {
    match p {
        Pretty::Num(q) if q.is_integer() => q.numer().to_string(),
        Pretty::Num(q) => format!("{}/{}", q.numer(), q.denom()),
        Pretty::Var(v) => v.to_string(),
        Pretty::Pow(v, e) => format!("{v}^{e}"),
        Pretty::Product(fs) => fs
            .iter()
            .map(|f| paren_if_sum(f, style))
            .collect::<Vec<_>>()
            .join(style.times),
        Pretty::Sum(ts) => {
            let mut out = String::new();
            for (i, (negative, t)) in ts.iter().enumerate() {
                let sep = match (i, negative, nested) {
                    (0, false, _) => String::new(),
                    (0, true, _) => style.minus.to_string(),
                    (_, false, false) => " + ".to_string(),
                    (_, true, false) => format!(" {} ", style.minus),
                    (_, false, true) => "+".to_string(),
                    (_, true, true) => style.minus.to_string(),
                };
                out += &sep;
                out += &paren_if_sum(t, style);
            }
            out
        }
    }
}

fn paren_if_sum(p: &Pretty, style: Style) -> String {
    if matches!(p, Pretty::Sum(_)) {
        format!("({})", render(p, style, true))
    } else {
        render(p, style, true)
    }
}

/// `N / D` em uma linha, com os parênteses necessários para a leitura
/// da esquerda para a direita ficar certa.
pub fn fraction(num: &Pretty, den: Option<&Pretty>, style: Style) -> String {
    let n = render(num, style, false);
    let Some(den) = den else {
        return n;
    };
    let n = if num.is_compound_sum() {
        format!("({n})")
    } else {
        n
    };
    // O denominador inteiro precisa ficar junto: a/(b·c), não a/b·c.
    let d = if den.is_atom() {
        render(den, style, false)
    } else {
        format!("({})", render(den, style, false))
    };
    format!("{n} / {d}")
}
