use std::collections::HashMap;
use std::ops::Deref;

use crate::{Instruction, Span, Value};
use crate::bytecode::backpatcher::{Backpatch, JumpBackpatcher, JumpIfFalseBackpatcher};

pub struct ChunkBuilder {
    pub instructions: Vec<Instruction>,
    pub spans: Vec<Span>,
    pub constants: Vec<Value>,
    constants_cache: HashMap<Value, usize>,
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
        if let Some(span) = self.spans.last() {
            self.write(instruction, span.clone());
        } else {
            self.write(instruction, Default::default())
        }
    }

    pub fn define(&mut self, constant: Value) -> usize {
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
    constants: Vec<Value>,
}

impl Chunk {
    pub fn len(&self) -> usize {
        self.instructions.len()
    }

    pub fn spans(&self) -> &Vec<Span> {
        &self.spans
    }

    pub fn constants(&self) -> &Vec<Value> {
        &self.constants
    }

    pub fn span(&self, index: usize) -> &Span {
        &self.spans[index]
    }

    pub fn constant(&self, index: usize) -> &Value {
        &self.constants[index]
    }
}

impl Deref for Chunk {
    type Target = [Instruction];

    fn deref(&self) -> &Self::Target {
        &self.instructions
    }
}

#[cfg(feature = "disassembler")]
mod disassembler {
    use std::fmt::Display;

    use crate::{Chunk, Instruction};

    impl Chunk {
        pub fn disassemble(&self, title: impl Display) {
            println!("== {} ==", title);
            for index in 0..self.instructions.len() {
                self.disassemble_instruction(index);
            }
        }

        pub fn disassemble_instruction(&self, index: usize) {
            print!("{index:04} ");

            let instruction = &self.instructions[index];
            match instruction {
                Instruction::LoadConstant(index) => {
                    constant_instruction("LoadConstant", self, index);
                }
                Instruction::Add => println!("Add"),
                Instruction::Subtract => println!("Subtract"),
                Instruction::Multiply => println!("Multiply"),
                Instruction::Divide => println!("Divide"),
                Instruction::Negate => println!("Negate"),
                Instruction::Not => println!("Not"),
                Instruction::Greater => println!("Greater"),
                Instruction::Less => println!("Less"),
                Instruction::Equal => println!("Equal"),
                Instruction::True => println!("True"),
                Instruction::False => println!("False"),
                Instruction::Nil => println!("Nil"),
                Instruction::Print => println!("Print"),
                Instruction::Pop => println!("Pop"),
                Instruction::DefineGlobal => println!("DefineGlobal"),
                Instruction::GetGlobal => println!("GetGlobal"),
                Instruction::SetGlobal => println!("SetGlobal"),
                Instruction::GetLocal(index) => index_instruction("GetLocal", index),
                Instruction::SetLocal(index) => index_instruction("SetLocal", index),
                Instruction::JumpIfFalse(offset) => offset_instruction("JumpIfFalse", offset),
                Instruction::Jump(offset) => offset_instruction("Jump", offset),
                Instruction::Return => println!("Return"),
            }
        }
    }

    fn constant_instruction(name: impl Display, chunk: &Chunk, index: &usize) {
        let constant = chunk.constant(*index);
        println!("{name:<16} {index:4} {constant:?}");
    }

    fn index_instruction(name: impl Display, index: &usize) {
        println!("{name:<16} {index:4}");
    }

    fn offset_instruction(name: impl Display, offset: &isize) {
        println!("{name:<16} {offset:4}");
    }
}
