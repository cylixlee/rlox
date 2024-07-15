use rlox_intermediate::{Chunk, DiagnosableResult, Instruction};

pub struct VirtualMachine {
    chunk: Chunk,
    program_count: usize,
}

impl VirtualMachine {
    pub fn new() -> Self {
        Self {
            chunk: Chunk::new(),
            program_count: 0,
        }
    }

    pub fn run(&mut self, chunk: Chunk) -> DiagnosableResult {
        self.chunk = chunk;
        self.program_count = 0;
        self.execute()
    }

    #[allow(unreachable_code)]
    fn execute(&mut self) -> DiagnosableResult {
        while self.program_count < self.chunk.len() {
            let instruction = &self.chunk[self.program_count];
            println!("{:?}", instruction);
            match instruction {
                Instruction::Return => break,
            }
            self.program_count += 1;
        }
        Ok(())
    }
}
