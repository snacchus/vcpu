use super::*;

#[test]
fn equal_false() {
    instruction_runs! {
        instr_alu!(SNE, T0, T1, T2),
        [T0 = 0xFFFF_FFFFu32, T1 = 876_234, T2 = 876_234] => [T0 = 0]
    }
}

#[test]
fn not_equal_true() {
    instruction_runs! {
        instr_alu!(SNE, T0, T1, T2),
        [T0 = 0xFFFF_FFFFu32, T1 = 876_234, T2 = 123_485] => [T0 = 1]
    }
}
