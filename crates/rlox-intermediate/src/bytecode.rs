use std::collections::HashMap;

pub use chunk::*;
pub use instruction::*;

mod backpatcher;
mod chunk;
mod instruction;

pub struct Function {
    pub chunk: Chunk,
    pub arity: usize,
}

pub struct Bytecode {
    pub functions: HashMap<String, Function>,
    pub script: Chunk,
}
