use super::*;

#[test]
fn positive() {
    instruction_runs! {
        instr_i!(SUBI, T0, T1, 1234),
        [T1 = 5678] => [T0 = 4444]
    };
}

#[test]
fn negative_immediate() {
    instruction_runs! {
        instr_i!(SUBI, T0, T1, -1234),
        [T1 = 5678] => [T0 = 6912]
    };
}

#[test]
fn negative_rs1() {
    instruction_runs! {
        instr_i!(SUBI, T0, T1, 1234),
        [T1 = -5678] => [T0 = -6912]
    };
}
