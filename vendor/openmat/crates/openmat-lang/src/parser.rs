use crate::ast::{Expr, Kind, Operator, Statement};
use crate::lexer::{self, Token, TokenKind as T};
use crate::{Diagnostic, Result, Span};

// MATLAB R2022b iskeyword inventory, verified through the reference runtime.
fn reserved(name: &str) -> bool {
    matches!(
        name,
        "break"
            | "case"
            | "catch"
            | "classdef"
            | "continue"
            | "else"
            | "elseif"
            | "end"
            | "for"
            | "function"
            | "global"
            | "if"
            | "otherwise"
            | "parfor"
            | "persistent"
            | "return"
            | "spmd"
            | "switch"
            | "try"
            | "while"
    )
}

pub fn parse(source: &str) -> Result<Vec<Statement>> {
    Parser {
        tokens: lexer::lex(source)?,
        pos: 0,
        nesting: 0,
    }
    .program()
}
struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    nesting: usize,
}
impl Parser {
    fn token(&self) -> &Token {
        &self.tokens[self.pos]
    }
    fn at(&self, kind: &T) -> bool {
        std::mem::discriminant(&self.token().kind) == std::mem::discriminant(kind)
    }
    fn advance(&mut self) -> Token {
        let t = self.token().clone();
        if !self.at(&T::Eof) {
            self.pos += 1;
        }
        t
    }
    fn error(&self, message: &str) -> Diagnostic {
        Diagnostic::syntax(message, self.token().span)
    }
    fn expect(&mut self, kind: T) -> Result<()> {
        if !self.at(&kind) {
            return Err(self.error(&format!("esperado {kind:?}")));
        }
        self.advance();
        Ok(())
    }
    fn new_expr(&self, kind: Kind, span: Span, depth: usize) -> Result<Expr> {
        openmat_numeric::execution::checkpoint(1).map_err(|e|Diagnostic::resource(e,span))?;
        let text=match &kind{Kind::Text(s)|Kind::Variable(s)=>s.len(),_=>0};
        openmat_numeric::execution::allocation((std::mem::size_of::<Expr>()+text) as u64).map_err(|e|Diagnostic::resource(e,span))?;
        if depth > 128 {
            return Err(Diagnostic::limit("expressão excede 128 níveis", span));
        }
        Ok(Expr { kind, span, depth })
    }
    fn program(&mut self) -> Result<Vec<Statement>> {
        let mut out = Vec::new();
        while !self.at(&T::Eof) {
            while self.at(&T::Newline) || self.at(&T::Semicolon) || self.at(&T::Comma) {
                self.advance();
            }
            if self.at(&T::Eof) {
                break;
            }
            let targets = if self.output_assignment() {
                Some(self.output_targets()?)
            } else {
                None
            };
            let mut indices = None;
            let indexed = targets.is_none() && self.indexed_assignment();
            let name = if indexed {
                let lhs = self.expression(0, false)?;
                let Kind::Apply(name, args) = lhs.kind else {
                    return Err(self.error("destino de indexação inválido"));
                };
                indices = Some(args);
                self.expect(T::Equal)?;
                Some(name)
            } else if targets.is_some() {
                None
            } else if let T::Ident(name) = self.token().kind.clone() {
                if reserved(&name) {
                    return Err(self.error("comando de controle ainda não suportado neste marco"));
                }
                if self
                    .tokens
                    .get(self.pos + 1)
                    .is_some_and(|t| t.kind == T::Equal)
                {
                    self.advance();
                    self.advance();
                    Some(name)
                } else {
                    None
                }
            } else {
                None
            };
            let expr = self.expression(0, false)?;
            let silent = self.at(&T::Semicolon);
            if !self.at(&T::Newline)
                && !self.at(&T::Semicolon)
                && !self.at(&T::Comma)
                && !self.at(&T::Eof)
            {
                return Err(
                    self.error("esperado separador de comandos; sintaxe ainda não suportada")
                );
            }
            openmat_numeric::execution::allocation(std::mem::size_of::<Statement>() as u64).map_err(|e|Diagnostic::resource(e,expr.span))?;
            out.push(Statement {
                name,
                targets,
                indices,
                expr,
                silent,
            });
        }
        Ok(out)
    }
    fn indexed_assignment(&self) -> bool {
        if !matches!(&self.token().kind,T::Ident(n) if !reserved(n))
            || self.tokens.get(self.pos + 1).map(|t| &t.kind) != Some(&T::LeftParen)
        {
            return false;
        }
        let mut depth = 0;
        for (i, t) in self.tokens.iter().enumerate().skip(self.pos + 1) {
            match t.kind {
                T::LeftParen => depth += 1,
                T::RightParen => {
                    depth -= 1;
                    if depth == 0 {
                        return self.tokens.get(i + 1).is_some_and(|t| t.kind == T::Equal);
                    }
                }
                T::Eof => break,
                _ => {}
            }
        }
        false
    }
    fn output_assignment(&self) -> bool {
        if !self.at(&T::LeftBracket) {
            return false;
        }
        let mut depth = 0usize;
        for (index, token) in self.tokens.iter().enumerate().skip(self.pos) {
            match token.kind {
                T::LeftBracket => depth += 1,
                T::RightBracket => {
                    depth -= 1;
                    if depth == 0 {
                        return self
                            .tokens
                            .get(index + 1)
                            .is_some_and(|t| t.kind == T::Equal);
                    }
                }
                T::Eof => return false,
                _ => {}
            }
        }
        false
    }
    fn output_targets(&mut self) -> Result<Vec<Option<String>>> {
        self.expect(T::LeftBracket)?;
        let mut targets = Vec::new();
        while self.at(&T::Newline) {
            self.advance();
        }
        loop {
            let target = match self.advance().kind {
                T::Ident(name) if !reserved(&name) => Some(name),
                T::Tilde => None,
                _ => return Err(self.error("saída requer nome de variável ou ~")),
            };
            targets.push(target);
            if self.at(&T::RightBracket) {
                break;
            }
            if self.at(&T::Comma) {
                self.advance();
                if self.at(&T::RightBracket) {
                    break;
                }
            } else if !self.token().gap {
                return Err(self.error("esperado separador de saídas"));
            }
            if self.at(&T::Newline) {
                return Err(
                    self.error("lista de saídas deve ter uma linha; use ... para continuar")
                );
            }
        }
        self.expect(T::RightBracket)?;
        self.expect(T::Equal)?;
        Ok(targets)
    }
    fn expression(&mut self, min_bp: u8, in_matrix: bool) -> Result<Expr> {
        self.nesting += 1;
        if self.nesting > 128 {
            return Err(self.error("aninhamento excede 128 níveis"));
        }
        let result = self.expression_inner(min_bp, in_matrix);
        self.nesting -= 1;
        result
    }
    fn expression_inner(&mut self, min_bp: u8, in_matrix: bool) -> Result<Expr> {
        let t = self.advance();
        let mut left = match t.kind {
            T::Number(n) => self.new_expr(Kind::Number(n), t.span, 1)?,
            T::Imaginary(n) => self.new_expr(Kind::Imaginary(n), t.span, 1)?,
            T::Text(s) => self.new_expr(Kind::Text(s), t.span, 1)?,
            T::Ident(s) => {
                if s == "end" {
                    self.new_expr(Kind::End, t.span, 1)?
                } else {
                    if reserved(&s) {
                        return Err(Diagnostic::syntax("palavra reservada em expressão", t.span));
                    }
                    self.new_expr(Kind::Variable(s), t.span, 1)?
                }
            }
            T::Colon => self.new_expr(Kind::All, t.span, 1)?,
            T::Tilde => {
                let rhs = self.expression(35, in_matrix)?;
                let depth = rhs.depth + 1;
                self.new_expr(Kind::Not(Box::new(rhs)), t.span, depth)?
            }
            T::Minus | T::Plus => {
                let negative = t.kind == T::Minus;
                let rhs = self.expression(35, in_matrix)?;
                let depth = rhs.depth + 1;
                self.new_expr(Kind::Unary(negative, Box::new(rhs)), t.span, depth)?
            }
            T::LeftParen => {
                let e = self.expression(0, false)?;
                self.expect(T::RightParen)?;
                e
            }
            T::LeftBracket => self.matrix(t.span)?,
            _ => return Err(Diagnostic::syntax("esperada expressão", t.span)),
        };
        loop {
            if self.at(&T::LeftParen) && 50 >= min_bp {
                let name = if let Kind::Variable(name) = &left.kind {
                    name.clone()
                } else {
                    return Err(self.error("indexação de expressão temporária ainda não suportada"));
                };
                self.advance();
                let mut args = Vec::new();
                if !self.at(&T::RightParen) {
                    loop {
                        args.push(self.expression(0, false)?);
                        if !self.at(&T::Comma) {
                            break;
                        }
                        self.advance();
                    }
                }
                self.expect(T::RightParen)?;
                let depth = args.iter().map(|e| e.depth).max().unwrap_or(0) + 1;
                left = self.new_expr(Kind::Apply(name, args), left.span, depth)?;
                continue;
            }
            // MATLAB puts transpose and power at the same precedence.
            if (self.at(&T::Transpose) || self.at(&T::NonConjugate)) && 40 >= min_bp {
                let conjugate = self.at(&T::Transpose);
                self.advance();
                let depth = left.depth + 1;
                left = self.new_expr(
                    Kind::Transpose(conjugate, Box::new(left.clone())),
                    left.span,
                    depth,
                )?;
                continue;
            }
            if self.at(&T::Colon) && 10 >= min_bp {
                self.advance();
                let middle = self.expression(11, in_matrix)?;
                let (step, end) = if self.at(&T::Colon) {
                    self.advance();
                    (Some(Box::new(middle)), self.expression(11, in_matrix)?)
                } else {
                    (None, middle)
                };
                let depth = left
                    .depth
                    .max(end.depth)
                    .max(step.as_ref().map_or(0, |e| e.depth))
                    + 1;
                let span = left.span;
                left = self.new_expr(
                    Kind::Range(Box::new(left), step, Box::new(end)),
                    span,
                    depth,
                )?;
                continue;
            }
            let (op, bp) = match self.token().kind {
                T::Plus => (Operator::Add, 20),
                T::Minus => (Operator::Sub, 20),
                T::Star => (Operator::Mul, 30),
                T::Slash => (Operator::Div, 30),
                T::Backslash => (Operator::LeftDiv, 30),
                T::DotStar => (Operator::DotMul, 30),
                T::DotSlash => (Operator::DotDiv, 30),
                T::DotBackslash => (Operator::DotLeftDiv, 30),
                T::Power => (Operator::Power, 40),
                T::DotPower => (Operator::DotPower, 40),
                T::Eq => (Operator::Eq, 8),
                T::Ne => (Operator::Ne, 8),
                T::Lt => (Operator::Lt, 8),
                T::Le => (Operator::Le, 8),
                T::Gt => (Operator::Gt, 8),
                T::Ge => (Operator::Ge, 8),
                T::And => (Operator::And, 6),
                T::Or => (Operator::Or, 5),
                _ => break,
            };
            if bp < min_bp {
                break;
            }
            // In [1 -2], whitespace before an adjacent unary sign starts a new element.
            if in_matrix
                && matches!(self.token().kind, T::Plus | T::Minus)
                && self.token().gap
                && self
                    .tokens
                    .get(self.pos + 1)
                    .is_some_and(|t| t.start == self.token().end)
            {
                break;
            }
            self.advance();
            let right = self.expression(bp + 1, in_matrix)?;
            let depth = left.depth.max(right.depth) + 1;
            let span = left.span;
            left = self.new_expr(
                Kind::Binary(op, Box::new(left), Box::new(right)),
                span,
                depth,
            )?;
        }
        Ok(left)
    }
    fn matrix(&mut self, span: Span) -> Result<Expr> {
        let mut rows = Vec::new();
        let mut row = Vec::new();
        while self.at(&T::Newline) {
            self.advance();
        }
        if self.at(&T::RightBracket) {
            self.advance();
            return self.new_expr(Kind::Matrix(rows), span, 1);
        }
        loop {
            row.push(self.expression(0, true)?);
            if self.at(&T::RightBracket) {
                self.advance();
                rows.push(row);
                break;
            }
            if self.at(&T::Semicolon) || self.at(&T::Newline) {
                rows.push(std::mem::take(&mut row));
                while self.at(&T::Semicolon) || self.at(&T::Newline) {
                    self.advance();
                }
                if self.at(&T::RightBracket) {
                    self.advance();
                    break;
                }
            } else if self.at(&T::Comma) {
                self.advance();
            } else if !self.token().gap {
                return Err(self.error("esperado separador de elementos da matriz"));
            }
        }
        let depth = rows.iter().flatten().map(|e| e.depth).max().unwrap_or(0) + 1;
        self.new_expr(Kind::Matrix(rows), span, depth)
    }
}
