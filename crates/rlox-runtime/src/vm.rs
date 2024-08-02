use std::collections::HashMap;
use std::ops::Deref;

use smallvec::{smallvec, SmallVec};

use rlox_intermediate::*;

const FRAMES_CAPACITY: usize = 128;
const STACK_CAPACITY: usize = 8192;

struct CallFrame {
    function: Reference<Function>,
    program_count: usize,
    offset: usize,
}

pub struct VirtualMachine {
    program_count: usize,
    stack: SmallVec<[Value; STACK_CAPACITY]>,
    heap: Heap,
    globals: HashMap<String, Value>,
    call_stack: SmallVec<[CallFrame; FRAMES_CAPACITY]>,
}

impl VirtualMachine {
    pub fn new(mut heap: Heap, function: Function) -> Self {
        let function = heap.spawn(function);
        Self {
            program_count: 0,
            stack: SmallVec::new(),
            heap,
            globals: HashMap::new(),
            call_stack: smallvec![CallFrame {
                function,
                program_count: 0,
                offset: 0,
            }],
        }
    }

    pub fn run(&mut self) -> DiagnosableResult {
        #[cfg(feature = "stack-monitor")]
        {
            println!("━━━━━ Stack Monitor ━━━━━");
        }

        loop {
            let instruction = &self.current_function()[self.program_count];
            let span = self.current_function().span(self.program_count).clone();

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

            #[cfg(feature = "stack-monitor")]
            self.current_function()
                .disassemble_instruction(self.program_count);

            match instruction {
                Instruction::LoadConstant(index) => {
                    let value = self.current_function().constant(*index).clone();
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
                Instruction::GetLocal(index) => {
                    let offset = self.current_stack_offset();
                    self.stack.push(self.stack[offset + *index].clone());
                }
                Instruction::SetLocal(index) => {
                    let value = self.stack.last().unwrap().clone();
                    let offset = self.current_stack_offset();
                    self.stack[offset + *index] = value;
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
                Instruction::Call(_) => todo!(),
                Instruction::Return => break,
            }

            #[cfg(feature = "stack-monitor")]
            if !self.stack.is_empty() {
                stack_monitor(&self.stack);
            }

            self.program_count += 1;
        }
        Ok(())
    }

    fn current_function(&self) -> Reference<Function> {
        self.call_stack.last().unwrap().function.clone()
    }

    fn current_stack_offset(&self) -> usize {
        self.call_stack.last().unwrap().offset
    }
}

fn stack_monitor<T>(stack: &SmallVec<T>)
where
    T: smallvec::Array<Item = Value>,
{
    for value in stack {
        print!("[ {value:?} ]");
    }
    println!();
}
