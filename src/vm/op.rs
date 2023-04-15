#[derive(Copy, Clone, Debug)]
#[repr(u8)]
pub enum OpCode {
    RETURN = 0,
    CONSTANT = 1,
    CONSTANTLONG = 2,
    NEGATE = 3,
    ADD = 4,
    SUBTRACT = 5,
    MULTIPLY = 6,
    DIVIDE = 7,
}

impl From<OpCode> for u8 {
    fn from(op: OpCode) -> Self {
        op as u8
    }
}

impl From<u8> for OpCode {
    fn from(op: u8) -> Self {
        match op {
            0 => OpCode::RETURN,
            1 => OpCode::CONSTANT,
            2 => OpCode::CONSTANTLONG,
            3 => OpCode::NEGATE,
            4 => OpCode::ADD,
            5 => OpCode::SUBTRACT,
            6 => OpCode::MULTIPLY,
            7 => OpCode::DIVIDE,
            _ => panic!("Unrecognized op code '{}'", op),
        }
    }
}
