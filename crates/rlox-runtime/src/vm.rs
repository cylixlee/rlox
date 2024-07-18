use std::collections::HashMap;
use std::ops::Deref;

use rlox_intermediate::*;

use crate::heap::Heap;
use crate::stack::Stack;
use crate::value::Value;

const STACK_SIZE: usize = 1024;

pub struct VirtualMachine {
    chunk: Chunk,
    program_count: usize,
    stack: Stack<Value, STACK_SIZE>,
    heap: Heap,
    globals: HashMap<String, Value>,
}

impl VirtualMachine {
    pub fn new(chunk: Chunk) -> Self {
        Self {
            chunk,
            program_count: 0,
            stack: Stack::new(),
            heap: Heap::new(),
            globals: HashMap::new(),
        }
    }

    pub fn run(&mut self) -> DiagnosableResult {
        #[cfg(feature = "stack-monitor")]
        println!("━━━━━━━ Stack Monitor ━━━━━━━");

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
                        Constant::String(string) => {
                            let reference = self.heap.spawn_string(string);
                            self.stack.push(Value::String(reference), span)?;
                        }
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
                        (Value::String(this), Value::String(that)) => {
                            let reference =
                                self.heap
                                    .spawn_string(format!("{}{}", this.deref(), that.deref()));
                            self.stack.push(Value::String(reference), span)?;
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
                    let value: bool = self.stack.pop(span.clone())?.boolean();
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
                Instruction::Print => {
                    let value = self.stack.pop(span)?;
                    println!("{value}");
                }
                Instruction::Pop => {
                    self.stack.pop(span)?;
                }
                Instruction::DefineGlobal => {
                    let name = match self.stack.pop(span.clone())? {
                        Value::String(identifier) => identifier,
                        _ => raise!("E0010", span),
                    };
                    let value = self.stack.pop(span.clone())?;
                    if self.globals.contains_key(name.deref()) {
                        raise!("E0011", span);
                    }
                    self.globals.insert(name.deref().clone(), value);
                }
                Instruction::GetGlobal => {
                    let name = match self.stack.pop(span.clone())? {
                        Value::String(identifier) => identifier,
                        _ => raise!("E0010", span),
                    };
                    if let Some(value) = self.globals.get(name.deref()) {
                        self.stack.push(value.clone(), span)?;
                    } else {
                        raise!("E0012", span);
                    }
                }
                Instruction::SetGlobal => {
                    let name = match self.stack.pop(span.clone())? {
                        Value::String(identifier) => identifier,
                        _ => raise!("E0010", span),
                    };
                    let value = self.stack.top(span.clone())?.clone();
                    if let Some(variable) = self.globals.get_mut(name.deref()) {
                        *variable = value;
                    } else {
                        raise!("E0012", span);
                    }
                }
                Instruction::GetLocal(index) => {
                    self.stack.push(self.stack[*index].clone(), span)?
                }
                Instruction::SetLocal(index) => {
                    let value = self.stack.top(span)?.clone();
                    self.stack[*index] = value;
                }
                Instruction::JumpIfFalse(offset) => {
                    let condition: bool = self.stack.top(span)?.boolean();
                    if !condition {
                        self.program_count = (self.program_count as isize + offset) as usize;
                        continue;
                    }
                }
                Instruction::Jump(offset) => {
                    self.program_count = (self.program_count as isize + offset) as usize;
                    continue;
                }
            }

            #[cfg(feature = "stack-monitor")]
            if !self.stack.is_empty() {
                println!("{:?}", self.stack);
            }

            self.program_count += 1;
        }

        #[cfg(feature = "stack-monitor")]
        println!();

        Ok(())
    }
}
