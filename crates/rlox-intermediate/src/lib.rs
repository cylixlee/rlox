#![allow(dead_code)]
#![allow(unused_variables)]

pub use ast::*;
pub use bytecode::*;
pub use heap::*;
pub use utility::*;
pub use value::*;

mod ast;
mod bytecode;
pub mod errors;
mod heap;
mod utility;
mod value;

// Lox language has no multi-source support, so we'll just use
// [`codespan_reporting::files::SimpleFile`] to represent source file.
//
// Thus, the [`FileId`] is unit type because there's no need for an id;
type FileId = ();
pub type Diagnostic = codespan_reporting::diagnostic::Diagnostic<FileId>;
pub type DiagnosableResult<T = ()> = Result<T, Box<Diagnostic>>;
