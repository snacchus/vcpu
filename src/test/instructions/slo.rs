use super::*;

#[test]
fn no_sign_bit() {
    instruction_runs! {
        instr_i!(SLO, T0, ZERO, 0x1234),
        [] => [T0 = 0x0000_1234]
    };
}

#[test]
fn does_not_extend_sign() {
    instruction_runs! {
        instr_i!(SLO, T0, ZERO, -1),
        [] => [T0 = 0x0000_FFFF]
    };
}

#[test]
fn does_not_overwrite_high_bits_no_sign_bit() {
    instruction_runs! {
        instr_i!(SLO, T0, ZERO, 0x4321),
        [T0 = 0x1234_5678] => [T0 = 0x1234_4321]
    };
}

#[test]
fn does_not_overwrite_high_bits_sign_bit() {
    instruction_runs! {
        instr_i!(SLO, T0, ZERO, 0xF321u16 as i16),
        [T0 = 0x1234_5678] => [T0 = 0x1234_F321]
    };
}
