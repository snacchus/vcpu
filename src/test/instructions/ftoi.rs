use super::*;

#[test]
fn zero() {
    instruction_runs! {
        instr_i!(FTOI, T0, T1, 0),
        [T1 = 0f32] => [T0 = 0]
    }
}

#[test]
fn whole_positive() {
    instruction_runs! {
        instr_i!(FTOI, T0, T1, 0),
        [T1 = 56f32] => [T0 = 56]
    }
}

#[test]
fn whole_negative() {
    instruction_runs! {
        instr_i!(FTOI, T0, T1, 0),
        [T1 = -56f32] => [T0 = -56]
    }
}

#[test]
fn real_positive() {
    instruction_runs! {
        instr_i!(FTOI, T0, T1, 0),
        [T1 = 87455.88f32] => [T0 = 87455]
    }
}

#[test]
fn real_negative() {
    instruction_runs! {
        instr_i!(FTOI, T0, T1, 0),
        [T1 = -2347.31f32] => [T0 = -2347]
    }
}

#[test]
fn small_positive() {
    instruction_runs! {
        instr_i!(FTOI, T0, T1, 0),
        [T1 = 0.000_452f32] => [T0 = 0]
    }
}

#[test]
fn small_negative() {
    instruction_runs! {
        instr_i!(FTOI, T0, T1, 0),
        [T1 = -0.000_31f32] => [T0 = 0]
    }
}

// Converting non-finite floats to integers is technically undefined behavior,
// but just assume it produces i32::MIN here. It these tests ever fail,
// go back to the implementation and handle these cases manually.

#[test]
fn nan() {
    instruction_runs! {
        instr_i!(FTOI, T0, T1, 0),
        [T1 = f32::NAN] => [T0 = i32::MIN]
    }
}

#[test]
fn positive_infinity() {
    instruction_runs! {
        instr_i!(FTOI, T0, T1, 0),
        [T1 = f32::INFINITY] => [T0 = i32::MIN]
    }
}

#[test]
fn negative_infinity() {
    instruction_runs! {
        instr_i!(FTOI, T0, T1, 0),
        [T1 = f32::NEG_INFINITY] => [T0 = i32::MIN]
    }
}
