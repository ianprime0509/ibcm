use std::fmt;

/// The different I/O operations.
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum IoOp {
    ReadHex,
    ReadChar,
    WriteHex,
    WriteChar,
}

/// The different shift operations.
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum ShiftOp {
    ShiftLeft,
    ShiftRight,
    RotateLeft,
    RotateRight,
}

/// A single IBCM instruction.
///
/// See the official IBCM documentation for a description
/// of each operation.
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum Instruction {
    /// `0x0 halt`
    Halt,
    /// `0x1 i/o` (takes operation)
    Io(IoOp),
    /// `0x2 shift` (takes operation and amount)
    Shift(ShiftOp, u16),
    /// `0x3 load` (takes address)
    Load(u16),
    /// `0x4 store` (takes address)
    Store(u16),
    /// `0x5 add` (takes address)
    Add(u16),
    /// `0x6 sub` (takes address)
    Sub(u16),
    /// `0x7 and` (takes address)
    And(u16),
    /// `0x8 or` (takes address)
    Or(u16),
    /// `0x9 xor` (takes address)
    Xor(u16),
    /// `0xA not`
    Not,
    /// `0xB nop`
    Nop,
    /// `0xC jmp` (takes address)
    Jmp(u16),
    /// `0xD jmpe` (takes address)
    Jmpe(u16),
    /// `0xE jmpl` (takes address)
    Jmpl(u16),
    /// `0xF brl` (takes address)
    Brl(u16),
}

impl Instruction {
    /// Parses an instruction from the given u16 (word).
    pub fn from_u16(word: u16) -> Instruction {
        match word >> 12 {
            0x0 => Instruction::Halt,
            0x1 => {
                Instruction::Io(match (word >> 10) & 0b11 {
                    0 => IoOp::ReadHex,
                    1 => IoOp::ReadChar,
                    2 => IoOp::WriteHex,
                    3 => IoOp::WriteChar,
                    _ => unreachable!(),
                })
            }
            0x2 => {
                Instruction::Shift(match (word >> 10) & 0b11 {
                                       0 => ShiftOp::ShiftLeft,
                                       1 => ShiftOp::ShiftRight,
                                       2 => ShiftOp::RotateLeft,
                                       3 => ShiftOp::RotateRight,
                                       _ => unreachable!(),
                                   },
                                   word & 0xf)
            }
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

    /// Returns the value of the current instruction as a u16.
    pub fn to_u16(&self) -> u16 {
        match *self {
            Instruction::Halt => 0x0000,
            Instruction::Io(IoOp::ReadHex) => 0x1000,
            Instruction::Io(IoOp::ReadChar) => 0x1400,
            Instruction::Io(IoOp::WriteHex) => 0x1800,
            Instruction::Io(IoOp::WriteChar) => 0x1c00,
            Instruction::Shift(ShiftOp::ShiftLeft, n) => 0x2000 | (n & 0xf),
            Instruction::Shift(ShiftOp::ShiftRight, n) => 0x2400 | (n & 0xf),
            Instruction::Shift(ShiftOp::RotateLeft, n) => 0x2800 | (n & 0xf),
            Instruction::Shift(ShiftOp::RotateRight, n) => 0x2c00 | (n & 0xf),
            Instruction::Load(addr) => 0x3000 | (addr & 0xfff),
            Instruction::Store(addr) => 0x4000 | (addr & 0xfff),
            Instruction::Add(addr) => 0x5000 | (addr & 0xfff),
            Instruction::Sub(addr) => 0x6000 | (addr & 0xfff),
            Instruction::And(addr) => 0x7000 | (addr & 0xfff),
            Instruction::Or(addr) => 0x8000 | (addr & 0xfff),
            Instruction::Xor(addr) => 0x9000 | (addr & 0xfff),
            Instruction::Not => 0xa000,
            Instruction::Nop => 0xb000,
            Instruction::Jmp(addr) => 0xc000 | (addr & 0xfff),
            Instruction::Jmpe(addr) => 0xd000 | (addr & 0xfff),
            Instruction::Jmpl(addr) => 0xe000 | (addr & 0xfff),
            Instruction::Brl(addr) => 0xf000 | (addr & 0xfff),
        }
    }

    /// Returns the name of the current instruction.
    pub fn name(&self) -> &'static str {
        match *self {
            Instruction::Halt => "halt",
            Instruction::Io(IoOp::ReadHex) => "readH",
            Instruction::Io(IoOp::ReadChar) => "readC",
            Instruction::Io(IoOp::WriteHex) => "printH",
            Instruction::Io(IoOp::WriteChar) => "printC",
            Instruction::Shift(ShiftOp::ShiftLeft, _) => "shiftL",
            Instruction::Shift(ShiftOp::ShiftRight, _) => "shiftR",
            Instruction::Shift(ShiftOp::RotateLeft, _) => "rotL",
            Instruction::Shift(ShiftOp::RotateRight, _) => "rotR",
            Instruction::Load(_) => "load",
            Instruction::Store(_) => "store",
            Instruction::Add(_) => "add",
            Instruction::Sub(_) => "sub",
            Instruction::And(_) => "and",
            Instruction::Or(_) => "or",
            Instruction::Xor(_) => "xor",
            Instruction::Not => "not",
            Instruction::Nop => "nop",
            Instruction::Jmp(_) => "jmp",
            Instruction::Jmpe(_) => "jmpe",
            Instruction::Jmpl(_) => "jmpl",
            Instruction::Brl(_) => "brl",
        }
    }

    /// Returns the address argument of the current instruction, if any.
    pub fn address(&self) -> Option<u16> {
        match *self {
            Instruction::Load(n) |
            Instruction::Store(n) |
            Instruction::Add(n) |
            Instruction::Sub(n) |
            Instruction::And(n) |
            Instruction::Or(n) |
            Instruction::Xor(n) |
            Instruction::Jmp(n) |
            Instruction::Jmpe(n) |
            Instruction::Jmpl(n) |
            Instruction::Brl(n) => Some(n),
            _ => None,
        }
    }

    /// Returns whether the current instruction is a control flow construct
    /// (i.e. a jump or a branch).
    pub fn is_jmp(&self) -> bool {
        match *self {
            Instruction::Jmp(_) |
            Instruction::Jmpe(_) |
            Instruction::Jmpl(_) |
            Instruction::Brl(_) => true,
            _ => false,
        }
    }
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Instruction::Shift(_, n) => write!(f, "{} {}", self.name(), n),
            _ => {
                if let Some(addr) = self.address() {
                    write!(f, "{} {:04x}", self.name(), addr)
                } else {
                    write!(f, "{}", self.name())
                }
            }
        }
    }
}
