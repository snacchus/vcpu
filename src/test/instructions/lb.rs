use super::*;

#[test]
fn success() {
    instruction_runs! {
        instr_i!(LB, T0, T1, -1),
        [T1 = 3] => [T0 = 0xFE],
        [0xFF, 0xFF, 0xFE, 0xFF] => [0xFF, 0xFF, 0xFE, 0xFF]
    };
}

#[test]
fn bad_access() {
    instruction_exits! {
        instr_i!(LB, T0, T1, 0),
        [T1 = -4] => [],
        [0xFF, 0xFF, 0xFE, 0xFF] => [0xFF, 0xFF, 0xFE, 0xFF],
        BadMemoryAccess
    };
}
