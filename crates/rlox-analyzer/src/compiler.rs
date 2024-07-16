use rlox_intermediate::{
    BinaryOperator, Chunk, ChunkBuilder, Constant, Declaration, Expression, Instruction, Literal,
    Statement,
};

struct Compiler {
    offset: usize,
    chunk_builder: ChunkBuilder,
}

impl Compiler {
    fn new() -> Self {
        Self {
            offset: 0,
            chunk_builder: ChunkBuilder::new(),
        }
    }

    fn compile(mut self, program: Vec<Declaration>) -> Chunk {
        while self.offset < program.len() {
            self.compile_declaration(&program[self.offset]);
            self.offset += 1;
        }
        self.chunk_builder.build()
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
            Expression::Literal(literal) => match &**literal {
                Literal::Number(number) => {
                    let index = self.chunk_builder.define(Constant::Number(*number));
                    self.chunk_builder
                        .write(Instruction::LoadConstant(index), literal.span.clone());
                }
                _ => unimplemented!(),
            },
            Expression::Binary {
                left,
                operator,
                right,
            } => {
                self.compile_expression(left);
                self.compile_expression(right);
                let span = operator.span.clone();
                match &**operator {
                    BinaryOperator::Add => self.chunk_builder.write(Instruction::Add, span),
                    BinaryOperator::Subtract => {
                        self.chunk_builder.write(Instruction::Subtract, span)
                    }
                    BinaryOperator::Multiply => {
                        self.chunk_builder.write(Instruction::Multiply, span)
                    }
                    BinaryOperator::Divide => self.chunk_builder.write(Instruction::Divide, span),
                    _ => unimplemented!(),
                }
            }
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
