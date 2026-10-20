use crate::{Diagnostic, Result, Span};

#[derive(Clone, Debug, PartialEq)]
pub enum TokenKind {
    Number(f64),
    Imaginary(f64),
    Text(String),
    Ident(String),
    Plus,
    Minus,
    Star,
    Slash,
    Backslash,
    Power,
    DotStar,
    DotSlash,
    DotBackslash,
    DotPower,
    Transpose,
    NonConjugate,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
    Equal,
    Tilde,
    Colon,
    Comma,
    Semicolon,
    Newline,
    LeftParen,
    RightParen,
    LeftBracket,
    RightBracket,
    Eof,
}
#[derive(Clone, Debug)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
    pub start: usize,
    pub end: usize,
    pub gap: bool,
}

pub fn lex(source: &str) -> Result<Vec<Token>> {
    if source.len() > 1_048_576 {
        return Err(Diagnostic::limit(
            "fonte excede 1 MiB",
            Span { line: 1, column: 1 },
        ));
    }
    let bytes = source.as_bytes();
    let (mut i, mut line, mut column) = (0, 1, 1);
    let mut tokens: Vec<Token> = Vec::new();
    let mut gap = false;
    while i < bytes.len() {
        openmat_numeric::execution::checkpoint(0).map_err(|e|Diagnostic::resource(e,Span{line,column}))?;
        let start = i;
        let span = Span { line, column };
        let b = bytes[i];
        if b == b' ' || b == b'\t' || b == b'\r' {
            i += 1;
            column += 1;
            gap = true;
            continue;
        }
        if b == b'%' {
            while i < bytes.len() && bytes[i] != b'\n' {
                i += 1;
                column += 1;
            }
            gap = true;
            continue;
        }
        if source[i..].starts_with("...") {
            i += 3;
            column += 3;
            while i < bytes.len() && bytes[i] != b'\n' {
                i += 1;
                column += 1;
            }
            if i < bytes.len() {
                i += 1;
                line += 1;
                column = 1;
            }
            gap = true;
            continue;
        }
        let can_transpose = tokens.last().is_some_and(|t| {
            matches!(
                t.kind,
                TokenKind::Number(_)
                    | TokenKind::Imaginary(_)
                    | TokenKind::Ident(_)
                    | TokenKind::RightParen
                    | TokenKind::RightBracket
                    | TokenKind::Transpose
                    | TokenKind::NonConjugate
            )
        });
        let kind = if b.is_ascii_digit()
            || (b == b'.' && bytes.get(i + 1).is_some_and(u8::is_ascii_digit))
        {
            while i < bytes.len() && bytes[i].is_ascii_digit() {
                i += 1;
            }
            // Do not consume the dot belonging to an element-wise operator.
            if bytes.get(i) == Some(&b'.')
                && !bytes
                    .get(i + 1)
                    .is_some_and(|c| matches!(c, b'*' | b'/' | b'\\' | b'^' | b'\''))
            {
                i += 1;
                while i < bytes.len() && bytes[i].is_ascii_digit() {
                    i += 1;
                }
            }
            if bytes.get(i).is_some_and(|c| matches!(c, b'e' | b'E')) {
                i += 1;
                if bytes.get(i).is_some_and(|c| matches!(c, b'+' | b'-')) {
                    i += 1;
                }
                let exponent_start = i;
                while i < bytes.len() && bytes[i].is_ascii_digit() {
                    i += 1;
                }
                if i == exponent_start {
                    return Err(Diagnostic::syntax("expoente numérico inválido", span));
                }
            }
            let number = source[start..i]
                .parse()
                .map_err(|_| Diagnostic::syntax("número inválido", span))?;
            if bytes.get(i).is_some_and(|c| matches!(c, b'i' | b'j')) {
                i += 1;
                TokenKind::Imaginary(number)
            } else {
                TokenKind::Number(number)
            }
        } else if b.is_ascii_alphabetic() || b == b'_' {
            i += 1;
            while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') {
                i += 1;
            }
            TokenKind::Ident(source[start..i].into())
        } else if b == b'"' || (b == b'\'' && (!can_transpose || gap)) {
            let quote = b;
            i += 1;
            let mut text = String::new();
            let mut closed = false;
            while i < bytes.len() {
                if bytes[i] == b'\n' {
                    return Err(Diagnostic::syntax("texto não terminado", span));
                }
                if bytes[i] == quote {
                    i += 1;
                    if bytes.get(i) == Some(&quote) {
                        text.push(quote as char);
                        i += 1;
                    } else {
                        closed = true;
                        break;
                    }
                } else {
                    let ch = source[i..].chars().next().unwrap();
                    text.push(ch);
                    i += ch.len_utf8();
                }
            }
            if !closed {
                return Err(Diagnostic::syntax("texto não terminado", span));
            }
            TokenKind::Text(text)
        } else {
            i += 1;
            match b {
                b'+' => TokenKind::Plus,
                b'-' => TokenKind::Minus,
                b'*' => TokenKind::Star,
                b'/' => TokenKind::Slash,
                b'\\' => TokenKind::Backslash,
                b'^' => TokenKind::Power,
                b'\'' => TokenKind::Transpose,
                b'=' if bytes.get(i) == Some(&b'=') => {
                    i += 1;
                    TokenKind::Eq
                }
                b'=' => TokenKind::Equal,
                b'~' if bytes.get(i) == Some(&b'=') => {
                    i += 1;
                    TokenKind::Ne
                }
                b'~' => TokenKind::Tilde,
                b'<' if bytes.get(i) == Some(&b'=') => {
                    i += 1;
                    TokenKind::Le
                }
                b'>' if bytes.get(i) == Some(&b'=') => {
                    i += 1;
                    TokenKind::Ge
                }
                b'<' => TokenKind::Lt,
                b'>' => TokenKind::Gt,
                b'&' => TokenKind::And,
                b'|' => TokenKind::Or,
                b':' => TokenKind::Colon,
                b',' => TokenKind::Comma,
                b';' => TokenKind::Semicolon,
                b'(' => TokenKind::LeftParen,
                b')' => TokenKind::RightParen,
                b'[' => TokenKind::LeftBracket,
                b']' => TokenKind::RightBracket,
                b'\n' => TokenKind::Newline,
                b'.' => {
                    let next = *bytes
                        .get(i)
                        .ok_or_else(|| Diagnostic::syntax("esperado operador após '.'", span))?;
                    i += 1;
                    match next {
                        b'*' => TokenKind::DotStar,
                        b'/' => TokenKind::DotSlash,
                        b'\\' => TokenKind::DotBackslash,
                        b'^' => TokenKind::DotPower,
                        b'\'' => TokenKind::NonConjugate,
                        _ => return Err(Diagnostic::syntax("operação '.' não suportada", span)),
                    }
                }
                _ => {
                    return Err(Diagnostic::syntax(
                        format!(
                            "caractere não suportado: {}",
                            source[start..].chars().next().unwrap()
                        ),
                        span,
                    ))
                }
            }
        };
        openmat_numeric::execution::allocation(std::mem::size_of::<Token>() as u64).map_err(|e|Diagnostic::resource(e,span))?;
        tokens.push(Token {
            kind,
            span,
            start,
            end: i,
            gap,
        });
        if tokens.len() > 100_000 {
            return Err(Diagnostic::syntax("limite de tokens excedido", span));
        }
        gap = false;
        if b == b'\n' {
            line += 1;
            column = 1;
        } else {
            column += source[start..i].chars().count();
        }
    }
    tokens.push(Token {
        kind: TokenKind::Eof,
        span: Span { line, column },
        start: i,
        end: i,
        gap,
    });
    Ok(tokens)
}
