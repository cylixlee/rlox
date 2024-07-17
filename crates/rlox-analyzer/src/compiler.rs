use rlox_intermediate::*;

struct Compiler {
    offset: usize,
    chunk: ChunkBuilder,
}

impl Compiler {
    fn new() -> Self {
        Self {
            offset: 0,
            chunk: ChunkBuilder::new(),
        }
    }

    fn compile(mut self, program: Vec<Declaration>) -> Chunk {
        while self.offset < program.len() {
            self.compile_declaration(&program[self.offset]);
            self.offset += 1;
        }
        self.chunk.build()
    }

    fn compile_declaration(&mut self, declaration: &Declaration) {
        match declaration {
            Declaration::Statement(statement) => self.compile_statement(statement),
            _ => unimplemented!(),
        }
    }

    fn compile_statement(&mut self, statement: &Statement) {
        match statement {
            Statement::Expression(expression) => self.compile_expression(expression),
            _ => unimplemented!(),
        }
    }

    fn compile_expression(&mut self, expression: &Expression) {
        match expression {
            Expression::Binary {
                left,
                operator,
                right,
            } => {
                self.compile_expression(left);
                self.compile_expression(right);
                let span = operator.span.clone();
                match &**operator {
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
                self.compile_expression(expression);
                let span = operator.span.clone();
                match &**operator {
                    UnaryOperator::Not => self.chunk.write(Instruction::Not, span),
                    UnaryOperator::Negate => self.chunk.write(Instruction::Negate, span),
                }
            }
            Expression::Literal(literal) => match &**literal {
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
                _ => unimplemented!(),
            },
            _ => unimplemented!(),
        }
    }
}

pub fn compile(program: Vec<Declaration>) -> Chunk {
    let chunk = Compiler::new().compile(program);
    #[cfg(feature = "bytecode-preview")]
    {
        println!("======= Instructions ========");
        for (index, instruction) in chunk.iter().enumerate() {
            println!("{index:04} {instruction:?}");
        }
        println!();

        println!("========= Constants =========");
        for (index, constant) in chunk.constants().iter().enumerate() {
            println!("{index:03}  {constant:?}");
        }
        println!();
    }
    chunk
}
