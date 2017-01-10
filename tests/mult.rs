//! Tests the IBCM simulator on the multiplication example
//! given in the documentation.

extern crate ibcm;

use ibcm::{Assembler, Simulator};

use std::collections::HashMap;

const MULT_IBCMASM: &'static [u8] = include_bytes!("programs/mult.ibcmasm");

#[test]
fn asm_mult() {
    // Test the program on several values
    let values = &[(3, 4), (6, 9), (10, 15), (30, 45)];
    let mut tests: HashMap<(u32, u32), u32> = HashMap::new();
    for &v in values {
        tests.insert(v, v.0 * v.1);
    }

    for ((m1, m2), sol) in tests {
        let input = format!("{:04x}\n{:04x}", m1, m2);
        let expected = format!("{:04x}", sol);
        let mut output = Vec::<u8>::new();

        {
            let mut sim =
                Simulator::from_instructions(Assembler::assemble(MULT_IBCMASM).unwrap().data())
                    .unwrap();
            sim.set_input(input.as_bytes());
            sim.set_output(&mut output, false);
            sim.run().expect("failed to run program");
        }

        assert_eq!(expected, String::from_utf8(output).unwrap().trim());
    }
}
