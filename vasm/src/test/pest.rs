use crate::{ Rule, VASMParser };
use ::pest::*;

#[test]
fn comment() {
    parses_to! {
        parser: VASMParser,
        input: "# oai0ß9jqp4o5gm66185 dA';:%",
        rule: Rule::comment,
        tokens: []
    };

    parses_to! {
        parser: VASMParser,
        input: "#balbalao40j3\n ughalsdhgf",
        rule: Rule::comment,
        tokens: []
    };
}

#[test]
fn dec_uint() {
    parses_to! {
        parser: VASMParser,
        input: "4492",
        rule: Rule::dec_uint,
        tokens: [dec_uint(0, 4)]
    };
}

#[test]
fn bin_uint() {
    parses_to! {
        parser: VASMParser,
        input: "0b01011",
        rule: Rule::bin_uint,
        tokens: [bin_uint(0, 7, [ bin_lit(2, 7) ])]
    };
}

#[test]
fn oct_uint() {
    parses_to! {
        parser: VASMParser,
        input: "0o22406",
        rule: Rule::oct_uint,
        tokens: [oct_uint(0, 7, [ oct_lit(2, 7) ])]
    };
}

#[test]
fn hex_uint() {
    parses_to! {
        parser: VASMParser,
        input: "0xF40a67",
        rule: Rule::hex_uint,
        tokens: [hex_uint(0, 8, [ hex_lit(2, 8) ])]
    };
}

#[test]
fn identifier() {
    parses_to! {
        parser: VASMParser,
        input: "some label",
        rule: Rule::identifier,
        tokens: [ identifier(0, 4) ]
    };
    parses_to! {
        parser: VASMParser,
        input: "_soGe56abel",
        rule: Rule::identifier,
        tokens: [ identifier(0, 11) ]
    };
    fails_with! {
        parser: VASMParser,
        input: "555___456sd",
        rule: Rule::identifier,
        positives: vec![Rule::identifier],
        negatives: vec![],
        pos: 0
    };
}

#[test]
fn data_block() {
    parses_to! {
        parser: VASMParser,
        input: ".block   0x16",
        rule: Rule::data_block,
        tokens: [ data_block(0, 13, [
                uint(9, 13, [ hex_uint(9, 13, [ hex_lit(11, 13) ]) ])
        ]) ]
    };
    fails_with! {
        parser: VASMParser,
        input: ".block -45",
        rule: Rule::data_block,
        positives: vec![Rule::uint],
        negatives: vec![],
        pos: 7
    };
    fails_with! {
        parser: VASMParser,
        input: ".block5",
        rule: Rule::data_block,
        positives: vec![Rule::data_block],
        negatives: vec![],
        pos: 0
    };
}

#[test]
fn data_byte() {
    parses_to! {
        parser: VASMParser,
        input: ".byte 34, -4,  0b001,0xFFa4",
        rule: Rule::data_byte,
        tokens: [ data_byte(0, 27, [ int_list(6, 27, [
            int(6, 8, [ dec_int(6, 8) ]),
            int(10, 12, [ dec_int(10, 12) ]),
            int(15, 20, [ bin_uint(15, 20, [ bin_lit(17, 20) ]) ]),
            int(21, 27, [ hex_uint(21, 27, [ hex_lit(23, 27) ]) ])
        ]) ]) ]
    };
    fails_with! {
        parser: VASMParser,
        input: ".byte",
        rule: Rule::data_byte,
        positives: vec![Rule::data_byte],
        negatives: vec![],
        pos: 0
    };
    fails_with! {
        parser: VASMParser,
        input: ".byte34,22,9",
        rule: Rule::data_byte,
        positives: vec![Rule::data_byte],
        negatives: vec![],
        pos: 0
    };
}

#[test]
fn labeled_data_element() {
    parses_to! {
        parser: VASMParser,
        input: "__something: .short 4",
        rule: Rule::labeled_data_element,
        tokens: [ labeled_data_element(0, 21, [
            label(0, 12, [ identifier(0, 11) ]),
            data_element(13, 21, [ data_short(13, 21, [ int_list(20, 21, [
                int(20, 21, [ dec_int(20, 21) ])
            ]) ]) ])
        ]) ]
    };
    parses_to! {
        parser: VASMParser,
        input: "label4  :.word 0b11,  0x0Aa",
        rule: Rule::labeled_data_element,
        tokens: [ labeled_data_element(0, 27, [
            label(0, 9, [ identifier(0, 6) ]),
            data_element(9, 27, [ data_word(9, 27, [ int_list(15, 27, [
                int(15, 19, [ bin_uint(15, 19, [ bin_lit(17, 19) ]) ]),
                int(22, 27, [ hex_uint(22, 27, [ hex_lit(24, 27) ]) ])
            ]) ]) ])
        ]) ]
    };
    parses_to! {
        parser: VASMParser,
        input: ".block 0b110",
        rule: Rule::labeled_data_element,
        tokens: [ labeled_data_element(0, 12, [
            data_element(0, 12, [ data_block(0, 12, [
                uint(7, 12, [ bin_uint(7, 12, [ bin_lit(9, 12) ]) ])
            ]) ])
        ]) ]
    };
}

#[test]
fn data() {
    parses_to! {
        parser: VASMParser,
        input: ".data .block 128",
        rule: Rule::data,
        tokens: [ data(0, 16, [
            labeled_data_element(6, 16, [
                data_element(6, 16, [ data_block(6, 16, [
                    uint(13, 16, [ dec_uint(13, 16) ])
                ]) ])
            ])
        ]) ]
    };
    fails_with! {
        parser: VASMParser,
        input: ".data.byte 3",
        rule: Rule::data,
        positives: vec![Rule::data],
        negatives: vec![],
        pos: 0
    };
}

#[test]
fn register() {
    parses_to! {
        parser: VASMParser,
        input: "$t0",
        rule: Rule::register,
        tokens: [ register(0, 3, [register_id(1, 3)]) ]
    };

    fails_with! {
        parser: VASMParser,
        input: "$bla",
        rule: Rule::register,
        positives: vec![Rule::register_id],
        negatives: vec![],
        pos: 1
    };

    fails_with! {
        parser: VASMParser,
        input: "$ t0",
        rule: Rule::register,
        positives: vec![Rule::register_id],
        negatives: vec![],
        pos: 1
    };
}

#[test]
fn instruction_alu() {
    parses_to! {
        parser: VASMParser,
        input: "XOR $s0,   $V1,$Rm",
        rule: Rule::instruction_alu,
        tokens: [ instruction_alu(0, 18, [
            opcode_alu_sep(0, 4, [ opcode_alu(0, 3) ]),
            register(4, 7, [ register_id(5, 7) ]),
            register(11, 14, [ register_id(12, 14) ]),
            register(15, 18, [ register_id(16, 18) ])
        ]) ]
    };
}

#[test]
fn instruction_i() {
    parses_to! {
        parser: VASMParser,
        input: "SLTI  $zErO,$RA ,-443",
        rule: Rule::instruction_i,
        tokens: [ instruction_i(0, 21, [ 
            opcode_i_sep(0, 6, [ opcode_i(0, 4) ]),
            register(6, 11, [ register_id(7, 11) ]),
            register(12, 15, [ register_id(13, 15) ]),
            int(17, 21, [ dec_int(17, 21) ])
        ]) ]
    };
}

#[test]
fn instruction_ds() {
    parses_to! {
        parser: VASMParser,
        input: "fLIp $ZERO , $rM",
        rule: Rule::instruction_ds,
        tokens: [ instruction_ds(0, 16, [
            opcode_ds_sep(0, 5, [ opcode_ds(0, 4) ]),
            register(5, 10, [ register_id(6, 10) ]),
            register(13, 16, [ register_id(14, 16) ])
        ]) ]
    };
}

#[test]
fn instruction_li() {
    parses_to! {
        parser: VASMParser,
        input: "LHI $T5 ,0o442",
        rule: Rule::instruction_li,
        tokens: [ instruction_li(0, 14, [
            opcode_li_sep(0, 4, [ opcode_li(0, 3) ]),
            register(4, 7, [ register_id(5, 7) ]),
            int(9, 14, [ oct_uint(9, 14, [ oct_lit(11, 14) ]) ])
        ]) ]
    };
}

#[test]
fn instruction_la() {
    parses_to! {
        parser: VASMParser,
        input: "la $s4, f_44ash__0",
        rule: Rule::instruction_la,
        tokens: [ instruction_la(0, 18, [
            opcode_la_sep(0, 3, [ opcode_la(0, 2) ]),
            register(3, 6, [ register_id(4, 6) ]),
            identifier(8, 18)
        ]) ]
    };
}

#[test]
fn instruction_e() {
    parses_to! {
        parser: VASMParser,
        input: "HaLT",
        rule: Rule::instruction_e,
        tokens: [ instruction_e(0, 4, [ opcode_e(0, 4) ]) ]
    };
}

#[test]
fn instruction_br() {
    parses_to! {
        parser: VASMParser,
        input: "BEZ $t6, 0xd1",
        rule: Rule::instruction_br,
        tokens: [ instruction_br(0, 13, [
            opcode_br_sep(0, 4, [ opcode_br(0, 3) ]),
            register(4, 7, [ register_id(5, 7) ]),
            jump_target(9, 13, [ int(9, 13, [ hex_uint(9, 13, [ hex_lit(11, 13) ]) ]) ])
        ]) ]
    };
    parses_to! {
        parser: VASMParser,
        input: "BnZ  $RM ,_55_arFd",
        rule: Rule::instruction_br,
        tokens: [ instruction_br(0, 18, [
            opcode_br_sep(0, 5, [ opcode_br(0, 3) ]),
            register(5, 8, [ register_id(6, 8) ]),
            jump_target(10, 18, [ identifier(10, 18) ])
        ]) ]
    };
}

#[test]
fn instruction_jr() {
    parses_to! {
        parser: VASMParser,
        input: "JLR   $zero",
        rule: Rule::instruction_jr,
        tokens: [ instruction_jr(0, 11, [
            opcode_jr_sep(0, 6, [ opcode_jr(0, 3) ]),
            register(6, 11, [ register_id(7, 11) ])
        ]) ]
    };
}

#[test]
fn instruction_ls() {
    parses_to! {
        parser: VASMParser,
        input: "sw $V1, -92 ($s5 )",
        rule: Rule::instruction_ls,
        tokens: [ instruction_ls(0, 18, [
            opcode_ls_sep(0, 3, [ opcode_ls(0, 2) ]),
            register(3, 6, [ register_id(4, 6) ]),
            int(8, 11, [ dec_int(8, 11) ]),
            register(13, 16, [ register_id(14, 16) ])
        ]) ]
    };
}

#[test]
fn instruction_j() {
    parses_to! {
        parser: VASMParser,
        input: "JMP 0b110110",
        rule: Rule::instruction_j,
        tokens: [ instruction_j(0, 12, [
            opcode_j_sep(0, 4, [ opcode_j(0, 3) ]),
            jump_target(4, 12, [ int(4, 12, [ bin_uint(4, 12, [ bin_lit(6, 12) ]) ]) ])
        ]) ]
    };
    parses_to! {
        parser: VASMParser,
        input: "jl oGfe_A34_",
        rule: Rule::instruction_j,
        tokens: [ instruction_j(0, 12, [
            opcode_j_sep(0, 3, [ opcode_j(0, 2) ]),
            jump_target(3, 12, [ identifier(3, 12) ])
        ]) ]
    };
}

#[test]
fn instruction() {
    fails_with! {
        parser: VASMParser,
        input: "DIV$t3,$ZERO,$a0",
        rule: Rule::instruction,
        positives: vec![Rule::instruction],
        negatives: vec![],
        pos: 0
    };
}

#[test]
fn labeled_instruction() {
    parses_to! {
        parser: VASMParser,
        input: "__aaaF5_: # comment \n JR $ra",
        rule: Rule::labeled_instruction,
        tokens: [ labeled_instruction(0, 28, [
            label(0, 9, [ identifier(0, 8) ]),
            instruction(22, 28, [ instruction_jr(22, 28, [
                opcode_jr_sep(22, 25, [ opcode_jr(22, 24) ]),
                register(25, 28, [ register_id(26, 28) ])
            ]) ])
        ]) ]
    };
    parses_to! {
        parser: VASMParser,
        input: "HALT",
        rule: Rule::labeled_instruction,
        tokens: [ labeled_instruction(0, 4, [
            instruction(0, 4, [ instruction_e(0, 4, [ opcode_e(0, 4) ]) ])
        ]) ]
    };
}

#[test]
fn instructions() {
    parses_to! {
        parser: VASMParser,
        input: ".instructions HALT",
        rule: Rule::instructions,
        tokens: [ instructions(0, 18, [
            labeled_instruction(14, 18, [
                instruction(14, 18, [ instruction_e(14, 18, [ opcode_e(14, 18) ]) ])
            ])
        ]) ]
    };
}

#[test]
fn program() {
    let src =
"# .data bla
.data
label: .block 9
.instructions
# bla:
ADD $t0, $t1, $t2";

    parses_to! {
        parser: VASMParser,
        input: src,
        rule: Rule::program,
        tokens: [ program(0, 72, [
            data(12, 33, [
                labeled_data_element(18, 33, [
                    label(18, 24, [ identifier(18, 23) ]),
                    data_element(25, 33, [ data_block(25, 33, [ uint(32, 33, [ dec_uint(32, 33) ]) ]) ])
                ])
            ]),
            instructions(34, 72, [
                labeled_instruction(55, 72, [ instruction(55, 72, [ instruction_alu(55, 72, [
                    opcode_alu_sep(55, 59, [ opcode_alu(55, 58) ]),
                    register(59, 62, [ register_id(60, 62) ]),
                    register(64, 67, [ register_id(65, 67) ]),
                    register(69, 72, [ register_id(70, 72) ])
                ]) ]) ])
            ])
        ]) ]
    };
}
