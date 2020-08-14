use super::*;

#[test]
fn success() {
    instruction_runs! {
        instr_i!(SW, T0, T1, 2),
        [T0 = 0x1234_5678u32, T1 = 1] => [],
        [0u8; 8] => [0, 0, 0, 0x78, 0x56, 0x34, 0x12, 0]
    };
}

#[test]
fn bad_access() {
    instruction_exits! {
        instr_i!(SW, T0, T1, 20),
        [T0 = 0x1234_5678u32] => [],
        [0u8; 8] => [0u8; 8],
        BadMemoryAccess
    };
}
