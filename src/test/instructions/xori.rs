use super::*;

#[test]
fn positive_immediate() {
    instruction_runs! {
        instr_i!(XORI, T0, T1, 0b0111_1101_1010_0111u16 as i16),
        [
            T1 = 0b0101_0011_0010_0011_1111_0100_0110_1011_u32
        ] => [
            T0 = 0b0101_0011_0010_0011_1000_1001_1100_1100_u32
        ]
    }
}

#[test]
fn negative_immediate() {
    instruction_runs! {
        instr_i!(XORI, T0, T1, 0b1111_1101_1010_0111u16 as i16),
        [
            T1 = 0b0101_0011_0010_0011_1111_0100_0110_1011_u32
        ] => [
            T0 = 0b1010_1100_1101_1100_0000_1001_1100_1100_u32
        ]
    }
}
