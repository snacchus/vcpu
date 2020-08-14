use super::*;

#[test]
fn positive() {
    instruction_runs! {
        instr_i!(LI, T0, ZERO, 0x1234),
        [] => [T0 = 0x0000_1234]
    };
}

#[test]
fn negative() {
    instruction_runs! {
        instr_i!(LI, T0, ZERO, -742),
        [] => [T0 = -742]
    };
}

#[test]
fn overrides_all_bits_no_sign_bit() {
    instruction_runs! {
        instr_i!(LI, T0, ZERO, 0x4321),
        [T0 = 0x1234_5678] => [T0 = 0x0000_4321]
    };
}

#[test]
fn overrides_all_bits_sign_bit() {
    instruction_runs! {
        instr_i!(LI, T0, ZERO, 0xF321u16 as i16),
        [T0 = 0x1234_5678] => [T0 = 0xFFFF_F321u32]
    };
}
