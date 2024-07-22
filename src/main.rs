#![allow(dead_code)]
#![allow(unused_variables)]

use std::fmt::Display;
use std::io;
use std::io::Write;
use std::ops::Deref;

use mimalloc::MiMalloc;

use rlox_analyzer::{compiler, parser, scanner};
use rlox_intermediate::*;

#[global_allocator]
static ALLOCATOR: MiMalloc = MiMalloc;

fn main() {
    let mut buffer = String::new();
    loop {
        print!(">> ");
        io::stdout().flush().unwrap();
        io::stdin().read_line(&mut buffer).unwrap();
        if buffer.trim().is_empty() {
            break;
        }
        let mut source = DiagnosableSource::new("<script>", &buffer);
        if let Err(diagnostic) = diagnosable_main(&source) {
            source.diagnose(&diagnostic);
        }
        buffer.clear();
    }
}

fn diagnosable_main<N, S>(source: &DiagnosableSource<N, S>) -> DiagnosableResult
where
    N: Display + Clone,
    S: AsRef<str>,
{
    let tokens = scanner::scan(source.deref())?;
    let declarations = parser::parse(tokens)?;
    let mut heap = Heap::new();
    let function = compiler::compile(&mut heap, declarations)?;
    Ok(())
}
