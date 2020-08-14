use super::*;

#[test]
fn success() {
    instruction_runs! {
        instr_i!(SB, T0, T1, 3),
        [T0 = 0x1234_5678u32, T1 = 3] => [],
        [0u8; 8] => [0, 0, 0, 0, 0, 0, 0x78u8, 0]
    };
}

#[test]
fn bad_access() {
    instruction_exits! {
        instr_i!(SB, T0, T1, 5),
        [T0 = 0x1234_5678u32, T1 = 3] => [],
        [0u8; 8] => [0u8; 8],
        BadMemoryAccess
    };
}
