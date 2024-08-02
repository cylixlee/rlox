use rlox_intermediate::{Instruction, Span, Value};

use crate::compiler::Compiler;

impl Compiler<'_> {
    pub(super) fn search_local(&self, identifier: &String) -> Option<usize> {
        for (index, local) in self.locals.iter().enumerate().rev() {
            if &local.name == identifier {
                return Some(self.locals.len() - index - 1);
            }
        }
        None
    }

    pub(super) fn begin_scope(&mut self) {
        self.depth += 1;
    }

    pub(super) fn end_scope(&mut self) {
        self.depth -= 1;
        while self.locals.len() > self.depth {
            self.locals.pop();
            self.current_function().append(Instruction::Pop);
        }
    }

    pub(super) fn prepare_identifier(&mut self, identifier: String, span: Span) {
        let identifier = self.heap.spawn_string(identifier);
        let index = self.current_function().define(Value::String(identifier));
        self.current_function()
            .write(Instruction::LoadConstant(index), span);
    }
}
