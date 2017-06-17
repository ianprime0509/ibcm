extern crate clap;
#[macro_use]
extern crate error_chain;

extern crate ibcm;

const VERSION: &str = env!("CARGO_PKG_VERSION");

use std::fs::File;
use std::io::{self, Read, Write};

use clap::{Arg, App, ArgMatches, SubCommand};

use ibcm::errors::*;
use ibcm::{Assembler, Debugger, Simulator};
use ibcm::ibcmc::lexer::Lexer;

quick_main!(run);

/// Program logic goes in this function (for more convenient error handling).
fn run() -> Result<()> {
    let matches = App::new("IBCM (Itty Bitty Computing Machine)")
        .version(VERSION)
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
        .subcommand(SubCommand::with_name("debug")
                        .arg(Arg::with_name("INPUT")
                                 .help("The program to debug")
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
        .subcommand(SubCommand::with_name("ibcmc")
                        .arg(Arg::with_name("INPUT")
                                 .help("The IBCMC source file to compile")
                                 .required(true))
                        .arg(Arg::with_name("output")
                                 .short("o")
                                 .long("output")
                                 .value_name("OUTPUT")
                                 .help("Sets the output file name")
                                 .takes_value(true)))
        .get_matches();

    match matches.subcommand() {
        ("compile", Some(sub_m)) => compile(sub_m),
        ("debug", Some(sub_m)) => debug(sub_m),
        ("execute", Some(sub_m)) => execute(sub_m),
        ("ibcmc", Some(sub_m)) => ibcmc(sub_m),
        _ => {
            println!("{}", matches.usage());
            Ok(())
        }
    }
}

/// The `compile` subcommand.
fn compile(m: &ArgMatches) -> Result<()> {
    let input = m.value_of("INPUT").unwrap();
    let f = File::open(input)
        .chain_err(|| ErrorKind::Io(format!("could not open input file `{}`", input)))?;
    // Read input file into a simulator (only needed for memory)
    let sim = if m.is_present("hex") {
        Simulator::from_hex(f)
    } else {
        Simulator::from_instructions(Assembler::assemble(f)?.data())
    }?;

    // Safe because we provided a default value
    let output = m.value_of("output").unwrap();
    let of =
        File::create(output)
            .chain_err(|| ErrorKind::Io(format!("could not create output file `{}`", output)))?;
    if m.is_present("binary") {
        sim.to_binary(of)?;
    } else {
        sim.to_hex(of)?;
    }

    Ok(())
}

/// The `debug` subcommand.
fn debug(m: &ArgMatches) -> Result<()> {
    // We can unwrap here since INPUT is a required argument
    let input = m.value_of("INPUT").unwrap();
    let f = File::open(input)
        .chain_err(|| ErrorKind::Io(format!("could not open input file `{}`", input)))?;
    // Read the input file into a simulator
    let sim = if m.is_present("binary") {
        Simulator::from_binary(f)
    } else if m.is_present("asm") {
        Simulator::from_instructions(Assembler::assemble(f)?.data())
    } else {
        Simulator::from_hex(f)
    }?;
    let mut debug = Debugger::new(sim);

    // Debug console
    loop {
        print!(">> ");
        io::stdout().flush().expect("could not flush stdout");

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .chain_err(|| ErrorKind::Io("could not read from stdin".into()))?;
        let input_parts = input.trim().split_whitespace().collect::<Vec<_>>();
        if input_parts.is_empty() {
            continue;
        }
        let command = input_parts[0];
        let args = &input_parts[1..];

        match debug.execute_command(command, args) {
            Ok(true) => break,
            Ok(false) => continue,
            Err(e @ Error(ErrorKind::Debug(_), _)) => {
                println!("error: {}", e);

                for e in e.iter().skip(1) {
                    println!("caused by: {}", e);
                }
                continue;
            }
            Err(e) => return Err(e),
        }
    }

    Ok(())
}

/// The `execute` subcommand.
fn execute(m: &ArgMatches) -> Result<()> {
    // We can unwrap here since INPUT is a required argument
    let input = m.value_of("INPUT").unwrap();
    let f = File::open(input)
        .chain_err(|| ErrorKind::Io(format!("could not open input file `{}`", input)))?;
    // Read the input file into a simulator
    let mut sim = if m.is_present("binary") {
        Simulator::from_binary(f)
    } else if m.is_present("asm") {
        Simulator::from_instructions(Assembler::assemble(f)?.data())
    } else {
        Simulator::from_hex(f)
    }?;

    // Run the simulator program
    sim.run()
}

/// The `ibcmc` subcommand.
fn ibcmc(m: &ArgMatches) -> Result<()> {
    let input = m.value_of("INPUT").unwrap();
    let f = File::open(input)
        .chain_err(|| ErrorKind::Io(format!("could not open input file `{}`", input)))?;

    for tok in Lexer::new(f.bytes()) {
        println!("{:?}", tok?);
    }

    Ok(())
}

