use codespan_reporting::diagnostic::Label;

use rlox_intermediate::Expression;

use crate::{DiagnosableResult, Diagnostic};
use crate::scanner::{Lexeme, Token};

mod expression;

struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    fn peek(&self) -> Option<&Token> {
        if self.current < self.tokens.len() {
            return Some(&self.tokens[self.current]);
        }
        None
    }

    fn must_peek(&self) -> DiagnosableResult<&Token> {
        if self.current < self.tokens.len() {
            return Ok(&self.tokens[self.current]);
        }
        let span = self.tokens[self.current - 1].span.clone();
        Err(Diagnostic::error()
            .with_code("E0003")
            .with_message("Unexpected EOF")
            .with_labels(vec![
                Label::primary((), span).with_message("incomplete code segment here")
            ]))
    }

    fn must_advance(&mut self) -> DiagnosableResult<&Token> {
        if self.current < self.tokens.len() {
            self.current += 1;
            return Ok(&self.tokens[self.current - 1]);
        }
        let span = self.tokens[self.current - 1].span.clone();
        Err(Diagnostic::error()
            .with_code("E0003")
            .with_message("Unexpected EOF")
            .with_labels(vec![
                Label::primary((), span).with_message("incomplete code segment here")
            ]))
    }

    fn must_consume(&mut self, lexeme: &Lexeme) -> DiagnosableResult {
        let Token { value, span } = self.must_advance()?;
        if value != lexeme {
            return Err(Diagnostic::error()
                .with_code("E0005")
                .with_message("Unexpected token")
                .with_labels(vec![Label::primary((), span.clone())
                    .with_message(format!("expected {lexeme:?}, found {value:?}"))]));
        }
        Ok(())
    }
}

pub fn parse_expression(tokens: Vec<Token>) -> DiagnosableResult<Expression> {
    let mut parser = Parser::new(tokens);
    parser.parse_expression()
}
