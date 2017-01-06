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
            /// An IO error.
            Io(s: String) {
                description("io error")
                display("io error: {}", s)
            }
        }
    }
}

pub use errors::*;

mod instruction;
mod simulator;

pub use simulator::Simulator;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}
