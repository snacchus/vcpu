use super::*;

#[test]
fn opcode() {
    instruction_exits! {
        0xFFFF_FFFFu32, [] => [], InvalidOpcode
    }
}

#[test]
fn alu_funct() {
    instruction_exits! {
        instr_r!(ALU, T0, T1, T2, 63), [] => [], InvalidOpcode
    }
}

#[test]
fn flop_funct() {
    instruction_exits! {
        instr_r!(FLOP, T0, T1, T2, 63), [] => [], InvalidOpcode
    }
}
