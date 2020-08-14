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
            instr_j!(JMP, -20),
            nop!(),
        ],
        [] => [],
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
            instr_j!(JMP, 16),
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
            instr_j!(JMP, 14),
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
            instr_j!(JMP, 44),
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
            instr_j!(JMP, -12),
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
