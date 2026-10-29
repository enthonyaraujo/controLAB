//! Análise léxica: quebra o texto em *tokens* (números, nomes,
//! operadores), cada um com sua posição.

use num_bigint::BigInt;
use num_rational::BigRational;
use num_traits::{One, Zero};

use super::error::{ParseError, ParseErrorKind, Span};

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    /// Número já convertido para fração exata. `integer` diz se foi
    /// escrito sem ponto decimal (importa para o expoente).
    Number {
        value: BigRational,
        integer: bool,
    },
    Ident(String),
    Plus,
    Minus,
    Star,
    Slash,
    Caret,
    LParen,
    RParen,
    End,
}

impl TokenKind {
    /// Descrição para mensagens de erro.
    pub fn describe(&self) -> String {
        match self {
            TokenKind::Number { value, .. } => format!("o número {value}"),
            TokenKind::Ident(name) => format!("'{name}'"),
            TokenKind::Plus => "'+'".into(),
            TokenKind::Minus => "'-'".into(),
            TokenKind::Star => "'*'".into(),
            TokenKind::Slash => "'/'".into(),
            TokenKind::Caret => "'^'".into(),
            TokenKind::LParen => "'('".into(),
            TokenKind::RParen => "')'".into(),
            TokenKind::End => "o fim da expressão".into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

/// Converte o texto em tokens; o último é sempre `End`.
pub fn tokenize(src: &str) -> Result<Vec<Token>, ParseError> {
    let chars: Vec<char> = src.chars().collect();
    let mut tokens = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        let start = i;
        let simple = match c {
            '+' => Some(TokenKind::Plus),
            // U+2212 é o "menos" tipográfico, comum ao colar de PDFs.
            '-' | '−' => Some(TokenKind::Minus),
            // '·' e '×' também valem como multiplicação.
            '*' | '·' | '×' => Some(TokenKind::Star),
            '/' => Some(TokenKind::Slash),
            '^' => Some(TokenKind::Caret),
            '(' => Some(TokenKind::LParen),
            ')' => Some(TokenKind::RParen),
            _ => None,
        };
        if let Some(kind) = simple {
            i += 1;
            tokens.push(Token {
                kind,
                span: Span::new(start, i),
            });
            continue;
        }
        if c.is_whitespace() {
            i += 1;
        } else if c.is_ascii_digit() || c == '.' {
            let (value, integer, next) = number(&chars, i)?;
            i = next;
            tokens.push(Token {
                kind: TokenKind::Number { value, integer },
                span: Span::new(start, i),
            });
        } else if c.is_alphabetic() {
            // Identificador: letra seguida de letras, dígitos ou '_'.
            // Por isso "G1s" é UM símbolo; para G1·s escreva "G1 s" ou "G1*s".
            while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                i += 1;
            }
            tokens.push(Token {
                kind: TokenKind::Ident(chars[start..i].iter().collect()),
                span: Span::new(start, i),
            });
        } else {
            return Err(ParseError::new(
                ParseErrorKind::InvalidChar(c),
                Span::new(start, start + 1),
            ));
        }
    }
    tokens.push(Token {
        kind: TokenKind::End,
        span: Span::new(chars.len(), chars.len() + 1),
    });
    Ok(tokens)
}

/// Lê um número decimal a partir de `start` e o converte para fração
/// exata: "2.75" = 275/100 = 11/4. Nada de ponto flutuante, então
/// 0.1 + 0.2 dá exatamente 3/10.
fn number(chars: &[char], start: usize) -> Result<(BigRational, bool, usize), ParseError> {
    let mut i = start;
    let mut digits = BigInt::zero();
    let mut denom = BigInt::one();
    while i < chars.len() && chars[i].is_ascii_digit() {
        digits = digits * 10 + chars[i].to_digit(10).expect("é dígito");
        i += 1;
    }
    let mut integer = true;
    if i < chars.len() && chars[i] == '.' {
        integer = false;
        i += 1;
        let frac_start = i;
        while i < chars.len() && chars[i].is_ascii_digit() {
            digits = digits * 10 + chars[i].to_digit(10).expect("é dígito");
            denom *= 10;
            i += 1;
        }
        // "2." ou "." sozinho: exigimos ao menos um dígito após o ponto
        // (e, no caso de ".", também antes não havia nenhum).
        if i == frac_start {
            return Err(ParseError::new(
                ParseErrorKind::BadNumber,
                Span::new(start, i),
            ));
        }
    }
    Ok((BigRational::new(digits, denom), integer, i))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kinds(src: &str) -> Vec<TokenKind> {
        tokenize(src).unwrap().into_iter().map(|t| t.kind).collect()
    }

    #[test]
    fn decimais_viram_fracao_exata() {
        let toks = kinds("2.75 .5 10");
        let q = |n: i64, d: i64| BigRational::new(n.into(), d.into());
        assert_eq!(
            toks[..3],
            [
                TokenKind::Number {
                    value: q(11, 4),
                    integer: false
                },
                TokenKind::Number {
                    value: q(1, 2),
                    integer: false
                },
                TokenKind::Number {
                    value: q(10, 1),
                    integer: true
                },
            ]
        );
    }

    #[test]
    fn identificadores() {
        assert_eq!(
            kinds("H_2 Kp ω_n"),
            [
                TokenKind::Ident("H_2".into()),
                TokenKind::Ident("Kp".into()),
                TokenKind::Ident("ω_n".into()),
                TokenKind::End
            ]
        );
    }

    #[test]
    fn posicao_em_caracteres_nao_bytes() {
        // 'ω' ocupa 2 bytes, mas 1 caractere: o '$' está na posição 2.
        let err = tokenize("ω $").unwrap_err();
        assert_eq!(err.span, Span::new(2, 3));
        assert_eq!(err.kind, ParseErrorKind::InvalidChar('$'));
    }

    #[test]
    fn ponto_sem_digito() {
        let err = tokenize("s + 2.").unwrap_err();
        assert_eq!(err.kind, ParseErrorKind::BadNumber);
        assert_eq!(err.span, Span::new(4, 6));
    }
}
