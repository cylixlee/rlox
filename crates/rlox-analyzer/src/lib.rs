use std::fmt::Display;
use std::ops::Deref;

use codespan_reporting::files::SimpleFile;
use codespan_reporting::term;
use codespan_reporting::term::Config;
use codespan_reporting::term::termcolor::{ColorChoice, StandardStream};

pub mod scanner;

// Lox language has no multi-source support, so we'll just use
// [`codespan_reporting::files::SimpleFile`] to represent source file.
//
// Thus, the [`FileId`] is unit type because there's no need for an id;
type FileId = ();

// Type aliases is for convenience.
type Diagnostic = codespan_reporting::diagnostic::Diagnostic<FileId>;
type DiagnosableResult<T = ()> = Result<T, Diagnostic>;

pub struct DiagnosableSource<N, S> {
    writer: StandardStream,
    config: Config,
    file: SimpleFile<N, S>,
}

impl<N, S> DiagnosableSource<N, S>
where
    N: Display + Clone,
    S: AsRef<str>,
{
    pub fn new(name: N, source: S) -> Self {
        Self {
            writer: StandardStream::stderr(ColorChoice::Always),
            config: Config::default(),
            file: SimpleFile::new(name, source),
        }
    }

    pub fn diagnose(&mut self, diagnostic: &Diagnostic) {
        term::emit(
            &mut self.writer.lock(),
            &self.config,
            &self.file,
            diagnostic,
        )
        .unwrap();
    }
}

impl<N, S> Deref for DiagnosableSource<N, S>
where
    N: Display,
    S: AsRef<str>,
{
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.file.source().as_ref()
    }
}
