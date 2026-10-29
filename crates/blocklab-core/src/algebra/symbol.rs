//! Variáveis das expressões: a variável de Laplace `s` e os símbolos
//! (G1, H_2, Kp, a...).
//!
//! A ordem entre símbolos é "natural": o nome é quebrado em pedaços de
//! letras e pedaços de dígitos, e os dígitos são comparados como números.
//! Assim G2 < G10 < H1, que é a ordem que um engenheiro espera ler.

use std::cmp::Ordering;
use std::fmt;
use std::sync::Arc;

/// Nome de um símbolo (parâmetro do sistema). Clonar é barato (`Arc`).
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Symbol(Arc<str>);

impl Symbol {
    /// Cria um símbolo. Não valida o nome: quem garante que é um
    /// identificador válido (e diferente de `s`) é o parser.
    pub fn new(name: &str) -> Self {
        debug_assert!(name != "s", "'s' é a variável de Laplace, não um símbolo");
        Symbol(Arc::from(name))
    }

    pub fn name(&self) -> &str {
        &self.0
    }
}

/// Um pedaço do nome para a comparação natural.
#[derive(PartialEq, Eq, PartialOrd, Ord)]
enum Chunk<'a> {
    // A ordem das variantes importa: texto vem antes de número no mesmo
    // ponto (raro; só acontece em nomes como "G_1" vs "G1").
    Text(&'a str),
    // Número comparado pelo valor (sem zeros à esquerda: primeiro o
    // comprimento, depois os dígitos) e, no empate, pelo texto original.
    Number(usize, &'a str, &'a str),
}

fn chunks(name: &str) -> Vec<Chunk<'_>> {
    let mut out = Vec::new();
    let bytes = name.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let start = i;
        let digit = bytes[i].is_ascii_digit();
        while i < bytes.len() && bytes[i].is_ascii_digit() == digit {
            i += 1;
        }
        let piece = &name[start..i];
        if digit {
            let trimmed = piece.trim_start_matches('0');
            out.push(Chunk::Number(trimmed.len(), trimmed, piece));
        } else {
            out.push(Chunk::Text(piece));
        }
    }
    out
}

impl Ord for Symbol {
    fn cmp(&self, other: &Self) -> Ordering {
        chunks(&self.0).cmp(&chunks(&other.0))
    }
}

impl PartialOrd for Symbol {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl fmt::Display for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl fmt::Debug for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Uma variável de polinômio.
///
/// `S` vem primeiro na ordem (variante declarada antes), então em todo
/// monômio o fator em `s` aparece antes dos símbolos.
#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Var {
    /// Variável de Laplace.
    S,
    /// Parâmetro simbólico.
    Sym(Symbol),
}

impl Var {
    /// `"s"` vira a variável de Laplace; qualquer outro nome vira símbolo.
    pub fn named(name: &str) -> Self {
        if name == "s" {
            Var::S
        } else {
            Var::Sym(Symbol::new(name))
        }
    }
}

impl fmt::Display for Var {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Var::S => f.write_str("s"),
            Var::Sym(sym) => sym.fmt(f),
        }
    }
}

impl fmt::Debug for Var {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sorted(names: &[&str]) -> Vec<String> {
        let mut v: Vec<Symbol> = names.iter().map(|n| Symbol::new(n)).collect();
        v.sort();
        v.iter().map(|s| s.name().to_string()).collect()
    }

    #[test]
    fn ordem_natural() {
        assert_eq!(
            sorted(&["H1", "G10", "G2", "G1", "H_2", "K"]),
            ["G1", "G2", "G10", "H1", "H_2", "K"]
        );
    }

    #[test]
    fn s_vem_antes_dos_simbolos() {
        assert!(Var::S < Var::named("A"));
        assert_eq!(Var::named("s"), Var::S);
    }
}
