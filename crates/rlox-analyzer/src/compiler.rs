use rlox_intermediate::*;

mod declaration;
mod expression;
mod statement;
mod utility;

enum FunctionType {
    Function,
    Script,
}

struct Local {
    name: String,
    depth: usize,
}

struct Compiler<'a> {
    offset: usize,
    locals: Vec<Local>,
    depth: usize,
    functions: Vec<FunctionBuilder>,
    function_type: FunctionType,

    /// Compile time heap allocation helps program runs faster at runtime.
    /// It does increase code complexity, though.
    heap: &'a mut Heap,
}

impl<'a> Compiler<'a> {
    fn new(heap: &'a mut Heap) -> Self {
        Self {
            offset: 0,
            locals: Vec::new(),
            depth: 0,
            functions: vec![FunctionBuilder::new()],
            function_type: FunctionType::Script,
            heap,
        }
    }

    fn current_function(&mut self) -> &mut FunctionBuilder {
        self.functions.last_mut().unwrap()
    }

    fn compile(&mut self, program: Vec<Declaration>) -> DiagnosableResult<Function> {
        while self.offset < program.len() {
            self.compile_declaration(&program[self.offset])?;
            self.offset += 1;
        }
        self.current_function().append(Instruction::Return);
        Ok(self.functions.pop().unwrap().build())
    }
}

pub fn compile(heap: &mut Heap, program: Vec<Declaration>) -> DiagnosableResult<Function> {
    let function = Compiler::new(heap).compile(program)?;
    #[cfg(feature = "disassembler")]
    {
        function.disassemble(function.name());
    }
    Ok(function)
}
