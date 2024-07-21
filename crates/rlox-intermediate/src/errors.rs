use codespan_reporting::diagnostic::Label;
use phf::phf_map;

use crate::{Diagnostic, Span};

struct ErrorInfo {
    message: &'static str,
    explanation: &'static str,
}

static ERROR_TABLE: phf::Map<&'static str, ErrorInfo> = phf_map! {
    "E0001" => ErrorInfo {
        message: "Unrecognized token",
        explanation: "invalid token encountered here",
    },
    "E0002" => ErrorInfo {
        message: "Unparsable float literal",
        explanation: "this float value may be valid, but cannot be parsed as f64",
    },
    "E0003" => ErrorInfo {
        message: "Early EOF",
        explanation: "expect some token (i.g. Semicolon) after this, got EOF",
    },
    "E0004" => ErrorInfo {
        message: "Invalid prefix expression",
        explanation: "this token cannot be prefix of an expression",
    },
    "E0005" => ErrorInfo {
        message: "Unexpected token",
        explanation: "this token is shouldn't be placed here",
    },
    "E0006" => ErrorInfo {
        message: "Stack overflow",
        explanation: "this operation caused VM stack overflow",
    },
    "E0007" => ErrorInfo {
        message: "Stack underflow",
        explanation: "this operation caused VM stack underflow",
    },
    "E0008" => ErrorInfo {
        message: "Invalid arithmetic operands",
        explanation: "this operation can only be applied to numbers",
    },
    "E0009" => ErrorInfo {
        message: "Invalid addition/concatenation operands",
        explanation: "this operation can only be applied to numbers or strings",
    },
    "E0010" => ErrorInfo {
        message: "Invalid identifier",
        explanation: "this token cannot be used as an identifier",
    },
    "E0011" => ErrorInfo {
        message: "Multiple definition",
        explanation: "this variable is already defined previously",
    },
    "E0012" => ErrorInfo {
        message: "Undefined variable",
        explanation: "this variable is never defined",
    },
    "E0013" => ErrorInfo {
        message: "Invalid assignment target",
        explanation: "the left operand is not assignable",
    },
};

pub fn raise(error_code: &'static str, span: Span) -> Diagnostic {
    let info = &ERROR_TABLE[error_code];
    Diagnostic::error()
        .with_code(error_code)
        .with_message(info.message)
        .with_labels(vec![Label::primary((), span).with_message(info.explanation)])
}

#[macro_export]
macro_rules! raise {
    ($error_code: expr, $span: expr) => {
        return Err(Box::new($crate::errors::raise($error_code, $span)))
    };

    ($error_code: expr, $span: expr, $($notes: expr), + $(,)?) => {
        return Err(Box::new($crate::errors::raise($error_code, $span).with_notes(vec![$($notes), +])))
    };
}
