//! Formatação de funções de transferência para leitura humana.
//!
//! ```
//! use blocklab_core::format::{to_latex, to_text, Mode};
//! use blocklab_core::parser::parse_rational;
//!
//! let t = parse_rational("(G1 G4 G2 + G1 G4 G3)/(1 + G1 G4 H1)").unwrap();
//! assert_eq!(to_text(&t, Mode::Symbolic), "G1·G4·(G2+G3) / (1 + G1·G4·H1)");
//! assert_eq!(
//!     to_latex(&t, Mode::Symbolic),
//!     r"\frac{G_{1} G_{4} \left(G_{2} + G_{3}\right)}{1 + G_{1} G_{4} H_{1}}"
//! );
//! ```

mod factor;
mod latex;
mod pretty;
mod text;

pub use pretty::Pretty;

use crate::algebra::{Poly, RationalFn};

/// Como apresentar os polinômios.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Mode {
    /// Fatorado e agrupado: G1·G4·(G2+G3).
    #[default]
    Symbolic,
    /// Polinômio em `s`, potências decrescentes: s² + 3s + 2.
    Numeric,
}

/// Numerador e denominador já prontos para exibir (a interface desenha
/// a fração com eles; `den = None` quando o denominador é 1).
#[derive(Clone, Debug, PartialEq)]
pub struct PrettyFraction {
    pub num: Pretty,
    pub den: Option<Pretty>,
}

impl PrettyFraction {
    /// Uma linha, com `·` e `−`.
    pub fn text(&self) -> String {
        text::fraction(&self.num, self.den.as_ref(), text::UNICODE)
    }

    /// Uma linha, só ASCII (`*` e `-`), para colar em outras ferramentas.
    pub fn ascii(&self) -> String {
        text::fraction(&self.num, self.den.as_ref(), text::ASCII)
    }

    /// LaTeX com `\frac{...}{...}`.
    pub fn latex(&self) -> String {
        latex::fraction(&self.num, self.den.as_ref())
    }
}

impl Pretty {
    /// Texto (Unicode) desta parte, sem parênteses externos.
    pub fn text(&self) -> String {
        text::render(self, text::UNICODE, false)
    }

    pub fn ascii(&self) -> String {
        text::render(self, text::ASCII, false)
    }

    pub fn latex(&self) -> String {
        latex::render(self)
    }
}

/// Árvore de apresentação de um polinômio.
pub fn pretty_poly(p: &Poly, mode: Mode) -> Pretty {
    match mode {
        Mode::Symbolic => factor::symbolic(p),
        Mode::Numeric => factor::numeric(p),
    }
}

pub fn pretty(f: &RationalFn, mode: Mode) -> PrettyFraction {
    PrettyFraction {
        num: pretty_poly(f.num(), mode),
        den: (*f.den() != Poly::one()).then(|| pretty_poly(f.den(), mode)),
    }
}

pub fn to_text(f: &RationalFn, mode: Mode) -> String {
    pretty(f, mode).text()
}

pub fn to_ascii(f: &RationalFn, mode: Mode) -> String {
    pretty(f, mode).ascii()
}

pub fn to_latex(f: &RationalFn, mode: Mode) -> String {
    pretty(f, mode).latex()
}

/// Nome de um símbolo em LaTeX (`G1` → `G_{1}`), para tabelas na
/// interface.
pub fn latex_symbol(name: &str) -> String {
    latex::symbol(name)
}

#[cfg(test)]
mod tests;
