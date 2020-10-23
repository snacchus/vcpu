//! Assembler for the [vcpu](../vcpu/index.html) virtual processor.
//!
//! The main functions provided by this crate are [`assemble`](fn.assemble.html) and [`assemble_addressed`](fn.assemble_addressed.html),
//! which accept some string input containing a program written in a custom assembly language, and output the assembled executable.
//! The executable is represented by the [`Executable`](../vex/struct.Executable.html) type from the [vex](../vex/index.html) crate.
//! Additionally, a [`SourceMap`](type.SourceMap.html) is returned, which associates each assembled instruction in the
//! executable with the corresponding line(s) in the source.
//!
//! Parsing the assembly language is implemented using [pest]. In fact, the main [`Error`](type.Error.html) type used by this
//! crate is just a type alias of `pest::error::Error`. This means that all functionality provided by [pest]
//! is also available, such as pretty formatting of errors.
//!
//! # VASM Assembler Language
//!
//! A VASM program consists of the two sections `.data` and `.instructions`, which are both always mandatory
//! and must appear in that order.
//!
//! The source can contain comments, which start with a hash-symbol `#` and continue to the end of the line.
//!
//! ## `.data` Section
//!
//! This section contains static, hardcoded data that comes with the executable and will be loaded into main
//! memory at runtime. Parts of this data can optionally be just reserved space, which will be initialized to zeroes.
//!
//! Individual data elements are declared in this section one after another.
//! These data elements will appear in the resulting executable in the exact same order.
//! The following table lists all available data element types and their syntax:
//!
//!  Keyword | Description | Syntax | Example
//! ---------|-------------|--------|--------
//! `.block` |simple block of memory which is initialized to zeroes.| `.block <size>` | `.block 1024`
//! `.byte`  |a list of integers, each a single byte big|`.byte <int> [, <int>]*`| `.byte 1, -45, 0xFF`
//! `.half`  |a list of integers, each two bytes big|`.half <int> [, <int>]*`| `.half 2037, -10228, 0x1234`
//! `.word`  |a list of integers, each four bytes big|`.word <int> [, <int>]*`| `.word 98273, -45455, 0xABCD1234`
//!
//! Note that integer literals can be signed decimal, hexadecimal (`0x`), octal (`0o`) or binary (`0b`).
//! Lists of integers are separated with commas.
//!
//! ## `.instructions` Section
//!
//! This section contains the instructions that make up the program.
//! Instructions are constructed using mnemonics.
//! Each mnemonic procudes one or more instructions.
//!
//! ### Register Identifiers
//!
//! Many mnemonics require registers to be specified so their id can be encoded in the resulting instruction(s).
//! To specify a register, prefix the name of the register with `$`. The following registers are available:
//!
//! Register    | Purpose
//! ------------|---------
//! `$ZERO`     | Always contains zero. Attempting to write to this register has no effect.
//! `$V0`-`$V1` | "Value". General purpose, but used for returning values from functions.
//! `$A0`-`$A4` | "Argument". General purpose, but used for passing arguments to functions.
//! `$T0`-`$T9` | "Temporary". General purpose, but used for holding temporary values (must be saved by caller).
//! `$S0`-`$S9` | "Saved". General purpose, but callees should save the contents to the stack.
//! `$SP`       | Stack pointer. Automatically updated by `PUSH` and `POP` mnemonics.
//! `$FP`       | Frame pointer. Contains address of current function's stack frame (must be maintained manually).
//! `$RM`       | Remainder. Contains the high bits of multiplication product, or the remainder of division.
//! `$RA`       | Return address. Contains address of the instruction to "return" to after jump and link instruction.
//!
//! ### Single Instruction Mnemonics
//!
//! Mnemonics that produce a single instruction correspond directly to one of the [`Opcode`](../vcpu/enum.Opcode.html)s
//! defined by the processor (or in the case of R-type instructions, to a function) and are
//! named after them.
//! For example, the `LB` mnemonic produces a single instruction that contains [`Opcode::LB`](../vcpu/enum.Opcode.html#variant.LB),
//! and the `ADD` mnemonic produces an instruction with [`Opcode::ALU`](../vcpu/enum.Opcode.html#variant.ALU)
//! and [`AluFunct::ADD`](../vcpu/enum.AluFunct.html#variant.ADD).
//!
//! Quick reference for all available single instruction mnemonics:
//!
//! Mnemonic | Short Description                            | Syntax
//! ---------|----------------------------------------------|----------------------
//! `NOP`    | No-op                                        | `NOP`
//! `HALT`   | Exit program                                 | `HALT`
//! `COPY`   | Copy register                                | `COPY rd, rs`
//! `ADD`    | Integer addition                             | `ADD rd, rs1, rs2`
//! `SUB`    | Integer subtraction                          | `SUB rd, rs1, rs2`
//! `MUL`    | Integer multiplication                       | `MUL rd, rs1, rs2`
//! `DIV`    | Integer division                             | `DIV rd, rs1, rs2`
//! `AND`    | Bitwise And                                  | `AND rd, rs1, rs2`
//! `OR`     | Bitwise Or                                   | `OR rd, rs1, rs2`
//! `XOR`    | Bitwise Xor                                  | `XOR rd, rs1, rs2`
//! `SLL`    | Shift left logical                           | `SLL rd, rs1, rs2`
//! `SRL`    | Shift right logical                          | `SRL rd, rs1, rs2`
//! `SRA`    | Shift right arithmetic                       | `SRA rd, rs1, rs2`
//! `SEQ`    | Set if equal                                 | `SEQ rd, rs1, rs2`
//! `SNE`    | Set if not equal                             | `SNE rd, rs1, rs2`
//! `SLT`    | Set if less than                             | `SLT rd, rs1, rs2`
//! `SGT`    | Set if greater than                          | `SGT rd, rs1, rs2`
//! `SLE`    | Set if less or equal                         | `SLE rd, rs1, rs2`
//! `SGE`    | Set if greater or equal                      | `SGE rd, rs1, rs2`
//! `SLTU`   | Set if less than unsigned                    | `SLTU rd, rs1, rs2`
//! `SGTU`   | Set if greater than unsigned                 | `SGTU rd, rs1, rs2`
//! `SLEU`   | Set if less or equal unsigned                | `SLEU rd, rs1, rs2`
//! `SGEU`   | Set if greater or equal unsigned             | `SGEU rd, rs1, rs2`
//! `LI`     | Load immediate value                         | `LI rd, value`
//! `LHI`    | Load immediate value high                    | `LHI rd, value`
//! `SLO`    | Set low bits                                 | `SLO rd, value`
//! `SHI`    | Set high bits                                | `SHI rd, value`
//! `LB`     | Load byte                                    | `LB rd, offset(rs)`
//! `LH`     | Load half                                    | `LH rd, offset(rs)`
//! `LW`     | Load word                                    | `LW rd, offset(rs)`
//! `SB`     | Store byte                                   | `SB rd, offset(rs)`
//! `SH`     | Store half                                   | `SH rd, offset(rs)`
//! `SW`     | Store word                                   | `SW rd, offset(rs)`
//! `ADDI`   | Integer addition immediate                   | `ADDI, rd, rs, value`
//! `SUBI`   | Integer subtraction immediate                | `SUBI, rd, rs, value`
//! `MULI`   | Integer multiplication immediate             | `MULI, rd, rs, value`
//! `DIVI`   | Integer division immediate                   | `DIVI, rd, rs, value`
//! `ANDI`   | Bitwise And immediate                        | `ANDI, rd, rs, value`
//! `ORI`    | Bitwise Or immediate                         | `ORI, rd, rs, value`
//! `XORI`   | Bitwise Xor immediate                        | `XORI, rd, rs, value`
//! `FLIP`   | Flip bits                                    | `FLIP, rd, rs`
//! `SLLI`   | Shift left logical immediate                 | `SLLI, rd, rs, value`
//! `SRLI`   | Shift right logical immediate                | `SRLI, rd, rs, value`
//! `SRAI`   | Shift right arithmetic immediate             | `SRAI, rd, rs, value`
//! `SEQI`   | Set if equal immediate                       | `SEQI, rd, rs, value`
//! `SNEI`   | Set if not equal immediate                   | `SNEI, rd, rs, value`
//! `SLTI`   | Set if less than immediate                   | `SLTI, rd, rs, value`
//! `SGTI`   | Set if greater than immediate                | `SGTI, rd, rs, value`
//! `SLEI`   | Set if less or equal immediate               | `SLEI, rd, rs, value`
//! `SGEI`   | Set if greater or equal immediate            | `SGEI, rd, rs, value`
//! `SLTUI`  | Set if less than unsigned immediate          | `SLTUI, rd, rs, value`
//! `SGTUI`  | Set if greater than unsigned immediate       | `SGTUI, rd, rs, value`
//! `SLEUI`  | Set if less or equal unsigned immediate      | `SLEUI, rd, rs, value`
//! `SGEUI`  | Set if greater or equal unsigned immediate   | `SGEUI, rd, rs, value`
//! `BEZ`    | Branch if zero                               | `BEZ rs, target`
//! `BNZ`    | Branch if not zero                           | `BNZ rs, target`
//! `JMP`    | Jump                                         | `JMP target`
//! `JL`     | Jump and link                                | `JL target`
//! `JR`     | Jump to register value                       | `JR rs`
//! `JLR`    | Jump to register and link                    | `JLR rs`
//! `ITOF`   | Integer to float                             | `ITOF rd, rs`
//! `FTOI`   | Float to integer                             | `FTOI rd, rs`
//! `FADD`   | Float addition                               | `FADD rd, rs1, rs2`
//! `FSUB`   | Float subtraction                            | `FSUB rd, rs1, rs2`
//! `FMUL`   | Float multiplication                         | `FMUL rd, rs1, rs2`
//! `FDIV`   | Float division                               | `FDIV rd, rs1, rs2`
//!
//! ### Shorthand Mnemonics
//!
//! Mnemonics that produce more than one instruction are purely an assembler feature and don't
//! directly correspond to processor-opcodes. They can be thought of as shorthands that make
//! common tasks more convenient, for example pushing to and popping from the stack.
//!
//! Quick reference for all available shorthand mnemonics:
//!
//! Mnemonic | Short Description                            | Syntax
//! ---------|----------------------------------------------|----------------------
//! `PUSH`   | Push register value onto stack               | `PUSH rs`
//! `POP`    | Pop register value from stack                | `POP rd`
//! `LWI`    | Load word immediate                          | `LWI rd, value`
//! `LDA`    | Load data address                            | `LDA rd, label`
//! `LIA`    | Load instruction address                     | `LIA rd, label`
//!
//! [pest]: https://docs.rs/pest/

// TODO: describe things like immediate values, jump offsets, address offsets, jump targets, labels
// TODO: describe data labels and instruction labels
// TODO: provide detailed documentation for each mnemonic (separate pages?)

mod data;
mod instructions;
mod int_util;
mod labels;
mod parser;
mod source_map;

#[cfg(test)]
mod test;

use parser::{Rule, VASMParser};
use pest::iterators::Pair;
use pest::{Parser, Span};
pub use source_map::{SourceMap, SourceMapItem};
use vex::Executable;

pub type Error = pest::error::Error<Rule>;

pub type Result<T> = std::result::Result<T, Error>;

pub fn assemble_addressed(input: &str, data_offset: u32) -> Result<(Executable, SourceMap)> {
    assemble_parsed(parse(input)?, data_offset)
}

pub fn assemble(input: &str) -> Result<(Executable, SourceMap)> {
    assemble_addressed(input, 0u32)
}

fn new_parser_error(span: Span, message: String) -> Error {
    Error::new_from_span(pest::error::ErrorVariant::CustomError { message }, span)
}

fn parse(input: &str) -> Result<Pair<Rule>> {
    Ok(VASMParser::parse(Rule::program, input)?.next().unwrap())
}

fn assemble_parsed(pair: Pair<Rule>, data_offset: u32) -> Result<(Executable, SourceMap)> {
    let mut pairs = pair.into_inner();

    let (data, data_labels) = data::process_data(pairs.next().unwrap())?;
    let (instr, instr_labels, source_map) =
        instructions::process_instructions(pairs.next().unwrap(), &data_labels, data_offset)?;

    Ok((
        Executable::from(
            data_offset,
            instructions::assemble_instructions(&instr, &instr_labels)?,
            data,
        ),
        source_map,
    ))
}
