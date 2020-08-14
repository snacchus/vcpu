use super::*;

#[test]
fn success() {
    instruction_runs! {
        instr_i!(LW, T0, ZERO, 0),
        [] => [T0 = 0xFFFE_FFFFu32],
        [0xFF, 0xFF, 0xFE, 0xFF] => [0xFF, 0xFF, 0xFE, 0xFF]
    };
}

#[test]
fn bad_access() {
    instruction_exits! {
        instr_i!(LW, T0, ZERO, -2),
        [] => [],
        [0xFF, 0xFF, 0xFE, 0xFF] => [0xFF, 0xFF, 0xFE, 0xFF],
        BadMemoryAccess
    };
}
