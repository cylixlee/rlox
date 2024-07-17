use rlox_intermediate::*;

use crate::parser::Parser;
use crate::scanner::{Lexeme, Token};

static IDENTIFIER: Lexeme = Lexeme::Identifier(String::new());

impl Parser {
    pub fn parse_declaration(&mut self) -> DiagnosableResult<Declaration> {
        Ok(match &self.must_peek()?.value {
            Lexeme::Class => self.parse_class_declaration()?,
            Lexeme::Fun => self.parse_fun_declaration()?,
            Lexeme::Var => self.parse_var_declaration()?,
            _ => Declaration::Statement(self.parse_statement()?),
        })
    }

    fn parse_class_declaration(&mut self) -> DiagnosableResult<Declaration> {
        self.must_consume(&Lexeme::Class)?;
        // parse class name and baseclass.
        let name = self.must_consume_identifier()?;
        let baseclass = if self.try_consume(&Lexeme::Less) {
            Some(self.must_consume_identifier()?)
        } else {
            None
        };
        // parse functions
        let mut functions = Vec::new();
        self.must_consume(&Lexeme::LeftBrace)?;
        while !self.try_consume(&Lexeme::RightBrace) {
            functions.push(self.parse_fun_item()?);
        }
        // note that right brace is already consumed in while condition.
        Ok(Declaration::Class {
            name,
            baseclass,
            functions,
        })
    }

    fn parse_fun_declaration(&mut self) -> DiagnosableResult<Declaration> {
        self.must_consume(&Lexeme::Fun)?;
        self.parse_fun_item()
    }

    // also needed by "for" statement.
    pub(super) fn parse_var_declaration(&mut self) -> DiagnosableResult<Declaration> {
        self.must_consume(&Lexeme::Var)?;
        let name = self.must_consume_identifier()?;
        let initializer = if self.try_consume(&Lexeme::Equal) {
            Some(self.parse_expression()?)
        } else {
            None
        };
        self.must_consume(&Lexeme::Semicolon)?;
        Ok(Declaration::Var { name, initializer })
    }
}

// Utility rules
type Parameters = Vec<Spanned<String>>;
type Arguments = Vec<Expression>;

// Utility functions
impl Parser {
    fn try_consume_identifier(&mut self) -> Option<Spanned<String>> {
        if let Some(Token { value, span }) = self.peek() {
            if let Lexeme::Identifier(identifier) = value {
                let identifier = Spanned::new(identifier.clone(), span.clone());
                self.advance();
                return Some(identifier);
            }
        }
        None
    }

    fn must_consume_identifier(&mut self) -> DiagnosableResult<Spanned<String>> {
        let Token { value, span } = self.must_consume(&IDENTIFIER)?;
        if let Lexeme::Identifier(identifier) = value {
            return Ok(Spanned::new(identifier.clone(), span.clone()));
        }
        unreachable!("incorrect token from must_consume")
    }

    // Parse function name, parameters and body into a function declaration.
    //
    // We need this function because methods (class functions) are pretty similar to
    // functions, except the "fun" keyword.
    fn parse_fun_item(&mut self) -> DiagnosableResult<Declaration> {
        let name = self.must_consume_identifier()?;
        self.must_consume(&Lexeme::LeftParenthesis)?;
        let parameters = self.parse_parameters()?;
        self.must_consume(&Lexeme::RightParenthesis)?;
        let body = Box::new(self.parse_block_statement()?);
        Ok(Declaration::Function {
            name,
            parameters,
            body,
        })
    }

    // Zero or more identifiers separated by comma.
    fn parse_parameters(&mut self) -> DiagnosableResult<Parameters> {
        let mut parameters = Parameters::new();
        if let Some(identifier) = self.try_consume_identifier() {
            parameters.push(identifier);
        }
        while self.try_consume(&Lexeme::Comma) {
            parameters.push(self.must_consume_identifier()?);
        }
        Ok(parameters)
    }

    // Zero or more expressions separated by comma.
    pub(super) fn parse_arguments(&mut self) -> DiagnosableResult<Arguments> {
        let mut arguments = Arguments::new();
        if let Ok(expression) = self.parse_expression() {
            arguments.push(expression);
        }
        while self.try_consume(&Lexeme::Comma) {
            arguments.push(self.parse_expression()?);
        }
        Ok(arguments)
    }
}
