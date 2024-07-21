use std::ops::{Deref, DerefMut};

use crate::{Chunk, ChunkBuilder};

pub struct FunctionBuilder {
    pub arity: usize,
    pub chunk: ChunkBuilder,
    pub name: String,
}

impl FunctionBuilder {
    pub fn new() -> Self {
        Self {
            arity: 0,
            chunk: ChunkBuilder::new(),
            name: String::new(),
        }
    }

    pub fn build(self) -> Function {
        Function {
            arity: self.arity,
            chunk: self.chunk.build(),
            name: self.name,
        }
    }
}

impl Deref for FunctionBuilder {
    type Target = ChunkBuilder;

    fn deref(&self) -> &Self::Target {
        &self.chunk
    }
}

impl DerefMut for FunctionBuilder {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.chunk
    }
}

pub struct Function {
    arity: usize,
    chunk: Chunk,
    name: String,
}

impl Function {
    pub fn arity(&self) -> usize {
        self.arity
    }

    pub fn chunk(&self) -> &Chunk {
        &self.chunk
    }

    pub fn name(&self) -> &String {
        &self.name
    }
}

impl Deref for Function {
    type Target = Chunk;

    fn deref(&self) -> &Self::Target {
        &self.chunk
    }
}
