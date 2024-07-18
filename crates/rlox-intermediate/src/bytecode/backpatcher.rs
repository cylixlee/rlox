pub trait Backpatch {
    fn backpatch_by(&mut self, value: isize);
    fn backpatch(&mut self);
}

macro_rules! define_backpatcher {
    ($($variant: ident), * $(,)?) => {
        use std::ptr::NonNull;
        use crate::bytecode::instruction::Instruction;
        $(
            paste::paste! {
                pub struct [<$variant Backpatcher>] {
                    instructions: NonNull<Vec<Instruction>>,
                    index: usize,
                }

                impl [<$variant Backpatcher>] {
                    pub fn new(instructions: &mut Vec<Instruction>, index: usize) -> Self {
                        unsafe {
                            Self {
                                instructions: NonNull::new_unchecked(instructions),
                                index,
                            }
                        }
                    }
                }

                impl Backpatch for [<$variant Backpatcher>] {
                    fn backpatch_by(&mut self, value: isize) {
                        let instructions = unsafe { self.instructions.as_mut() };
                        instructions[self.index] = Instruction::$variant(value - self.index as isize);
                    }

                    fn backpatch(&mut self) {
                        let instructions = unsafe { self.instructions.as_mut() };
                        self.backpatch_by(instructions.len() as isize);
                    }
                }
            }
        )*
    };
}

define_backpatcher!(JumpIfFalse, Jump);
