use byteorder::ByteOrder;
use matches::*;
use num::{Num, NumCast, Signed, ToPrimitive, Unsigned};
use pest::iterators::Pair;
use pest::{Parser, Span};
use pest_derive::Parser;
use std::collections::HashMap;
use std::mem;
use std::num::ParseIntError;
use std::str::FromStr;
use util::ParseEnumError;
use vcpu::*;
use vexfile::Program;

// TODO: write documentation with a list of all mnemonics and their behavior
// TODO: rename "opcode*" rules to "mnemonic*"

pub type Error = pest::error::Error<crate::Rule>;

pub fn new_parser_error(span: Span, message: String) -> Error {
    Error::new_from_span(pest::error::ErrorVariant::CustomError { message }, span)
}

#[derive(Debug, PartialEq)]
enum JumpTarget<'i, T: Num + Copy> {
    Address(T),
    Label(Span<'i>),
}

#[derive(Debug, PartialEq)]
enum ParsedInstruction<'i> {
    Complete(Word),

    Branch {
        opcode: OpCode,
        rs1: RegisterId,
        target: JumpTarget<'i, Immediate>,
    },

    Jump {
        opcode: OpCode,
        target: JumpTarget<'i, Address>,
    },

    LoadInstructionAddress {
        label: Span<'i>,
        rd: RegisterId,
        upper: bool,
    },
}

type Result<T> = std::result::Result<T, Error>;

#[derive(Parser)]
#[grammar = "vasm.pest"]
struct VASMParser;

type LabelMap<'i> = HashMap<&'i str, u32>;
type InstrVec<'i> = Vec<ParsedInstruction<'i>>;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SourceMapItem {
    pub start_line: u32,
    pub line_count: u32,
}

pub type SourceMap = Vec<SourceMapItem>;

trait GetUnsigned: Signed {
    type Unsigned;
}

impl GetUnsigned for i8 {
    type Unsigned = u8;
}

impl GetUnsigned for i16 {
    type Unsigned = u16;
}

impl GetUnsigned for i32 {
    type Unsigned = u32;
}

trait ToPrimitiveTrunc: Sized {
    fn to_i8(&self) -> i8;
    fn to_i16(&self) -> i16;
    fn to_i32(&self) -> i32;
}

macro_rules! impl_to_prim_trunc {
    ($T:ty) => {
        impl ToPrimitiveTrunc for $T {
            #[inline]
            fn to_i8(&self) -> i8 {
                *self as i8
            }
            #[inline]
            fn to_i16(&self) -> i16 {
                *self as i16
            }
            #[inline]
            fn to_i32(&self) -> i32 {
                *self as i32
            }
        }
    };
}

impl_to_prim_trunc!(u8);
impl_to_prim_trunc!(u16);
impl_to_prim_trunc!(u32);

trait NumCastTrunc: Sized {
    fn from<T: ToPrimitiveTrunc>(n: T) -> Self;
}

macro_rules! impl_num_cast_trunc {
    ($T:ty, $conv:ident) => {
        impl NumCastTrunc for $T {
            #[inline]
            fn from<N: ToPrimitiveTrunc>(n: N) -> $T {
                n.$conv()
            }
        }
    };
}

impl_num_cast_trunc!(i8, to_i8);
impl_num_cast_trunc!(i16, to_i16);
impl_num_cast_trunc!(i32, to_i32);

fn process_num_lit<T>(pair: Pair<Rule>, base: u32) -> Result<T>
where
    T: Num<FromStrRadixErr = ParseIntError>,
{
    let span = pair.as_span();
    T::from_str_radix(span.as_str(), base)
        .map_err(|err| new_parser_error(span, format!("Parsing integer failed: {}", err)))
}

fn process_unsigned_lit<T>(pair: Pair<Rule>, base: u32) -> Result<T>
where
    T: GetUnsigned + NumCastTrunc,
    <T as GetUnsigned>::Unsigned: Num<FromStrRadixErr = ParseIntError> + ToPrimitiveTrunc,
{
    Ok(NumCastTrunc::from(process_num_lit::<T::Unsigned>(
        pair, base,
    )?))
}

fn process_uint<T>(pair: Pair<Rule>) -> Result<T>
where
    T: Unsigned + Num<FromStrRadixErr = ParseIntError>,
{
    let inner = pair.into_inner().next().unwrap();
    match inner.as_rule() {
        Rule::bin_uint => process_num_lit(inner.into_inner().next().unwrap(), 2),
        Rule::oct_uint => process_num_lit(inner.into_inner().next().unwrap(), 8),
        Rule::hex_uint => process_num_lit(inner.into_inner().next().unwrap(), 16),
        Rule::dec_uint => process_num_lit(inner, 10),
        _ => unreachable!(),
    }
}

fn process_int<T>(pair: Pair<Rule>) -> Result<T>
where
    T: GetUnsigned + Num<FromStrRadixErr = ParseIntError> + NumCastTrunc,
    <T as GetUnsigned>::Unsigned: Num<FromStrRadixErr = ParseIntError> + ToPrimitiveTrunc,
{
    let inner = pair.into_inner().next().unwrap();
    match inner.as_rule() {
        Rule::bin_uint => process_unsigned_lit(inner.into_inner().next().unwrap(), 2),
        Rule::oct_uint => process_unsigned_lit(inner.into_inner().next().unwrap(), 8),
        Rule::hex_uint => process_unsigned_lit(inner.into_inner().next().unwrap(), 16),
        Rule::dec_int => process_num_lit(inner, 10),
        _ => unreachable!(),
    }
}

fn process_int_list<T>(pair: Pair<Rule>, data: &mut Vec<u8>) -> Result<()>
where
    T: GetUnsigned + Num<FromStrRadixErr = ParseIntError> + ToPrimitive + NumCastTrunc,
    <T as GetUnsigned>::Unsigned: Num<FromStrRadixErr = ParseIntError> + ToPrimitiveTrunc,
{
    let pairs = pair.into_inner();
    let element_size = mem::size_of::<T>();
    let (lower, upper) = pairs.size_hint();
    let estimated_size = if let Some(upper_bound) = upper {
        upper_bound
    } else {
        lower
    };
    data.reserve(estimated_size * element_size);

    for int in pairs {
        let span = int.as_span();
        let value = process_int::<T>(int)?
            .to_i64()
            .ok_or_else(|| new_parser_error(span, "Cannot cast integer".to_owned()))?;
        let current_size = data.len();
        let new_size = current_size + element_size;
        data.resize(new_size, 0u8);
        Endian::write_int(&mut data[current_size..new_size], value, element_size);
    }
    Ok(())
}

fn process_data_element(pair: Pair<Rule>, data: &mut Vec<u8>) -> Result<()> {
    debug_assert_matches!(pair.as_rule(), Rule::data_element);
    let inner = pair.into_inner().next().unwrap();
    let span = inner.as_span();

    match inner.as_rule() {
        Rule::data_block => {
            let element_size = process_uint::<usize>(inner.into_inner().next().unwrap())?;
            let new_size = data.len().checked_add(element_size).ok_or_else(|| {
                new_parser_error(span.clone(), "Data block is too big".to_owned())
            })?;
            data.resize(new_size, 0u8);
        }
        Rule::data_byte => process_int_list::<i8>(inner.into_inner().next().unwrap(), data)?,
        Rule::data_short => process_int_list::<i16>(inner.into_inner().next().unwrap(), data)?,
        Rule::data_word => process_int_list::<i32>(inner.into_inner().next().unwrap(), data)?,
        _ => unreachable!(),
    };

    let max_size = u32::max_value() as usize - 1;

    if data.len() > max_size {
        Err(new_parser_error(
            span,
            format!("Data exceeds maximum size of {} bytes", max_size),
        ))
    } else {
        Ok(())
    }
}

fn process_labeled_element<'i, F>(
    pair: Pair<'i, Rule>,
    labels: &mut LabelMap<'i>,
    rule: Rule,
    len: u32,
    op: F,
) -> Result<()>
where
    F: FnOnce(Pair<'i, Rule>) -> Result<()>,
{
    let mut pairs = pair.into_inner();
    let first = pairs.next().unwrap();
    let r = first.as_rule();
    if r == Rule::label {
        let label_str = first.into_inner().next().unwrap().as_span().as_str();
        labels.insert(label_str, len);
        op(pairs.next().unwrap())?;
    } else if r == rule {
        op(first)?;
    } else {
        unreachable!();
    }

    Ok(())
}

fn process_data(pair: Pair<Rule>) -> Result<(Vec<u8>, LabelMap)> {
    debug_assert_matches!(pair.as_rule(), Rule::data);

    let mut data = Vec::new();
    let mut labels = HashMap::new();

    for labeled_data_element in pair.into_inner() {
        process_labeled_element(
            labeled_data_element,
            &mut labels,
            Rule::data_element,
            data.len() as u32,
            |p| process_data_element(p, &mut data),
        )?;
    }

    Ok((data, labels))
}

fn process_enum_inner<'i, T: FromStr<Err = ParseEnumError>>(pair: &Pair<'i, Rule>) -> Result<T> {
    pair.as_str()
        .to_uppercase()
        .parse()
        .map_err(|err| new_parser_error(pair.as_span(), format!("{}", err)))
}

fn process_enum<T: FromStr<Err = ParseEnumError>>(pair: Pair<Rule>) -> Result<T> {
    process_enum_inner(&pair.into_inner().next().unwrap())
}

fn process_jump_target<T>(pair: Pair<Rule>) -> Result<JumpTarget<T>>
where
    T: GetUnsigned + Num<FromStrRadixErr = ParseIntError> + NumCastTrunc + Copy,
    <T as GetUnsigned>::Unsigned: Num<FromStrRadixErr = ParseIntError> + ToPrimitiveTrunc,
{
    let inner = pair.into_inner().next().unwrap();
    let rule = inner.as_rule();
    let target = match rule {
        Rule::int => JumpTarget::Address(process_int(inner)?),
        Rule::identifier => JumpTarget::Label(inner.as_span()),
        _ => unreachable!(),
    };
    Ok(target)
}

fn process_instruction<'i>(
    pair: Pair<'i, Rule>,
    instr: &mut InstrVec<'i>,
    data_labels: &LabelMap<'i>,
    data_offset: u32,
) -> Result<usize> {
    let span = pair.as_span();
    let inner = pair.into_inner().next().unwrap();
    let rule = inner.as_rule();
    let mut pairs = inner.into_inner();

    let old_len = instr.len();

    match rule {
        Rule::instruction_alu => {
            let alu_funct = process_enum(pairs.next().unwrap())?;
            let rd = process_enum(pairs.next().unwrap())?;
            let rs1 = process_enum(pairs.next().unwrap())?;
            let rs2 = process_enum(pairs.next().unwrap())?;
            instr.push(ParsedInstruction::Complete(make_alu_instruction(
                alu_funct, rd, rs1, rs2,
            )));
        }
        Rule::instruction_flop => {
            let flop_funct = process_enum(pairs.next().unwrap())?;
            let rd = process_enum(pairs.next().unwrap())?;
            let rs1 = process_enum(pairs.next().unwrap())?;
            let rs2 = process_enum(pairs.next().unwrap())?;
            instr.push(ParsedInstruction::Complete(make_flop_instruction(
                flop_funct, rd, rs1, rs2,
            )));
        }
        Rule::instruction_i => {
            let opcode = process_enum(pairs.next().unwrap())?;
            let rd = process_enum(pairs.next().unwrap())?;
            let rs1 = process_enum(pairs.next().unwrap())?;
            let immediate = process_int(pairs.next().unwrap())?;
            instr.push(ParsedInstruction::Complete(make_i_instruction(
                opcode, rd, rs1, immediate,
            )));
        }
        Rule::instruction_iu => {
            let opcode = process_enum(pairs.next().unwrap())?;
            let rd = process_enum(pairs.next().unwrap())?;
            let rs1 = process_enum(pairs.next().unwrap())?;
            let immediate = process_uint::<u16>(pairs.next().unwrap())?;
            instr.push(ParsedInstruction::Complete(make_i_instruction(
                opcode,
                rd,
                rs1,
                immediate as i16,
            )));
        }
        Rule::instruction_ds => {
            let opcode = process_enum(pairs.next().unwrap())?;
            let rd = process_enum(pairs.next().unwrap())?;
            let rs1 = process_enum(pairs.next().unwrap())?;
            instr.push(ParsedInstruction::Complete(make_i_instruction(
                opcode, rd, rs1, 0i16,
            )));
        }
        Rule::instruction_li => {
            let opcode = process_enum(pairs.next().unwrap())?;
            let rd = process_enum(pairs.next().unwrap())?;
            let immediate = process_int(pairs.next().unwrap())?;
            instr.push(ParsedInstruction::Complete(make_i_instruction(
                opcode,
                rd,
                RegisterId::ZERO,
                immediate,
            )));
        }
        Rule::instruction_si => {
            let opcode = process_enum(pairs.next().unwrap())?;
            let rd = process_enum(pairs.next().unwrap())?;
            let immediate = process_uint::<u16>(pairs.next().unwrap())?;
            instr.push(ParsedInstruction::Complete(make_i_instruction(
                opcode,
                rd,
                RegisterId::ZERO,
                immediate as i16,
            )));
        }
        Rule::instruction_e => {
            let opcode = process_enum_inner(&pairs.next().unwrap())?;
            instr.push(ParsedInstruction::Complete(make_i_instruction(
                opcode,
                RegisterId::ZERO,
                RegisterId::ZERO,
                0i16,
            )));
        }
        Rule::instruction_br => {
            let opcode = process_enum(pairs.next().unwrap())?;
            let rs1 = process_enum(pairs.next().unwrap())?;
            let target = process_jump_target(pairs.next().unwrap())?;
            instr.push(ParsedInstruction::Branch {
                opcode,
                rs1,
                target,
            });
        }
        Rule::instruction_jr => {
            let opcode = process_enum(pairs.next().unwrap())?;
            let rs1 = process_enum(pairs.next().unwrap())?;
            instr.push(ParsedInstruction::Complete(make_i_instruction(
                opcode,
                RegisterId::ZERO,
                rs1,
                0,
            )));
        }
        Rule::instruction_ls => {
            let opcode = process_enum(pairs.next().unwrap())?;
            let rd = process_enum(pairs.next().unwrap())?;
            let immediate = process_int(pairs.next().unwrap())?;
            let rs1 = process_enum(pairs.next().unwrap())?;
            instr.push(ParsedInstruction::Complete(make_i_instruction(
                opcode, rd, rs1, immediate,
            )));
        }
        Rule::instruction_j => {
            let opcode = process_enum(pairs.next().unwrap())?;
            let target = process_jump_target(pairs.next().unwrap())?;
            instr.push(ParsedInstruction::Jump { opcode, target });
        }
        Rule::macro_push => {
            let register = process_enum(pairs.next().unwrap())?;
            instr.push(ParsedInstruction::Complete(make_i_instruction(
                OpCode::SW,
                register,
                RegisterId::SP,
                -4i16,
            )));
            instr.push(ParsedInstruction::Complete(make_i_instruction(
                OpCode::SUBI,
                RegisterId::SP,
                RegisterId::SP,
                4i16,
            )));
        }
        Rule::macro_pop => {
            let register = process_enum(pairs.next().unwrap())?;
            instr.push(ParsedInstruction::Complete(make_i_instruction(
                OpCode::LW,
                register,
                RegisterId::SP,
                0i16,
            )));
            instr.push(ParsedInstruction::Complete(make_i_instruction(
                OpCode::ADDI,
                RegisterId::SP,
                RegisterId::SP,
                4i16,
            )));
        }
        Rule::macro_lwi => {
            let register = process_enum(pairs.next().unwrap())?;
            let value: i32 = process_int(pairs.next().unwrap())?;
            instr.push(ParsedInstruction::Complete(make_i_instruction(
                OpCode::SLO,
                register,
                RegisterId::ZERO,
                value as i16,
            )));
            instr.push(ParsedInstruction::Complete(make_i_instruction(
                OpCode::SHI,
                register,
                RegisterId::ZERO,
                (value >> 16) as i16,
            )));
        }
        Rule::macro_lda => {
            let rd = process_enum(pairs.next().unwrap())?;
            let label_span = pairs.next().unwrap().as_span();
            let label = label_span.as_str();
            let address = data_labels.get(label).ok_or_else(|| {
                new_parser_error(label_span, "Data label was not found".to_owned())
            })?;
            let offset_address = *address + data_offset;

            instr.push(ParsedInstruction::Complete(make_i_instruction(
                OpCode::SLO,
                rd,
                RegisterId::ZERO,
                offset_address as i16,
            )));
            instr.push(ParsedInstruction::Complete(make_i_instruction(
                OpCode::SHI,
                rd,
                RegisterId::ZERO,
                (offset_address >> 16) as i16,
            )));
        }
        Rule::macro_lia => {
            let rd = process_enum(pairs.next().unwrap())?;
            let label = pairs.next().unwrap();

            instr.push(ParsedInstruction::LoadInstructionAddress {
                label: label.as_span(),
                rd,
                upper: false,
            });
            instr.push(ParsedInstruction::LoadInstructionAddress {
                label: label.as_span(),
                rd,
                upper: true,
            });
        }
        _ => unreachable!(),
    }

    let max_size = (u32::max_value() / constants::WORD_BYTES) as usize - 1;
    let new_len = instr.len();

    if new_len > max_size {
        Err(new_parser_error(
            span,
            format!("Instructions exceed maximum size of {} bytes", max_size),
        ))
    } else {
        Ok(new_len - old_len)
    }
}

fn process_instructions<'i>(
    pair: Pair<'i, Rule>,
    data_labels: &LabelMap<'i>,
    data_offset: u32,
) -> Result<(InstrVec<'i>, LabelMap<'i>, SourceMap)> {
    debug_assert_matches!(pair.as_rule(), Rule::instructions);

    let mut instructions = Vec::new();
    let mut labels = HashMap::new();
    let mut source_map = Vec::new();

    for labeled_instruction in pair.into_inner() {
        let span = labeled_instruction.as_span();
        let start_line = span.start_pos().line_col().0 as u32;
        let end_line = span.end_pos().line_col().0 as u32;
        let line_count = end_line - start_line + 1;
        let source_map_item = SourceMapItem {
            start_line,
            line_count,
        };

        process_labeled_element(
            labeled_instruction,
            &mut labels,
            Rule::instruction,
            instructions.len() as u32,
            |p| {
                let count = process_instruction(p, &mut instructions, &data_labels, data_offset)?;
                for _ in 0..count {
                    source_map.push(source_map_item);
                }

                Ok(())
            },
        )?;
    }

    Ok((instructions, labels, source_map))
}

fn resolve_jump_target<T: NumCast + Num + Copy>(
    labels: &LabelMap,
    target: &JumpTarget<T>,
    current_instr: u32,
) -> Result<T> {
    match target {
        JumpTarget::Address(address) => Ok(*address),
        JumpTarget::Label(label) => {
            let absolute =
                Into::<i64>::into(*labels.get(label.as_str()).ok_or_else(|| {
                    new_parser_error(label.clone(), "Label not found".to_owned())
                })?);

            let relative = absolute - Into::<i64>::into(current_instr);
            let byte_dist = relative * Into::<i64>::into(constants::WORD_BYTES);
            num::NumCast::from(byte_dist)
                .ok_or_else(|| new_parser_error(label.clone(), "Jump distance too far".to_owned()))
        }
    }
}

fn finalize_instruction(
    labels: &LabelMap,
    instr: &ParsedInstruction,
    current_instr: u32,
) -> Result<Word> {
    Ok(match *instr {
        ParsedInstruction::Complete(word) => word,
        ParsedInstruction::Branch {
            ref opcode,
            ref rs1,
            ref target,
        } => make_i_instruction(
            *opcode,
            RegisterId::ZERO,
            *rs1,
            resolve_jump_target(labels, &target, current_instr)?,
        ),
        ParsedInstruction::Jump {
            ref opcode,
            ref target,
        } => make_j_instruction(
            *opcode,
            resolve_jump_target(labels, &target, current_instr)?,
        ),
        ParsedInstruction::LoadInstructionAddress {
            ref label,
            ref rd,
            ref upper,
        } => {
            let address = *labels
                .get(label.as_str())
                .ok_or_else(|| new_parser_error(label.clone(), "Label not found".to_owned()))?
                as u32
                * constants::WORD_BYTES;
            if *upper {
                make_i_instruction(OpCode::SHI, *rd, RegisterId::ZERO, (address >> 16) as i16)
            } else {
                make_i_instruction(OpCode::SLO, *rd, RegisterId::ZERO, address as i16)
            }
        }
    })
}

fn assemble_instructions(instr: &[ParsedInstruction], labels: &LabelMap) -> Result<Vec<u8>> {
    let result_size = instr.len() * constants::WORD_BYTES as usize;
    let mut result = vec![0; result_size];

    for (i, pi) in instr.iter().enumerate() {
        let instr = finalize_instruction(labels, pi, i as u32)?;
        let start = i * constants::WORD_BYTES as usize;
        let end = start + constants::WORD_BYTES as usize;
        Endian::write_u32(&mut result[start..end], instr);
    }

    Ok(result)
}

fn assemble_parsed(pair: Pair<Rule>, data_offset: u32) -> Result<(Program, SourceMap)> {
    let mut pairs = pair.into_inner();

    let (data, data_labels) = process_data(pairs.next().unwrap())?;
    let (instr, instr_labels, source_map) =
        process_instructions(pairs.next().unwrap(), &data_labels, data_offset)?;

    Ok((
        Program::from(
            data_offset,
            assemble_instructions(&instr, &instr_labels)?,
            data,
        ),
        source_map,
    ))
}

fn parse(input: &str) -> Result<Pair<Rule>> {
    Ok(VASMParser::parse(Rule::program, input)?.next().unwrap())
}

pub fn assemble_addressed(input: &str, data_offset: u32) -> Result<(Program, SourceMap)> {
    assemble_parsed(parse(input)?, data_offset)
}

pub fn assemble(input: &str) -> Result<(Program, SourceMap)> {
    assemble_addressed(input, 0u32)
}

#[cfg(test)]
mod test;
