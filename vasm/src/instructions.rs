use crate::int_util::*;
use crate::labels::*;
use crate::*;
use byteorder::ByteOrder;
use matches::debug_assert_matches;
use num::*;
use std::collections::HashMap;
use std::num::ParseIntError;
use std::str::FromStr;
use util::ParseEnumError;
use vcpu::*;

type InstrVec<'i> = Vec<ParsedInstruction<'i>>;

#[derive(Debug, PartialEq)]
pub enum JumpTarget<'i, T: Num + Copy> {
    Address(T),
    Label(Span<'i>),
}

#[derive(Debug, PartialEq)]
pub enum ParsedInstruction<'i> {
    Complete(Word),

    Branch {
        opcode: Opcode,
        rs1: RegisterId,
        target: JumpTarget<'i, Immediate>,
    },

    Jump {
        opcode: Opcode,
        target: JumpTarget<'i, Address>,
    },

    LoadInstructionAddress {
        label: Span<'i>,
        rd: RegisterId,
        upper: bool,
    },
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
            let alu_funct = process_enum_inner(&pairs.next().unwrap())?;
            let rd = process_enum(pairs.next().unwrap())?;
            let rs1 = process_enum(pairs.next().unwrap())?;
            let rs2 = process_enum(pairs.next().unwrap())?;
            instr.push(ParsedInstruction::Complete(make_alu_instruction(
                alu_funct, rd, rs1, rs2,
            )));
        }
        Rule::instruction_flop => {
            let flop_funct = process_enum_inner(&pairs.next().unwrap())?;
            let rd = process_enum(pairs.next().unwrap())?;
            let rs1 = process_enum(pairs.next().unwrap())?;
            let rs2 = process_enum(pairs.next().unwrap())?;
            instr.push(ParsedInstruction::Complete(make_flop_instruction(
                flop_funct, rd, rs1, rs2,
            )));
        }
        Rule::instruction_i => {
            let opcode = process_enum_inner(&pairs.next().unwrap())?;
            let rd = process_enum(pairs.next().unwrap())?;
            let rs1 = process_enum(pairs.next().unwrap())?;
            let immediate = process_int(pairs.next().unwrap())?;
            instr.push(ParsedInstruction::Complete(make_i_instruction(
                opcode, rd, rs1, immediate,
            )));
        }
        Rule::instruction_iu => {
            let opcode = process_enum_inner(&pairs.next().unwrap())?;
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
            let opcode = process_enum_inner(&pairs.next().unwrap())?;
            let rd = process_enum(pairs.next().unwrap())?;
            let rs1 = process_enum(pairs.next().unwrap())?;
            instr.push(ParsedInstruction::Complete(make_i_instruction(
                opcode, rd, rs1, 0i16,
            )));
        }
        Rule::instruction_li => {
            let opcode = process_enum_inner(&pairs.next().unwrap())?;
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
            let opcode = process_enum_inner(&pairs.next().unwrap())?;
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
            let opcode = process_enum_inner(&pairs.next().unwrap())?;
            let rs1 = process_enum(pairs.next().unwrap())?;
            let target = process_jump_target(pairs.next().unwrap())?;
            instr.push(ParsedInstruction::Branch {
                opcode,
                rs1,
                target,
            });
        }
        Rule::instruction_jr => {
            let opcode = process_enum_inner(&pairs.next().unwrap())?;
            let rs1 = process_enum(pairs.next().unwrap())?;
            instr.push(ParsedInstruction::Complete(make_i_instruction(
                opcode,
                RegisterId::ZERO,
                rs1,
                0,
            )));
        }
        Rule::instruction_ls => {
            let opcode = process_enum_inner(&pairs.next().unwrap())?;
            let rd = process_enum(pairs.next().unwrap())?;
            let immediate = process_int(pairs.next().unwrap())?;
            let rs1 = process_enum(pairs.next().unwrap())?;
            instr.push(ParsedInstruction::Complete(make_i_instruction(
                opcode, rd, rs1, immediate,
            )));
        }
        Rule::instruction_j => {
            let opcode = process_enum_inner(&pairs.next().unwrap())?;
            let target = process_jump_target(pairs.next().unwrap())?;
            instr.push(ParsedInstruction::Jump { opcode, target });
        }
        Rule::instruction_push => {
            let register = process_enum(pairs.next().unwrap())?;
            instr.push(ParsedInstruction::Complete(make_i_instruction(
                Opcode::SW,
                register,
                RegisterId::SP,
                -4i16,
            )));
            instr.push(ParsedInstruction::Complete(make_i_instruction(
                Opcode::SUBI,
                RegisterId::SP,
                RegisterId::SP,
                4i16,
            )));
        }
        Rule::instruction_pop => {
            let register = process_enum(pairs.next().unwrap())?;
            instr.push(ParsedInstruction::Complete(make_i_instruction(
                Opcode::LW,
                register,
                RegisterId::SP,
                0i16,
            )));
            instr.push(ParsedInstruction::Complete(make_i_instruction(
                Opcode::ADDI,
                RegisterId::SP,
                RegisterId::SP,
                4i16,
            )));
        }
        Rule::instruction_lwi => {
            let register = process_enum(pairs.next().unwrap())?;
            let value: i32 = process_int(pairs.next().unwrap())?;
            instr.push(ParsedInstruction::Complete(make_i_instruction(
                Opcode::SLO,
                register,
                RegisterId::ZERO,
                value as i16,
            )));
            instr.push(ParsedInstruction::Complete(make_i_instruction(
                Opcode::SHI,
                register,
                RegisterId::ZERO,
                (value >> 16) as i16,
            )));
        }
        Rule::instruction_lda => {
            let rd = process_enum(pairs.next().unwrap())?;
            let label_span = pairs.next().unwrap().as_span();
            let label = label_span.as_str();
            let address = data_labels.get(label).ok_or_else(|| {
                new_parser_error(label_span, "Data label was not found".to_owned())
            })?;
            let offset_address = *address + data_offset;

            instr.push(ParsedInstruction::Complete(make_i_instruction(
                Opcode::SLO,
                rd,
                RegisterId::ZERO,
                offset_address as i16,
            )));
            instr.push(ParsedInstruction::Complete(make_i_instruction(
                Opcode::SHI,
                rd,
                RegisterId::ZERO,
                (offset_address >> 16) as i16,
            )));
        }
        Rule::instruction_lia => {
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

    let max_size = (u32::max_value() / WORD_BYTES) as usize - 1;
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

pub fn process_instructions<'i>(
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
            let byte_dist = relative * Into::<i64>::into(WORD_BYTES);
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
                * WORD_BYTES;
            if *upper {
                make_i_instruction(Opcode::SHI, *rd, RegisterId::ZERO, (address >> 16) as i16)
            } else {
                make_i_instruction(Opcode::SLO, *rd, RegisterId::ZERO, address as i16)
            }
        }
    })
}

pub fn assemble_instructions(instr: &[ParsedInstruction], labels: &LabelMap) -> Result<Vec<u8>> {
    let result_size = instr.len() * WORD_BYTES as usize;
    let mut result = vec![0; result_size];

    for (i, pi) in instr.iter().enumerate() {
        let instr = finalize_instruction(labels, pi, i as u32)?;
        let start = i * WORD_BYTES as usize;
        let end = start + WORD_BYTES as usize;
        Endian::write_u32(&mut result[start..end], instr);
    }

    Ok(result)
}
