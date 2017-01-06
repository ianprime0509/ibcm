//! The IBCM simulation.

use std::io::{self, Read, Write, BufRead, BufReader};

use errors::*;
use instruction::{Instruction, IoOp, ShiftOp};

/// The IBCM machine simulator.
pub struct Simulator {
    /// Internal memory
    memory: [u16; 4096],
    /// The accumulator
    acc: i16,
    /// Instruction register
    ir: u16,
    /// Program counter
    pc: u16,
    /// Whether the machine has been halted
    halted: bool,
}

impl Simulator {
    /// Load the simulator from the given instructions.
    pub fn from_instructions(input: &[u16]) -> Result<Simulator> {
        if input.len() > 4096 {
            return Err(ErrorKind::ProgramTooLong.into());
        }

        let mut data = [0u16; 4096];
        data[..input.len()].copy_from_slice(input);

        Ok(Simulator {
            memory: data,
            acc: 0,
            ir: 0,
            pc: 0,
            halted: false,
        })
    }

    /// Load the simulator from the given binary data.
    pub fn from_binary<R: Read>(input: R) -> Result<Simulator> {
        let mut data = [0u16; 4096];
        let mut i = 0;
        // Whether we're filling the top half of the byte
        let mut upper = true;

        for b in input.bytes() {
            let b = b.chain_err(|| ErrorKind::Io("could not read from binary input".into()))?;
            if i >= data.len() {
                return Err(ErrorKind::ProgramTooLong.into());
            }

            if upper {
                data[i] |= (b as u16) << 8;
            } else {
                data[i] |= b as u16;
                i += 1;
            }
            upper = !upper;
        }

        Ok(Simulator {
            memory: data,
            acc: 0,
            ir: 0,
            pc: 0,
            halted: false,
        })
    }

    /// Load the simulator from text input containing the instructions in hex format.
    ///
    /// Expects one instruction per line, and lines may begin with `//` to denote a comment.
    pub fn from_hex<R: Read>(input: R) -> Result<Simulator> {
        let mut data = [0u16; 4096];
        let mut i = 0;
        let br = BufReader::new(input);

        for l in br.lines() {
            let l = l.chain_err(|| ErrorKind::Io("could not read from hex input".into()))?;
            let l = l.trim();
            if l.is_empty() || l.starts_with("//") {
                continue;
            }
            // Try to read a word
            let word = u16::from_str_radix(&l[..4], 16).chain_err(|| ErrorKind::UserInput(format!("expected hexadecimal word at start of line: '{}'", l)))?;
            if i >= data.len() {
                return Err(ErrorKind::ProgramTooLong.into());
            }
            data[i] = word;
            i += 1;
        }

        Ok(Simulator {
            memory: data,
            acc: 0,
            ir: 0,
            pc: 0,
            halted: false,
        })
    }

    /// Returns a reference to the memory.
    pub fn memory(&self) -> &[u16] {
        &self.memory
    }

    /// Dumps memory in a nice format to stdout.
    pub fn dump(&self, amt: usize) {
        println!("    |   0|   1|   2|   3|   4|   5|   6|   7|   8|   9|   A|   B|   C|   D|   E|   F");
        println!("------------------------------------------------------------------------------------");
        let mut row = 0;
        for chunk in self.memory[..amt].chunks(16) {
            print!("  {:02x}", row);
            for w in chunk {
                print!("|{:04x}", w);
            }
            println!("");
            row += 1;
        }
    }

    /// Performs a single step in the code.
    ///
    /// If the step was successful, returns whether the
    /// machine was halted. Note that if the machine is already
    /// halted when this method is called, there will be an error.
    pub fn step(&mut self) -> Result<bool> {
        // Load the instruction and increment the program counter
        if self.pc >= self.memory.len() as u16 {
            return Err(ErrorKind::OutOfBounds.into());
        }
        self.ir = self.memory[self.pc as usize];
        self.pc += 1;

        let ins = self.ir;
        self.execute(Instruction::from_u16(ins))?;
        Ok(self.halted)
    }

    /// Runs the loaded program until it halts.
    pub fn run(&mut self) -> Result<()> {
        loop {
            match self.step() {
                Ok(false) => {}
                Ok(true) => return Ok(()),
                Err(e) => return Err(e),
            }
        }
    }

    /// Executes a single instruction.
    ///
    /// This will return an error if the machine has been halted.
    fn execute(&mut self, ins: Instruction) -> Result<()> {
        if self.halted {
            return Err(ErrorKind::Halted.into());
        }
        match ins {
            Instruction::Halt => {
                self.halted = true;
            }
            Instruction::Io(IoOp::ReadHex) => {
                self.acc = read_hex()? as i16;
            }
            Instruction::Io(IoOp::ReadChar) => {
                self.acc = read_u8()? as i16;
            }
            Instruction::Io(IoOp::WriteHex) => {
                println!("{:04x}", self.acc);
            }
            Instruction::Io(IoOp::WriteChar) => {
                println!("{}", self.acc as u8 as char);
            }
            Instruction::Shift(ShiftOp::ShiftLeft, n) => {
                self.acc <<= n;
            }
            Instruction::Shift(ShiftOp::ShiftRight, n) => {
                // Appears to be an unsigned shift in the canonical source code
                self.acc = ((self.acc as u16) >> n) as i16;
            }
            Instruction::Shift(ShiftOp::RotateLeft, n) => {
                self.acc = self.acc.rotate_left(n as u32);
            }
            Instruction::Shift(ShiftOp::RotateRight, n) => {
                self.acc = self.acc.rotate_right(n as u32);
            }
            Instruction::Load(addr) => {
                self.acc = self.memory[addr as usize] as i16;
            }
            Instruction::Store(addr) => {
                self.memory[addr as usize] = self.acc as u16;
            }
            Instruction::Add(addr) => {
                self.acc = self.acc.wrapping_add(self.memory[addr as usize] as i16);
            }
            Instruction::Sub(addr) => {
                self.acc = self.acc.wrapping_sub(self.memory[addr as usize] as i16);
            }
            Instruction::And(addr) => {
                self.acc &= self.memory[addr as usize] as i16;
            }
            Instruction::Or(addr) => {
                self.acc |= self.memory[addr as usize] as i16;
            }
            Instruction::Xor(addr) => {
                self.acc ^= self.memory[addr as usize] as i16;
            }
            Instruction::Not => {
                self.acc = !self.acc;
            }
            Instruction::Nop => {}
            Instruction::Jmp(addr) => {
                self.pc = addr;
            }
            Instruction::Jmpe(addr) => {
                if self.acc == 0 {
                    self.pc = addr;
                }
            }
            Instruction::Jmpl(addr) => {
                if self.acc < 0 {
                    self.pc = addr;
                }
            }
            Instruction::Brl(addr) => {
                self.acc = self.pc as i16;
                self.pc = addr;
            }
        }

        Ok(())
    }
}

/// Reads a hexadecimal word from stdin.
fn read_hex() -> Result<u16> {
    print!("Enter hexadecimal word: ");
    io::stdout().flush().expect("could not flush stdout");

    let mut input = String::new();
    io::stdin().read_line(&mut input).chain_err(|| ErrorKind::Io("could not read user input".into()))?;
    let hex = input.trim();
    
    // Validate input
    if hex.len() >= 1 && hex.len() <= 4 {
        Ok(u16::from_str_radix(hex, 16).chain_err(|| ErrorKind::UserInput(format!("'{}' is not a valid hexadecimal word", hex)))?)
    } else {
        Err(ErrorKind::UserInput(format!("'{}' is not a valid hexadecimal word (should be at most 4 hexadecimal digits)", hex)).into())
    }
}

/// Reads a single ASCII character from stdin.
fn read_u8() -> Result<u8> {
    print!("Enter ASCII character: ");
    io::stdout().flush().expect("could not flush stdout");

    let mut input = String::new();
    io::stdin().read_line(&mut input).chain_err(|| ErrorKind::Io("could not read user input".into()))?;
    let tr = input.trim();
    let ch = tr.as_bytes();

    if ch.len() == 1 {
        Ok(ch[0])
    } else {
        Err(ErrorKind::UserInput(format!("expected a single ASCII character; got '{}'", tr)).into())
    }
}
