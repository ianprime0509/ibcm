#[macro_use]
extern crate error_chain;

pub mod errors {
    error_chain! {
        errors {
            /// Tried to execute an instruction on a halted machine.
            Halted {
                description("tried to execute an instruction on a halted machine")
            }
            /// There was an error in user input format.
            UserInput(s: String) {
                description("user input error")
                display("user input error: {}", s)
            }
            /// The program counter pointed to an invalid memory location.
            OutOfBounds {
                description("program ran out of bounds")
            }
            /// The given input program is too long.
            ProgramTooLong {
                description("input program is too long")
            }

            /// There was an error when parsing assembly code.
            ///
            /// Contains error description and line number of error.
            Asm(s: String, n: usize) {
                description("error parsing assembly")
                display("error parsing assembly on line {}: {}", n, s)
            }

            /// There was an error in the debugger.
            ///
            /// Basically just a simple message designated as a debugger
            /// error for better handling.
            Debug(s: String) {
                description("debugger error")
                display("{}", s)
            }

            /// An IO error.
            Io(s: String) {
                description("io error")
                display("io error: {}", s)
            }
        }
    }
}

pub use errors::*;

mod asm;
mod debug;
mod instruction;
mod simulator;

pub use asm::Assembler;
pub use debug::Debugger;
pub use instruction::Instruction;
pub use simulator::Simulator;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}
