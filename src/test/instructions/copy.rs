use super::*;

#[test]
fn works() {
    instruction_runs! {
        instr_i!(COPY, T1, T0, 0),
        [T0 = 0x1234_5678] => [T1 = 0x1234_5678]
    };
}
