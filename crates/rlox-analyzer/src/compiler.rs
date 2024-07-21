use std::ops::Deref;

use rlox_intermediate::*;

enum FunctionType {
    Function,
    Script,
}

struct Compiler<'a> {
    offset: usize,
    locals: Vec<String>,
    blocks: Vec<usize>,
    functions: Vec<FunctionBuilder>,
    function_type: FunctionType,

    heap: &'a mut Heap,
}

impl<'a> Compiler<'a> {
    fn new(heap: &'a mut Heap) -> Self {
        Self {
            offset: 0,
            locals: Vec::new(),
            blocks: Vec::new(),
            functions: vec![FunctionBuilder::new()],
            function_type: FunctionType::Script,
            heap,
        }
    }

    fn current_function(&mut self) -> &mut FunctionBuilder {
        self.functions.last_mut().unwrap()
    }

    // fn compile(mut self, program: Vec<Declaration>) -> DiagnosableResult<Chunk> {
    //     while self.offset < program.len() {
    //         self.compile_declaration(&program[self.offset])?;
    //         self.offset += 1;
    //     }
    //     Ok(self.current_function().build())
    // }

    fn compile_declaration(&mut self, declaration: &Declaration) -> DiagnosableResult {
        match declaration {
            Declaration::Var { name, initializer } => {
                // initial value
                if let Some(initializer) = initializer {
                    self.compile_expression(initializer)?;
                } else {
                    self.current_function()
                        .write(Instruction::Nil, name.span.clone());
                }
                // determine whether it is global or local
                if self.blocks.is_empty() {
                    // load global variable name (identifier)
                    let identifier = self.heap.spawn_string(name.deref().clone());
                    let index = self.current_function().define(Value::String(identifier));
                    self.current_function()
                        .write(Instruction::LoadConstant(index), name.span.clone());
                    self.current_function().append(Instruction::DefineGlobal);
                } else {
                    // there's no need to generate SetLocal.
                    // local variables are defined once initializer expression calculated.
                    self.locals.push(name.deref().clone());
                }
            }
            Declaration::Statement(statement) => self.compile_statement(statement)?,
            _ => unimplemented!(),
        }
        Ok(())
    }

    fn compile_statement(&mut self, statement: &Statement) -> DiagnosableResult {
        match statement {
            Statement::Expression(expression) => {
                self.compile_expression(expression)?;
                self.current_function().append(Instruction::Pop);
            }
            Statement::For {
                initializer,
                condition,
                incrementer,
                body,
            } => {
                self.begin_scope();
                if let Some(initializer) = initializer {
                    match initializer {
                        ForLoopInitializer::VarDeclaration(declaration) => {
                            self.compile_declaration(declaration)?
                        }
                        ForLoopInitializer::VarInitialization(expression) => {
                            self.compile_expression(expression)?
                        }
                    }
                }
                let condition_tag = self.current_function().instructions.len();
                if let Some(condition) = condition {
                    self.compile_expression(condition)?;
                }
                let mut outer_backpatch = self
                    .current_function()
                    .append_backpatch(Instruction::JumpIfFalse(0));
                self.current_function().append(Instruction::Pop);
                let mut body_backpatch = self
                    .current_function()
                    .append_backpatch(Instruction::Jump(0));
                let incrementer_tag = self.current_function().instructions.len();
                if let Some(incrementer) = incrementer {
                    self.compile_expression(incrementer)?;
                    self.current_function().append(Instruction::Pop);
                }
                self.current_function()
                    .append_backpatch(Instruction::Jump(0))
                    .backpatch_by(condition_tag as isize);
                body_backpatch.backpatch();
                self.compile_statement(body)?;
                self.current_function()
                    .append_backpatch(Instruction::Jump(0))
                    .backpatch_by(incrementer_tag as isize);
                outer_backpatch.backpatch();
                self.current_function().append(Instruction::Pop);
                self.end_scope();
            }
            Statement::If {
                condition,
                then,
                otherwise,
            } => {
                self.compile_expression(condition)?;
                let mut else_backpatch = self
                    .current_function()
                    .append_backpatch(Instruction::JumpIfFalse(0));
                self.current_function().append(Instruction::Pop);
                self.compile_statement(then)?;
                let mut outer_backpatch = self
                    .current_function()
                    .append_backpatch(Instruction::Jump(0));
                else_backpatch.backpatch();
                self.current_function().append(Instruction::Pop);
                if let Some(otherwise) = otherwise {
                    self.compile_statement(otherwise)?;
                }
                outer_backpatch.backpatch();
            }
            Statement::Print(expression) => {
                self.compile_expression(expression)?;
                self.current_function().append(Instruction::Print);
            }
            Statement::While { condition, body } => {
                let condition_tag = self.current_function().instructions.len();
                self.compile_expression(condition)?;
                let mut outer_backpatch = self
                    .current_function()
                    .append_backpatch(Instruction::JumpIfFalse(0));
                self.current_function().append(Instruction::Pop);
                self.compile_statement(body)?;
                self.current_function()
                    .append_backpatch(Instruction::Jump(0))
                    .backpatch_by(condition_tag as isize);
                outer_backpatch.backpatch();
                self.current_function().append(Instruction::Pop);
            }
            Statement::Block(declarations) => {
                self.begin_scope();
                for declaration in declarations {
                    self.compile_declaration(declaration)?;
                }
                self.end_scope();
            }
            _ => unimplemented!(),
        }
        Ok(())
    }

    fn compile_expression(&mut self, expression: &Expression) -> DiagnosableResult {
        match expression {
            Expression::Assignment { left, span, right } => {
                // prepare assignment value
                self.compile_expression(right)?;
                match left.deref() {
                    Expression::Binary { .. } => unimplemented!(),
                    Expression::Literal(literal) => match literal.deref() {
                        Literal::Identifier(identifier) => {
                            match self.search_local(identifier) {
                                None => {
                                    // load global variable name (identifier)
                                    let name = self.heap.spawn_string(identifier.clone());
                                    let index = self.current_function().define(Value::String(name));
                                    self.current_function().write(
                                        Instruction::LoadConstant(index),
                                        literal.span.clone(),
                                    );
                                    self.current_function().append(Instruction::SetGlobal);
                                }
                                Some(index) => {
                                    self.current_function().append(Instruction::SetLocal(index))
                                }
                            }
                        }
                        _ => raise!("E0013", literal.span.clone()),
                    },
                    _ => raise!("E0013", span.clone()),
                }
            }
            Expression::Binary {
                left,
                operator,
                right,
            } => {
                let span = operator.span.clone();
                match operator.deref() {
                    BinaryOperator::And => {
                        self.compile_expression(left)?;
                        let mut outer_backpatch = self
                            .current_function()
                            .append_backpatch(Instruction::JumpIfFalse(0));
                        self.current_function().append(Instruction::Pop);
                        self.compile_expression(right)?;
                        outer_backpatch.backpatch();
                    }
                    BinaryOperator::Or => {
                        self.compile_expression(left)?;
                        let mut right_backpatch = self
                            .current_function()
                            .append_backpatch(Instruction::JumpIfFalse(0));
                        let mut outer_backpatch = self
                            .current_function()
                            .append_backpatch(Instruction::Jump(0));
                        right_backpatch.backpatch();
                        self.current_function().append(Instruction::Pop);
                        self.compile_expression(right)?;
                        outer_backpatch.backpatch();
                    }
                    _ => {
                        self.compile_expression(left)?;
                        self.compile_expression(right)?;
                        match operator.deref() {
                            BinaryOperator::Add => {
                                self.current_function().write(Instruction::Add, span)
                            }
                            BinaryOperator::Subtract => {
                                self.current_function().write(Instruction::Subtract, span)
                            }
                            BinaryOperator::Multiply => {
                                self.current_function().write(Instruction::Multiply, span)
                            }
                            BinaryOperator::Divide => {
                                self.current_function().write(Instruction::Divide, span)
                            }
                            BinaryOperator::Equal => {
                                self.current_function().write(Instruction::Equal, span)
                            }
                            BinaryOperator::Greater => {
                                self.current_function().write(Instruction::Greater, span)
                            }
                            BinaryOperator::Less => {
                                self.current_function().write(Instruction::Less, span)
                            }
                            BinaryOperator::NotEqual => {
                                self.current_function().write(Instruction::Equal, span);
                                self.current_function().append(Instruction::Not);
                            }
                            BinaryOperator::GreaterEqual => {
                                self.current_function().write(Instruction::Less, span);
                                self.current_function().append(Instruction::Not);
                            }
                            BinaryOperator::LessEqual => {
                                self.current_function().write(Instruction::Greater, span);
                                self.current_function().append(Instruction::Not);
                            }
                            _ => unimplemented!(),
                        }
                    }
                }
            }
            Expression::Unary {
                operator,
                expression,
            } => {
                self.compile_expression(expression)?;
                let span = operator.span.clone();
                match operator.deref() {
                    UnaryOperator::Not => self.current_function().write(Instruction::Not, span),
                    UnaryOperator::Negate => {
                        self.current_function().write(Instruction::Negate, span)
                    }
                }
            }
            Expression::Literal(literal) => match literal.deref() {
                Literal::Nil => self
                    .current_function()
                    .write(Instruction::Nil, literal.span.clone()),
                Literal::Boolean(boolean) => {
                    if *boolean {
                        self.current_function()
                            .write(Instruction::True, literal.span.clone());
                    } else {
                        self.current_function()
                            .write(Instruction::False, literal.span.clone());
                    }
                }
                Literal::Number(number) => {
                    let index = self.current_function().define(Value::Number(*number));
                    self.current_function()
                        .write(Instruction::LoadConstant(index), literal.span.clone());
                }
                Literal::String(string) => {
                    let string = self.heap.spawn_string(string.clone());
                    let index = self.current_function().define(Value::String(string));
                    self.current_function()
                        .write(Instruction::LoadConstant(index), literal.span.clone());
                }
                Literal::Identifier(identifier) => {
                    // determine whether it is global or local
                    match self.search_local(identifier) {
                        None => {
                            // load global variable name (identifier)
                            let name = self.heap.spawn_string(identifier.clone());
                            let index = self.current_function().define(Value::String(name));
                            self.current_function()
                                .write(Instruction::LoadConstant(index), literal.span.clone());
                            self.current_function().append(Instruction::GetGlobal);
                        }
                        Some(index) => self.current_function().append(Instruction::GetLocal(index)),
                    }
                }
                _ => unimplemented!(),
            },
            _ => unimplemented!(),
        }
        Ok(())
    }

    fn search_local(&self, identifier: &String) -> Option<usize> {
        let mut local_index = None;
        for (index, name) in self.locals.iter().rev().enumerate() {
            if name == identifier {
                local_index = Some(self.locals.len() - index - 1);
                break;
            }
        }
        local_index
    }

    fn begin_scope(&mut self) {
        self.blocks.push(self.locals.len());
    }

    fn end_scope(&mut self) {
        let frame = self.blocks.pop().unwrap();
        while self.locals.len() > frame {
            self.locals.pop();
            self.current_function().append(Instruction::Pop);
        }
    }
}

pub fn compile(program: Vec<Declaration>) -> DiagnosableResult<Chunk> {
    // let chunk = Compiler::new().compile(program)?;
    // #[cfg(feature = "bytecode-preview")]
    // {
    //     println!("━━━━━━━ Instructions ━━━━━━━━");
    //     for (index, instruction) in chunk.iter().enumerate() {
    //         println!("{index:04} {instruction:?}");
    //     }
    //
    //     println!("━━━━━━━━━ Constants ━━━━━━━━━");
    //     for (index, constant) in chunk.constants().iter().enumerate() {
    //         println!("{index:03}  {constant:?}");
    //     }
    // }
    // Ok(chunk)
    unimplemented!()
}
