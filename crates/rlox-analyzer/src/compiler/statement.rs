use rlox_intermediate::{
    DiagnosableResult, Expression, ForLoopInitializer, Instruction, Statement,
};

use crate::compiler::Compiler;

impl Compiler<'_> {
    pub(super) fn compile_statement(&mut self, statement: &Statement) -> DiagnosableResult {
        match statement {
            Statement::Expression(expression) => {
                self.compile_expression(expression)?;
                self.current_function().append(Instruction::Pop);
                Ok(())
            }

            Statement::For {
                initializer,
                condition,
                incrementer,
                body,
            } => self.compile_for(initializer, condition, incrementer, body),

            Statement::If {
                condition,
                then,
                otherwise,
            } => self.compile_if(condition, then, otherwise),

            Statement::Print(expression) => {
                self.compile_expression(expression)?;
                self.current_function().append(Instruction::Print);
                Ok(())
            }

            Statement::While { condition, body } => self.compile_while(condition, body),

            Statement::Block(declarations) => {
                self.begin_scope();
                for declaration in declarations {
                    self.compile_declaration(declaration)?;
                }
                self.end_scope();
                Ok(())
            }

            Statement::Return(expression) => self.compile_return(expression),
        }
    }

    fn compile_for(
        &mut self,
        initializer: &Option<ForLoopInitializer>,
        condition: &Option<Expression>,
        incrementer: &Option<Expression>,
        body: &Statement,
    ) -> DiagnosableResult {
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
        Ok(())
    }

    fn compile_if(
        &mut self,
        condition: &Expression,
        then: &Statement,
        otherwise: &Option<Box<Statement>>,
    ) -> DiagnosableResult {
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
        Ok(())
    }

    fn compile_while(&mut self, condition: &Expression, body: &Statement) -> DiagnosableResult {
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
        Ok(())
    }

    fn compile_return(&mut self, expression: &Option<Expression>) -> DiagnosableResult {
        unimplemented!()
    }
}
