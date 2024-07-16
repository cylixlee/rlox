#[derive(Debug)]
pub enum Instruction {
    LoadConstant(usize),
    Add,
    Subtract,
    Multiply,
    Divide,
}
