use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::mem;
use std::ops::Deref;

use crate::{Instruction, Span};
use crate::bytecode::backpatcher::{Backpatch, JumpBackpatcher, JumpIfFalseBackpatcher};

#[derive(Debug, Clone)]
pub enum Constant {
    Number(f64),
    String(String),
}

impl PartialEq for Constant {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Constant::Number(this), Constant::Number(that)) => (this - that).abs() < f64::EPSILON,
            (Constant::String(this), Constant::String(that)) => this == that,
            _ => false,
        }
    }
}

impl Eq for Constant {}

impl Hash for Constant {
    fn hash<H: Hasher>(&self, state: &mut H) {
        mem::discriminant(self).hash(state);
        match self {
            Constant::Number(number) => number.to_bits().hash(state),
            Constant::String(string) => string.hash(state),
        }
    }
}

pub struct ChunkBuilder {
    pub instructions: Vec<Instruction>,
    pub spans: Vec<Span>,
    pub constants: Vec<Constant>,
    constants_cache: HashMap<Constant, usize>,
}

impl ChunkBuilder {
    pub fn new() -> Self {
        Self {
            instructions: Vec::new(),
            spans: Vec::new(),
            constants: Vec::new(),
            constants_cache: HashMap::new(),
        }
    }

    pub fn build(self) -> Chunk {
        Chunk {
            instructions: self.instructions,
            spans: self.spans,
            constants: self.constants,
        }
    }

    pub fn write(&mut self, instruction: Instruction, span: Span) {
        self.instructions.push(instruction);
        self.spans.push(span);
    }

    pub fn append_backpatch(&mut self, instruction: Instruction) -> Box<dyn Backpatch> {
        let index = self.instructions.len();
        self.append(instruction.clone());
        match instruction {
            Instruction::JumpIfFalse(_) => {
                Box::new(JumpIfFalseBackpatcher::new(&mut self.instructions, index))
            }
            Instruction::Jump(_) => Box::new(JumpBackpatcher::new(&mut self.instructions, index)),
            _ => unreachable!(),
        }
    }

    pub fn append(&mut self, instruction: Instruction) {
        let span = self.spans.last().unwrap().clone();
        self.write(instruction, span);
    }

    pub fn define(&mut self, constant: Constant) -> usize {
        if let Some(index) = self.constants_cache.get(&constant) {
            return *index;
        }
        let index = self.constants.len();
        self.constants_cache.insert(constant.clone(), index);
        self.constants.push(constant);
        index
    }
}

pub struct Chunk {
    instructions: Vec<Instruction>,
    spans: Vec<Span>,
    constants: Vec<Constant>,
}

impl Chunk {
    pub fn len(&self) -> usize {
        self.instructions.len()
    }

    pub fn is_empty(&self) -> bool {
        self.instructions.is_empty()
    }

    pub fn spans(&self) -> &Vec<Span> {
        &self.spans
    }

    pub fn constants(&self) -> &Vec<Constant> {
        &self.constants
    }

    pub fn span(&self, index: usize) -> &Span {
        &self.spans[index]
    }

    pub fn constant(&self, index: usize) -> &Constant {
        &self.constants[index]
    }
}

impl Deref for Chunk {
    type Target = [Instruction];

    fn deref(&self) -> &Self::Target {
        &self.instructions
    }
}
