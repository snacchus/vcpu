use super::*;

#[test]
fn no_sign_bit() {
    instruction_runs! {
        instr_i!(SHI, T0, ZERO, 0x1234),
        [] => [T0 = 0x1234_0000]
    };
}

#[test]
fn does_not_extend_sign() {
    instruction_runs! {
        instr_i!(SHI, T0, ZERO, -1),
        [] => [T0 = 0xFFFF_0000]
    };
}

#[test]
fn does_not_overwrite_high_bits_no_sign_bit() {
    instruction_runs! {
        instr_i!(SHI, T0, ZERO, 0x4321),
        [T0 = 0x1234_5678] => [T0 = 0x4321_5678]
    };
}

#[test]
fn does_not_overwrite_high_bits_sign_bit() {
    instruction_runs! {
        instr_i!(SHI, T0, ZERO, 0xF321),
        [T0 = 0x1234_5678] => [T0 = 0xF321_5678]
    };
}
