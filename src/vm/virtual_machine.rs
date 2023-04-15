use super::{Chunk, OpCode, Value};

pub const STACK_SIZE: usize = 256;

#[derive(Debug)]
pub enum InterpretError {
    RuntimeError,
}

pub struct VirtualMachine {
    ip: usize,
    chunk: Chunk,
    stack: Vec<Value>,
}

impl VirtualMachine {
    pub fn new(chunk: Chunk) -> Self {
        Self {
            ip: 0,
            chunk,
            stack: Vec::with_capacity(STACK_SIZE),
        }
    }

    fn next(&mut self) -> u8 {
        self.ip += 1;
        self.chunk.code[self.ip - 1]
    }

    fn next_instruction(&mut self) -> OpCode {
        self.next().into()
    }

    fn push(&mut self, value: Value) -> Result<(), InterpretError> {
        if self.stack.len() >= STACK_SIZE {
            return Err(InterpretError::RuntimeError);
        }
        self.stack.push(value);
        Ok(())
    }

    fn pop(&mut self) -> Result<Value, InterpretError> {
        match self.stack.pop() {
            Some(value) => Ok(value),
            None => Err(InterpretError::RuntimeError),
        }
    }

    pub fn run(&mut self) -> Result<Value, InterpretError> {
        self.ip = 0; // Reset instruction pointer before we start

        loop {
            // TODO: Install a loggin crate that can turn this off when not in debug
            println!(
                "\t[{}]",
                self.stack
                    .iter()
                    .map(|v| format!("{:#?}", v))
                    .collect::<Vec<String>>()
                    .join(", ")
            );
            println!("{}", self.chunk.disassemple_inctruction(self.ip).0);

            match self.next_instruction() {
                OpCode::RETURN => {
                    return self.pop();
                },
                OpCode::ADD => {
                    let (b, a) = (self.pop()?, self.pop()?);
                    self.push(a + b)?;
                }
                OpCode::SUBTRACT => {
                    let (b, a) = (self.pop()?, self.pop()?);
                    self.push(a - b)?;
                }
                OpCode::MULTIPLY => {
                    let (b, a) = (self.pop()?, self.pop()?);
                    self.push(a * b)?;
                }
                OpCode::DIVIDE => {
                    let (b, a) = (self.pop()?, self.pop()?);
                    self.push(a / b)?;
                }
                OpCode::CONSTANT => {
                    let constant = self.next();
                    let value = self.chunk.constants[constant as usize];
                    self.push(value)?;
                }
                OpCode::CONSTANTLONG => {
                    let constant = u32::from_le_bytes([self.next(), self.next(), self.next(), 0]);
                    let value = self.chunk.constants[constant as usize];
                    self.push(value)?;
                }
                OpCode::NEGATE => {
                    let value = self.pop()?;
                    self.push(-value)?;
                }
            }
        }
    }
}
