use super::*;

#[test]
fn negative() {
    instructions_execute! {
        [
            nop!(),
            nop!(),
            nop!(),
            nop!(),
            nop!(),
            nop!(),
            nop!(),
            instr_j!(JL, -20),
            nop!(),
        ],
        [] => [RA = 32],
        empty_storage!() => empty_storage!(),
        8,
        None,
        8
    }
}

#[test]
fn positive() {
    instructions_execute! {
        [
            nop!(),
            nop!(),
            instr_j!(JL, 16),
            nop!(),
            nop!(),
            nop!(),
            nop!(),
            nop!(),
            nop!(),
        ],
        [] => [RA = 12],
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
            instr_j!(JL, 14),
            nop!(),
            nop!(),
            nop!(),
            nop!(),
            nop!(),
            nop!(),
        ],
        [] => [],
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
            instr_j!(JL, 44),
            nop!(),
            nop!(),
            nop!(),
            nop!(),
            nop!(),
            nop!(),
        ],
        [] => [],
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
            instr_j!(JL, -12),
            nop!(),
            nop!(),
            nop!(),
            nop!(),
            nop!(),
            nop!(),
        ],
        [] => [],
        empty_storage!() => empty_storage!(),
        3,
        Some(ExitCode::BadJump),
        8
    }
}
