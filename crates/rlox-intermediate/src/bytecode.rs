use std::collections::HashMap;
use std::rc::Rc;

pub use chunk::*;
pub use instruction::*;

mod backpatcher;
mod chunk;
mod instruction;

pub struct Function {
    pub chunk: Rc<Chunk>,
    pub arity: usize,
}

pub struct Bytecode {
    pub functions: HashMap<String, Function>,
    pub script: Rc<Chunk>,
}
