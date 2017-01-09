//! The debugger.
use errors::*;
use simulator::Simulator;

/// The help string for the debugger
static HELP: &'static str = "The following commands are recognized:
quit            Exit the debugger.
help            Print this message.
dump <amt>      Display the contents of the first <amt>
                memory locations.
run             Run the program until it halts.
status          Output the content of all registers and print
                the current instruction.
step <n>        Execute the next <n> instructions.";

/// A debugger, which is a wrapper around a `Simulator` that
/// processes debug instructions.
pub struct Debugger {
    /// The underlying `Simulator`.
    sim: Simulator,
}

impl Debugger {
    /// Construct a new `Debugger` from the given `Simulator`.
    pub fn new(sim: Simulator) -> Debugger {
        Debugger {
            sim: sim,
        }
    }

    /// Executes the specified command with the given arguments.
    ///
    /// Returns `true` if the debugger should quit.
    pub fn execute_command(&mut self, command: &str, args: &[&str]) -> Result<bool> {
        match command {
            "quit" => Ok(true),
            "help" => {
                println!("{}", HELP);
                Ok(false)
            }
            "dump" => self.dump(args),
            "run" => self.run(args),
            "status" => self.status(args),
            "step" => self.step(args),
            s => Err(ErrorKind::Debug(format!("unknown command '{}'", s)).into()),
        }
    }

    /// The `dump` command.
    fn dump(&mut self, args: &[&str]) -> Result<bool> {
        if args.len() != 1 {
            return Err(ErrorKind::Debug("must specify amount of memory to dump".into()).into());
        }
        let amt = args[0].parse().chain_err(|| ErrorKind::Debug("invalid amount to dump".into()))?;
        self.sim.dump(amt);

        Ok(false)
    }

    /// The `run` command.
    fn run(&mut self, args: &[&str]) -> Result<bool> {
        if !args.is_empty() {
            return Err(ErrorKind::Debug("did not expect any arguments".into()).into());
        }

        if self.sim.is_halted() {
            return Err(ErrorKind::Debug("machine is halted".into()).into());
        }

        // We want to print out if the machine halted,
        // so we shouldn't use the sim.run() method.
        let mut steps = 0;
        while !self.sim.step()? {
            steps += 1;
        }
        println!("machine halted after {} step(s)", steps);
        Ok(false)
    }

    /// The `status` command.
    fn status(&mut self, args: &[&str]) -> Result<bool> {
        if !args.is_empty() {
            return Err(ErrorKind::Debug("did not expect any arguments".into()).into());
        }

        // Print out registers and whether the machine is halted
        let (acc, ir, pc) = self.sim.regs();
        println!("acc:    {}", acc);
        println!("ir:     {}", ir);
        println!("pc:     {}", pc);
        println!("halted? {}", self.sim.is_halted());
        // Print out the current instruction with a backtrace
        let mut ins = self.sim.current_instruction()?;
        println!("current instruction: {}", ins);
        while ins.is_jmp() {
            let addr = ins.address().unwrap();
            ins = self.sim.instruction_at(addr);
            println!("--> (@ {:04x}) {}", addr, ins);
        }

        Ok(false)
    }

    /// The `step` command.
    fn step(&mut self, args: &[&str]) -> Result<bool> {
        if args.len() > 1 {
            return Err(ErrorKind::Debug("expected no more than 1 argument".into()).into());
        }
        // Number of steps to execute
        let n = if args.len() == 1 {
            args[0].parse().chain_err(|| ErrorKind::Debug("invalid number of steps".into()))?
        } else {
            1
        };

        // Quit if the machine is halted
        if self.sim.is_halted() {
            return Err(ErrorKind::Debug("machine is halted".into()).into());
        }

        // Execute the steps
        for i in 0..n {
            if self.sim.step()? {
                println!("halted after {} step(s)", i + 1);
                return Ok(false);
            }
        }
        println!("executed {} step(s)", n);
        Ok(false)
    }
}
