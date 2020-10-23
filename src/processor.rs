mod logic;

use crate::StorageMut;
use crate::{constants, register_index, Address, Endian, Immediate, Register, RegisterId, Word};
use logic::TickResult;
use util::InteropGetName;
use util_derive::InteropGetName;

use byteorder::ByteOrder;
use num_derive::{FromPrimitive, ToPrimitive};

pub const fn jmp_addr_i16(offset: i16) -> Immediate {
    offset * (constants::WORD_BYTES as i16)
}

pub const fn jmp_addr_i32(offset: i32) -> Address {
    offset * (constants::WORD_BYTES as i32)
}

pub fn instructions_from_words(vec: &[Word]) -> Vec<u8> {
    let mut byte_vec = vec![0; vec.len() * constants::WORD_BYTES as usize];
    Endian::write_u32_into(&vec[..], &mut byte_vec[..]);
    byte_vec
}

fn get_next_pc(pc: u32, instr_len: u32) -> u32 {
    let result = pc.wrapping_add(constants::WORD_BYTES);
    if result < instr_len {
        result
    } else {
        0
    }
}

#[derive(PartialEq, Eq, Debug, Clone, Copy, FromPrimitive, ToPrimitive, InteropGetName)]
pub enum ExitCode {
    /// HALT instruction was executed (Normal shutdown).
    Halted,
    /// Attempted integer division by zero.
    DivisionByZero,
    /// Attempted to access main memory at invalid address.
    BadMemoryAccess,
    /// Jump address was not aligned to word boundaries.
    BadAlignment,
    /// Jump address was out of instruction memory range.
    BadJump,
    /// Opcode or funct was not recognized.
    InvalidOpcode,
    /// Program counter is out of instruction memory range.
    BadProgramCounter,
}

pub struct Processor {
    registers: [Register; constants::REGISTER_COUNT],
    program_counter: u32,
    state: Option<ExitCode>,
}

impl Processor {
    pub fn new() -> Processor {
        Default::default()
    }

    pub fn registers(&self) -> &[Register; constants::REGISTER_COUNT] {
        &self.registers
    }

    pub fn registers_mut(&mut self) -> &mut [Register; constants::REGISTER_COUNT] {
        &mut self.registers
    }

    pub fn register(&self, id: RegisterId) -> &Register {
        &self.registers[register_index(id)]
    }

    pub fn register_mut(&mut self, id: RegisterId) -> &mut Register {
        &mut self.registers[register_index(id)]
    }

    pub fn program_counter(&self) -> u32 {
        self.program_counter
    }

    pub fn state(&self) -> Option<ExitCode> {
        self.state
    }

    pub fn is_stopped(&self) -> bool {
        self.state.is_some()
    }

    pub fn tick(&mut self, instructions: &[u8], storage: &mut dyn StorageMut) -> Option<ExitCode> {
        if !self.is_stopped() {
            self.state = self.get_new_state(instructions, storage);
        }

        self.state
    }

    pub fn reset(&mut self) {
        self.registers = [Default::default(); constants::REGISTER_COUNT];
        self.program_counter = 0u32;
        self.state = None;
    }

    fn get_new_state(
        &mut self,
        instructions: &[u8],
        storage: &mut dyn StorageMut,
    ) -> Option<ExitCode> {
        let instr_len = instructions.len() as u32;
        if self.program_counter + constants::WORD_BYTES > instr_len {
            Some(ExitCode::BadProgramCounter)
        } else {
            let pc = self.program_counter as usize;

            let instruction =
                Endian::read_u32(&instructions[pc..(pc + constants::WORD_BYTES as usize)]);

            let tick_result = logic::tick(
                &mut self.registers,
                storage,
                instruction,
                self.program_counter,
            );

            match tick_result {
                TickResult::Next => {
                    self.program_counter = get_next_pc(self.program_counter, instr_len);
                    None
                }
                TickResult::Jump(new_pc, link) => {
                    if (new_pc % (constants::WORD_BYTES as u32)) != 0 {
                        Some(ExitCode::BadAlignment)
                    } else if new_pc >= instr_len {
                        Some(ExitCode::BadJump)
                    } else {
                        let old_pc = self.program_counter;
                        if link {
                            self.register_mut(RegisterId::RA)
                                .set_u(get_next_pc(old_pc, instr_len));
                        }
                        self.program_counter = new_pc;
                        None
                    }
                }
                TickResult::Stop(exit_code) => Some(exit_code),
            }
        }
    }

    pub fn run(&mut self, instructions: &[u8], storage: &mut dyn StorageMut) -> ExitCode {
        loop {
            if let Some(exit_code) = self.tick(instructions, storage) {
                return exit_code;
            }
        }
    }
}

impl Default for Processor {
    fn default() -> Processor {
        Processor {
            registers: [Default::default(); constants::REGISTER_COUNT],
            program_counter: 0u32,
            state: None,
        }
    }
}
