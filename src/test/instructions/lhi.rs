use super::*;

#[test]
fn works() {
    instruction_runs! {
        instr_i!(LHI, T0, ZERO, 0x1234),
        [] => [T0 = 0x1234_0000]
    };
}

#[test]
fn overrides_all_bits() {
    instruction_runs! {
        instr_i!(LHI, T0, ZERO, 0x4321),
        [T0 = 0x1234_5678] => [T0 = 0x4321_0000]
    };
}
