#![allow(dead_code)]
#![allow(unused_variables)]

pub use ast::*;
pub use bytecode::*;
pub use utility::*;

mod ast;
mod bytecode;
pub mod errors;
mod utility;

// Lox language has no multi-source support, so we'll just use
// [`codespan_reporting::files::SimpleFile`] to represent source file.
//
// Thus, the [`FileId`] is unit type because there's no need for an id;
type FileId = ();
pub type Diagnostic = codespan_reporting::diagnostic::Diagnostic<FileId>;
pub type DiagnosableResult<T = ()> = Result<T, Box<Diagnostic>>;
