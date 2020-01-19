use crate::{assemble, JumpTarget, ParsedInstruction, Program, Rule, VASMParser};
use ::pest::{error::Error as PestError, iterators::Pair, Parser, Span};
use byteorder::ByteOrder;
use std::collections::HashMap;
use vcpu::*;

mod pest;

macro_rules! hashmap {
    ($( $key: expr => $val: expr ),*) => {{
         let mut map = ::std::collections::HashMap::new();
         $( map.insert($key, $val); )*
         map
    }}
}

fn parse_rule(rule: Rule, input: &str) -> Result<Pair<Rule>, PestError<Rule>> {
    Ok(VASMParser::parse(rule, input)?.next().unwrap())
}

#[test]
fn process_instructions_add() {
    let input = ".instructions
LI $t0, 23
LI $t1, 34
ADD $t0, $t0, $t1
HALT";

    let expected_instr = vec![
        ParsedInstruction::Complete(instr_i(OpCode::LI, RegisterId::T0, RegisterId::ZERO, 23)),
        ParsedInstruction::Complete(instr_i(OpCode::LI, RegisterId::T1, RegisterId::ZERO, 34)),
        ParsedInstruction::Complete(instr_alu(
            ALUFunct::ADD,
            RegisterId::T0,
            RegisterId::T0,
            RegisterId::T1,
        )),
        ParsedInstruction::Complete(instr_i(OpCode::HALT, RegisterId::ZERO, RegisterId::ZERO, 0)),
    ];

    let expected_labels = HashMap::new();

    let pair = parse_rule(Rule::instructions, input).unwrap();
    let (instr, labels) = super::process_instructions(pair, &HashMap::new()).unwrap();

    assert_eq!(instr, expected_instr);
    assert_eq!(labels, expected_labels);
}

#[test]
fn process_instructions_loop() {
    let input = ".instructions
loop: SLTI $t2, $t0, 32
      BEZ  $t2, end
      SLLI $t1, $t0, 2
      SW   $t0, 0($t1)
      ADDI $t0, $t0, 1
      JMP loop
end:  HALT";

    let expected_instr = vec![
        ParsedInstruction::Complete(instr_i(OpCode::SLTI, RegisterId::T2, RegisterId::T0, 32)),
        ParsedInstruction::Branch {
            opcode: OpCode::BEZ,
            rs1: RegisterId::T2,
            target: JumpTarget::Label(Span::new(input, 54, 57).unwrap()),
        },
        ParsedInstruction::Complete(instr_i(OpCode::SLLI, RegisterId::T1, RegisterId::T0, 2)),
        ParsedInstruction::Complete(instr_i(OpCode::SW, RegisterId::T0, RegisterId::T1, 0)),
        ParsedInstruction::Complete(instr_i(OpCode::ADDI, RegisterId::T0, RegisterId::T0, 1)),
        ParsedInstruction::Jump {
            opcode: OpCode::JMP,
            target: JumpTarget::Label(Span::new(input, 137, 141).unwrap()),
        },
        ParsedInstruction::Complete(instr_i(OpCode::HALT, RegisterId::ZERO, RegisterId::ZERO, 0)),
    ];

    let expected_labels = hashmap![
        "loop" => 0,
        "end" => 6
    ];

    let pair = parse_rule(Rule::instructions, input).unwrap();
    let (instr, labels) = super::process_instructions(pair, &HashMap::new()).unwrap();

    assert_eq!(instr, expected_instr);
    assert_eq!(labels, expected_labels);
}

fn transmute_vec(vec: Vec<Word>) -> Vec<u8> {
    let mut byte_vec = vec![0; vec.len() * constants::WORD_BYTES as usize];
    Endian::write_u32_into(&vec[..], &mut byte_vec[..]);
    byte_vec
}

#[test]
fn assemble_loop() {
    let input = ".data
.block 128
.instructions
loop: SLTI $t2, $t0, 32
      BEZ  $t2, end
      SLLI $t1, $t0, 2
      SW   $t0, 0($t1)
      ADDI $t0, $t0, 1
      JMP loop
end:  HALT";

    let expected_data = vec![0; 128];

    let expected_instr = transmute_vec(vec![
        instr_i(OpCode::SLTI, RegisterId::T2, RegisterId::T0, 32),
        instr_i(
            OpCode::BEZ,
            RegisterId::ZERO,
            RegisterId::T2,
            jmp_addr_i16(5),
        ),
        instr_i(OpCode::SLLI, RegisterId::T1, RegisterId::T0, 2),
        instr_i(OpCode::SW, RegisterId::T0, RegisterId::T1, 0),
        instr_i(OpCode::ADDI, RegisterId::T0, RegisterId::T0, 1),
        instr_j(OpCode::JMP, jmp_addr_i32(-5)),
        instr_i(OpCode::HALT, RegisterId::ZERO, RegisterId::ZERO, 0),
    ]);

    let program = assemble(input).unwrap();

    assert_eq!(program, Program::from(expected_data, expected_instr));
}
