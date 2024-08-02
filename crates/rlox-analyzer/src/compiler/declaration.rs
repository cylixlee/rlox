use rlox_intermediate::{Declaration, DiagnosableResult, Expression, Instruction, Span};

use crate::compiler::{Compiler, Local};

impl Compiler<'_> {
    pub(super) fn compile_declaration(&mut self, declaration: &Declaration) -> DiagnosableResult {
        match declaration {
            Declaration::Var { name, initializer } => {
                self.compile_variable(name, name.span.clone(), initializer)?
            }
            Declaration::Statement(statement) => self.compile_statement(statement)?,
            _ => unimplemented!(),
        }
        Ok(())
    }

    fn compile_variable(
        &mut self,
        name: &String,
        span: Span,
        initializer: &Option<Expression>,
    ) -> DiagnosableResult {
        // initial value
        if let Some(initializer) = initializer {
            self.compile_expression(initializer)?;
        } else {
            self.current_function()
                .write(Instruction::Nil, span.clone());
        }
        // determine whether it is global or local
        if self.depth == 0 {
            self.prepare_identifier(name.clone(), span);
            self.current_function().append(Instruction::DefineGlobal);
        } else {
            // there's no need to generate SetLocal.
            // local variables are defined once initializer expression calculated.
            self.locals.push(Local {
                name: name.clone(),
                depth: self.depth,
            });
        }
        Ok(())
    }
}
