use super::*;

#[test]
fn positive() {
    instruction_runs! {
        instr_alu!(DIV, T0, T1, T2),
        [T1 = 2096, T2 = 37] => [T0 = 56, RM = 24]
    }
}

#[test]
fn negative_rs2() {
    instruction_runs! {
        instr_alu!(DIV, T0, T1, T2),
        [T1 = 2096, T2 = -37] => [T0 = -56, RM = 24]
    }
}

#[test]
fn negative_rs1() {
    instruction_runs! {
        instr_alu!(DIV, T0, T1, T2),
        [T1 = -2096, T2 = 37] => [T0 = -56, RM = -24]
    }
}

#[test]
fn no_remainder_override_rm() {
    instruction_runs! {
        instr_alu!(DIV, T0, T1, T2),
        [T1 = 2072, T2 = 37, RM = 0x8127_6634u32] => [T0 = 56, RM = 0]
    }
}

#[test]
fn by_zero() {
    instruction_exits! {
        instr_alu!(DIV, T0, T1, T2),
        [T1 = 2072, T2 = 0, RM = 0x8127_6634u32] => [],
        DivisionByZero
    }
}
