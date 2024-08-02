use rlox_intermediate::{
    BinaryOperator, DiagnosableResult, Expression, Instruction, Literal, raise, Span,
    UnaryOperator, Value,
};

use crate::compiler::Compiler;

impl Compiler<'_> {
    pub(super) fn compile_expression(&mut self, expression: &Expression) -> DiagnosableResult {
        match expression {
            Expression::Assignment { left, span, right } => {
                self.compile_assignment(left, span, right)
            }

            Expression::Binary {
                left,
                operator,
                right,
            } => {
                let span = operator.span.clone();
                match &**operator {
                    BinaryOperator::And => self.compile_conditional_and(left, right),
                    BinaryOperator::Or => self.compile_conditional_or(left, right),
                    _ => {
                        self.compile_expression(left)?;
                        self.compile_expression(right)?;
                        self.compile_binary_operator(operator, span)
                    }
                }
            }

            Expression::Unary {
                operator,
                expression,
            } => {
                self.compile_expression(expression)?;
                self.compile_unary_operator(operator, operator.span.clone())
            }

            Expression::Literal(literal) => self.compile_literal(literal, literal.span.clone()),

            Expression::Invocation {
                expression,
                arguments,
            } => self.compile_invocation(expression, arguments),
        }
    }

    fn compile_assignment(
        &mut self,
        left: &Expression,
        span: &Span,
        right: &Expression,
    ) -> DiagnosableResult {
        // prepare assignment value
        self.compile_expression(right)?;
        match left {
            Expression::Binary { .. } => unimplemented!(),
            Expression::Literal(literal) => match &**literal {
                Literal::Identifier(identifier) => match self.search_local(identifier) {
                    None => {
                        self.prepare_identifier(identifier.clone(), literal.span.clone());
                        self.current_function().append(Instruction::SetGlobal);
                    }
                    Some(index) => self.current_function().append(Instruction::SetLocal(index)),
                },
                _ => raise!("E0013", literal.span.clone()),
            },
            _ => raise!("E0013", span.clone()),
        }
        Ok(())
    }

    fn compile_conditional_and(
        &mut self,
        left: &Expression,
        right: &Expression,
    ) -> DiagnosableResult {
        self.compile_expression(left)?;
        let mut outer_backpatch = self
            .current_function()
            .append_backpatch(Instruction::JumpIfFalse(0));
        self.current_function().append(Instruction::Pop);
        self.compile_expression(right)?;
        outer_backpatch.backpatch();
        Ok(())
    }

    fn compile_conditional_or(
        &mut self,
        left: &Expression,
        right: &Expression,
    ) -> DiagnosableResult {
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
        Ok(())
    }

    fn compile_binary_operator(
        &mut self,
        operator: &BinaryOperator,
        span: Span,
    ) -> DiagnosableResult {
        match operator {
            BinaryOperator::Add => self.current_function().write(Instruction::Add, span),
            BinaryOperator::Subtract => self.current_function().write(Instruction::Subtract, span),
            BinaryOperator::Multiply => self.current_function().write(Instruction::Multiply, span),
            BinaryOperator::Divide => self.current_function().write(Instruction::Divide, span),
            BinaryOperator::Equal => self.current_function().write(Instruction::Equal, span),
            BinaryOperator::Greater => self.current_function().write(Instruction::Greater, span),
            BinaryOperator::Less => self.current_function().write(Instruction::Less, span),
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
        Ok(())
    }

    fn compile_unary_operator(
        &mut self,
        operator: &UnaryOperator,
        span: Span,
    ) -> DiagnosableResult {
        match operator {
            UnaryOperator::Not => self.current_function().write(Instruction::Not, span),
            UnaryOperator::Negate => self.current_function().write(Instruction::Negate, span),
        }
        Ok(())
    }

    fn compile_literal(&mut self, literal: &Literal, span: Span) -> DiagnosableResult {
        match literal {
            Literal::Nil => self.current_function().write(Instruction::Nil, span),
            Literal::Boolean(boolean) => {
                if *boolean {
                    self.current_function().write(Instruction::True, span);
                } else {
                    self.current_function().write(Instruction::False, span);
                }
            }
            Literal::Number(number) => {
                let index = self.current_function().define(Value::Number(*number));
                self.current_function()
                    .write(Instruction::LoadConstant(index), span);
            }
            Literal::String(string) => {
                let string = self.heap.spawn_string(string.clone());
                let index = self.current_function().define(Value::String(string));
                self.current_function()
                    .write(Instruction::LoadConstant(index), span);
            }
            Literal::Identifier(identifier) => {
                // determine whether it is global or local
                match self.search_local(identifier) {
                    None => {
                        self.prepare_identifier(identifier.clone(), span);
                        self.current_function().append(Instruction::GetGlobal);
                    }
                    Some(index) => self.current_function().append(Instruction::GetLocal(index)),
                }
            }
            _ => unimplemented!(),
        }
        Ok(())
    }

    fn compile_invocation(
        &mut self,
        expression: &Expression,
        arguments: &Vec<Expression>,
    ) -> DiagnosableResult {
        unimplemented!()
    }
}
