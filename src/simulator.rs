//! The IBCM simulation.

use std::io::{self, Read, Write, BufRead, BufReader, BufWriter};

use errors::*;
use instruction::{Instruction, IoOp, ShiftOp};

/// The IBCM machine simulator.
///
/// This manages the state of a simulated IBCM machine, which consists
/// of 4096 words (i.e. `u16`s) of memory and the three registers
/// (the accumulator, instruction register, and program counter).
/// Since the IBCM contains I/O instructions, by default the simulator
/// will use the standard input and output to handle these instructions.
/// In some circumstances, it may be necessary to redirect these,
/// which can be done by means of the `set_input` and `set_output` methods.
///
/// # Examples
///
/// A simple program, which copies the contents of one memory cell to another:
///
/// ```
/// use ibcm::Simulator;
///
/// let program = "// Jump to beginning
/// c003
/// // Source
/// 1234
/// // Destination
/// 0000
/// // Load source and then store in destination
/// 3001
/// 4002
/// // End program
/// 0000";
///
/// // Simulate the program
/// let mut sim = Simulator::from_hex(program.as_bytes()).unwrap();
///
/// assert_eq!(&[0x1234, 0x0000], &sim.memory()[1..3]);
/// sim.run().unwrap();
/// assert_eq!(&[0x1234, 0x1234], &sim.memory()[1..3]);
/// ```
///
/// For more complicated programs, it is much more convenient to write
/// IBCM assembly and to use an `Assembler` to convert it to this format.
pub struct Simulator<'a, 'b> {
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
    /// The actual length of the program
    len: usize,
    /// The source of input data
    input: Box<BufRead + 'a>,
    /// The destination of output data
    output: Box<Write + 'b>,
    /// Whether to show a prompt for input
    show_prompt: bool,
}

impl<'a, 'b> Simulator<'a, 'b> {
    /// Load the simulator from the given memory buffer.
    ///
    /// Requires an argument specifying the length of the program,
    /// for correct compilation output.
    fn from_memory(memory: [u16; 4096], len: usize) -> Self {
        Simulator {
            memory: memory,
            acc: 0,
            ir: 0,
            pc: 0,
            halted: false,
            len: len,
            input: Box::new(BufReader::new(io::stdin())),
            output: Box::new(io::stdout()),
            show_prompt: true,
        }
    }

    /// Load the simulator from the given instructions.
    ///
    /// # Examples
    ///
    /// ```
    /// use ibcm::Simulator;
    ///
    /// let mem = &[0x1000, 0x1800, 0x0000];
    /// let sim = Simulator::from_instructions(mem).unwrap();
    /// 
    /// assert_eq!(mem, &sim.memory()[..3]);
    /// ```
    pub fn from_instructions(input: &[u16]) -> Result<Self> {
        if input.len() > 4096 {
            return Err(ErrorKind::ProgramTooLong.into());
        }

        let mut data = [0u16; 4096];
        data[..input.len()].copy_from_slice(input);

        Ok(Simulator::from_memory(data, input.len()))
    }

    /// Load the simulator from the given binary data.
    ///
    /// # Examples
    ///
    /// ```
    /// use ibcm::Simulator;
    ///
    /// let input: &[u8] = &[0x00, 0x10, 0x00, 0x18, 0x00, 0x00];
    /// let sim = Simulator::from_binary(input).unwrap();
    ///
    /// assert_eq!(&[0x1000, 0x1800, 0x0000], &sim.memory()[..3]);
    /// ```
    pub fn from_binary<R: Read>(input: R) -> Result<Self> {
        let mut data = [0u16; 4096];
        let mut i = 0;
        // Whether we're filling the top half of the byte.
        // This is initially false because we're treating input as
        // little-endian for compatibility with the reference
        // implementation.
        let mut upper = false;

        for b in input.bytes() {
            let b = b.chain_err(|| ErrorKind::Io("could not read from binary input".into()))?;
            if i >= data.len() {
                return Err(ErrorKind::ProgramTooLong.into());
            }

            if upper {
                data[i] |= (b as u16) << 8;
                i += 1;
            } else {
                data[i] |= b as u16;
            }
            upper = !upper;
        }

        Ok(Simulator::from_memory(data, i))
    }

    /// Load the simulator from text input containing the instructions in hex format.
    ///
    /// Expects one instruction per line, and lines may begin with `//` to denote a comment.
    ///
    /// # Examples
    ///
    /// ```
    /// use ibcm::Simulator;
    ///
    /// let program = "// A comment
    /// 1000
    ///     1800  // Indentation is supported
    /// 0000";
    /// let sim = Simulator::from_hex(program.as_bytes()).unwrap();
    ///
    /// assert_eq!(&[0x1000, 0x1800, 0x0000], &sim.memory()[..3]);
    /// ```
    pub fn from_hex<R: Read>(input: R) -> Result<Self> {
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
            let word = u16::from_str_radix(&l[..4], 16).chain_err(|| {
                    ErrorKind::UserInput(format!("expected hexadecimal word at start of line: \
                                                  '{}'",
                                                 l))
                })?;
            if i >= data.len() {
                return Err(ErrorKind::ProgramTooLong.into());
            }
            data[i] = word;
            i += 1;
        }

        Ok(Simulator::from_memory(data, i))
    }

    /// Writes the memory of the simulator in binary format.
    ///
    /// # Examples
    ///
    /// ```
    /// use ibcm::Simulator;
    ///
    /// let mut output = Vec::new();
    /// {
    ///     let sim = Simulator::from_instructions(&[0x1000, 0x1800, 0x0000]).unwrap();
    ///     sim.to_binary(&mut output).unwrap();
    /// }
    ///
    /// assert_eq!(&[0x00, 0x10, 0x00, 0x18, 0x00, 0x00], output.as_slice());
    /// ```
    pub fn to_binary<W: Write>(&self, input: W) -> Result<()> {
        let mut bw = BufWriter::new(input);

        // Output the binary
        for &w in &self.memory[..self.len] {
            // The IBCM is big-endian, but output should be
            // little-endian for compatibility with the reference
            // implementation (which does not support big-endian
            // output).
            bw.write(&[(w & 0xff) as u8, ((w >> 8) & 0xff) as u8])
                .chain_err(|| ErrorKind::Io("could not write to file".into()))?;
        }

        Ok(())
    }

    /// Writes the memory of the simulator in hexadecimal format.
    ///
    /// # Examples
    ///
    /// ```
    /// use ibcm::Simulator;
    ///
    /// let mut output = Vec::new();
    /// {
    ///     let sim = Simulator::from_instructions(&[0x1000, 0x1800, 0x0000]).unwrap();
    ///     sim.to_hex(&mut output).unwrap();
    /// }
    /// let expected = b"1000
    /// 1800
    /// 0000
    /// ";
    ///
    /// assert_eq!(expected, output.as_slice());
    /// ```
    pub fn to_hex<W: Write>(&self, input: W) -> Result<()> {
        let mut bw = BufWriter::new(input);

        // Output the hex file
        for w in &self.memory[..self.len] {
            writeln!(bw, "{:04x}", w).chain_err(|| ErrorKind::Io("could not write to file".into()))?;
        }

        Ok(())
    }

    /// Returns a reference to the memory.
    pub fn memory(&self) -> &[u16] {
        &self.memory
    }

    /// Returns the instruction at the given position in memory.
    ///
    /// # Panics
    ///
    /// This will panic if the address given is out of range of the memory
    /// (e.g. if `addr >= 4096`).
    pub fn instruction_at(&self, addr: u16) -> Instruction {
        Instruction::from_u16(self.memory[addr as usize])
    }

    /// Returns the current instruction, returning an error if
    /// the program has overflowed its memory.
    pub fn current_instruction(&self) -> Result<Instruction> {
        if self.pc >= self.memory.len() as u16 {
            return Err(ErrorKind::OutOfBounds.into());
        }
        Ok(self.instruction_at(self.pc))
    }

    /// Returns the registers: (acc, ir, pc).
    pub fn regs(&self) -> (i16, u16, u16) {
        (self.acc, self.ir, self.pc)
    }

    /// Returns whether the machine has been halted.
    pub fn is_halted(&self) -> bool {
        self.halted
    }

    /// Sets the input stream of the program.
    pub fn set_input<R: BufRead + 'a>(&mut self, input: R) {
        self.input = Box::new(input);
    }

    /// Sets the output stream of the program, and takes an additional
    /// argument specifying whether a prompt should be shown for input.
    pub fn set_output<W: Write + 'b>(&mut self, output: W, show_prompt: bool) {
        self.output = Box::new(output);
        self.show_prompt = show_prompt;
    }

    /// Dumps memory in a nice format to the output.
    pub fn dump(&mut self, amt: usize) -> Result<()> {
        for (i, chunk) in (&self.memory[..amt]).chunks(8).enumerate() {
            write!(&mut self.output, "{:03x}:", 8 * i).chain_err(|| ErrorKind::Io("could not write to output".into()))?;
            for w in chunk {
                write!(&mut self.output, " {:04x}", w).chain_err(|| ErrorKind::Io("could not write to output".into()))?;
            }
            writeln!(&mut self.output, "").chain_err(|| ErrorKind::Io("could not write to output".into()))?;
        }

        Ok(())
    }

    /// Performs a single step in the code.
    ///
    /// If the step was successful, returns whether the
    /// machine was halted. Note that if the machine is already
    /// halted when this method is called, there will be an error.
    pub fn step(&mut self) -> Result<bool> {
        // Load the instruction and increment the program counter
        let ins = self.current_instruction()?;
        self.pc += 1;

        self.execute(ins)?;
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
                self.acc = self.read_hex()? as i16;
            }
            Instruction::Io(IoOp::ReadChar) => {
                self.acc = self.read_u8()? as i16;
            }
            Instruction::Io(IoOp::WriteHex) => {
                writeln!(&mut self.output, "{:04x}", self.acc).chain_err(|| ErrorKind::Io("could not write to output".into()))?;
            }
            Instruction::Io(IoOp::WriteChar) => {
                writeln!(&mut self.output, "{}", self.acc as u8 as char).chain_err(|| ErrorKind::Io("could not write to output".into()))?;
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

    /// Reads a hexadecimal word from stdin.
    fn read_hex(&mut self) -> Result<u16> {
        // Show a prompt if this feature is enabled
        if self.show_prompt {
            write!(&mut self.output, "Enter hexadecimal word: ").chain_err(|| ErrorKind::Io("could not write to output".into()))?;
            self.output.flush().chain_err(|| ErrorKind::Io("could not display prompt".into()))?;
        }

        // We expect one hexadecimal word (4 bytes) per line
        let mut input = String::new();
        self.input
            .read_line(&mut input)
            .chain_err(|| ErrorKind::Io("could not read user input".into()))?;
        let hex = input.trim();

        // Validate input
        if hex.len() >= 1 && hex.len() <= 4 {
            Ok(u16::from_str_radix(hex, 16).chain_err(|| {
                    ErrorKind::UserInput(format!("'{}' is not a valid hexadecimal word", hex))
                })?)
        } else {
            Err(ErrorKind::UserInput(format!("'{}' is not a valid hexadecimal word (should be \
                                              at most 4 hexadecimal digits)",
                                             hex))
                .into())
        }
    }

    /// Reads a single ASCII character from stdin.
    fn read_u8(&mut self) -> Result<u8> {
        if self.show_prompt {
            write!(&mut self.output, "Enter ASCII character: ").chain_err(|| ErrorKind::Io("could not write to output".into()))?;
            self.output.flush().chain_err(|| ErrorKind::Io("could not display prompt".into()))?;
        }

        // We expect one character per line
        let mut input = String::new();
        self.input
            .read_line(&mut input)
            .chain_err(|| ErrorKind::Io("could not read user input".into()))?;
        let tr = input.trim();
        let ch = tr.as_bytes();

        if ch.len() == 1 {
            Ok(ch[0])
        } else {
            Err(ErrorKind::UserInput(format!("expected a single ASCII character; got '{}'", tr))
                .into())
        }
    }
}
