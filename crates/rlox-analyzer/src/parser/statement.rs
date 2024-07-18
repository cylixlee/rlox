use rlox_intermediate::*;

use crate::parser::Parser;
use crate::scanner::Lexeme;

impl Parser {
    pub fn parse_statement(&mut self) -> DiagnosableResult<Statement> {
        Ok(match &self.must_peek()?.value {
            Lexeme::For => self.parse_for_statement()?,
            Lexeme::If => self.parse_if_statement()?,
            Lexeme::Print => self.parse_print_statement()?,
            Lexeme::Return => self.parse_return_statement()?,
            Lexeme::While => self.parse_while_statement()?,
            Lexeme::LeftBrace => self.parse_block_statement()?,
            _ => self.parse_expression_statement()?,
        })
    }

    fn parse_expression_statement(&mut self) -> DiagnosableResult<Statement> {
        let expression = self.parse_expression()?;
        self.must_consume(&Lexeme::Semicolon)?;
        Ok(Statement::Expression(expression))
    }

    // very long because all 3 items in for-loop (initializer, condition and
    // incrementer) are optional.
    fn parse_for_statement(&mut self) -> DiagnosableResult<Statement> {
        self.must_consume(&Lexeme::For)?;
        self.must_consume(&Lexeme::LeftParenthesis)?;
        // parse initializer
        let initializer: Option<ForLoopInitializer>;
        if !self.try_consume(&Lexeme::Semicolon) {
            if let Lexeme::Var = self.must_peek()?.value {
                initializer = Some(ForLoopInitializer::VarDeclaration(Box::new(
                    self.parse_var_declaration()?,
                )));
                // a little bit lazy here: parsing var declaration will consume the
                // following semicolon.
            } else {
                initializer = Some(ForLoopInitializer::VarInitialization(Box::new(
                    self.parse_expression()?,
                )));
                self.must_consume(&Lexeme::Semicolon)?;
            }
        } else {
            initializer = None;
        }
        // parse condition
        let condition: Option<Expression>;
        if !self.try_consume(&Lexeme::Semicolon) {
            condition = Some(self.parse_expression()?);
            self.must_consume(&Lexeme::Semicolon)?;
        } else {
            condition = None;
        }
        // parse incrementer
        let incrementer: Option<Expression>;
        if !self.try_consume(&Lexeme::RightParenthesis) {
            incrementer = Some(self.parse_expression()?);
            self.must_consume(&Lexeme::RightParenthesis)?;
        } else {
            incrementer = None;
        }
        let body = Box::new(self.parse_statement()?);
        Ok(Statement::For {
            initializer,
            condition,
            incrementer,
            body,
        })
    }

    fn parse_if_statement(&mut self) -> DiagnosableResult<Statement> {
        self.must_consume(&Lexeme::If)?;
        self.must_consume(&Lexeme::LeftParenthesis)?;
        let condition = self.parse_expression()?;
        self.must_consume(&Lexeme::RightParenthesis)?;
        let then = Box::new(self.parse_statement()?);
        let otherwise = if self.try_consume(&Lexeme::Else) {
            Some(Box::new(self.parse_statement()?))
        } else {
            None
        };
        Ok(Statement::If {
            condition,
            then,
            otherwise,
        })
    }

    fn parse_print_statement(&mut self) -> DiagnosableResult<Statement> {
        self.must_consume(&Lexeme::Print)?;
        let expression = self.parse_expression()?;
        self.must_consume(&Lexeme::Semicolon)?;
        Ok(Statement::Print(expression))
    }

    fn parse_return_statement(&mut self) -> DiagnosableResult<Statement> {
        self.must_consume(&Lexeme::Return)?;
        let expression = if self.try_consume(&Lexeme::Semicolon) {
            None
        } else {
            Some(self.parse_expression()?)
        };
        self.must_consume(&Lexeme::Semicolon)?;
        Ok(Statement::Return(expression))
    }

    fn parse_while_statement(&mut self) -> DiagnosableResult<Statement> {
        self.must_consume(&Lexeme::While)?;
        self.must_consume(&Lexeme::LeftParenthesis)?;
        let condition = self.parse_expression()?;
        self.must_consume(&Lexeme::RightParenthesis)?;
        let body = Box::new(self.parse_statement()?);
        Ok(Statement::While { condition, body })
    }

    // also needed by function declaration.
    pub(super) fn parse_block_statement(&mut self) -> DiagnosableResult<Statement> {
        self.must_consume(&Lexeme::LeftBrace)?;
        let mut declarations = Vec::new();
        while !self.try_consume(&Lexeme::RightBrace) {
            declarations.push(self.parse_declaration()?);
        }
        // note that right brace is consumed in while condition.
        Ok(Statement::Block(declarations))
    }
}
