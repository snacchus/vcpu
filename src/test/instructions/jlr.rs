use super::*;

#[test]
fn success() {
    instructions_execute! {
        [
            nop!(),
            nop!(),
            instr_i!(JLR, ZERO, T0, 0),
            nop!(),
            nop!(),
            nop!(),
            nop!(),
            nop!(),
            nop!(),
        ],
        [T0 = 32] => [RA = 12],
        empty_storage!() => empty_storage!(),
        3,
        None,
        32
    }
}

#[test]
fn bad_alignment() {
    instructions_execute! {
        [
            nop!(),
            nop!(),
            instr_i!(JLR, ZERO, T0, 0),
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
            instr_i!(JLR, ZERO, T0, 0),
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
