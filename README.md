# IBCM

The IBCM (Itty Bitty Computing Machine) is a vastly simplified design for a
computer (more precisely, a processor architecture) for the purpose of teaching
basic assembly programming as part of the University of Virginia's "Program and
Data Representation" course. The original specification of the IBCM
architecture can be found
[here](https://aaronbloomfield.github.io/pdr/book/ibcm-chapter.pdf), and the
GitHub repository containing the "reference interpreter" along with several
examples of IBCM code can be found
[here](https://github.com/aaronbloomfield/pdr/tree/master/ibcm).

It should be noted that I did not create or contribute to the original design
in any way; in fact, I didn't even take the course that it is associated with.
One of my friends took the course and I thought it would be an interesting
project to fill in one of the "gaps" of the original project: the lack of an
assembler.  Actually, I bet that the lack of an assembler is a "feature", since
it teaches students how to think about raw machine code :)

Nevertheless, this project represents my attempt both to rewrite the original
simulation and provide it with a functional assembler. Currently, the following
features are provided:
* A simulation, fully compatible with the original
* An assembler
* A bare-bones debugger, offering basic "stepwise" execution

## Simulation

Currently, the simulation is capable of simulating programs in three formats:
IBCM assembly (described below), a list of IBCM instructions in hexadecimal
(one instruction per line), and an IBCM binary file. The documentation
states that the IBCM is big-endian; nevertheless, the "reference simulator"
processes these binary files as little-endian, so this implementation
does the same to maintain compatibility. The hexadecimal instruction format,
however, is big-endian.

As an example, here is a simple program in IBCM assembly, which reads a hexadecimal
word into the accumulator, adds 1, and outputs it again:

```text
        // Go to beginning of program
        jmp     init
        // Variables
1:      dw      1       // Labels can be anything

init:
        readH           // Read hex word
        add     1       // Add 1
        printH          // Print hex word
        halt            // End of program
```

Here is the same program, in hexadecimal format:

```text
c002
0001
1000
5001
1008
0000
```

In binary format, the bytes would be (with spaces added for legibility:
`02c0 0100 0001 0150 0810 0000`.

The command `ibcm compile` can convert assembly or hexadecimal into a hexadecimal
or binary format, similarly to the `-comp` option in the original simulator.
The `ibcm simulate` command is similar to the `-sim` option, but can run programs
directly from hexadecimal or assembly format without needing to compile them first.
Use the `ibcm help` command, along with `ibcm help compile` and `ibcm help simulate`
commands, for more information on the available arguments.

## Assembler

### Specification

The IBCM assembly language is not actually defined in the language
specification itself, which defines the machine only in terms of the raw `u16`
instructions on which it operates. In the example programs, however, several
"opcodes" are used to clarify the hexadecimal instructions (the example
programs may be found in the [official
repository](https://github.com/aaronbloomfield/pdr/tree/master/ibcm) of the
IBCM language). These opcodes were used as the basis for the assembly language,
which adds the very convenient feature of labels (avoiding the need for manual
calculation of memory locations, which is error-prone).

An IBCM assembly program consists of a sequence of *statements*, each of which
must occupy its own line. A statement consists of an opcode and, if applicable,
an argument, separated by whitespace. For example:

```text
halt
dw      000A
jmp     label
```

IBCM assembly may also contain *labels*, which consist of a sequence of
non-whitespace characters followed by a single colon (`:`). Each label refers
to the statement directly following it; you may place up to one label on the
same line as a statement, and an arbitrary number on the lines preceding it
(which will all refer to the same statement). For example, all the labels in
the following refer to the `halt` statement:

```text
label1:
@!#:
01234:
标签: halt
```

As can be seen in the example above, the only requirement for a label is that
it not contain any whitespace or colons (I might change this later, but this is
unlikely), and that it must be valid UTF8. This also means that all arguments
to opcodes expecting an address must be labels, and cannot be (say) references
to a specific memory location.  However, since this isn't actual assembly where
such things are useful, this shouldn't be a problem.

Indentation and whitespace within a line is ignored, allowing for clearer
formatting.  Additionally, comments may appear in the code: the characters `//`
will cause the rest of the line to be treated as a comment, as in C++.

### Usage

The assembler is invoked using either the `ibcm simulate` (for running)
or `ibcm compile` (for outputting hexadecimal or binary code) command with the `-s` option,
which will treat the input file as an IBCM assembly file. The same option can
be used with the debugger (`ibcm debug`) as well, but note that this currently does
not provide any additional functionality (such as viewing the contents of labelled
memory locations).

## Debugger

The `ibcm debug` command can be used to provide a debugging interface for IBCM code.
This allows for execution of a program step by step, and provides the basic abilities
to dump the contents of memory and inspect register values. Inside the debug interface,
the `help` command can be used for a summary of commands, which are also summarized below.

* `quit`: Exits the debugger.
* `help`: Shows a basic help message with commands.
* `dump <amt>`: Displays the contents of the first `<amt>` memory locations.
* `run`: Runs the program until it halts (eventually, breakpoints may be added to this feature).
* `status`: Outputs the content of all registers, including a "backtrace" of the current
instruction (i.e. if the current instruction is a jump, the referenced instruction will be
printed, and so on).
* `step <n>`: Executes `<n>` instructions (or until the machine halts).

## Planned features

Even though this project is pretty useless, it's also fun and significantly easier
than trying to make an assembler for a real architecture :) Here are some features I hope
to implement, eventually:

* Disassembler (including debugger integration)
* Breakpoints in debugger

## Additional notes
Here are a few notes for things that are undocumented and/or bugs in the original
implementation:

1. If you forget the `halt` instruction at the end of the program, the reference
simulator will repeat the last instruction twice before halting (probably a bug).
This implementation does not do this.
2. As noted above, the format of binary files used by the reference simulator
is *little-endian*, not big-endian as might be expected (if we interpret the
binary files as a "dump" of the ICBM's internal memory). In the reference implementation,
it looks like there's supposed to be support for big-endian target machines,
but this hasn't been added yet and I'm not sure if it would change the format.
