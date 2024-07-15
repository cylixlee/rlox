pub use expression::*;
pub use utility::*;

pub mod errors;
mod expression;
mod utility;

// Lox language has no multi-source support, so we'll just use
// [`codespan_reporting::files::SimpleFile`] to represent source file.
//
// Thus, the [`FileId`] is unit type because there's no need for an id;
pub type FileId = ();
pub type Diagnostic = codespan_reporting::diagnostic::Diagnostic<FileId>;
pub type DiagnosableResult<T = ()> = Result<T, Diagnostic>;
