extern crate num;
#[macro_use]
extern crate num_derive;

pub mod constants;
pub mod processor;
pub mod memory;

type Word = u32;
type Immediate = i16;
type Address = i32;

mod tests {
    use processor::*;
    use super::*;

    #[test]
    fn wrapping_arithmetic() {
        use std::num::Wrapping;

        let i = -50;

        let a = Wrapping(100u32);
        let b = Wrapping(i as u32);
        let c = a + b;

        assert_eq!(c, Wrapping(50u32));
    }

    fn test_program(program: &[Word], expected_code: ExitCode) -> Processor {

        let mut processor = Processor::new();

        processor.load_program(program);
        let exit_code = processor.run();

        assert_eq!(exit_code, expected_code);

        processor
    }

    #[test]
    fn program_halt() {
        let program = vec![
            make_instruction_i(OpCode::HALT, RegisterId::ZERO, RegisterId::ZERO, 0)
        ];

        test_program(&program[..], ExitCode::Halted);
    }

    #[test]
    fn program_add() {
        let program = vec![
            make_instruction_i(OpCode::LI, RegisterId::T0, RegisterId::ZERO, 42),
            make_instruction_i(OpCode::LI, RegisterId::T1, RegisterId::ZERO, 64),
            make_instruction_r(OpCodeR::ADD, RegisterId::T2, RegisterId::T0, RegisterId::T1),
            make_instruction_i(OpCode::HALT, RegisterId::ZERO, RegisterId::ZERO, 0)
        ];

        let processor = test_program(&program[..], ExitCode::Halted);

        assert_eq!(processor.register(RegisterId::T2).i(), 106);
    }
}
