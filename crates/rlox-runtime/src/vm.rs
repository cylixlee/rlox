use rlox_intermediate::*;

use crate::heap::Heap;
use crate::stack::Stack;
use crate::value::Value;

const STACK_SIZE: usize = 4 * 1024;

pub struct VirtualMachine {
    chunk: Chunk,
    program_count: usize,
    stack: Stack<Value, STACK_SIZE>,
    heap: Heap,
}

impl VirtualMachine {
    pub fn new(chunk: Chunk) -> Self {
        Self {
            chunk,
            program_count: 0,
            stack: Stack::new(),
            heap: Heap::new(),
        }
    }

    pub fn run(&mut self) -> DiagnosableResult {
        #[cfg(feature = "stack-monitor")]
        println!("======= Stack Monitor =======");

        while self.program_count < self.chunk.len() {
            let instruction = &self.chunk[self.program_count];
            let span = self.chunk.span(self.program_count).clone();

            macro_rules! binary {
                ($variant: ident, $operator: tt) => {{
                    let right = self.stack.pop(span.clone())?;
                    let left = self.stack.pop(span.clone())?;
                    if let (Value::Number(left), Value::Number(right)) = (left, right) {
                        self.stack.push(Value::$variant(left $operator right), span)?;
                    } else {
                        raise!("E0008", span)
                    }
                }};

                (arithmetic $operator: tt) => { binary!(Number, $operator) };
                (relational $operator: tt) => { binary!(Boolean, $operator) };
            }

            #[cfg(feature = "stack-monitor")]
            println!("{:04} {:?}", self.program_count, instruction);

            match instruction {
                Instruction::LoadConstant(index) => {
                    let constant = self.chunk.constant(*index).clone();
                    match constant {
                        Constant::Number(number) => self.stack.push(Value::Number(number), span)?,
                        Constant::String(string) => unsafe {
                            let reference = self.heap.spawn_string(string);
                            self.stack.push(Value::Object(reference.cast()), span)?;
                        },
                    }
                }
                Instruction::Add => {
                    let right = self.stack.pop(span.clone())?;
                    let left = self.stack.pop(span.clone())?;
                    match (left, right) {
                        // arithmetic addition
                        (Value::Number(left), Value::Number(right)) => {
                            self.stack.push(Value::Number(left + right), span)?;
                        }
                        // string concatenation
                        (Value::Object(this), Value::Object(that)) => {
                            match (this.downcast_ref::<String>(), that.downcast_ref::<String>()) {
                                (Some(this), Some(that)) => unsafe {
                                    let reference = self.heap.spawn_string(format!("{this}{that}"));
                                    self.stack.push(Value::Object(reference.cast()), span)?;
                                },
                                _ => raise!("E0009", span),
                            }
                        }
                        _ => raise!("E0009", span),
                    }
                }
                Instruction::Subtract => binary!(arithmetic -),
                Instruction::Multiply => binary!(arithmetic *),
                Instruction::Divide => binary!(arithmetic /),
                Instruction::Negate => {
                    if let Value::Number(number) = self.stack.pop(span.clone())? {
                        self.stack.push(Value::Number(-number), span)?;
                    } else {
                        raise!("E0008", span);
                    }
                }
                Instruction::Not => {
                    let value: bool = self.stack.pop(span.clone())?.into();
                    self.stack.push(Value::Boolean(!value), span)?;
                }
                Instruction::Greater => binary!(relational >),
                Instruction::Less => binary!(relational <),
                Instruction::Equal => {
                    let right = self.stack.pop(span.clone())?;
                    let left = self.stack.pop(span.clone())?;
                    self.stack.push(Value::Boolean(left == right), span)?;
                }
                Instruction::True => self.stack.push(Value::Boolean(true), span)?,
                Instruction::False => self.stack.push(Value::Boolean(false), span)?,
                Instruction::Nil => self.stack.push(Value::Nil, span)?,
            }

            #[cfg(feature = "stack-monitor")]
            println!("{:?}", self.stack);

            self.program_count += 1;
        }

        #[cfg(feature = "stack-monitor")]
        println!();

        Ok(())
    }
}
