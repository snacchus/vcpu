use super::*;

#[test]
fn not_greater_false() {
    instruction_runs! {
        instr_alu!(SGEU, T0, T1, T2),
        [T0 = 0xFFFF_FFFFu32, T1 = 123_843, T2 = 876_234] => [T0 = 0]
    }
}

#[test]
fn equal_true() {
    instruction_runs! {
        instr_alu!(SGEU, T0, T1, T2),
        [T0 = 0xFFFF_FFFFu32, T1 = 876_234, T2 = 876_234] => [T0 = 1]
    }
}

#[test]
fn different_signs_false() {
    instruction_runs! {
        instr_alu!(SGEU, T0, T1, T2),
        [T0 = 0xFFFF_FFFFu32, T1 = 876_234, T2 = -1] => [T0 = 0]
    }
}

#[test]
fn greater_true() {
    instruction_runs! {
        instr_alu!(SGEU, T0, T1, T2),
        [T0 = 0xFFFF_FFFFu32, T1 = 887_367, T2 = 123_485] => [T0 = 1]
    }
}

#[test]
fn different_signs_true() {
    instruction_runs! {
        instr_alu!(SGEU, T0, T1, T2),
        [T0 = 0xFFFF_FFFFu32, T1 = -1, T2 = 887_367] => [T0 = 1]
    }
}
