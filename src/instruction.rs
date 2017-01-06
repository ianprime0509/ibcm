/// The different I/O operations
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum IoOp {
    ReadHex,
    ReadChar,
    WriteHex,
    WriteChar,
}

/// The different shift operations
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum ShiftOp {
    ShiftLeft,
    ShiftRight,
    RotateLeft,
    RotateRight,
}

/// A single IBCM instruction
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum Instruction {
    Halt,
    Io(IoOp),
    Shift(ShiftOp, u16),
    Load(u16),
    Store(u16),
    Add(u16),
    Sub(u16),
    And(u16),
    Or(u16),
    Xor(u16),
    Not,
    Nop,
    Jmp(u16),
    Jmpe(u16),
    Jmpl(u16),
    Brl(u16),
}

impl Instruction {
    /// Parses an instruction from the given u16 (word)
    pub fn from_u16(word: u16) -> Instruction {
        match word >> 12 {
            0x0 => Instruction::Halt,
            0x1 => Instruction::Io(match (word >> 10) & 0b11 {
                0 => IoOp::ReadHex,
                1 => IoOp::ReadChar,
                2 => IoOp::WriteHex,
                3 => IoOp::WriteChar,
                _ => unreachable!(),
            }),
            0x2 => Instruction::Shift(match (word >> 10) & 0b11 {
                0 => ShiftOp::ShiftLeft,
                1 => ShiftOp::ShiftRight,
                2 => ShiftOp::RotateLeft,
                3 => ShiftOp::RotateRight,
                _ => unreachable!(),
            }, word & 0xf),
            0x3 => Instruction::Load(word & 0xfff),
            0x4 => Instruction::Store(word & 0xfff),
            0x5 => Instruction::Add(word & 0xfff),
            0x6 => Instruction::Sub(word & 0xfff),
            0x7 => Instruction::And(word & 0xfff),
            0x8 => Instruction::Or(word & 0xfff),
            0x9 => Instruction::Xor(word & 0xfff),
            0xA => Instruction::Not,
            0xB => Instruction::Nop,
            0xC => Instruction::Jmp(word & 0xfff),
            0xD => Instruction::Jmpe(word & 0xfff),
            0xE => Instruction::Jmpl(word & 0xfff),
            0xF => Instruction::Brl(word & 0xfff),
            _ => unreachable!(),
        }
    }
}

