use super::*;

#[test]
fn not_less_false() {
    instruction_runs! {
        instr_alu!(SLT, T0, T1, T2),
        [T0 = 0xFFFF_FFFFu32, T1 = 876_234, T2 = 123_843] => [T0 = 0]
    }
}

#[test]
fn equal_false() {
    instruction_runs! {
        instr_alu!(SLT, T0, T1, T2),
        [T0 = 0xFFFF_FFFFu32, T1 = 876_234, T2 = 876_234] => [T0 = 0]
    }
}

#[test]
fn different_signs_false() {
    instruction_runs! {
        instr_alu!(SLT, T0, T1, T2),
        [T0 = 0xFFFF_FFFFu32, T1 = 876_234, T2 = -1] => [T0 = 0]
    }
}

#[test]
fn less_true() {
    instruction_runs! {
        instr_alu!(SLT, T0, T1, T2),
        [T0 = 0xFFFF_FFFFu32, T1 = 123_485, T2 = 887_367] => [T0 = 1]
    }
}

#[test]
fn different_signs_true() {
    instruction_runs! {
        instr_alu!(SLT, T0, T1, T2),
        [T0 = 0xFFFF_FFFFu32, T1 = -1, T2 = 887_367] => [T0 = 1]
    }
}
