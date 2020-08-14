use super::*;

#[test]
fn positive_amount() {
    instruction_runs! {
        instr_alu!(SLL, T0, T1, T2),
        [
            T1 = 0b0101_0011_0010_0011_1111_0100_0110_1011_u32,
            T2 = 13
        ] => [
            T0 = 0b0111_1110_1000_1101_0110_0000_0000_0000_u32
        ]
    }
}

#[test]
fn negative_amount() {
    instruction_runs! {
        instr_alu!(SLL, T0, T1, T2),
        [
            T1 = 0b0101_0011_0010_0011_1111_0100_0110_1011_u32,
            T2 = -6
        ] => [
            T0 = 0b1010_1100_0000_0000_0000_0000_0000_0000_u32
        ]
    }
}
