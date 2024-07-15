use crate::{Expression, Spanned, Statement};

#[derive(Debug)]
pub enum Declaration {
    Class {
        name: Spanned<String>,
        baseclass: Option<Spanned<String>>,
        functions: Vec<Declaration>,
    },
    Function {
        name: Spanned<String>,
        parameters: Vec<Spanned<String>>,
        body: Box<Statement>,
    },
    Var {
        name: Spanned<String>,
        initializer: Option<Expression>,
    },
}
