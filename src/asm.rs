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
pub struct Assembler {
    /// The statements that have been processed, along with their line numbers.
    stmts: Vec<(usize, Stmt)>,
    /// A map giving the position of labels.
    labels: HashMap<String, u16>,
}

impl Assembler {
    /// Assembles the assembly code from the given reader.
    ///
    /// Returns a vector of IBCM instructions, or an error.
    pub fn assemble<R: Read>(input: R) -> Result<Vec<u16>> {
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
            if part.ends_with(':') {
                // Add the label to the label table
                let label = (&part[..part.len()-1]).trim();
                if label.is_empty() {
                    return Err(ErrorKind::Asm("found empty label".into(), n).into());
                }

                let label = label.to_owned();
                if labels.contains_key(&label) {
                    return Err(ErrorKind::Asm(format!("found duplicate label: '{}'", label), n).into());
                }
                labels.insert(label, stmts.len() as u16);

                // Get next part (the actual instruction)
                part = match iter.next() {
                    Some(s) => s,
                    None => continue,
                };
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
    fn second_pass(self) -> Result<Vec<u16>> {
        let mut code = Vec::new();

        // Replace address labels
        for &(n, ref stmt) in &self.stmts {
            match stmt {
                &Stmt::Data(ref s) => code.push(u16::from_str_radix(s.as_str(), 16).chain_err(|| ErrorKind::Asm("invalid data declaration (must be a hexadecimal word)".into(), n))?),
                &Stmt::Instr { instr, ref addr } => code.push(self.assemble_instr(n, instr, addr)?),
            }
        }

        Ok(code)
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
    fn resolve_label(&self, label: String, linum: usize) -> Result<u16> {
        if let Some(&addr) = self.labels.get(&label) {
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
fn require_arg(instr: Instruction, arg: &Option<String>, linum: usize) -> Result<String> {
    arg.clone().ok_or(ErrorKind::Asm(format!("expected argument to '{}'", instr.name()), linum).into())
}
