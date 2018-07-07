extern crate byteorder;
#[macro_use]
extern crate matches;
extern crate num;
#[cfg_attr(test, macro_use)]
extern crate pest;
#[macro_use]
extern crate pest_derive;
extern crate vcpu;

pub mod error;
pub use error::*;

use std::str::FromStr;
use byteorder::ByteOrder;
use num::{Num, Unsigned, Signed, NumCast};
use std::num::ParseIntError;
use std::collections::HashMap;
use std::mem;
use pest::Parser;
use pest::iterators::Pair;
use vcpu::*;

#[derive(Debug, PartialEq)]
pub struct Program {
    pub data: Vec<u8>,
    pub instructions: Vec<u8>
}

type Endian = byteorder::LittleEndian;

#[derive(Debug, PartialEq)]
enum JumpTarget<'i, T: Num+Copy> {
    Address(T),
    Label(&'i str)
}

#[derive(Debug, PartialEq)]
enum ParsedInstruction<'i> {
    Complete(Word),

    Branch {
        opcode: OpCode,
        rs1: RegisterId,
        target: JumpTarget<'i, Immediate>
    },

    Jump {
        opcode: OpCode,
        target: JumpTarget<'i, Address>
    }
}

type ParseResult<'i, T> = std::result::Result<T, ParseError<'i>>;
type AssembleResult<T> = std::result::Result<T, AssembleError>;

type Result<'i, T> = std::result::Result<T, error::Error<'i>>;

#[cfg(debug_assertions)]
const _GRAMMAR: &'static str = include_str!("vasm.pest");

#[derive(Parser)]
#[grammar = "vasm.pest"]
struct VASMParser;

type LabelMap<'i> = HashMap<&'i str, u32>;
type DataVec = Vec<u8>;
type InstrVec<'i> = Vec<ParsedInstruction<'i>>;
type DataInfo<'i> = (DataVec, LabelMap<'i>);
type InstrInfo<'i> = (InstrVec<'i>, LabelMap<'i>);

trait UnsignedFromStr: Signed + NumCast {
    fn unsigned_from_str_radix(str: &str, radix: u32) -> ParseResult<Self>;
}

impl UnsignedFromStr for i8 {
    fn unsigned_from_str_radix(str: &str, radix: u32) -> ParseResult<i8> {
        NumCast::from(u8::from_str_radix(str, radix)?).ok_or(ParseError::CastInt)
    }
}

impl UnsignedFromStr for i16 {
    fn unsigned_from_str_radix(str: &str, radix: u32) -> ParseResult<i16> {
        NumCast::from(u16::from_str_radix(str, radix)?).ok_or(ParseError::CastInt)
    }
}

impl UnsignedFromStr for i32 {
    fn unsigned_from_str_radix(str: &str, radix: u32) -> ParseResult<i32> {
        NumCast::from(u32::from_str_radix(str, radix)?).ok_or(ParseError::CastInt)
    }
}

fn process_int_lit<T: Num<FromStrRadixErr=ParseIntError>>(pair: Pair<Rule>, base: u32) -> ParseResult<T> {
    T::from_str_radix(pair.into_span().as_str(), base).map_err(From::from)
}

fn process_uint_lit<T: UnsignedFromStr>(pair: Pair<Rule>, base: u32) -> ParseResult<T> {
    T::unsigned_from_str_radix(pair.into_span().as_str(), base)
}

fn process_uint<T>(pair: Pair<Rule>) -> ParseResult<T> 
    where T: Unsigned + Num<FromStrRadixErr=ParseIntError> {
    let inner = pair.into_inner().next().unwrap();
    match inner.as_rule() {
        Rule::bin_uint => process_int_lit(inner.into_inner().next().unwrap(), 2),
        Rule::oct_uint => process_int_lit(inner.into_inner().next().unwrap(), 8),
        Rule::hex_uint => process_int_lit(inner.into_inner().next().unwrap(), 16),
        Rule::dec_uint => process_int_lit(inner, 10),
        _ => unreachable!()
    }
}

fn process_int<T>(pair: Pair<Rule>) -> ParseResult<T>
    where T: Signed + UnsignedFromStr + Num<FromStrRadixErr=ParseIntError> {

    let inner = pair.into_inner().next().unwrap();
    match inner.as_rule() {
        Rule::bin_uint => process_uint_lit(inner.into_inner().next().unwrap(), 2),
        Rule::oct_uint => process_uint_lit(inner.into_inner().next().unwrap(), 8),
        Rule::hex_uint => process_uint_lit(inner.into_inner().next().unwrap(), 16),
        Rule::dec_int => process_int_lit(inner, 10),
        _ => unreachable!()
    }
}

fn process_int_list<'i, T>(pair: Pair<'i, Rule>, data: &mut DataVec) -> ParseResult<'i, ()>
    where T: UnsignedFromStr + Num<FromStrRadixErr=ParseIntError> {
    let pairs = pair.into_inner();
    let element_size = mem::size_of::<T>();
    let (lower, upper) = pairs.size_hint();
    let estimated_size = if let Some(upper_bound) = upper { upper_bound } else { lower };
    data.reserve(estimated_size * element_size);

    for int in pairs {
        let value = process_int::<T>(int)?.to_u64().ok_or(ParseError::CastInt)?;
        let current_size = data.len();
        let new_size = current_size + element_size;
        data.resize(new_size, 0u8);
        Endian::write_uint(&mut data[current_size..new_size], value, element_size);
    }
    Ok(())
}

fn process_data_element<'i>(pair: Pair<'i, Rule>, data: &mut DataVec) -> Result<'i, ()> {
    debug_assert_matches!(pair.as_rule(), Rule::data_element);
    let inner = pair.into_inner().next().unwrap();
    
    match inner.as_rule() {
        Rule::data_block => {
            let element_size = process_uint::<usize>(inner.into_inner().next().unwrap())?;
            let new_size = data.len().checked_add(element_size).ok_or(AssembleError::Misc)?;
            data.resize(new_size, 0u8);
        }
        Rule::data_byte => process_int_list::<i8>(inner.into_inner().next().unwrap(), data)?,
        Rule::data_short => process_int_list::<i16>(inner.into_inner().next().unwrap(), data)?,
        Rule::data_word => process_int_list::<i32>(inner.into_inner().next().unwrap(), data)?,
        _ => unreachable!()
    };

    if data.len() >= u32::max_value() as usize {
        return Err(From::from(AssembleError::Misc));
    }

    Ok(())
}

fn process_labeled_element<'i, F>(pair: Pair<'i, Rule>, labels: &mut LabelMap<'i>, rule: Rule, len: u32, op: F) -> Result<'i, ()>
    where F: FnOnce(Pair<'i, Rule>) -> Result<()> {

    let mut pairs = pair.into_inner();
    let first = pairs.next().unwrap();
    let r = first.as_rule();
    if r == Rule::label {
        let label_str = first.into_inner().next().unwrap().into_span().as_str();
        labels.insert(label_str, len);
        op(pairs.next().unwrap())?;
    } else if r == rule {
        op(first)?;
    } else {
        unreachable!();
    }

    Ok(())
}

fn process_data<'i>(pair: Pair<'i, Rule>) -> Result<'i, DataInfo<'i>> {
    debug_assert_matches!(pair.as_rule(), Rule::data);

    let mut data = Vec::new();
    let mut labels = HashMap::new();

    for labeled_data_element in pair.into_inner() {
        process_labeled_element(labeled_data_element, &mut labels, Rule::data_element, data.len() as u32,
            |p| process_data_element(p, &mut data))?;
    }

    Ok((data, labels))
}

fn process_enum_inner<'i, T: FromStr<Err=ParseEnumError>>(pair: Pair<'i, Rule>) -> ParseResult<'i, T> {
    pair.as_str().to_uppercase().parse().map_err(From::from)
}

fn process_enum<'i, T: FromStr<Err=ParseEnumError>>(pair: Pair<'i, Rule>) -> ParseResult<'i, T> {
    process_enum_inner(pair.into_inner().next().unwrap())
}

fn process_jump_target<'i, T>(pair: Pair<'i, Rule>) -> ParseResult<'i, JumpTarget<'i, T>>
    where T: UnsignedFromStr + Num<FromStrRadixErr=ParseIntError> + Copy {
    let inner = pair.into_inner().next().unwrap();
    let rule = inner.as_rule();
    let target = match rule {
        Rule::int => {
            JumpTarget::Address(process_int(inner)?)
        }
        Rule::identifier => {
            JumpTarget::Label(inner.as_str())
        }
        _ => unreachable!()
    };
    Ok(target)
}

fn process_instruction<'i>(pair: Pair<'i, Rule>, instr: &mut InstrVec<'i>, data_labels: &LabelMap<'i>) -> Result<'i, ()> {
    let inner = pair.into_inner().next().unwrap();
    let rule = inner.as_rule();
    let mut pairs = inner.into_inner();

    match rule {
        Rule::instruction_alu => {
            let alu_funct = process_enum(pairs.next().unwrap())?;
            let rd = process_enum(pairs.next().unwrap())?;
            let rs1 = process_enum(pairs.next().unwrap())?;
            let rs2 = process_enum(pairs.next().unwrap())?;
            instr.push(ParsedInstruction::Complete(instr_alu(alu_funct, rd, rs1, rs2)));
        }
        Rule::instruction_i => {
            let opcode = process_enum(pairs.next().unwrap())?;
            let rd = process_enum(pairs.next().unwrap())?;
            let rs1 = process_enum(pairs.next().unwrap())?;
            let immediate = process_int(pairs.next().unwrap())?;
            instr.push(ParsedInstruction::Complete(instr_i(opcode, rd, rs1, immediate)));
        }
        Rule::instruction_ds => {
            let opcode = process_enum(pairs.next().unwrap())?;
            let rd = process_enum(pairs.next().unwrap())?;
            let rs1 = process_enum(pairs.next().unwrap())?;
            instr.push(ParsedInstruction::Complete(instr_i(opcode, rd, rs1, 0i16)));
        }
        Rule::instruction_li => {
            let opcode = process_enum(pairs.next().unwrap())?;
            let rd = process_enum(pairs.next().unwrap())?;
            let immediate = process_int(pairs.next().unwrap())?;
            instr.push(ParsedInstruction::Complete(instr_i(opcode, rd, RegisterId::ZERO, immediate)));
        }
        Rule::instruction_la => {
            pairs.next();
            let rd = process_enum(pairs.next().unwrap())?;
            let label = pairs.next().unwrap().into_span().as_str();
            let address = data_labels.get(label).ok_or(AssembleError::Misc)?;
            instr.push(ParsedInstruction::Complete(instr_i(OpCode::LI, rd, RegisterId::ZERO, *address as i16)));
            instr.push(ParsedInstruction::Complete(instr_i(OpCode::LHI, rd, RegisterId::ZERO, (*address >> 16) as i16)));
        }
        Rule::instruction_e => {
            let opcode = process_enum_inner(pairs.next().unwrap())?;
            instr.push(ParsedInstruction::Complete(instr_i(opcode, RegisterId::ZERO, RegisterId::ZERO, 0i16)));
        }
        Rule::instruction_br => {
            let opcode = process_enum(pairs.next().unwrap())?;
            let rs1 = process_enum(pairs.next().unwrap())?;
            let target = process_jump_target(pairs.next().unwrap())?;
            instr.push(ParsedInstruction::Branch {
                opcode: opcode,
                rs1: rs1,
                target: target
            });
        }
        Rule::instruction_jr => {
            let opcode = process_enum(pairs.next().unwrap())?;
            let rs1 = process_enum(pairs.next().unwrap())?;
            instr.push(ParsedInstruction::Complete(instr_i(opcode, RegisterId::ZERO, rs1, 0)));
        }
        Rule::instruction_ls => {
            let opcode = process_enum(pairs.next().unwrap())?;
            let rd = process_enum(pairs.next().unwrap())?;
            let immediate = process_int(pairs.next().unwrap())?;
            let rs1 = process_enum(pairs.next().unwrap())?;
            instr.push(ParsedInstruction::Complete(instr_i(opcode, rd, rs1, immediate)));
        }
        Rule::instruction_j => {
            let opcode = process_enum(pairs.next().unwrap())?;
            let target = process_jump_target(pairs.next().unwrap())?;
            instr.push(ParsedInstruction::Jump {
                opcode: opcode,
                target: target
            });
        }
        _ => unreachable!()
    }

    if instr.len() >= (u32::max_value() / constants::WORD_BYTES) as usize {
        return Err(From::from(AssembleError::Misc));
    }

    Ok(())
}

fn process_instructions<'i>(pair: Pair<'i, Rule>, data_labels: LabelMap<'i>) -> Result<'i, InstrInfo<'i>> {
    debug_assert_matches!(pair.as_rule(), Rule::instructions);

    let mut instructions = Vec::new();
    let mut labels = HashMap::new();
    
    for labeled_instruction in pair.into_inner() {
        process_labeled_element(labeled_instruction, &mut labels, Rule::instruction, instructions.len() as u32,
            |p| process_instruction(p, &mut instructions, &data_labels))?;
    }

    Ok((instructions, labels))
}

struct Assembler<'i> {
    parsed_instructions: InstrVec<'i>,
    instruction_labels: LabelMap<'i>
}

impl<'i> Assembler<'i> {
    fn resolve_jump_target<T: NumCast+Num+Copy>(&self, target: &JumpTarget<T>, current_instr: u32) -> AssembleResult<T> {
        match target {
            &JumpTarget::Address(address) => Ok(address),
            &JumpTarget::Label(label) => {
                let absolute = *self.instruction_labels.get(label).ok_or(AssembleError::Misc)? as i64;
                let relative = absolute - current_instr as i64;
                let byte_dist = relative * constants::WORD_BYTES as i64;
                NumCast::from(byte_dist).ok_or(From::from(AssembleError::Misc))
            }
        }
    }

    fn finalize_instruction(&self, instr: &ParsedInstruction, current_instr: u32) -> AssembleResult<Word> {
        Ok(match instr {
            &ParsedInstruction::Complete(word) => word,
            &ParsedInstruction::Branch { ref opcode, ref rs1, ref target } => {
                instr_i(*opcode, RegisterId::ZERO, *rs1, self.resolve_jump_target(&target, current_instr)?)
            },
            &ParsedInstruction::Jump { ref opcode, ref target } => {
                instr_j(*opcode, self.resolve_jump_target(&target, current_instr)?)
            }
        })
    }

    pub fn assemble_instructions(self) -> AssembleResult<Vec<u8>> {
        let result_size = self.parsed_instructions.len() * constants::WORD_BYTES as usize;
        let mut result = vec![0; result_size];

        for (i, pi) in self.parsed_instructions.iter().enumerate() {
            let instr = self.finalize_instruction(pi, i as u32)?;
            let start = i * constants::WORD_BYTES as usize;
            let end = start + constants::WORD_BYTES as usize;
            Endian::write_u32(&mut result[start..end], instr);
        }

        return Ok(result);
    }
}

fn assemble(pair: Pair<Rule>) -> Result<Program> {
    let mut pairs = pair.into_inner();

    let (data, data_labels) = process_data(pairs.next().unwrap())?;
    let (instr, instr_labels) = process_instructions(pairs.next().unwrap(), data_labels)?;

    let assembler = Assembler {
        parsed_instructions: instr,
        instruction_labels: instr_labels
    };

    Ok(Program {
        data: data,
        instructions: assembler.assemble_instructions()?
    })
}

fn parse<'i>(input: &'i str) -> ParseResult<'i, Pair<'i, Rule>> {
    Ok(VASMParser::parse(Rule::program, input)?.next().unwrap())
}

pub fn parse_and_assemble<'i>(input: &'i str) -> Result<Program> {
    assemble(parse(input)?)
}

#[cfg(test)]
mod test;
