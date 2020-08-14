use super::*;

#[test]
fn sign_bit_not_set_positive_amount() {
    instruction_runs! {
        instr_alu!(SRA, T0, T1, T2),
        [
            T1 = 0b0101_0011_0010_0011_1111_0100_0110_1011_u32,
            T2 = 13
        ] => [
            T0 = 0b0000_0000_0000_0010_1001_1001_0001_1111_u32
        ]
    }
}

#[test]
fn sign_bit_set_positive_amount() {
    instruction_runs! {
        instr_alu!(SRA, T0, T1, T2),
        [
            T1 = 0b1101_0011_0010_0011_1111_0100_0110_1011_u32,
            T2 = 13
        ] => [
            T0 = 0b1111_1111_1111_1110_1001_1001_0001_1111_u32
        ]
    }
}

#[test]
fn sign_bit_not_set_negative_amount() {
    instruction_runs! {
        instr_alu!(SRA, T0, T1, T2),
        [
            T1 = 0b0101_0011_0010_0011_1111_0100_0110_1011_u32,
            T2 = -6
        ] => [
            T0 = 0b0000_0000_0000_0000_0000_0000_0001_0100_u32
        ]
    }
}

#[test]
fn sign_bit_set_negative_amount() {
    instruction_runs! {
        instr_alu!(SRA, T0, T1, T2),
        [
            T1 = 0b1101_0011_0010_0011_1111_0100_0110_1011_u32,
            T2 = -6
        ] => [
            T0 = 0b1111_1111_1111_1111_1111_1111_1111_0100_u32
        ]
    }
}
