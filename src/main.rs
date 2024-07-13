use rlox_analyzer::{DiagnosableSource, scanner};

fn main() {
    let mut source = DiagnosableSource::new("<script>", "11.4?514!");
    match scanner::scan(&*source) {
        Ok(tokens) => {
            for token in &tokens {
                println!("{:?}", token);
            }
        }
        Err(diagnostic) => source.diagnose(&diagnostic),
    }
}
