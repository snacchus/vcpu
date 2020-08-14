use super::*;

#[test]
fn positive() {
    instruction_runs! {
        instr_i!(MULI, T0, T1, 2873),
        [T1 = 8345] => [T0 = 23_975_185, RM = 0]
    };
}

#[test]
fn negative_immediate() {
    instruction_runs! {
        instr_i!(MULI, T0, T1, -2873),
        [T1 = 8345] => [T0 = -23_975_185, RM = -1]
    };
}

#[test]
fn negative_rs1() {
    instruction_runs! {
        instr_i!(MULI, T0, T1, 2873),
        [T1 = -8345] => [T0 = -23_975_185, RM = -1]
    };
}

#[test]
fn overflow_positive() {
    instruction_runs! {
        instr_i!(MULI, T0, T1, i16::MAX),
        [T1 = 0x12_345_678] => [T0 = 0x1907_A988, RM = 0x91A]
    };
}

#[test]
fn overflow_negative() {
    instruction_runs! {
        instr_i!(MULI, T0, T1, i16::MIN),
        [T1 = 1_234_567] => [T0 = 0x94BC_8000u32, RM = 0xFFFF_FFF6u32]
    };
}

#[test]
fn no_overflow_overwrite_rm_positive() {
    instruction_runs! {
        instr_i!(MULI, T0, T1, 10),
        [T1 = 10, RM = 0x1234_5678] => [T0 = 100, RM = 0]
    };
}

#[test]
fn no_overflow_overwrite_rm_negative() {
    instruction_runs! {
        instr_i!(MULI, T0, T1, -10),
        [T1 = 10, RM = 0x1234_5678] => [T0 = -100, RM = -1]
    };
}
