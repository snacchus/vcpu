use super::*;

#[test]
fn positive() {
    instruction_runs! {
        instr_alu!(SUB, T0, T1, T2),
        [T1 = 5678, T2 = 1234] => [T0 = 4444]
    };
}

#[test]
fn negative_rs2() {
    instruction_runs! {
        instr_alu!(SUB, T0, T1, T2),
        [T1 = 5678, T2 = -1234] => [T0 = 6912]
    };
}

#[test]
fn negative_rs1() {
    instruction_runs! {
        instr_alu!(SUB, T0, T1, T2),
        [T1 = -5678, T2 = 1234] => [T0 = -6912]
    };
}
