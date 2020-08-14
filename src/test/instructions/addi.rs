use super::*;

#[test]
fn positive() {
    instruction_runs! {
        instr_i!(ADDI, T0, T1, 1234),
        [T1 = 5678] => [T0 = 6912]
    };
}

#[test]
fn negative_immediate() {
    instruction_runs! {
        instr_i!(ADDI, T0, T1, -1234),
        [T1 = 5678] => [T0 = 4444]
    };
}

#[test]
fn negative_rs1() {
    instruction_runs! {
        instr_i!(ADDI, T0, T1, 1234),
        [T1 = -5678] => [T0 = -4444]
    };
}

#[test]
fn overflow() {
    instruction_runs! {
        instr_i!(ADDI, T0, T1, i16::MAX),
        [T1 = 0xFFFF_FFFFu32] => [T0 = 0x0000_7FFEu32]
    };
}
