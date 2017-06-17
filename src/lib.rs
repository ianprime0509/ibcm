//! This library is used to simulate and debug programs written for
//! the IBCM (Itty Bitty Computing Machine), as described in
//! the [documentation](https://aaronbloomfield.github.io/pdr/book/ibcm-chapter.pdf)
//! for the machine. In short, the IBCM is a vastly simplified "computer",
//! with only 16 instructions and one usable register (the accumulator).
//!
//! Included in this library is a simulator, which is intended to be
//! compatible with the reference implementation (although such things
//! as input and output may be handled differently, e.g. with different prompts).
//! In addition, this library adds an assembler, which aims to mimic the
//! sample assembly language given in the IBCM documentation, and a debugger,
//! which fills in for some of the other features in the reference interpreter.
//!
//! For examples of use, see the main binary in the `src/bin` folder and the tests
//! in the `tests` folder.

#![warn(missing_docs)]

#[macro_use]
extern crate error_chain;
extern crate itertools;

pub mod errors {
    //! The error types for this crate, generated using `error-chain`.
    #![allow(missing_docs)]

    error_chain! {
        links {
            Ibcmc(::ibcmc::errors::Error, ::ibcmc::errors::ErrorKind);
        }

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

mod asm;
mod debug;
pub mod ibcmc;
mod instruction;
mod simulator;

pub use errors::*;

pub use asm::Assembler;
pub use debug::Debugger;
pub use instruction::Instruction;
pub use simulator::Simulator;

#[cfg(test)]
mod tests {
    use super::*;

    /// A helper function to assemble code into a simulator.
    fn sim_asm(code: &str) -> Simulator {
        Simulator::from_instructions(Assembler::assemble(code.as_bytes()).unwrap().data()).unwrap()
    }

    /// Test the `halt` operation.
    #[test]
    fn halt() {
        let program = "halt";
        let mut sim = sim_asm(program);

        // Should halt after a single step
        assert_eq!(true, sim.step().unwrap());
    }

    /// Test I/O operations.
    #[test]
    fn io() {
        let program = "readH
        printH
        readC
        printC";
        // Buffers for input and output
        let input = "12ab\nh";
        let mut output = Vec::<u8>::new();

        {
            let mut sim = sim_asm(program);
            sim.set_input(input.as_bytes());
            sim.set_output(&mut output, false);
            sim.run().unwrap();
        }

        assert_eq!(input, String::from_utf8(output).unwrap().trim());
    }

    /// Test shift operations.
    #[test]
    fn shift() {
        let program = "jmp init
        shl: dw 0010
        shr: dw 0010
        rotl: dw f000
        rotr: dw c00f

        init:
        load shl
        shiftL 8
        store shl
        
        load shr
        shiftR 4
        store shr
        
        load rotl
        rotL 4
        store rotl
        
        load rotr
        rotR 4
        store rotr
        
        halt";

        let mut sim = sim_asm(program);
        sim.run().unwrap();

        assert_eq!(&[0x1000, 0x0001, 0x000f, 0xfc00], &sim.memory()[1..5]);
    }

    /// Test `load` and `store`.
    #[test]
    fn load_store() {
        let program = "jmp init
        src: dw 1234
        dest: dw 0000

        init:
        load src
        store dest
        halt";

        let mut sim = sim_asm(program);
        sim.run().unwrap();

        assert_eq!(&[0x1234, 0x1234], &sim.memory()[1..3]);
    }

    /// Test `add` and `sub`.
    #[test]
    fn add_sub() {
        let program = "jmp init
        add7: dw 0
        sub15: dw 0
        7: dw 0007
        15: dw 000f

        init:
        load add7
        add 7
        store add7

        load sub15
        sub 15
        store sub15

        halt";

        let mut sim = sim_asm(program);
        sim.run().unwrap();

        assert_eq!(&[7, 0u16.wrapping_sub(15)], &sim.memory()[1..3]);
    }

    /// Test bitwise operations (`and`, `or`, `xor`, `not`).
    #[test]
    fn bitwise() {
        let program = "jmp init
        a: dw abcd
        b: dw 1234
        and: dw 0
        or: dw 0
        xor: dw 0
        not: dw 0

        init:
        load a
        and b
        store and

        load a
        or b
        store or

        load a
        xor b
        store xor

        load a
        not
        store not

        halt";

        let mut sim = sim_asm(program);
        sim.run().unwrap();

        assert_eq!(&[0xabcd & 0x1234, 0xabcd | 0x1234, 0xabcd ^ 0x1234, !0xabcd], &sim.memory()[3..7]);
    }

    /// Test `nop`.
    #[test]
    fn nop() {
        let program = "jmp init
        a: dw 4
        init:
        nop
        nop
        nop
        nop
        nop
        halt";

        let mut sim = sim_asm(program);
        sim.run().unwrap();

        assert_eq!(4, sim.memory()[1]);
    }

    /// Test the various jumps (`jmp`, `jmpe`, `jmpl`)
    #[test]
    fn jmp() {
        let program = "jmp init
        pass: dw 0
        0: dw 0
        1: dw 1
        2: dw 2

        fail:
        load 0
        store pass
        halt

        test3:
        halt

        test2:
        load 0
        add 1
        jmpl fail
        sub 2
        jmpl test3
        jmp fail

        test1:
        load 0
        add 1
        jmpe fail
        sub 1
        jmpe test2
        jmp fail

        init:
        jmp test1
        halt";

        let mut sim = sim_asm(program);
        sim.run().unwrap();

        assert_eq!(0, sim.memory()[1]);
    }

    /// Test branching (`brl`)
    #[test]
    fn brl() {
        let program = "jmp init
        success: store addr
        halt
        addr: dw 0
        1: dw 1
        2: dw 2

        init:
        brl success
        halt";

        let mut sim = sim_asm(program);
        sim.run().unwrap();
        let (acc, _, _) = sim.regs();
        
        // Make sure we set the register and got to the part
        // where we store the value
        assert_eq!(7, acc, "wrong accumulator value");
        assert_eq!(7, sim.memory()[3], "did not jump");
    }
}
