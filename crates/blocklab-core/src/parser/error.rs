//! Erros do parser, sempre com a posição do caractere.

use std::fmt;

use thiserror::Error;

/// Trecho do texto, em índices de **caractere** (não de byte), com fim
/// exclusivo. Assim a posição bate com o que o usuário vê, mesmo com
/// acentos ou letras gregas.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Span { start, end }
    }

    /// Menor trecho que cobre os dois.
    pub fn join(self, other: Span) -> Span {
        Span::new(self.start.min(other.start), self.end.max(other.end))
    }
}

/// O que deu errado.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ParseErrorKind {
    #[error("expressão vazia")]
    Empty,
    #[error("caractere inválido '{0}'")]
    InvalidChar(char),
    #[error("número mal formado: falta dígito depois do ponto decimal")]
    BadNumber,
    #[error("esperava {expected}, encontrei {found}")]
    Unexpected {
        expected: &'static str,
        found: String,
    },
    #[error("'(' sem o ')' correspondente")]
    UnclosedParen,
    #[error("')' sem o '(' correspondente")]
    UnmatchedParen,
    #[error("o expoente precisa ser um inteiro literal, não {0}")]
    NonIntegerExponent(&'static str),
    #[error(
        "exponencial e^(...) não é suportada: atraso de transporte não é \
         uma função racional de s"
    )]
    Exponential,
    #[error("o expoente precisa ser um inteiro literal, não o símbolo '{0}'")]
    SymbolicExponent(String),
    #[error("expoente grande demais (o máximo é ±{max})")]
    ExponentTooLarge { max: u32 },
    #[error("potências encadeadas são ambíguas; use parênteses, como (s^2)^3")]
    ChainedPower,
    #[error("divisão por zero: este divisor vale 0")]
    DivisionByZero,
}

/// Erro com a posição onde ocorreu.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub struct ParseError {
    pub span: Span,
    pub kind: ParseErrorKind,
}

impl ParseError {
    pub fn new(kind: ParseErrorKind, span: Span) -> Self {
        ParseError { span, kind }
    }

    /// Posição do primeiro caractere com problema, contando de 1 (como um
    /// editor de texto mostra a coluna).
    pub fn column(&self) -> usize {
        self.span.start + 1
    }

    /// Mostra a expressão com o trecho do erro sublinhado:
    ///
    /// ```text
    /// 10/(s+1
    ///    ^ posição 4: '(' sem o ')' correspondente
    /// ```
    pub fn render(&self, src: &str) -> String {
        let width = (self.span.end.saturating_sub(self.span.start)).max(1);
        format!(
            "{src}\n{}{} posição {}: {}",
            " ".repeat(self.span.start),
            "^".repeat(width),
            self.column(),
            self.kind
        )
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "posição {}: {}", self.column(), self.kind)
    }
}
