//! The assembler.

use std::collections::HashMap;
use std::io::{Read, BufRead, BufReader};

use instruction::{Instruction, IoOp, ShiftOp};
use errors::*;

/// A single statement, which may have as its argument a label
/// whose position is not yet known.
enum Stmt {
    /// An instruction with optional argument.
    Instr {
        instr: Instruction,
        addr: Option<String>,
    },
    /// A `dw` statement.
    Data(String),
}

/// Represents the state of the assembler.
///
/// This struct should never be used directly as an object; its only use
/// for library consumers is to use the `assemble` associated function, which
/// will process IBCM assembly and return an assembled `Program`, which in turn
/// can be used in a `Simulator`.
///
/// # Assembly language
///
/// The IBCM assembly language is not actually defined in the language specification
/// itself, which defines the machine only in terms of the raw `u16` instructions
/// on which it operates. In the example programs, however, several "opcodes" are used
/// to clarify the hexadecimal instructions (the example programs may be found
/// in the [official repository](https://github.com/aaronbloomfield/pdr/tree/master/ibcm)
/// of the IBCM language). These opcodes were used as the basis for the assembly language,
/// which adds the very convenient feature of labels (avoiding the need for manual
/// calculation of memory locations, which is error-prone).
///
/// An IBCM assembly program consists of a sequence of *statements*, each of which
/// must occupy its own line. A statement consists of an opcode and, if applicable,
/// an argument, separated by whitespace. For example:
///
/// ```text
/// halt
/// dw      000A
/// jmp     label
/// ```
///
/// IBCM assembly may also contain *labels*, which consist of a sequence of non-whitespace
/// characters followed by a single colon (`:`). Each label refers to the statement directly
/// following it; you may place up to one label on the same line as a statement,
/// and an arbitrary number on the lines preceding it (which will all refer to the same
/// statement). For example, all the labels in the following refer to the `halt` statement:
///
/// ```text
/// label1:
/// @!#:
/// 01234:
/// 标签: halt
/// ```
///
/// As can be seen in the example above, the only requirement for a label is that it not
/// contain any whitespace or colons (I might change this later, but this is unlikely),
/// and that it must be valid UTF8. This also means that all arguments to opcodes expecting
/// an address must be labels, and cannot be (say) references to a specific memory location.
/// However, since this isn't actual assembly where such things are useful, this shouldn't
/// be a problem.
///
/// Indentation and whitespace within a line is ignored, allowing for clearer formatting.
/// Additionally, comments may appear in the code: the characters `//` will cause the
/// rest of the line to be treated as a comment, as in C++.
///
/// For examples of IBCM assembly, see the section below, as well as the examples in the
/// `tests` directory of this project.
///
/// # Examples
///
/// A simple program which copies a value in memory, as seen in the documentation of
/// `ibcm::Simulator`.
///
/// ```
/// use ibcm::Assembler;
///
/// let program = "// Jump to beginning
///         jmp     init
/// // Source
/// src:    dw      1234
/// // Destination
/// dest:   dw      0000
///
/// init:
///         // Load source and then store in destination
///         load    src
///         store   dest
///         // End program
///         halt";
///
/// let assembled = Assembler::assemble(program.as_bytes()).unwrap();
/// 
/// assert_eq!(assembled.data(), &[0xc003, 0x1234, 0x0000, 0x3001, 0x4002, 0x0000]);
/// ```
///
/// A more complicated program, which multiplies two numbers:
///
/// ```
/// use ibcm::{Assembler, Simulator};
///
/// let program = "// Jump to beginning
///         jmp     init
/// // Numbers to multiply
/// m:      dw      5
/// n:      dw      7
/// // The result
/// prod:   dw      0
/// // Constants
/// 1:      dw      1
///
/// init:
///         // In a real program, we would read in
///         // m and n as input here
///         load    m
///         // Loop: decrement m until it equals 0,
///         // each time adding another n to the product
/// loop:   jmpe    end
///         store   m
///         load    prod
///         add     n
///         store   prod
///         load    m
///         sub     1
///         jmp     loop
///
/// // Here is where we'd output the result
/// end:    load    prod
///         // printH
///         halt";
///
/// let assembled = Assembler::assemble(program.as_bytes()).unwrap();
/// let mut sim = Simulator::from_instructions(assembled.data()).unwrap();
///
/// sim.run().unwrap();
/// // At this point, the accumulator should hold the result
/// let (acc, _, _) = sim.regs();
/// 
/// assert_eq!(acc, 35);
/// ```
///
/// A much more complicated version of the above, using a recursive multiplication routine,
/// has been transcribed from the official IBCM documentation and can be found in the `tests`
/// directory.
pub struct Assembler {
    /// The statements that have been processed, along with their line numbers.
    stmts: Vec<(usize, Stmt)>,
    /// A map giving the position of labels.
    labels: HashMap<String, u16>,
}

/// Represents an assembled program.
///
/// Currently, this contains the actual assembled program as a list of
/// `u16` instructions, as well as a `HashMap` which gives the position
/// of labels in the code.
pub struct Program {
    data: Vec<u16>,
    labels: HashMap<String, u16>,
}

impl Program {
    /// Returns the program instructions.
    pub fn data(&self) -> &[u16] {
        self.data.as_slice()
    }

    /// Returns the labels.
    pub fn labels(&self) -> &HashMap<String, u16> {
        &self.labels
    }
}

impl Assembler {
    /// Assembles the assembly code from the given reader.
    ///
    /// Returns a vector of IBCM instructions, or an error. See the documentation
    /// for the `Assembler` struct for a description of the assembly code format
    /// and examples.
    pub fn assemble<R: Read>(input: R) -> Result<Program> {
        let asm = Assembler::first_pass(input)?;
        asm.second_pass()
    }

    /// First pass: parse the input to get the initial list of statements and labels
    fn first_pass<R: Read>(input: R) -> Result<Assembler> {
        let br = BufReader::new(input);
        let mut stmts = Vec::new();
        let mut labels = HashMap::new();

        for (n, l) in br.lines().enumerate() {
            // Adjust line number
            let n = n + 1;
            let l = l.chain_err(|| ErrorKind::Io("could not read line".into()))?;
            
            // Get rid of any comments
            let l = if let Some(n) = l.find("//") {
                &l[..n]
            } else {
                l.as_str()
            };

            // Try to get the label/instruction
            let mut iter = l.split_whitespace();
            let mut part = match iter.next() {
                Some(s) => s,
                None => continue,
            };

            // See if we have a label
            if let Some(idx) = part.find(':') {
                // Add the label to the label table
                let label = (&part[..idx]).trim();
                if label.is_empty() {
                    return Err(ErrorKind::Asm("found empty label".into(), n).into());
                }

                let label = label.to_owned();
                if labels.contains_key(&label) {
                    return Err(ErrorKind::Asm(format!("found duplicate label: '{}'", label), n).into());
                }
                labels.insert(label, stmts.len() as u16);

                // Get next part (the actual instruction)
                if idx == part.len() - 1 {
                    // Get the next part from the iterator
                    part = match iter.next() {
                        Some(s) => s,
                        None => continue,
                    };
                } else {
                    // Use the rest of this part
                    part = &part[idx + 1..];
                }
            }

            // Return an error if the program is too long
            if stmts.len() == u16::max_value() as usize {
                return Err(ErrorKind::ProgramTooLong.into());
            }

            // Get the instruction and any arguments (there should only be one argument)
            let instr = part;
            let arg = iter.next();
            if let Some(s) = iter.next() {
                return Err(ErrorKind::Asm(format!("unexpected argument {}", s), n).into());
            }

            // Get the statement and add it to the list
            stmts.push((n, get_stmt(instr, arg, n)?));
        }

        Ok(Assembler {
            stmts: stmts,
            labels: labels,
        })
    }

    /// Second pass: replace address labels with their corresponding locations.
    fn second_pass(self) -> Result<Program> {
        let mut code = Vec::new();

        // Replace address labels
        for &(n, ref stmt) in &self.stmts {
            match *stmt {
                Stmt::Data(ref s) => code.push(self.assemble_data(n, s)?),
                Stmt::Instr { instr, ref addr } => code.push(self.assemble_instr(n, instr, addr)?),
            }
        }

        Ok(Program {
            data: code,
            labels: self.labels,
        })
    }

    /// Assemble a data declaration.
    fn assemble_data(&self, linum: usize, s: &str) -> Result<u16> {
        u16::from_str_radix(s, 16).chain_err(|| ErrorKind::Asm("invalid data declaration (must be a hexadecimal word)".into(), linum))
    }

    /// Assemble instruction from the base instruction and an optional address.
    fn assemble_instr(&self, linum: usize, instr: Instruction, addr: &Option<String>) -> Result<u16> {
        // Match instruction and use or reject the address as necessary
        // This is pretty ugly
        let new_instr = match instr {
            Instruction::Halt | Instruction::Io(_) | Instruction::Not | Instruction::Nop => {
                refuse_arg(instr, addr, linum)?;
                instr
            }
            Instruction::Shift(_, _) => instr,
            Instruction::Load(_) => Instruction::Load(self.resolve_label(require_arg(instr, addr, linum)?, linum)?),
            Instruction::Store(_) => Instruction::Store(self.resolve_label(require_arg(instr, addr, linum)?, linum)?),
            Instruction::Add(_) => Instruction::Add(self.resolve_label(require_arg(instr, addr, linum)?, linum)?),
            Instruction::Sub(_) => Instruction::Sub(self.resolve_label(require_arg(instr, addr, linum)?, linum)?),
            Instruction::And(_) => Instruction::And(self.resolve_label(require_arg(instr, addr, linum)?, linum)?),
            Instruction::Or(_) => Instruction::Or(self.resolve_label(require_arg(instr, addr, linum)?, linum)?),
            Instruction::Xor(_) => Instruction::Xor(self.resolve_label(require_arg(instr, addr, linum)?, linum)?),
            Instruction::Jmp(_) => Instruction::Jmp(self.resolve_label(require_arg(instr, addr, linum)?, linum)?),
            Instruction::Jmpe(_) => Instruction::Jmpe(self.resolve_label(require_arg(instr, addr, linum)?, linum)?),
            Instruction::Jmpl(_) => Instruction::Jmpl(self.resolve_label(require_arg(instr, addr, linum)?, linum)?),
            Instruction::Brl(_) => Instruction::Brl(self.resolve_label(require_arg(instr, addr, linum)?, linum)?),
        };

        Ok(new_instr.to_u16())
    }

    /// Attempts to resolve the label with the given name to an address.
    fn resolve_label(&self, label: &str, linum: usize) -> Result<u16> {
        if let Some(&addr) = self.labels.get(label) {
            Ok(addr)
        } else {
            Err(ErrorKind::Asm(format!("label '{}' is undefined", label), linum).into())
        }
    }
}

/// A helper function to get a `Stmt` from an instruction and an optional argument.
fn get_stmt(instr: &str, arg: Option<&str>, linum: usize) -> Result<Stmt> {
    // See if we have a data declaration (`dw`)
    if instr == "dw" {
        return Ok(Stmt::Data(match arg {
            Some(s) => s.into(),
            None => return Err(ErrorKind::Asm("expected data declaration after 'dw'".into(), linum).into()),
        }));
    }

    // Get the instruction
    let ins = match instr {
        "halt" => Instruction::Halt,
        "readH" => Instruction::Io(IoOp::ReadHex),
        "readC" => Instruction::Io(IoOp::ReadChar),
        "printH" => Instruction::Io(IoOp::WriteHex),
        "printC" => Instruction::Io(IoOp::WriteChar),
        "shiftL" => Instruction::Shift(ShiftOp::ShiftLeft, get_shift_amt(arg, linum)?),
        "shiftR" => Instruction::Shift(ShiftOp::ShiftRight, get_shift_amt(arg, linum)?),
        "rotL" => Instruction::Shift(ShiftOp::RotateLeft, get_shift_amt(arg, linum)?),
        "rotR" => Instruction::Shift(ShiftOp::RotateRight, get_shift_amt(arg, linum)?),
        "load" => Instruction::Load(0),
        "store" => Instruction::Store(0),
        "add" => Instruction::Add(0),
        "sub" => Instruction::Sub(0),
        "and" => Instruction::And(0),
        "or" => Instruction::Or(0),
        "xor" => Instruction::Xor(0),
        "not" => Instruction::Not,
        "nop" => Instruction::Nop,
        "jmp" => Instruction::Jmp(0),
        "jmpe" => Instruction::Jmpe(0),
        "jmpl" => Instruction::Jmpl(0),
        "brl" => Instruction::Brl(0),
        s @ _ => return Err(ErrorKind::Asm(format!("unknown instruction '{}'", s), linum).into()),
    };

    Ok(Stmt::Instr {
        instr: ins,
        addr: arg.map(|s| s.to_owned()),
    })
}

/// Helper method to parse a shift amount from an optional argument.
fn get_shift_amt(arg: Option<&str>, linum: usize) -> Result<u16> {
    let amt = match arg {
        Some(s) => s,
        None => return Err(ErrorKind::Asm("must specify amount to shift".into(), linum).into()),
    };
    let amt = amt.parse::<u16>().chain_err(|| ErrorKind::Asm("invalid shift amount".into(), linum))?;
    if amt >= 16 {
        return Err(ErrorKind::Asm("invalid shift amount (must be between 0 and 15, inclusive)".into(), linum).into());
    }

    Ok(amt)
}

/// Helper method to return an error if an argument was given.
///
/// Accepts as an argument the instruction, for better error messages.
fn refuse_arg(instr: Instruction, arg: &Option<String>, linum: usize) -> Result<()> {
    if let &Some(_) = arg {
        Err(ErrorKind::Asm(format!("unexpected argument to '{}'", instr.name()), linum).into())
    } else {
        Ok(())
    }
}

/// Helper method to extract a required argument from an option.
fn require_arg(instr: Instruction, arg: &Option<String>, linum: usize) -> Result<&str> {
    match *arg {
        Some(ref s) => Ok(s),
        None => Err(ErrorKind::Asm(format!("expected argument to '{}'", instr.name()), linum).into()),
    }
}
