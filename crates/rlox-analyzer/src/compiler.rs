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

    fn compile(mut self, program: Vec<Declaration>) -> DiagnosableResult<Chunk> {
        while self.offset < program.len() {
            self.compile_declaration(&program[self.offset])?;
            self.offset += 1;
        }
        Ok(self.chunk.build())
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
            Statement::Print(expression) => {
                self.compile_expression(expression)?;
                self.chunk.append(Instruction::Print);
            }
            Statement::Block(declarations) => {
                self.blocks.push(self.locals.len());
                for declaration in declarations {
                    self.compile_declaration(declaration)?;
                }
                let frame = *self.blocks.last().unwrap();
                while self.locals.len() > frame {
                    self.locals.pop();
                    self.chunk.append(Instruction::Pop);
                }
                self.blocks.pop();
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
                            // determine whether it is global or local
                            if self.blocks.is_empty() {
                                // load global variable name (identifier)
                                let index = self.chunk.define(Constant::String(identifier.clone()));
                                self.chunk
                                    .write(Instruction::LoadConstant(index), literal.span.clone());
                                self.chunk.append(Instruction::SetGlobal);
                            } else {
                                let index = self.locals.len();
                                self.locals.push(identifier.clone());
                                self.chunk.append(Instruction::SetLocal(index));
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
                self.compile_expression(left)?;
                self.compile_expression(right)?;
                let span = operator.span.clone();
                match operator.deref() {
                    BinaryOperator::Add => self.chunk.write(Instruction::Add, span),
                    BinaryOperator::Subtract => self.chunk.write(Instruction::Subtract, span),
                    BinaryOperator::Multiply => self.chunk.write(Instruction::Multiply, span),
                    BinaryOperator::Divide => self.chunk.write(Instruction::Divide, span),
                    BinaryOperator::Equal => self.chunk.write(Instruction::Equal, span),
                    BinaryOperator::Greater => self.chunk.write(Instruction::Greater, span),
                    BinaryOperator::Less => self.chunk.write(Instruction::Less, span),
                    BinaryOperator::NotEqual => {
                        self.chunk.write(Instruction::Equal, span.clone());
                        self.chunk.write(Instruction::Not, span);
                    }
                    BinaryOperator::GreaterEqual => {
                        self.chunk.write(Instruction::Less, span.clone());
                        self.chunk.write(Instruction::Not, span);
                    }
                    BinaryOperator::LessEqual => {
                        self.chunk.write(Instruction::Greater, span.clone());
                        self.chunk.write(Instruction::Not, span.clone());
                    }
                    _ => unimplemented!(),
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
                    let mut local_index = None;
                    for (index, name) in self.locals.iter().rev().enumerate() {
                        if name == identifier {
                            local_index = Some(self.locals.len() - index - 1);
                            break;
                        }
                    }
                    match local_index {
                        None => {
                            // load global variable name (identifier)
                            let index = self.chunk.define(Constant::String(identifier.clone()));
                            self.chunk
                                .write(Instruction::LoadConstant(index), literal.span.clone());
                            self.chunk.append(Instruction::GetGlobal);
                        }
                        Some(index) => self.chunk.append(Instruction::GetLocal(index)),
                    }
                }
                _ => unimplemented!(),
            },
            _ => unimplemented!(),
        }
        Ok(())
    }
}

pub fn compile(program: Vec<Declaration>) -> DiagnosableResult<Chunk> {
    let chunk = Compiler::new().compile(program)?;
    #[cfg(feature = "bytecode-preview")]
    {
        println!("━━━━━━━ Instructions ━━━━━━━━");
        for (index, instruction) in chunk.iter().enumerate() {
            println!("{index:04} {instruction:?}");
        }

        println!("━━━━━━━━━ Constants ━━━━━━━━━");
        for (index, constant) in chunk.constants().iter().enumerate() {
            println!("{index:03}  {constant:?}");
        }
    }
    Ok(chunk)
}
