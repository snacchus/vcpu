use super::*;

#[test]
fn success() {
    instruction_runs! {
        instr_i!(LH, T0, T1, 0),
        [T1 = 2] => [T0 = 0xFFFE],
        [0xFF, 0xFF, 0xFE, 0xFF] => [0xFF, 0xFF, 0xFE, 0xFF]
    };
}

#[test]
fn bad_access() {
    instruction_exits! {
        instr_i!(LH, T0, T1, 34),
        [T1 = 2] => [],
        [0xFF, 0xFF, 0xFE, 0xFF] => [0xFF, 0xFF, 0xFE, 0xFF],
        BadMemoryAccess
    };
}
