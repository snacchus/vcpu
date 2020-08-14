use super::*;

#[test]
fn equal_false() {
    instruction_runs! {
        instr_i!(SNEI, T0, T1, -25678),
        [T0 = 0xFFFF_FFFFu32, T1 = -25678] => [T0 = 0]
    }
}

#[test]
fn not_equal_true() {
    instruction_runs! {
        instr_i!(SNEI, T0, T1, -1234),
        [T0 = 0xFFFF_FFFFu32, T1 = 876_234] => [T0 = 1]
    }
}
