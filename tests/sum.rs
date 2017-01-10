//! Tests the IBCM simulator on the simple sum example
//! given in the documentation.

extern crate ibcm;

use ibcm::{Assembler, Simulator};

use std::collections::HashMap;

const SUM_IBCM: &'static [u8] = include_bytes!("programs/sum.ibcm");
const SUM_IBCMASM: &'static [u8] = include_bytes!("programs/sum.ibcmasm");

#[test]
fn sum() {
    // Test the program on several values
    let values = &[5, 10, 15, 20];
    let mut tests: HashMap<u32, u32> = HashMap::new();
    for &v in values {
        tests.insert(v, (1..v + 1).sum());
    }

    for (test, sol) in tests {
        let input = format!("{:04x}", test);
        let expected = format!("{:04x}", sol);
        let mut output = Vec::<u8>::new();

        {
            let mut sim = Simulator::from_hex(SUM_IBCM).unwrap();
            sim.set_input(input.as_bytes());
            sim.set_output(&mut output, false);
            sim.run().expect("failed to run program");
        }

        assert_eq!(expected, String::from_utf8(output).unwrap().trim());
    }
}

#[test]
fn asm_sum() {
    // Test the program on several values
    let values = &[4, 8, 12, 16];
    let mut tests: HashMap<u32, u32> = HashMap::new();
    for &v in values {
        tests.insert(v, (1..v + 1).sum());
    }

    for (test, sol) in tests {
        let input = format!("{:04x}", test);
        let expected = format!("{:04x}", sol);
        let mut output = Vec::<u8>::new();

        {
            let mut sim = Simulator::from_instructions(Assembler::assemble(SUM_IBCMASM).unwrap().data())
                .unwrap();
            sim.set_input(input.as_bytes());
            sim.set_output(&mut output, false);
            sim.run().expect("failed to run program");
        }

        assert_eq!(expected, String::from_utf8(output).unwrap().trim());
    }
}
