use std::mem;

use rlox_intermediate::{DiagnosableResult, raise};

use crate::parser::Parser;
use crate::scanner::{Lexeme, Token};

// Utility functions
impl Parser {
    pub(super) fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    pub(super) fn has_reached_end(&self) -> bool {
        self.peek().is_none()
    }

    pub(super) fn peek(&self) -> Option<&Token> {
        if self.current < self.tokens.len() {
            return Some(&self.tokens[self.current]);
        }
        None
    }

    pub(super) fn advance(&mut self) -> Option<&Token> {
        if self.current < self.tokens.len() {
            self.current += 1;
            return Some(&self.tokens[self.current - 1]);
        }
        None
    }

    pub(super) fn try_consume(&mut self, lexeme: &Lexeme) -> bool {
        if let Some(peek) = self.peek() {
            if mem::discriminant(&peek.value) == mem::discriminant(lexeme) {
                self.advance();
                return true;
            }
        }
        false
    }
}

// "Must version" of utility functions
impl Parser {
    pub(super) fn must_peek(&self) -> DiagnosableResult<&Token> {
        if let Some(peek) = self.peek() {
            return Ok(peek);
        }
        raise!("E0003", self.tokens[self.current - 1].span.clone())
    }

    pub(super) fn must_advance(&mut self) -> DiagnosableResult<&Token> {
        // if we call `self.advance` using if-let, the borrow checker will complain.
        //
        // because the returned reference has the same lifetime as self, `raise!` in an
        // else block will not work -- that borrows self as immutable.
        if self.has_reached_end() {
            raise!("E0003", self.tokens[self.current - 1].span.clone());
        }
        Ok(self.advance().unwrap())
    }

    // a little bit different here: try_consume returns bool, but must_consume needs to
    // return a reference to the consumed token.
    pub(super) fn must_consume(&mut self, lexeme: &Lexeme) -> DiagnosableResult<&Token> {
        let token = self.must_advance()?;
        if mem::discriminant(&token.value) == mem::discriminant(lexeme) {
            return Ok(token);
        }
        raise! {
            "E0005", token.span.clone(),
            format!("expected {:?}, found {:?}", lexeme, token.value),
        }
    }
}
