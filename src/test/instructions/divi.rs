use super::*;

#[test]
fn positive() {
    instruction_runs! {
        instr_i!(DIVI, T0, T1, 37),
        [T1 = 2096] => [T0 = 56, RM = 24]
    }
}

#[test]
fn negative_immediate() {
    instruction_runs! {
        instr_i!(DIVI, T0, T1, -37),
        [T1 = 2096] => [T0 = -56, RM = 24]
    }
}

#[test]
fn negative_rs1() {
    instruction_runs! {
        instr_i!(DIVI, T0, T1, 37),
        [T1 = -2096] => [T0 = -56, RM = -24]
    }
}

#[test]
fn no_remainder_override_rm() {
    instruction_runs! {
        instr_i!(DIVI, T0, T1, 37),
        [T1 = 2072, RM = 0x8127_6634u32] => [T0 = 56, RM = 0]
    }
}

#[test]
fn by_zero() {
    instruction_exits! {
        instr_i!(DIVI, T0, T1, 0),
        [T1 = 2072, RM = 0x8127_6634u32] => [],
        DivisionByZero
    }
}
