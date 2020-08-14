use super::*;

#[test]
fn not_greater_false() {
    instruction_runs! {
        instr_i!(SGEUI, T0, T1, 6_234),
        [T0 = 0xFFFF_FFFFu32, T1 = 3_843] => [T0 = 0]
    }
}

#[test]
fn equal_true() {
    instruction_runs! {
        instr_i!(SGEUI, T0, T1, 6_234),
        [T0 = 0xFFFF_FFFFu32, T1 = 6_234] => [T0 = 1]
    }
}

#[test]
fn does_not_extend_sign() {
    instruction_runs! {
        instr_i!(SGEUI, T0, T1, 65535_u16 as i16),
        [T0 = 0xFFFF_FFFFu32, T1 = 887_367] => [T0 = 1]
    }
}

#[test]
fn greater_true() {
    instruction_runs! {
        instr_i!(SGEUI, T0, T1, 3_485),
        [T0 = 0xFFFF_FFFFu32, T1 = 7_367] => [T0 = 1]
    }
}

#[test]
fn uses_unsigned_arithmetic() {
    instruction_runs! {
        instr_i!(SGEUI, T0, T1, 7_367),
        [T0 = 0xFFFF_FFFFu32, T1 = -1] => [T0 = 1]
    }
}
