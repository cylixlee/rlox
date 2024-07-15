use crate::ast::declaration::Declaration;
use crate::Expression;

#[derive(Debug)]
pub enum ForLoopInitializer {
    VarDeclaration(Declaration),
    VarInitialization(Expression),
}

#[derive(Debug)]
pub enum Statement {
    Expression(Expression),
    For {
        initializer: Option<ForLoopInitializer>,
        condition: Option<Expression>,
        incrementer: Option<Expression>,
        body: Box<Statement>,
    },
    If {
        condition: Expression,
        then: Box<Statement>,
        otherwise: Option<Box<Statement>>,
    },
    Print(Expression),
    Return(Option<Expression>),
    While {
        condition: Expression,
        body: Box<Statement>,
    },
    Block(Vec<Declaration>),
}
