use std::ops::Index;

use getset::Getters;

use crate::{Instruction, Span};

#[derive(Debug)]
pub enum Constant {
    Number(f64),
    String(String),
}

#[derive(Getters)]
#[rustfmt::skip]
pub struct Chunk {
    #[getset(get = "pub")] instructions: Vec<Instruction>,
    #[getset(get = "pub")] spans: Vec<Span>,
    #[getset(get = "pub")] constants: Vec<Constant>,
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            instructions: Vec::new(),
            spans: Vec::new(),
            constants: Vec::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.instructions.len()
    }

    pub fn write(&mut self, instruction: Instruction, span: Span) {
        self.instructions.push(instruction);
        self.spans.push(span);
    }

    pub fn define(&mut self, constant: Constant) -> usize {
        self.constants.push(constant);
        self.constants.len() - 1
    }
}

impl Index<usize> for Chunk {
    type Output = Instruction;

    fn index(&self, index: usize) -> &Self::Output {
        &self.instructions[index]
    }
}
