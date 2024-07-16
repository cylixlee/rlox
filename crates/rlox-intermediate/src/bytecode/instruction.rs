#[derive(Debug)]
pub enum Instruction {
    LoadConstant(usize),

    // Arithmetics
    Add,
    Subtract,
    Multiply,
    Divide,
    Negate,
    Not,

    // Relational
    Greater,
    Less,
    Equal,

    // Special literals
    True,
    False,
    Nil,
}
