use super::*;

#[test]
fn works() {
    instruction_runs! {
        instr_i!(FLIP, T0, T1, 0),
        [
            T1 = 0b0101_0011_0010_0011_1111_0100_0110_1011_u32
        ] => [
            T0 = 0b1010_1100_1101_1100_0000_1011_1001_0100_u32
        ]
    }
}