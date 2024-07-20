use std::collections::HashMap;
use std::mem;
use std::ops::Deref;
use std::rc::Rc;
use std::time::Instant;

use rlox_intermediate::*;

use crate::heap::{Heap, Reference};
use crate::stack::Stack;
use crate::value::Value;

const STACK_SIZE: usize = 1024;

type NativeFunction = fn(&mut VirtualMachine) -> Value;

pub struct VirtualMachine {
    bytecode: Bytecode,
    program_count: usize,
    stack_offset: usize,
    stack: Stack<Value, STACK_SIZE>,
    heap: Heap,
    globals: HashMap<String, Value>,

    // Invocation fields
    native_functions: HashMap<String, NativeFunction>,
    chunks: Vec<Rc<Chunk>>,
    last_program_counts: Vec<usize>,
    last_stack_offsets: Vec<usize>,
    next_stack_offsets: Vec<usize>,
    return_value: Value,

    // Timer
    started: Instant,

    // Stack Monitor
    #[cfg(feature = "stack-monitor")]
    call_stack: Vec<String>,
}

impl VirtualMachine {
    pub fn new(bytecode: Bytecode) -> Self {
        let chunk = Rc::clone(&bytecode.script);
        let exit_program_count = chunk.len();
        let mut native_functions: HashMap<String, NativeFunction> = HashMap::new();
        native_functions.insert(String::from("clock"), native_clock);
        Self {
            bytecode,
            program_count: 0,
            stack_offset: 0,
            stack: Stack::new(),
            heap: Heap::new(),
            globals: HashMap::new(),
            native_functions,
            chunks: vec![chunk],
            last_program_counts: vec![exit_program_count],
            last_stack_offsets: vec![0],
            next_stack_offsets: Vec::new(),
            return_value: Value::Nil,
            started: Instant::now(),
            #[cfg(feature = "stack-monitor")]
            call_stack: vec!["<script>".into()],
        }
    }

    pub fn run(&mut self) -> DiagnosableResult {
        #[cfg(feature = "stack-monitor")]
        println!("━━━━━━━ Stack Monitor ━━━━━━━");

        while !self.chunks.is_empty() {
            while self.program_count < self.current_chunk().len() {
                let instruction = self.current_chunk()[self.program_count].clone();
                let span = self.current_chunk().span(self.program_count).clone();

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
                {
                    let function_name = self.call_stack.last().unwrap();
                    println!(
                        "{function_name}::{:04} {:?}",
                        self.program_count, instruction
                    );
                }

                match instruction {
                    Instruction::LoadConstant(index) => {
                        let constant = self.current_chunk().constant(index).clone();
                        match constant {
                            Constant::Number(number) => {
                                self.stack.push(Value::Number(number), span)?
                            }
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
                                let reference = self.heap.spawn_string(format!(
                                    "{}{}",
                                    this.deref(),
                                    that.deref()
                                ));
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
                        let name = self.pop_identifier(span.clone())?;
                        let value = self.stack.pop(span.clone())?;
                        if self.globals.contains_key(name.deref()) {
                            raise!("E0011", span);
                        }
                        self.globals.insert(name.deref().clone(), value);
                    }
                    Instruction::GetGlobal => {
                        let name = self.pop_identifier(span.clone())?;
                        self.stack
                            .push(self.global_ref(name, span.clone())?.clone(), span)?;
                    }
                    Instruction::SetGlobal => {
                        let name = self.pop_identifier(span.clone())?;
                        let value = self.stack.top(span.clone())?.clone();
                        *self.global_mut(name, span)? = value;
                    }
                    Instruction::GetLocal(index) => {
                        self.stack
                            .push(self.stack[self.stack_offset + index].clone(), span)?;
                    }
                    Instruction::SetLocal(index) => {
                        let value = self.stack.top(span)?.clone();
                        self.stack[self.stack_offset + index] = value;
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
                    }
                    Instruction::PrepareInvoke => self.next_stack_offsets.push(self.stack.len()),
                    Instruction::Invoke => {
                        let name = self.pop_identifier(span.clone())?;
                        if let Some(native_function) = self.native_functions.get(name.deref()) {
                            self.next_stack_offsets.pop(); // Native functions don't need stack frames.
                            let value = (*native_function)(self);
                            self.stack.push(value, span)?;
                        } else {
                            let function = match self.bytecode.functions.get(name.deref()) {
                                Some(function) => function,
                                None => raise!("E0015", span),
                            };

                            #[cfg(feature = "stack-monitor")]
                            self.call_stack.push(name.deref().clone());

                            self.last_stack_offsets.push(self.stack_offset);
                            self.stack_offset = self.next_stack_offsets.pop().unwrap();

                            let argument_count = self.stack.len() - self.stack_offset;
                            if argument_count != function.arity {
                                raise! {
                                    "E0016", span,
                                    format!(
                                        "expected {} arguments, found {}",
                                        function.arity, argument_count
                                    )
                                }
                            }
                            self.last_program_counts.push(self.program_count + 1);
                            self.program_count = 0;
                            self.chunks.push(Rc::clone(&function.chunk));
                            self.return_value = Value::Nil;
                            continue;
                        }
                    }
                    Instruction::Return => {
                        self.return_value = self.stack.pop(span)?;
                        break;
                    }
                }

                #[cfg(feature = "stack-monitor")]
                if !self.stack.is_empty() {
                    println!("{:?}", self.stack);
                }

                self.program_count += 1;
            }

            while self.stack.len() > self.stack_offset {
                self.stack.try_pop().unwrap();
            }
            self.stack_offset = self.last_stack_offsets.pop().unwrap();
            self.stack
                .try_push(mem::replace(&mut self.return_value, Value::Nil));
            let last_program_count = self.last_program_counts.pop().unwrap();
            self.program_count = last_program_count;
            self.chunks.pop().unwrap();
        }

        #[cfg(feature = "stack-monitor")]
        println!();

        Ok(())
    }

    fn current_chunk(&self) -> &Chunk {
        self.chunks.last().unwrap()
    }

    fn pop_identifier(&mut self, span: Span) -> DiagnosableResult<Reference<String>> {
        match self.stack.pop(span.clone())? {
            Value::String(identifier) => Ok(identifier),
            _ => raise!("E0010", span),
        }
    }

    fn global_ref(&self, name: impl AsRef<str>, span: Span) -> DiagnosableResult<&Value> {
        if let Some(value) = self.globals.get(name.as_ref()) {
            return Ok(value);
        }
        raise!("E0012", span);
    }

    fn global_mut(&mut self, name: impl AsRef<str>, span: Span) -> DiagnosableResult<&mut Value> {
        if let Some(value) = self.globals.get_mut(name.as_ref()) {
            return Ok(value);
        }
        raise!("E0012", span);
    }
}

fn native_clock(vm: &mut VirtualMachine) -> Value {
    Value::Number(vm.started.elapsed().as_millis() as f64)
}
