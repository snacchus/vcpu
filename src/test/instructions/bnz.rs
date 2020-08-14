use super::*;

#[test]
fn branch_negative() {
    instructions_execute! {
        [
            nop!(),
            nop!(),
            nop!(),
            instr_i!(BNZ, ZERO, T0, -8),
            nop!(),
        ],
        [T0 = 6] => [],
        empty_storage!() => empty_storage!(),
        4,
        None,
        4
    }
}

#[test]
fn branch_positive() {
    instructions_execute! {
        [
            nop!(),
            instr_i!(BNZ, ZERO, T0, 12),
            nop!(),
            nop!(),
            nop!(),
        ],
        [T0 = 2] => [],
        empty_storage!() => empty_storage!(),
        2,
        None,
        16
    }
}

#[test]
fn no_branch_negative() {
    instructions_execute! {
        [
            nop!(),
            nop!(),
            nop!(),
            instr_i!(BNZ, ZERO, T0, -8),
            nop!(),
        ],
        [T0 = 0] => [],
        empty_storage!() => empty_storage!(),
        4,
        None,
        16
    }
}

#[test]
fn no_branch_positive() {
    instructions_execute! {
        [
            nop!(),
            instr_i!(BNZ, ZERO, T0, 12),
            nop!(),
            nop!(),
            nop!(),
        ],
        [T0 = 0] => [],
        empty_storage!() => empty_storage!(),
        2,
        None,
        8
    }
}

#[test]
fn bad_alignment() {
    instructions_execute! {
        [
            nop!(),
            instr_i!(BNZ, ZERO, T0, 5),
            nop!(),
            nop!(),
            nop!(),
        ],
        [T0 = 1] => [],
        empty_storage!() => empty_storage!(),
        2,
        Some(ExitCode::BadAlignment),
        4
    }
}

#[test]
fn bad_jump_positive() {
    instructions_execute! {
        [
            nop!(),
            instr_i!(BNZ, ZERO, T0, 32),
            nop!(),
            nop!(),
            nop!(),
        ],
        [T0 = 1] => [],
        empty_storage!() => empty_storage!(),
        2,
        Some(ExitCode::BadJump),
        4
    }
}

#[test]
fn bad_jump_negative() {
    instructions_execute! {
        [
            nop!(),
            instr_i!(BNZ, ZERO, T0, -32),
            nop!(),
            nop!(),
            nop!(),
        ],
        [T0 = 1] => [],
        empty_storage!() => empty_storage!(),
        2,
        Some(ExitCode::BadJump),
        4
    }
}
