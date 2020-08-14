use super::*;

#[test]
fn zero() {
    instruction_runs! {
        instr_i!(ITOF, T0, T1, 0),
        [T1 = 0] => [T0 = 0f32]
    };
}

#[test]
fn positive() {
    instruction_runs! {
        instr_i!(ITOF, T0, T1, 0),
        [T1 = 234] => [T0 = 234f32]
    };
}

#[test]
fn negative() {
    instruction_runs! {
        instr_i!(ITOF, T0, T1, 0),
        [T1 = -2392] => [T0 = -2392f32]
    };
}

#[test]
fn max_value() {
    instruction_runs! {
        instr_i!(ITOF, T0, T1, 0),
        [T1 = i32::MAX] => [T0 = 2_147_483_647_f32]
    };
}

#[test]
fn min_value() {
    instruction_runs! {
        instr_i!(ITOF, T0, T1, 0),
        [T1 = i32::MIN] => [T0 = -2_147_483_648_f32]
    };
}
