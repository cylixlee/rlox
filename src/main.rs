use std::fmt::Display;
use std::io;

use rlox_analyzer::{parser, scanner};
use rlox_intermediate::{DiagnosableResult, DiagnosableSource};

fn main() {
    let mut buffer = String::new();
    loop {
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
    let tokens = scanner::scan(&**source)?;
    let expression = parser::parse_expression(tokens)?;
    println!("{:#?}", expression);
    Ok(())
}
