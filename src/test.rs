use super::*;

macro_rules! instr {
    (a $opcode:ident $rd:ident $rs1:ident $rs2:ident) => {
        instr_alu!($opcode, $rd, $rs1, $rs2)
    };
    (i $opcode:ident $rd:ident $rs1:ident $imm:expr) => {
        instr_i!($opcode, $rd, $rs1, $imm)
    };
    (j $opcode:ident $addr:expr) => {
        instr_j!($opcode, $addr)
    };
}

macro_rules! instructions {
    [$( ($( $x:tt )+) ),*] => {
        [$(instr!($($x)+)),*]
    };
}

macro_rules! empty_storage {
    () => {
        [0u8; 0]
    };
}

#[test]
fn wrapping_arithmetic() {
    let i = -20;

    let a = 20u32;
    let b = i as u32;
    let c = a.wrapping_add(b);

    assert_eq!(c, 0u32);
}

#[allow(dead_code)]
fn test_instructions_me(
    mem_size: u32,
    instructions: &[u8],
    expected_code: ExitCode,
) -> (Processor, Vec<u8>) {
    let mut processor = Processor::default();
    let mut memory = vec![0; mem_size as usize];

    let exit_code = processor.run(&instructions, &mut memory);

    assert_eq!(exit_code, expected_code);

    (processor, memory)
}

#[allow(dead_code)]
fn test_instructions_e(instructions: &[u8], expected_code: ExitCode) -> (Processor, Vec<u8>) {
    test_instructions_me(1024, instructions, expected_code)
}

#[allow(dead_code)]
fn test_instructions_m(mem_size: u32, instructions: &[u8]) -> (Processor, Vec<u8>) {
    test_instructions_me(mem_size, instructions, ExitCode::Halted)
}

#[allow(dead_code)]
fn test_instructions(instructions: &[u8]) -> (Processor, Vec<u8>) {
    test_instructions_e(instructions, ExitCode::Halted)
}

#[test]
fn instructions_halt() {
    let instructions = instructions_from_words(&[instr_i!(HALT, ZERO, ZERO, 0)]);

    test_instructions(&instructions[..]);
}

#[test]
fn instructions_add() {
    let instructions = instructions_from_words(&instructions![
        (i LI T0 ZERO 42),
        (i LI T1 ZERO 64),
        (a ADD T2 T0 T1),
        (i HALT ZERO ZERO 0)
    ]);

    let (processor, _) = test_instructions(&instructions[..]);

    assert_eq!(processor.register(RegisterId::T2).i(), 106);
}

#[test]
fn instructions_loop() {
    let iterations = 32i32;

    let instructions = instructions_from_words(&instructions![
        (i SLTI T2 T0 iterations as i16),
        (i BEZ ZERO T2 jmp_addr_i16(5)),
        (i SLLI T1 T0 2),
        (i SW T0 T1 0),
        (i ADDI T0 T0 1),
        (j JMP jmp_addr_i32(-5)),
        (i HALT ZERO ZERO 0)
    ]);

    let (_, storage) = test_instructions(&instructions[..]);

    for i in 0..iterations {
        let value = storage.read_word(i as u32 * constants::WORD_BYTES).unwrap() as i32;
        assert_eq!(value, i);
    }
}

#[test]
fn negative_immediate_value() {
    let instructions = instructions_from_words(&instructions![
        (i LI T0 ZERO 16),
        (i ADDI T0 T0 -4),
        (i HALT ZERO ZERO 0)
    ]);

    let (processor, _) = test_instructions(&instructions[..]);

    assert_eq!(12i32, processor.register(RegisterId::T0).i());
}

#[test]
fn instruction_sw_negative_offset() {
    let instructions = instructions_from_words(&instructions![
        (i LI T0 ZERO 23),
        (i LI T1 ZERO 16),
        (i SW T0 T1 -4),
        (i HALT ZERO ZERO 0)
    ]);

    let (_, storage) = test_instructions_me(64, &instructions[..], ExitCode::Halted);

    assert_eq!(23, storage.read_word(12).unwrap());
}

#[test]
fn truncation_in_store_instruction() {
    let instructions = instructions_from_words(&instructions![
        (i FLIP T0 T0 0),
        (i SB T0 ZERO 0),
        (i HALT ZERO ZERO 0)
    ]);

    let (processor, storage) = test_instructions(&instructions[..]);

    assert_eq!(0xFFFF_FFFF, processor.register(RegisterId::T0).u());
    assert_eq!(0xFF, storage[0]);
}

mod instructions;
