use std::num::ParseFloatError;
use std::ops::Range;

use codespan_reporting::diagnostic::Label;
use logos::{Lexer, Logos};

use crate::{DiagnosableResult, Diagnostic};

#[rustfmt::skip]
#[derive(Logos, Debug, PartialEq)]
#[logos(skip r"[ \t\r\n]+")]
#[logos(error = Option<ParseFloatError>)]
pub enum Lexeme {
    // Operators
    #[token("(")] LeftParenthesis,
    #[token(")")] RightParenthesis,
    #[token("{")] LeftBrace,
    #[token("}")] RightBrace,
    #[token(",")] Comma,
    #[token(".")] Dot,
    #[token("-")] Minus,
    #[token("+")] Plus,
    #[token(";")] Semicolon,
    #[token("/")] Slash,
    #[token("*")] Star,
    #[token("!")] Bang,
    #[token("=")] Equal,
    #[token(">")] Greater,
    #[token("<")] Less,
    #[token("!=")] BangEqual,
    #[token("==")] EqualEqual,
    #[token(">=")] GreaterEqual,
    #[token("<=")] LessEqual,

    // Literals
    #[regex("[a-zA-Z][a-zA-Z0-9]*", scan_identifier)]
    Identifier(String),

    #[regex("\"[^\"]*\"", scan_string)]
    String(String),

    #[regex("[0-9]+(\\.[0-9]+)?", scan_number)]
    Number(f64),

    // Keywords.
    #[token("and")]    And,
    #[token("class")]  Class,
    #[token("else")]   Else,
    #[token("false")]  False,
    #[token("for")]    For,
    #[token("fun")]    Fun,
    #[token("if")]     If,
    #[token("nil")]    Nil,
    #[token("or")]     Or,
    #[token("print")]  Print,
    #[token("return")] Return,
    #[token("super")]  Super,
    #[token("this")]   This,
    #[token("true")]   True,
    #[token("var")]    Var,
    #[token("while")]  While,

    // Comments are skipped.
    #[regex("//[^\n]*", logos::skip)]
    Comment,
}

fn scan_identifier(lexer: &mut Lexer<Lexeme>) -> String {
    lexer.slice().into()
}

fn scan_string(lexer: &mut Lexer<Lexeme>) -> String {
    let slice = lexer.slice();
    (&slice[1..slice.len() - 1]).into() // Drop quotes.
}

fn scan_number(lexer: &mut Lexer<Lexeme>) -> Result<f64, ParseFloatError> {
    lexer.slice().parse::<f64>()
}

#[derive(Debug)]
pub struct Token {
    pub lexeme: Lexeme,
    pub span: Range<usize>,
}

pub fn scan(source: impl AsRef<str>) -> DiagnosableResult<Vec<Token>> {
    let lexer = Lexeme::lexer(source.as_ref());
    let mut tokens = Vec::new();
    for (lexeme, span) in lexer.spanned() {
        let lexeme = match lexeme {
            Ok(lexeme) => lexeme,
            Err(error) => {
                return match error {
                    None => Err(Diagnostic::error()
                        .with_code("E0001")
                        .with_message("Unrecognized token")
                        .with_labels(vec![
                            Label::primary((), span).with_message("invalid token encountered here")
                        ])),
                    Some(error) => Err(Diagnostic::error()
                        .with_code("E0002")
                        .with_message("Unparsable float literal")
                        .with_labels(vec![Label::primary((), span).with_message(
                            "this float value may be valid, but cannot be parsed as f64",
                        )])
                        .with_notes(vec![format!("internal reason: {}", error)])),
                }
            }
        };
        tokens.push(Token { lexeme, span })
    }
    Ok(tokens)
}
