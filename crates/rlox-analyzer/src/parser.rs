use rlox_intermediate::{DiagnosableResult, Expression, raise};

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
        raise!("E0003", self.tokens[self.current - 1].span.clone())
    }

    fn must_advance(&mut self) -> DiagnosableResult<&Token> {
        if self.current < self.tokens.len() {
            self.current += 1;
            return Ok(&self.tokens[self.current - 1]);
        }
        raise!("E0003", self.tokens[self.current - 1].span.clone())
    }

    fn must_consume(&mut self, lexeme: &Lexeme) -> DiagnosableResult {
        let Token { value, span } = self.must_advance()?;
        if value == lexeme {
            return Ok(());
        }
        raise! {
            "E0005", span.clone(),
            format!("expected {lexeme:?}, found {value:?}"),
        }
    }
}

pub fn parse_expression(tokens: Vec<Token>) -> DiagnosableResult<Expression> {
    let mut parser = Parser::new(tokens);
    parser.parse_expression()
}
