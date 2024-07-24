use std::collections::HashMap;
use std::ops::Deref;

use smallvec::SmallVec;

use rlox_intermediate::*;

const STACK_SIZE: usize = 4 * 1024;

pub struct VirtualMachine {
    chunk: Chunk,
    program_count: usize,
    stack: SmallVec<[Value; STACK_SIZE]>,
    heap: Heap,
    globals: HashMap<String, Value>,
}

impl VirtualMachine {
    pub fn new(chunk: Chunk) -> Self {
        Self {
            chunk,
            program_count: 0,
            stack: SmallVec::new(),
            heap: Heap::new(),
            globals: HashMap::new(),
        }
    }

    pub fn run(&mut self) -> DiagnosableResult {
        #[cfg(feature = "stack-monitor")]
        println!("━━━━━━━ Stack Monitor ━━━━━━━");

        loop {
            let instruction = &self.chunk[self.program_count];
            let span = self.chunk.span(self.program_count).clone();

            macro_rules! binary {
                ($variant: ident, $operator: tt) => {{
                    let right = self.stack.pop().unwrap();
                    let left = self.stack.pop().unwrap();
                    if let (Value::Number(left), Value::Number(right)) = (left, right) {
                        self.stack.push(Value::$variant(left $operator right));
                    } else {
                        raise!("E0008", span)
                    }
                }};

                (arithmetic $operator: tt) => { binary!(Number, $operator) };
                (relational $operator: tt) => { binary!(Boolean, $operator) };
            }

            // TODO: new stack-monitor code here...
            // #[cfg(feature = "stack-monitor")]
            // println!("{:04} {:?}", self.program_count, instruction);

            match instruction {
                Instruction::LoadConstant(index) => {
                    let value = self.chunk.constant(*index).clone();
                    match value {
                        Value::String(string) => {
                            let reference = self.heap.spawn_string(string.deref().clone());
                            self.stack.push(Value::String(reference));
                        }
                        _ => self.stack.push(value),
                    }
                }
                Instruction::Add => {
                    let right = self.stack.pop().unwrap();
                    let left = self.stack.pop().unwrap();
                    match (left, right) {
                        // arithmetic addition
                        (Value::Number(left), Value::Number(right)) => {
                            self.stack.push(Value::Number(left + right));
                        }
                        // string concatenation
                        (Value::String(this), Value::String(that)) => {
                            let reference =
                                self.heap
                                    .spawn_string(format!("{}{}", this.deref(), that.deref()));
                            self.stack.push(Value::String(reference));
                        }
                        _ => raise!("E0009", span),
                    }
                }
                Instruction::Subtract => binary!(arithmetic -),
                Instruction::Multiply => binary!(arithmetic *),
                Instruction::Divide => binary!(arithmetic /),
                Instruction::Negate => {
                    if let Value::Number(number) = self.stack.pop().unwrap() {
                        self.stack.push(Value::Number(-number));
                    } else {
                        raise!("E0008", span);
                    }
                }
                Instruction::Not => {
                    let value: bool = self.stack.pop().unwrap().boolean();
                    self.stack.push(Value::Boolean(!value));
                }
                Instruction::Greater => binary!(relational >),
                Instruction::Less => binary!(relational <),
                Instruction::Equal => {
                    let right = self.stack.pop().unwrap();
                    let left = self.stack.pop().unwrap();
                    self.stack.push(Value::Boolean(left == right));
                }
                Instruction::True => self.stack.push(Value::Boolean(true)),
                Instruction::False => self.stack.push(Value::Boolean(false)),
                Instruction::Nil => self.stack.push(Value::Nil),
                Instruction::Print => {
                    let value = self.stack.pop().unwrap();
                    println!("{value}");
                }
                Instruction::Pop => {
                    self.stack.pop().unwrap();
                }
                Instruction::DefineGlobal => {
                    let name = match self.stack.pop().unwrap() {
                        Value::String(identifier) => identifier,
                        _ => raise!("E0010", span),
                    };
                    let value = self.stack.pop().unwrap();
                    if self.globals.contains_key(name.deref()) {
                        raise!("E0011", span);
                    }
                    self.globals.insert(name.deref().clone(), value);
                }
                Instruction::GetGlobal => {
                    let name = match self.stack.pop().unwrap() {
                        Value::String(identifier) => identifier,
                        _ => raise!("E0010", span),
                    };
                    if let Some(value) = self.globals.get(name.deref()) {
                        self.stack.push(value.clone());
                    } else {
                        raise!("E0012", span);
                    }
                }
                Instruction::SetGlobal => {
                    let name = match self.stack.pop().unwrap() {
                        Value::String(identifier) => identifier,
                        _ => raise!("E0010", span),
                    };
                    let value = self.stack.last().unwrap().clone();
                    if let Some(variable) = self.globals.get_mut(name.deref()) {
                        *variable = value;
                    } else {
                        raise!("E0012", span);
                    }
                }
                Instruction::GetLocal(index) => self.stack.push(self.stack[*index].clone()),
                Instruction::SetLocal(index) => {
                    let value = self.stack.last().unwrap().clone();
                    self.stack[*index] = value;
                }
                Instruction::JumpIfFalse(offset) => {
                    let condition: bool = self.stack.last().unwrap().boolean();
                    if !condition {
                        self.program_count = (self.program_count as isize + offset) as usize;
                        continue;
                    }
                }
                Instruction::Jump(offset) => {
                    self.program_count = (self.program_count as isize + offset) as usize;
                    continue;
                }
                Instruction::Return => break,
            }

            // TODO: new stack-monitor code here.
            // #[cfg(feature = "stack-monitor")]
            // if !self.stack.is_empty() {
            //     println!("{:?}", self.stack);
            // }

            self.program_count += 1;
        }
        Ok(())
    }
}
