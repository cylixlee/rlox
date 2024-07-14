use std::mem;

use codespan_reporting::diagnostic::Label;

use rlox_intermediate::{BinaryOperator, Expression, Literal, Spanned, UnaryOperator};

use crate::{DiagnosableResult, Diagnostic};
use crate::parser::Parser;
use crate::scanner::{Lexeme, Token};

#[repr(u8)]
#[derive(Ord, PartialOrd, Eq, PartialEq, Copy, Clone)]
enum InfixPrecedence {
    None,
    Assignment,     // right associative
    ConditionalOr,  // left associative
    ConditionalAnd, // left associative
    Equality,       // left associative
    Relational,     // left associative
    Additive,       // left associative
    Multiplicative, // left associative
    Property,       // left associative
    Impossible,
}

impl InfixPrecedence {
    pub fn increase(&self) -> Self {
        let number = *self as u8;
        if number < Self::Impossible as u8 {
            unsafe { mem::transmute(number + 1) }
        } else {
            Self::Impossible
        }
    }
}

impl Lexeme {
    fn precedence(&self) -> InfixPrecedence {
        match self {
            Lexeme::Equal => InfixPrecedence::Assignment,
            Lexeme::Or => InfixPrecedence::ConditionalOr,
            Lexeme::And => InfixPrecedence::ConditionalAnd,
            Lexeme::EqualEqual | Lexeme::BangEqual => InfixPrecedence::Equality,
            Lexeme::Greater | Lexeme::GreaterEqual | Lexeme::Less | Lexeme::LessEqual => {
                InfixPrecedence::Relational
            }
            Lexeme::Plus | Lexeme::Minus => InfixPrecedence::Additive,
            Lexeme::Star | Lexeme::Slash => InfixPrecedence::Multiplicative,
            Lexeme::Dot => InfixPrecedence::Property,
            _ => InfixPrecedence::None,
        }
    }
}

impl Parser {
    pub fn parse_expression(&mut self) -> DiagnosableResult<Expression> {
        self.parse_precedence(InfixPrecedence::Assignment)
    }

    fn parse_precedence(&mut self, precedence: InfixPrecedence) -> DiagnosableResult<Expression> {
        // parse prefix expressions.
        let Token { value, span } = self.must_peek()?.clone();
        macro_rules! literal {
            ($variant: ident) => {{
                self.must_advance()?;
                Expression::Literal(Spanned::new(Literal::$variant, span))
            }};
            ($variant: ident, $value: expr) => {{
                self.must_advance()?;
                Expression::Literal(Spanned::new(Literal::$variant($value), span))
            }};
        }
        let mut expression = match value {
            // literal
            Lexeme::Nil => literal!(Nil),
            Lexeme::True => literal!(Boolean, true),
            Lexeme::False => literal!(Boolean, false),
            Lexeme::Number(number) => literal!(Number, number),
            Lexeme::String(string) => literal!(String, string),
            Lexeme::Identifier(identifier) => literal!(Identifier, identifier),
            // parenthesized
            Lexeme::LeftParenthesis => self.parse_parenthesized()?,
            // unary
            Lexeme::Bang | Lexeme::Minus => self.parse_unary()?,
            _ => {
                return Err(Diagnostic::error()
                    .with_code("E0004")
                    .with_message("Invalid prefix expression")
                    .with_labels(vec![Label::primary((), span.clone())
                        .with_message("this token cannot be prefix of an expression")]))
            }
        };

        while let Some(infix) = self.peek() {
            if infix.precedence() < precedence {
                break;
            }
            expression = self.parse_binary(expression)?;
        }
        Ok(expression)
    }

    fn parse_parenthesized(&mut self) -> DiagnosableResult<Expression> {
        self.must_consume(&Lexeme::LeftParenthesis)?;
        let expression = self.parse_expression()?;
        self.must_consume(&Lexeme::RightParenthesis)?;
        Ok(expression)
    }

    fn parse_unary(&mut self) -> DiagnosableResult<Expression> {
        #[rustfmt::skip]
        let Token { value: operator, span } = self.must_advance()?.clone();
        let expression = self.parse_precedence(InfixPrecedence::Property)?;
        Ok(match operator {
            Lexeme::Bang => Expression::Unary {
                operator: Spanned::new(UnaryOperator::Not, span.clone()),
                expression: Box::new(expression),
            },
            Lexeme::Minus => Expression::Unary {
                operator: Spanned::new(UnaryOperator::Negate, span.clone()),
                expression: Box::new(expression),
            },
            _ => unreachable!("incorrect unary operator forwarded from precedence parsing"),
        })
    }

    fn parse_binary(&mut self, left: Expression) -> DiagnosableResult<Expression> {
        #[rustfmt::skip]
        let Token { value: operator, span } = self.must_advance()?.clone();
        let precedence = operator.precedence();
        let expression = match operator {
            Lexeme::Equal => self.parse_precedence(precedence),
            _ => self.parse_precedence(precedence.increase()),
        }?;
        let operator = match operator {
            Lexeme::Equal => BinaryOperator::Assign,
            Lexeme::Or => BinaryOperator::Or,
            Lexeme::And => BinaryOperator::And,
            Lexeme::EqualEqual => BinaryOperator::Equal,
            Lexeme::BangEqual => BinaryOperator::NotEqual,
            Lexeme::Greater => BinaryOperator::Greater,
            Lexeme::GreaterEqual => BinaryOperator::GreaterEqual,
            Lexeme::Less => BinaryOperator::Less,
            Lexeme::LessEqual => BinaryOperator::LessEqual,
            Lexeme::Plus => BinaryOperator::Add,
            Lexeme::Minus => BinaryOperator::Subtract,
            Lexeme::Star => BinaryOperator::Multiply,
            Lexeme::Slash => BinaryOperator::Divide,
            Lexeme::Dot => BinaryOperator::PropertyAccess,
            _ => unreachable!("incorrect binary operator forwarded from precedence parsing"),
        };
        Ok(Expression::Binary {
            left: Box::new(left),
            operator: Spanned::new(operator, span.clone()),
            right: Box::new(expression),
        })
    }
}
