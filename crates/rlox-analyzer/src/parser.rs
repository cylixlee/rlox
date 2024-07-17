use rlox_intermediate::*;

use crate::scanner::Token;

mod declaration;
mod expression;
mod statement;
mod utility;

struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

pub fn parse(tokens: Vec<Token>) -> DiagnosableResult<Vec<Declaration>> {
    let mut parser = Parser::new(tokens);
    let mut declarations = Vec::new();
    while !parser.has_reached_end() {
        declarations.push(parser.parse_declaration()?);
    }
    Ok(declarations)
}
