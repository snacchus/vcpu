use super::*;

#[test]
fn success() {
    instructions_execute! {
        [
            nop!(),
            nop!(),
            instr_i!(JR, ZERO, T0, 0),
            nop!(),
            nop!(),
            nop!(),
            nop!(),
            nop!(),
            nop!(),
        ],
        [T0 = 24] => [],
        empty_storage!() => empty_storage!(),
        3,
        None,
        24
    }
}

#[test]
fn bad_alignment() {
    instructions_execute! {
        [
            nop!(),
            nop!(),
            instr_i!(JR, ZERO, T0, 0),
            nop!(),
            nop!(),
            nop!(),
            nop!(),
            nop!(),
            nop!(),
        ],
        [T0 = 14] => [],
        empty_storage!() => empty_storage!(),
        3,
        Some(ExitCode::BadAlignment),
        8
    }
}

#[test]
fn bad_jump_positive() {
    instructions_execute! {
        [
            nop!(),
            nop!(),
            instr_i!(JR, ZERO, T0, 0),
            nop!(),
            nop!(),
            nop!(),
            nop!(),
            nop!(),
            nop!(),
        ],
        [T0 = 44] => [],
        empty_storage!() => empty_storage!(),
        3,
        Some(ExitCode::BadJump),
        8
    }
}

#[test]
fn bad_jump_negative() {
    instructions_execute! {
        [
            nop!(),
            nop!(),
            instr_i!(JR, ZERO, T0, 0),
            nop!(),
            nop!(),
            nop!(),
            nop!(),
            nop!(),
            nop!(),
        ],
        [T0 = -12] => [],
        empty_storage!() => empty_storage!(),
        3,
        Some(ExitCode::BadJump),
        8
    }
}
