use super::*;

#[test]
fn not_equal_false() {
    instruction_runs! {
        instr_i!(SEQI, T0, T1, 12_348),
        [T0 = 0xFFFF_FFFFu32, T1 = 876_234] => [T0 = 0]
    }
}

#[test]
fn equal_true() {
    instruction_runs! {
        instr_i!(SEQI, T0, T1, -25678),
        [T0 = 0xFFFF_FFFFu32, T1 = -25678] => [T0 = 1]
    }
}
