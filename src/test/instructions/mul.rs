use super::*;

#[test]
fn positive() {
    instruction_runs! {
        instr_alu!(MUL, T0, T1, T2),
        [T1 = 8345, T2 = 2873] => [T0 = 23_975_185, RM = 0]
    };
}

#[test]
fn negative_rs2() {
    instruction_runs! {
        instr_alu!(MUL, T0, T1, T2),
        [T1 = 8345, T2 = -2873] => [T0 = -23_975_185, RM = -1]
    };
}

#[test]
fn negative_rs1() {
    instruction_runs! {
        instr_alu!(MUL, T0, T1, T2),
        [T1 = -8345, T2 = 2873] => [T0 = -23_975_185, RM = -1]
    };
}

#[test]
fn overflow_positive() {
    instruction_runs! {
        instr_alu!(MUL, T0, T1, T2),
        [T1 = 0x1234_5678, T2 = 0x1234_5678] => [T0 = 0x1DF4_D840, RM = 0x14B_66DC]
    };
}

#[test]
fn overflow_negative() {
    instruction_runs! {
        instr_alu!(MUL, T0, T1, T2),
        [T1 = 1_234_567, T2 = -1_234_567] => [T0 = 0x213E_04CF, RM = 0xFFFF_FE9Du32]
    };
}

#[test]
fn no_overflow_overwrite_rm_positive() {
    instruction_runs! {
        instr_alu!(MUL, T0, T1, T2),
        [T1 = 10, T2 = 10, RM = 0x1234_5678] => [T0 = 100, RM = 0]
    };
}

#[test]
fn no_overflow_overwrite_rm_negative() {
    instruction_runs! {
        instr_alu!(MUL, T0, T1, T2),
        [T1 = 10, T2 = -10, RM = 0x1234_5678] => [T0 = -100, RM = -1]
    };
}
