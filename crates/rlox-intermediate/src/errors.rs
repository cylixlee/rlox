use std::ops::Range;

use codespan_reporting::diagnostic::Label;
use phf::phf_map;

use crate::Diagnostic;

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
        explanation: "expect some token or Semicolon after this, got EOF",
    },
    "E0004" => ErrorInfo {
        message: "Invalid prefix expression",
        explanation: "this token cannot be prefix of an expression",
    },
    "E0005" => ErrorInfo {
        message: "Unexpected token",
        explanation: "this token is shouldn't be placed here",
    },
};

pub fn raise(error_code: &'static str, span: Range<usize>) -> Diagnostic {
    let info = &ERROR_TABLE[error_code];
    Diagnostic::error()
        .with_code(error_code)
        .with_message(info.message)
        .with_labels(vec![Label::primary((), span).with_message(info.explanation)])
}

#[macro_export]
macro_rules! raise {
    ($error_code: expr, $span: expr) => {
        return Err($crate::errors::raise($error_code, $span))
    };

    ($error_code: expr, $span: expr, $($notes: expr), + $(,)?) => {
        return Err($crate::errors::raise($error_code, $span).with_notes(vec![$($notes), +]))
    };
}
