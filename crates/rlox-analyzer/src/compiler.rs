use std::collections::HashMap;
use std::ops::Deref;

use rlox_intermediate::*;

struct Compiler {
    offset: usize,
    chunk: ChunkBuilder,
    locals: Vec<String>,
    blocks: Vec<usize>,
}

impl Compiler {
    fn new() -> Self {
        Self {
            offset: 0,
            chunk: ChunkBuilder::new(),
            locals: Vec::new(),
            blocks: Vec::new(),
        }
    }

    fn predefine_parameters(&mut self, parameters: Vec<Spanned<String>>) {
        for parameter in parameters {
            self.locals.push(parameter.into_inner());
        }
    }

    fn compile(mut self, program: Vec<Declaration>) -> DiagnosableResult<Chunk> {
        while self.offset < program.len() {
            self.compile_declaration(&program[self.offset])?;
            self.offset += 1;
        }
        Ok(self.emit())
    }

    fn emit(self) -> Chunk {
        self.chunk.build()
    }

    fn compile_declaration(&mut self, declaration: &Declaration) -> DiagnosableResult {
        match declaration {
            Declaration::Var { name, initializer } => {
                // initial value
                if let Some(initializer) = initializer {
                    self.compile_expression(initializer)?;
                } else {
                    self.chunk.write(Instruction::Nil, name.span.clone());
                }
                // determine whether it is global or local
                if self.blocks.is_empty() {
                    // load global variable name (identifier)
                    let index = self.chunk.define(Constant::String(name.deref().clone()));
                    self.chunk
                        .write(Instruction::LoadConstant(index), name.span.clone());
                    self.chunk.append(Instruction::DefineGlobal);
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
                self.chunk.append(Instruction::Pop);
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
                let condition_tag = self.chunk.instructions.len();
                if let Some(condition) = condition {
                    self.compile_expression(condition)?;
                }
                let mut outer_backpatch = self.chunk.append_backpatch(Instruction::JumpIfFalse(0));
                self.chunk.append(Instruction::Pop);
                let mut body_backpatch = self.chunk.append_backpatch(Instruction::Jump(0));
                let incrementer_tag = self.chunk.instructions.len();
                if let Some(incrementer) = incrementer {
                    self.compile_expression(incrementer)?;
                }
                self.chunk.append(Instruction::Pop);
                self.chunk
                    .append_backpatch(Instruction::Jump(0))
                    .backpatch_by(condition_tag as isize);
                body_backpatch.backpatch();
                self.compile_statement(body)?;
                self.chunk
                    .append_backpatch(Instruction::Jump(0))
                    .backpatch_by(incrementer_tag as isize);
                outer_backpatch.backpatch();
                self.chunk.append(Instruction::Pop);
                self.end_scope();
            }
            Statement::If {
                condition,
                then,
                otherwise,
            } => {
                self.compile_expression(condition)?;
                let mut else_backpatch = self.chunk.append_backpatch(Instruction::JumpIfFalse(0));
                self.chunk.append(Instruction::Pop);
                self.compile_statement(then)?;
                let mut outer_backpatch = self.chunk.append_backpatch(Instruction::Jump(0));
                else_backpatch.backpatch();
                self.chunk.append(Instruction::Pop);
                if let Some(otherwise) = otherwise {
                    self.compile_statement(otherwise)?;
                }
                outer_backpatch.backpatch();
            }
            Statement::Print(expression) => {
                self.compile_expression(expression)?;
                self.chunk.append(Instruction::Print);
            }
            Statement::While { condition, body } => {
                let condition_tag = self.chunk.instructions.len();
                self.compile_expression(condition)?;
                let mut outer_backpatch = self.chunk.append_backpatch(Instruction::JumpIfFalse(0));
                self.chunk.append(Instruction::Pop);
                self.compile_statement(body)?;
                self.chunk
                    .append_backpatch(Instruction::Jump(0))
                    .backpatch_by(condition_tag as isize);
                outer_backpatch.backpatch();
                self.chunk.append(Instruction::Pop);
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
                                    let index =
                                        self.chunk.define(Constant::String(identifier.clone()));
                                    self.chunk.write(
                                        Instruction::LoadConstant(index),
                                        literal.span.clone(),
                                    );
                                    self.chunk.append(Instruction::SetGlobal);
                                }
                                Some(index) => self.chunk.append(Instruction::SetLocal(index)),
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
                        let mut outer_backpatch =
                            self.chunk.append_backpatch(Instruction::JumpIfFalse(0));
                        self.chunk.append(Instruction::Pop);
                        self.compile_expression(right)?;
                        outer_backpatch.backpatch();
                    }
                    BinaryOperator::Or => {
                        self.compile_expression(left)?;
                        let mut right_backpatch =
                            self.chunk.append_backpatch(Instruction::JumpIfFalse(0));
                        let mut outer_backpatch = self.chunk.append_backpatch(Instruction::Jump(0));
                        right_backpatch.backpatch();
                        self.chunk.append(Instruction::Pop);
                        self.compile_expression(right)?;
                        outer_backpatch.backpatch();
                    }
                    _ => {
                        self.compile_expression(left)?;
                        self.compile_expression(right)?;
                        match operator.deref() {
                            BinaryOperator::Add => self.chunk.write(Instruction::Add, span),
                            BinaryOperator::Subtract => {
                                self.chunk.write(Instruction::Subtract, span)
                            }
                            BinaryOperator::Multiply => {
                                self.chunk.write(Instruction::Multiply, span)
                            }
                            BinaryOperator::Divide => self.chunk.write(Instruction::Divide, span),
                            BinaryOperator::Equal => self.chunk.write(Instruction::Equal, span),
                            BinaryOperator::Greater => self.chunk.write(Instruction::Greater, span),
                            BinaryOperator::Less => self.chunk.write(Instruction::Less, span),
                            BinaryOperator::NotEqual => {
                                self.chunk.write(Instruction::Equal, span);
                                self.chunk.append(Instruction::Not);
                            }
                            BinaryOperator::GreaterEqual => {
                                self.chunk.write(Instruction::Less, span);
                                self.chunk.append(Instruction::Not);
                            }
                            BinaryOperator::LessEqual => {
                                self.chunk.write(Instruction::Greater, span);
                                self.chunk.append(Instruction::Not);
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
                    UnaryOperator::Not => self.chunk.write(Instruction::Not, span),
                    UnaryOperator::Negate => self.chunk.write(Instruction::Negate, span),
                }
            }
            Expression::Invocation {
                expression,
                arguments,
            } => {
                for argument in arguments {
                    self.compile_expression(argument)?;
                }
                match expression.deref() {
                    Expression::Literal(literal) => match literal.deref() {
                        Literal::Identifier(identifier) => {
                            let index = self.chunk.define(Constant::String(identifier.clone()));
                            self.chunk
                                .write(Instruction::LoadConstant(index), literal.span.clone());
                        }
                        _ => raise!("E0014", literal.span.clone()),
                    },
                    _ => unimplemented!(),
                }
                self.chunk.append(Instruction::Invoke);
            }
            Expression::Literal(literal) => match literal.deref() {
                Literal::Nil => self.chunk.write(Instruction::Nil, literal.span.clone()),
                Literal::Boolean(boolean) => {
                    if *boolean {
                        self.chunk.write(Instruction::True, literal.span.clone());
                    } else {
                        self.chunk.write(Instruction::False, literal.span.clone());
                    }
                }
                Literal::Number(number) => {
                    let index = self.chunk.define(Constant::Number(*number));
                    self.chunk
                        .write(Instruction::LoadConstant(index), literal.span.clone());
                }
                Literal::String(string) => {
                    let index = self.chunk.define(Constant::String(string.clone()));
                    self.chunk
                        .write(Instruction::LoadConstant(index), literal.span.clone());
                }
                Literal::Identifier(identifier) => {
                    // determine whether it is global or local
                    match self.search_local(identifier) {
                        None => {
                            // load global variable name (identifier)
                            let index = self.chunk.define(Constant::String(identifier.clone()));
                            self.chunk
                                .write(Instruction::LoadConstant(index), literal.span.clone());
                            self.chunk.append(Instruction::GetGlobal);
                        }
                        Some(index) => self
                            .chunk
                            .write(Instruction::GetLocal(index), literal.span.clone()),
                    }
                }
                _ => unimplemented!(),
            },
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
            self.chunk.append(Instruction::Pop);
        }
    }
}

pub fn compile(program: Vec<Declaration>) -> DiagnosableResult<Bytecode> {
    let mut script = Vec::new();
    let mut functions = HashMap::new();
    for declaration in program {
        match declaration {
            Declaration::Function {
                name,
                parameters,
                body,
            } => {
                let mut compiler = Compiler::new();
                let arity = parameters.len();
                compiler.begin_scope();
                compiler.predefine_parameters(parameters);
                compiler.compile_statement(&body)?;
                compiler.end_scope();
                let function = Function {
                    chunk: compiler.emit(),
                    arity,
                };
                functions.insert(name.into_inner(), function);
            }
            _ => script.push(declaration),
        }
    }
    let script = Compiler::new().compile(script)?;
    let bytecode = Bytecode { functions, script };
    #[cfg(feature = "bytecode-preview")]
    {
        println!("━━━━━━━━━━ Bytecode Preview Start ━━━━━━━━━━");
        for (name, function) in &bytecode.functions {
            println!("function \"{name}\", arity = {}", function.arity);
            preview_chunk(&function.chunk);
            println!();
        }
        println!("<entrypoint>");
        preview_chunk(&bytecode.script);
    }
    Ok(bytecode)
}

#[cfg(feature = "bytecode-preview")]
fn preview_chunk(chunk: &Chunk) {
    if !chunk.is_empty() {
        println!("INSTRUCTIONS ({}):", chunk.len());
        for (index, instruction) in chunk.iter().enumerate() {
            println!("    {index:04} {instruction:?}");
        }
        if !chunk.constants().is_empty() {
            println!("CONSTANTS ({}):", chunk.constants().len());
            for (index, constant) in chunk.constants().iter().enumerate() {
                println!("    {index:03} {constant:?}");
            }
        }
    }
}
