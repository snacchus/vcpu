use super::*;

#[test]
fn success() {
    instruction_runs! {
        instr_i!(SH, T0, T1, -6),
        [T0 = 0x1234_5678u32, T1 = 8] => [],
        [0u8; 8] => [0, 0, 0x78, 0x56, 0, 0, 0, 0]
    };
}

#[test]
fn bad_access() {
    instruction_exits! {
        instr_i!(SH, T0, T1, 0),
        [T0 = 0x1234_5678u32, T1 = 8] => [],
        [0u8; 8] => [0u8; 8],
        BadMemoryAccess
    };
}
