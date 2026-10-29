//! Parser descendente recursivo.
//!
//! Gramática, da menor para a maior precedência (cada regra chama a de
//! baixo, então a de baixo "amarra" mais forte):
//!
//! ```text
//! expr     := term (('+' | '-') term)*
//! term     := unary (('*' | '/') unary)*
//! unary    := ('+' | '-') unary | implicit
//! implicit := power power*                  ← justaposição: 2s, s(s+1)
//! power    := atom ('^' exponent)?
//! atom     := NÚMERO | NOME | '(' expr ')'
//! exponent := ['+' | '-'] INTEIRO | '(' ['+' | '-'] INTEIRO ')'
//! ```
//!
//! **Multiplicação implícita amarra mais forte que `*` e `/`.** É a
//! convenção de quem escreve funções de transferência à mão:
//! `10/s(s+1)` significa 10/(s·(s+1)), e `1/2s` significa 1/(2s).
//! Com `*` explícito vale a regra usual da esquerda para a direita:
//! `1/2*s` = s/2.

use num_traits::ToPrimitive;

use super::ast::{BinOp, Expr, ExprKind};
use super::error::{ParseError, ParseErrorKind, Span};
use super::lexer::{Token, TokenKind, tokenize};
use crate::algebra::Var;

/// Maior |expoente| aceito. Evita que `(s+1)^100000` congele o programa.
pub const MAX_EXPONENT: u32 = 100;

/// Converte o texto em AST.
pub fn parse(src: &str) -> Result<Expr, ParseError> {
    let tokens = tokenize(src)?;
    let mut p = Parser {
        tokens,
        pos: 0,
        depth: 0,
    };
    if p.peek().kind == TokenKind::End {
        return Err(ParseError::new(ParseErrorKind::Empty, Span::new(0, 1)));
    }
    let expr = p.expr()?;
    let tok = p.peek();
    match tok.kind {
        TokenKind::End => Ok(expr),
        TokenKind::RParen => Err(ParseError::new(ParseErrorKind::UnmatchedParen, tok.span)),
        _ => Err(p.unexpected("um operador ou o fim da expressão")),
    }
}

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    /// Quantos '(' estão abertos agora.
    depth: usize,
}

impl Parser {
    fn peek(&self) -> &Token {
        &self.tokens[self.pos]
    }

    fn next(&mut self) -> Token {
        let t = self.tokens[self.pos].clone();
        if t.kind != TokenKind::End {
            self.pos += 1;
        }
        t
    }

    fn unexpected(&self, expected: &'static str) -> ParseError {
        let tok = self.peek();
        ParseError::new(
            ParseErrorKind::Unexpected {
                expected,
                found: tok.kind.describe(),
            },
            tok.span,
        )
    }

    fn bin(op: BinOp, a: Expr, b: Expr) -> Expr {
        let span = a.span.join(b.span);
        Expr::new(ExprKind::Bin(op, Box::new(a), Box::new(b)), span)
    }

    fn expr(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.term()?;
        loop {
            let op = match self.peek().kind {
                TokenKind::Plus => BinOp::Add,
                TokenKind::Minus => BinOp::Sub,
                _ => return Ok(left),
            };
            self.next();
            let right = self.term()?;
            left = Self::bin(op, left, right);
        }
    }

    fn term(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.unary()?;
        loop {
            let op = match self.peek().kind {
                TokenKind::Star => BinOp::Mul,
                TokenKind::Slash => BinOp::Div,
                _ => return Ok(left),
            };
            self.next();
            let right = self.unary()?;
            left = Self::bin(op, left, right);
        }
    }

    fn unary(&mut self) -> Result<Expr, ParseError> {
        match self.peek().kind {
            TokenKind::Plus => {
                self.next();
                self.unary()
            }
            TokenKind::Minus => {
                let op = self.next().span;
                let inner = self.unary()?;
                let span = op.join(inner.span);
                Ok(Expr::new(ExprKind::Neg(Box::new(inner)), span))
            }
            _ => self.implicit(),
        }
    }

    /// Justaposição: enquanto o próximo token puder começar um fator
    /// (número, nome ou '('), é multiplicação implícita.
    fn implicit(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.power()?;
        loop {
            let starts_factor = matches!(
                self.peek().kind,
                TokenKind::Number { .. } | TokenKind::Ident(_) | TokenKind::LParen
            );
            if !starts_factor {
                return Ok(left);
            }
            // "2 3" quase sempre é erro de digitação, não 6.
            let prev_is_number = matches!(self.tokens[self.pos - 1].kind, TokenKind::Number { .. });
            if prev_is_number && matches!(self.peek().kind, TokenKind::Number { .. }) {
                return Err(self.unexpected("um operador entre os dois números"));
            }
            let right = self.power()?;
            left = Self::bin(BinOp::Mul, left, right);
        }
    }

    fn power(&mut self) -> Result<Expr, ParseError> {
        let base = self.atom()?;
        if self.peek().kind != TokenKind::Caret {
            return Ok(base);
        }
        self.next();
        let (exp, exp_span) = self.exponent(&base)?;
        if self.peek().kind == TokenKind::Caret {
            return Err(ParseError::new(
                ParseErrorKind::ChainedPower,
                self.peek().span,
            ));
        }
        let span = base.span.join(exp_span);
        Ok(Expr::new(ExprKind::Pow(Box::new(base), exp), span))
    }

    fn atom(&mut self) -> Result<Expr, ParseError> {
        let tok = self.peek().clone();
        match tok.kind {
            TokenKind::Number { value, .. } => {
                self.next();
                Ok(Expr::new(ExprKind::Num(value), tok.span))
            }
            TokenKind::Ident(name) => {
                self.next();
                Ok(Expr::new(ExprKind::Var(Var::named(&name)), tok.span))
            }
            TokenKind::LParen => {
                self.next();
                self.depth += 1;
                let mut inner = self.expr()?;
                self.depth -= 1;
                let close = self.peek().clone();
                match close.kind {
                    TokenKind::RParen => {
                        self.next();
                        // O trecho passa a incluir os parênteses, para o
                        // erro de divisão por zero sublinhar "(s - s)".
                        inner.span = tok.span.join(close.span);
                        Ok(inner)
                    }
                    TokenKind::End => Err(ParseError::new(ParseErrorKind::UnclosedParen, tok.span)),
                    _ => Err(self.unexpected("')' ou um operador")),
                }
            }
            TokenKind::RParen if self.depth == 0 => {
                Err(ParseError::new(ParseErrorKind::UnmatchedParen, tok.span))
            }
            _ => Err(self.unexpected("um número, um símbolo ou '('")),
        }
    }

    /// Lê o expoente depois de '^'. Só aceita inteiro literal (com sinal
    /// opcional, com ou sem parênteses). Tudo o mais vira um erro que
    /// explica *por que* não é aceito.
    fn exponent(&mut self, base: &Expr) -> Result<(i32, Span), ParseError> {
        let is_e = matches!(&base.kind, ExprKind::Var(Var::Sym(sym)) if sym.name() == "e");
        let start = self.pos;

        if let Some(result) = self.try_integer_literal() {
            return result;
        }
        // Não é inteiro literal: volta ao início do expoente e descobre o
        // que é, para dar a mensagem certa.
        self.pos = start;
        let tok = self.peek().clone();
        let exponential =
            |span: Span| ParseError::new(ParseErrorKind::Exponential, base.span.join(span));
        match tok.kind {
            TokenKind::Number { integer: false, .. } if !is_e => Err(ParseError::new(
                ParseErrorKind::NonIntegerExponent("um número decimal"),
                tok.span,
            )),
            TokenKind::Ident(name) if !is_e => Err(ParseError::new(
                ParseErrorKind::SymbolicExponent(name),
                tok.span,
            )),
            TokenKind::Number { .. }
            | TokenKind::Ident(_)
            | TokenKind::LParen
            | TokenKind::Minus
            | TokenKind::Plus => {
                // Mede o trecho da expressão do expoente para sublinhar
                // tudo. Se nem ela fizer sentido, vale o erro dela.
                let span = self.unary_span()?;
                if is_e {
                    Err(exponential(span))
                } else {
                    Err(ParseError::new(
                        ParseErrorKind::NonIntegerExponent("uma expressão"),
                        span,
                    ))
                }
            }
            _ => Err(self.unexpected("um expoente inteiro")),
        }
    }

    /// `['+'|'-'] INTEIRO` ou `'(' ['+'|'-'] INTEIRO ')'`. Devolve `None`
    /// (sem garantia de posição) se o formato não bate.
    fn try_integer_literal(&mut self) -> Option<Result<(i32, Span), ParseError>> {
        let open = (self.peek().kind == TokenKind::LParen).then(|| self.next().span);
        let first = self.peek().span;
        let negative = match self.peek().kind {
            TokenKind::Minus => {
                self.next();
                true
            }
            TokenKind::Plus => {
                self.next();
                false
            }
            _ => false,
        };
        let tok = self.next();
        let TokenKind::Number {
            value,
            integer: true,
        } = tok.kind
        else {
            return None;
        };
        let mut span = first.join(tok.span);
        if let Some(open) = open {
            if self.peek().kind != TokenKind::RParen {
                return None;
            }
            span = open.join(self.next().span);
        }
        let magnitude = value.to_integer().to_u32().filter(|m| *m <= MAX_EXPONENT);
        Some(match magnitude {
            // Cabe em i32 porque MAX_EXPONENT é pequeno.
            Some(m) => Ok((if negative { -(m as i32) } else { m as i32 }, span)),
            None => Err(ParseError::new(
                ParseErrorKind::ExponentTooLarge { max: MAX_EXPONENT },
                span,
            )),
        })
    }

    /// Consome um `unary` só para saber o trecho que ele ocupa.
    fn unary_span(&mut self) -> Result<Span, ParseError> {
        self.unary().map(|e| e.span)
    }
}
