extern crate num;
extern crate num_integer;

#[macro_use]
extern crate num_derive;

extern crate byteorder;

pub mod constants;
pub mod processor;
pub mod memory;

type Word = u32;
type Immediate = i16;
type Address = i32;

type Endian = byteorder::LittleEndian;

#[cfg(test)]
mod tests {
    use std::rc::Rc;
    use std::cell::RefCell;
    use byteorder::ByteOrder;
    use super::*;
    use super::processor::*;
    use super::memory::*;

    type MemoryRef = Rc<RefCell<Memory>>;

    #[test]
    fn wrapping_arithmetic() {
        let i = -20;

        let a = 20u32;
        let b = i as u32;
        let c = a.wrapping_add(b);

        assert_eq!(c, 0u32);
    }

    #[allow(dead_code)]
    fn test_program_me(mem_size: usize, program: &[u8], expected_code: ExitCode) -> (Processor, MemoryRef) {
        let memory = Rc::new(RefCell::new(Memory::new(mem_size)));
        let mut processor = Processor::new(memory.clone());

        processor.load_program(program).unwrap();
        let exit_code = processor.run();

        assert_eq!(exit_code, expected_code);

        (processor, memory)
    }

    #[allow(dead_code)]
    fn test_program_e(program: &[u8], expected_code: ExitCode) -> (Processor, MemoryRef) {
        test_program_me(1024, program, expected_code)
    }

    #[allow(dead_code)]
    fn test_program_m(mem_size: usize, program: &[u8]) -> (Processor, MemoryRef) {
        test_program_me(mem_size, program, ExitCode::Halted)
    }

    #[allow(dead_code)]
    fn test_program(program: &[u8]) -> (Processor, MemoryRef) {
        test_program_e(program, ExitCode::Halted)
    }

    fn transmute_vec(vec: Vec<Word>) -> Vec<u8> {
        let mut byte_vec = vec![0; vec.len() * constants::WORD_BYTES];
        Endian::write_u32_into(&vec[..], &mut byte_vec[..]);
        byte_vec
    }

    #[test]
    fn program_halt() {
        let program = transmute_vec(vec![
            instr_i(OpCode::HALT, RegisterId::ZERO, RegisterId::ZERO, 0)
        ]);

        test_program(&program[..]);
    }

    #[test]
    fn program_add() {
        let program = transmute_vec(vec![
            instr_i(OpCode::LI, RegisterId::T0, RegisterId::ZERO, 42),
            instr_i(OpCode::LI, RegisterId::T1, RegisterId::ZERO, 64),
            instr_r(OpCodeR::ADD, RegisterId::T2, RegisterId::T0, RegisterId::T1),
            instr_i(OpCode::HALT, RegisterId::ZERO, RegisterId::ZERO, 0)
        ]);

        let (processor, _) = test_program(&program[..]);

        assert_eq!(processor.register(RegisterId::T2).i(), 106);
    }

    #[test]
    fn program_loop() {
        let iterations = 32i32;

        let program = transmute_vec(vec![
            instr_i(OpCode::SLTI, RegisterId::T2, RegisterId::T0, iterations as i16),
            instr_i(OpCode::BEZ, RegisterId::ZERO, RegisterId::T2, jmp_addr_i16(5)),
            instr_i(OpCode::SLLI, RegisterId::T1, RegisterId::T0, 2),
            instr_i(OpCode::SW, RegisterId::T0, RegisterId::T1, 0),
            instr_i(OpCode::ADDI, RegisterId::T0, RegisterId::T0, 1),
            instr_j(OpCode::JMP, jmp_addr_i32(-5)),
            instr_i(OpCode::HALT, RegisterId::ZERO, RegisterId::ZERO, 0)
        ]);

        let (_, memory) = test_program(&program[..]);

        let mem_ref = memory.borrow();

        for i in 0..iterations {
            let value = mem_ref.read_word((i as usize) * constants::WORD_BYTES).unwrap() as i32;
            assert_eq!(value, i);
        }
    }
}
