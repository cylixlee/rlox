use rlox_intermediate::{Chunk, DiagnosableResult, Instruction, raise};

use crate::stack::Stack;
use crate::value::Value;

const STACK_SIZE: usize = 4 * 1024;

pub struct VirtualMachine {
    chunk: Chunk,
    program_count: usize,
    stack: Stack<Value, STACK_SIZE>,
}

impl VirtualMachine {
    pub fn new(chunk: Chunk) -> Self {
        Self {
            chunk,
            program_count: 0,
            stack: Stack::new(),
        }
    }

    pub fn run(&mut self) -> DiagnosableResult {
        macro_rules! arithmetic {
            ($operator: tt) => {{
                let span = self.chunk.span(self.program_count);
                let right = self.stack.pop(span.clone())?;
                let left = self.stack.pop(span.clone())?;
                if let (Value::Number(left), Value::Number(right)) = (left, right) {
                    self.stack.push(Value::Number(left $operator right), span.clone())?;
                } else {
                    raise!("E0008", span.clone())
                }
            }};
        }

        #[cfg(feature = "stack-monitor")]
        println!("======= Stack Monitor =======");

        while self.program_count < self.chunk.len() {
            let instruction = &self.chunk[self.program_count];

            #[cfg(feature = "stack-monitor")]
            println!("{:04} {:?}", self.program_count, instruction);

            match instruction {
                Instruction::LoadConstant(index) => {
                    self.stack.push(
                        self.chunk.constant(*index).clone().into(),
                        self.chunk.span(*index).clone(),
                    )?;
                }
                Instruction::Add => arithmetic!(+),
                Instruction::Subtract => arithmetic!(-),
                Instruction::Multiply => arithmetic!(*),
                Instruction::Divide => arithmetic!(/),
            }

            #[cfg(feature = "stack-monitor")]
            println!("{:?}", self.stack);

            self.program_count += 1;
        }
        Ok(())
    }
}
