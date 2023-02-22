use super::{Chunk, OpCode};

#[derive(Debug)]
pub enum InterpretError{ 
    CompileError, RuntimeError
}

pub struct VirtualMachine {
    ip: usize,
    chunk: Chunk
}

impl VirtualMachine {
    pub fn new(chunk: Chunk) -> Self {
        Self { ip: 0, chunk }
    }

    fn next(&mut self) -> u8 {
        self.ip += 1;
        self.chunk.code[self.ip - 1]
    }

    fn next_instruction(&mut self) -> OpCode {
        self.next().into()
    }

    pub fn run(&mut self) -> Result<(), InterpretError> {
        self.ip = 0; // Reset instruction pointer before we start
        loop {
            match self.next_instruction() {
                OpCode::RETURN => return Ok(()),
                OpCode::CONSTANT => {
                    let constant = self.next();
                    let value = self.chunk.constants[constant as usize];
                    println!("{:#?}", value);
                }
                OpCode::CONSTANTLONG => {
                    let constant = u32::from_le_bytes([
                        self.next(), self.next(), self.next(), 0
                    ]);
                    let value = self.chunk.constants[constant as usize];
                    println!("{:#?}", value);
                }
                _ => {}
            }

            
        }
    }
}


