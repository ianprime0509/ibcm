extern crate ibcm;

use ibcm::Simulator;

fn main() {
    let input = "c006
    0000
    0000
    0000
    0001
    0000
    1000
    4003
    3004
    4001
    3005
    4002
    3003
    6001
    e016
    3002
    5001
    4002
    3001
    5004
    4001
    c00c
    3002
    1800
    0000";
    let mut sim = Simulator::from_hex(input.as_bytes()).unwrap();

    if let Err(e) = sim.run() {
        println!("error: {}", e);
    }
}
