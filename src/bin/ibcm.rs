extern crate clap;

extern crate ibcm;

use std::fs::File;
use std::io::{self, Write};
use std::process;

use clap::{Arg, App, ArgMatches, SubCommand};

use ibcm::errors::*;
use ibcm::{Assembler, Simulator};

fn main() {
    let matches = App::new("IBCM (Itty Bitty Computing Machine)")
        .version("0.1.0")
        .author("Ian Johnson <ianprime0509@gmail.com>")
        .subcommand(SubCommand::with_name("compile")
            .arg(Arg::with_name("INPUT")
                .help("The program data file to compile")
                .required(true))
            .arg(Arg::with_name("binary")
                .short("b")
                .long("binary")
                .help("Outputs a binary file instead of a hexadecimal listing"))
            .arg(Arg::with_name("hex")
                .short("x")
                .long("hex")
                .help("Processes the input as a hexadecimal listing"))
            .arg(Arg::with_name("output")
                .short("o")
                .long("output")
                .value_name("FILE")
                .default_value("ibcm.out")
                .help("Sets the output file name")
                .takes_value(true)))
        .subcommand(SubCommand::with_name("execute")
            .arg(Arg::with_name("INPUT")
                .help("The program data file to load")
                .required(true))
            .arg(Arg::with_name("asm")
                .conflicts_with("binary")
                .short("s")
                .long("asm")
                .help("Processes the input as an ICBM assembly file"))
            .arg(Arg::with_name("binary")
                .short("b")
                .long("binary")
                .help("Processes the input as a binary file")))
        .get_matches();

    if let Err(ref e) = run(&matches) {
        let mut stderr = io::stderr();
        let errmsg = "Error writing to stderr";

        writeln!(stderr, "error: {}", e).expect(errmsg);

        for e in e.iter().skip(1) {
            writeln!(stderr, "caused by: {}", e).expect(errmsg);
        }

        if let Some(backtrace) = e.backtrace() {
            writeln!(stderr, "backtrace: {:?}", backtrace).expect(errmsg);
        }

        process::exit(1);
    }
}

/// Program logic goes in this function (for more convenient error handling).
fn run(m: &ArgMatches) -> Result<()> {
    match m.subcommand() {
        ("compile", Some(sub_m)) => compile(sub_m),
        ("execute", Some(sub_m)) => execute(sub_m),
        _ => {
            println!("{}", m.usage());
            Ok(())
        }
    }
}

/// The `compile` subcommand.
fn compile(m: &ArgMatches) -> Result<()> {
    let input = m.value_of("INPUT").unwrap();
    let f = File::open(input).chain_err(|| ErrorKind::Io("could not open input file".into()))?;
    // Read input file into a simulator (only needed for memory)
    let sim = if m.is_present("hex") {
        Simulator::from_hex(f)
    } else {
        Simulator::from_instructions(&Assembler::assemble(f)?)
    }?;

    // Safe because we provided a default value
    let output = m.value_of("output").unwrap();
    let of =
        File::create(output).chain_err(|| ErrorKind::Io("could not create output file".into()))?;
    if m.is_present("binary") {
        sim.to_binary(of)?;
    } else {
        sim.to_hex(of)?;
    }

    Ok(())
}

/// The `execute` subcommand.
fn execute(m: &ArgMatches) -> Result<()> {
    // We can unwrap here since INPUT is a required argument
    let input = m.value_of("INPUT").unwrap();
    let f = File::open(input).chain_err(|| ErrorKind::Io("could not open input file".into()))?;
    // Read the input file into a simulator
    let mut sim = if m.is_present("binary") {
        Simulator::from_binary(f)
    } else if m.is_present("asm") {
        Simulator::from_instructions(&Assembler::assemble(f)?)
    } else {
        Simulator::from_hex(f)
    }?;

    // Run the simulator program
    sim.run()
}
