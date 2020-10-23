use crate::instructions::*;
use crate::*;
use ::pest::{iterators::Pair, Parser, Span};
use byteorder::ByteOrder;
use std::collections::HashMap;
use vcpu::*;

mod pest;

// TODO: more unit tests

macro_rules! hashmap {
    ($( $key: expr => $val: expr ),*) => {{
         let mut map = ::std::collections::HashMap::new();
         $( map.insert($key, $val); )*
         map
    }}
}

pub fn parse_rule(rule: Rule, input: &str) -> Result<Pair<Rule>> {
    Ok(VASMParser::parse(rule, input)?.next().unwrap())
}

#[test]
fn large_unsigned_literal() {
    let input = ".instructions 
    LI $t0, 0xFFFF
    LI $t1, 0x7FFF";

    let expected_instr = vec![
        ParsedInstruction::Complete(instr_i!(LI, T0, ZERO, -1i16)),
        ParsedInstruction::Complete(instr_i!(LI, T1, ZERO, i16::MAX)),
    ];

    let pair = parse_rule(Rule::instructions, input).unwrap();
    let (instr, _, _) = process_instructions(pair, &HashMap::new(), 0).unwrap();

    assert_eq!(instr, expected_instr);
}

#[test]
fn process_instructions_add() {
    let input = ".instructions
LI $t0, 23
LI $t1, 34
ADD $t0, $t0, $t1
HALT";

    let expected_instr = vec![
        ParsedInstruction::Complete(instr_i!(LI, T0, ZERO, 23)),
        ParsedInstruction::Complete(instr_i!(LI, T1, ZERO, 34)),
        ParsedInstruction::Complete(instr_alu!(ADD, T0, T0, T1)),
        ParsedInstruction::Complete(instr_i!(HALT, ZERO, ZERO, 0)),
    ];

    let expected_labels = HashMap::new();

    let pair = parse_rule(Rule::instructions, input).unwrap();
    let (instr, labels, _) = process_instructions(pair, &HashMap::new(), 0).unwrap();

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
        ParsedInstruction::Complete(instr_i!(SLTI, T2, T0, 32)),
        ParsedInstruction::Branch {
            opcode: Opcode::BEZ,
            rs1: RegisterId::T2,
            target: JumpTarget::Label(Span::new(input, 54, 57).unwrap()),
        },
        ParsedInstruction::Complete(instr_i!(SLLI, T1, T0, 2)),
        ParsedInstruction::Complete(instr_i!(SW, T0, T1, 0)),
        ParsedInstruction::Complete(instr_i!(ADDI, T0, T0, 1)),
        ParsedInstruction::Jump {
            opcode: Opcode::JMP,
            target: JumpTarget::Label(Span::new(input, 137, 141).unwrap()),
        },
        ParsedInstruction::Complete(instr_i!(HALT, ZERO, ZERO, 0)),
    ];

    let expected_labels = hashmap![
        "loop" => 0,
        "end" => 6
    ];

    let pair = parse_rule(Rule::instructions, input).unwrap();
    let (instr, labels, _) = process_instructions(pair, &HashMap::new(), 0).unwrap();

    assert_eq!(instr, expected_instr);
    assert_eq!(labels, expected_labels);
}

fn transmute_vec(vec: Vec<Word>) -> Vec<u8> {
    let mut byte_vec = vec![0; vec.len() * WORD_BYTES as usize];
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
        instr_i!(SLTI, T2, T0, 32),
        instr_i!(BEZ, ZERO, T2, jmp_addr_i16(5)),
        instr_i!(SLLI, T1, T0, 2),
        instr_i!(SW, T0, T1, 0),
        instr_i!(ADDI, T0, T0, 1),
        instr_j!(JMP, jmp_addr_i32(-5)),
        instr_i!(HALT, ZERO, ZERO, 0),
    ]);

    let (executable, source_map) = assemble(input).unwrap();

    assert_eq!(
        executable,
        Executable::from(0u32, expected_instr, expected_data)
    );
    assert_eq!(
        source_map,
        vec![
            SourceMapItem {
                start_line: 4,
                line_count: 1
            },
            SourceMapItem {
                start_line: 5,
                line_count: 1
            },
            SourceMapItem {
                start_line: 6,
                line_count: 1
            },
            SourceMapItem {
                start_line: 7,
                line_count: 1
            },
            SourceMapItem {
                start_line: 8,
                line_count: 1
            },
            SourceMapItem {
                start_line: 9,
                line_count: 1
            },
            SourceMapItem {
                start_line: 10,
                line_count: 1
            },
        ]
    );
}

#[test]
fn single_line_multiple_instructions() {
    let input = ".data
stuff: .block 16
.instructions
LDA $T0, stuff
HALT
";
    let (_, source_map) = assemble(input).unwrap();

    assert_eq!(
        source_map,
        vec![
            SourceMapItem {
                start_line: 4,
                line_count: 1,
            },
            SourceMapItem {
                start_line: 4,
                line_count: 1,
            },
            SourceMapItem {
                start_line: 5,
                line_count: 1,
            },
        ]
    );
}

#[test]
fn non_contiguous_instructions() {
    let input = ".data

.instructions
NOP
                label:
   NOP #comment

  NOP   NOP

            ADD
                    $T0,
          $T1,

            #hi
    $T2

 HALT
 
 #hi again";

    let (_, source_map) = assemble(input).unwrap();

    assert_eq!(
        source_map,
        vec![
            SourceMapItem {
                start_line: 4,
                line_count: 1,
            },
            SourceMapItem {
                start_line: 5,
                line_count: 2,
            },
            SourceMapItem {
                start_line: 8,
                line_count: 1,
            },
            SourceMapItem {
                start_line: 8,
                line_count: 1,
            },
            SourceMapItem {
                start_line: 10,
                line_count: 6,
            },
            SourceMapItem {
                start_line: 17,
                line_count: 1,
            },
        ]
    );
}

#[test]
fn unsigned_immediate() {
    let input = ".data
.instructions
SLTUI $T0, $T1, 62994
HALT";

    let expected_instr = transmute_vec(vec![
        instr_i!(SLTUI, T0, T1, 62994u16 as i16),
        instr_i!(HALT, ZERO, ZERO, 0),
    ]);

    let (executable, _) = assemble(input).unwrap();
    assert_eq!(executable.instructions(), &expected_instr[..]);
}

#[test]
fn unsigned_set_immediate() {
    let input = ".data
.instructions
SLO $T0, 62994
SHI $T0, 62994
HALT";

    let expected_instr = transmute_vec(vec![
        instr_i!(SLO, T0, ZERO, 62994u16 as i16),
        instr_i!(SHI, T0, ZERO, 62994u16 as i16),
        instr_i!(HALT, ZERO, ZERO, 0),
    ]);

    let (executable, _) = assemble(input).unwrap();
    assert_eq!(executable.instructions(), &expected_instr[..]);
}

#[test]
fn macro_push() {
    let input = ".data
.instructions
PUSH $T6
HALT";

    let expected_instr = transmute_vec(vec![
        instr_i!(SW, T6, SP, -4),
        instr_i!(SUBI, SP, SP, 4),
        instr_i!(HALT, ZERO, ZERO, 0),
    ]);

    let (executable, _) = assemble(input).unwrap();
    assert_eq!(executable.instructions(), &expected_instr[..]);
}

#[test]
fn macro_pop() {
    let input = ".data
.instructions
POP $T6
HALT";

    let expected_instr = transmute_vec(vec![
        instr_i!(LW, T6, SP, 0),
        instr_i!(ADDI, SP, SP, 4),
        instr_i!(HALT, ZERO, ZERO, 0),
    ]);

    let (executable, _) = assemble(input).unwrap();
    assert_eq!(executable.instructions(), &expected_instr[..]);
}

// TODO: add loading macro optimizations (only use one instruction when possible)

#[test]
fn macro_lwi_signed() {
    let input = ".data
.instructions
LWI $T4, -2634987
HALT";

    let expected_instr = transmute_vec(vec![
        instr_i!(SLO, T4, ZERO, 0xCB15u16 as i16),
        instr_i!(SHI, T4, ZERO, 0xFFD7u16 as i16),
        instr_i!(HALT, ZERO, ZERO, 0),
    ]);

    let (executable, _) = assemble(input).unwrap();
    assert_eq!(executable.instructions(), &expected_instr[..]);
}

// TODO: fix this test (requires messing with integer parsing)
// #[test]
// fn macro_lwi_unsigned() {
//     let input = ".data
// .instructions
// LWI $T4, 3310837087u
// HALT";

//     let expected_instr = transmute_vec(vec![
//         instr_i!(SLO, T4, ZERO, 0x5D5Fu16 as i16),
//         instr_i!(SHI, T4, ZERO, 0xC557u16 as i16),
//         instr_i!(HALT, ZERO, ZERO, 0),
//     ]);

//     let (executable, _) = assemble(input).unwrap();
//     assert_eq!(executable.instructions(), &expected_instr[..]);
// }

#[test]
fn macro_lda() {
    let input = ".data
.block 64
stuff: .word 1234
.instructions
LDA $T4, stuff
HALT";

    let expected_instr = transmute_vec(vec![
        instr_i!(SLO, T4, ZERO, 64),
        instr_i!(SHI, T4, ZERO, 0),
        instr_i!(HALT, ZERO, ZERO, 0),
    ]);

    let (executable, _) = assemble(input).unwrap();
    assert_eq!(executable.instructions(), &expected_instr[..]);
}

#[test]
fn macro_lia() {
    let input = ".data
.instructions
LIA $T4, stuff
NOP
stuff: NOP
HALT";

    let expected_instr = transmute_vec(vec![
        instr_i!(SLO, T4, ZERO, 12),
        instr_i!(SHI, T4, ZERO, 0),
        nop!(),
        nop!(),
        instr_i!(HALT, ZERO, ZERO, 0),
    ]);

    let (executable, _) = assemble(input).unwrap();
    assert_eq!(executable.instructions(), &expected_instr[..]);
}
