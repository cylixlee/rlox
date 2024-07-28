#[derive(Clone)]
pub enum Instruction {
    LoadConstant(usize),

    /* Arithmetics */
    Add,
    Subtract,
    Multiply,
    Divide,
    Negate,
    Not,

    /* Relational */
    Greater,
    Less,
    Equal,

    /* Special literals */
    True,
    False,
    Nil,

    /* Stack operation */
    Print,
    Pop,

    /* Variable operation */
    DefineGlobal,
    GetGlobal,
    SetGlobal,
    // there's no DefineLocal because locals are "defined" when initializer expression evaluated.
    GetLocal(usize),
    SetLocal(usize),

    /* Control flow */
    JumpIfFalse(isize),
    Jump(isize),
    Call(usize),
    Return,
}
