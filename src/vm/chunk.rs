use std::fmt::Debug;

use super::{OpCode, Value};

pub struct Chunk {
    // TODO: lines could be run-length encoded to save space
    /// Position information for error handling
    lines: Vec<usize>,

    constants: Vec<Value>,
    code: Vec<u8>,
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            lines: Vec::new(),
            constants: Vec::new(),
            code: Vec::new(),
        }
    }

    /// Pushes a byte to the chunk returning its position
    pub fn push<T: Into<u8>>(&mut self, byte: T, line: usize) -> usize {
        self.code.push(byte.into());
        self.lines.push(line);
        self.code.len() - 1
    }

    /// Pushes a constant to the chunk and writes the approriate instruction to load that constant
    /// NOTE: `push_constant` will write CONSTANT instructions for the first 2^8 constants pushed.
    /// Afterwards it will start using CONTANTLONG instructions to ensure enough addreses are
    /// available.
    pub fn push_constant<T: Into<Value>>(&mut self, constant: T, line: usize) {
        self.constants.push(constant.into());
        if self.constants.len() > 256 {
            self.push(OpCode::CONSTANTLONG, line);
            // Take first 3 bytes to make 24 bit location
            for byte in (self.constants.len() - 1).to_le_bytes().iter().take(3) {
                self.push(*byte, line);
            }
        } else {
            self.push(OpCode::CONSTANT, line);
            self.push((self.constants.len() - 1) as u8, line);
        }
    }

    /// Disassemples the chunk by constructing a string representing the inner bytecode
    fn disassemple(&self) -> String {
        let mut out = String::new();
        let mut offset = 0;
        while offset < self.code.len() {
            out += &format!("{:0>4} ", offset);

            if offset > 0 && self.lines[offset - 1] == self.lines[offset] {
                out += "   | ";
            } else {
                out += &format!("{:0>4} ", self.lines[offset]);
            }

            let op: OpCode = self.code[offset].into();
            offset += 1;
            out += &format!("{:<12} ", format!("{:?}", op));
            let immidiates = match op {
                OpCode::RETURN => String::new(),
                OpCode::CONSTANT => {
                    let constant = self.code[offset];
                    let value = self.constants[constant as usize];
                    offset += 1;
                    format!("{:0>4} '{:#?}'", constant, value)
                }
                OpCode::CONSTANTLONG => {
                    let constant = u32::from_le_bytes([
                        self.code[offset],
                        self.code[offset + 1],
                        self.code[offset + 2],
                        0,
                    ]);
                    let value = self.constants[constant as usize];
                    offset += 3;
                    format!("{:0>12} '{:#?}'", constant, value)
                }
            };
            out += &immidiates;
            out += "\n";
        }

        out
    }
}

impl Debug for Chunk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.disassemple())
    }
}
