use crate::{Span, Spanned};

#[derive(Debug)]
pub enum Literal {
    Nil,
    Boolean(bool),
    Number(f64),
    String(String),
    Identifier(String),
    This,
    Super,
}

#[derive(Debug)]
pub enum UnaryOperator {
    Not,
    Negate,
}

#[derive(Debug)]
pub enum BinaryOperator {
    Assign,         // precedence: Assignment
    Or,             // precedence: Conditional Or
    And,            // precedence: Conditional And
    Equal,          // precedence: Equality
    NotEqual,       // precedence: Equality
    Greater,        // precedence: Relational
    GreaterEqual,   // precedence: Relational
    Less,           // precedence: Relational
    LessEqual,      // precedence: Relational
    Add,            // precedence: Additive
    Subtract,       // precedence: Additive
    Multiply,       // precedence: Multiplicative
    Divide,         // precedence: Multiplicative
    PropertyAccess, // precedence: Property
}

#[derive(Debug)]
pub enum Expression {
    Assignment {
        left: Box<Expression>,
        span: Span,
        right: Box<Expression>,
    },
    Binary {
        left: Box<Expression>,
        operator: Spanned<BinaryOperator>,
        right: Box<Expression>,
    },
    Unary {
        operator: Spanned<UnaryOperator>,
        expression: Box<Expression>,
    },
    Invocation {
        expression: Box<Expression>,
        arguments: Vec<Expression>,
    },
    Literal(Spanned<Literal>),
}
