mod core;
mod enums;

pub use self::enums::*;
use crate::memory::StorageMut;

use std::error::Error as StdError;
use std::fmt;

use byteorder::ByteOrder;

use self::core::{Core, TickResult};
use super::{constants, Address, Endian, Immediate, Word};

pub const fn jmp_addr_i16(offset: i16) -> Immediate {
    offset * (constants::WORD_BYTES as i16)
}

pub const fn jmp_addr_i32(offset: i32) -> Address {
    offset * (constants::WORD_BYTES as i32)
}

pub fn program_from_words(vec: &[Word]) -> Vec<u8> {
    let mut byte_vec = vec![0; vec.len() * constants::WORD_BYTES as usize];
    Endian::write_u32_into(&vec[..], &mut byte_vec[..]);
    byte_vec
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum ExitCode {
    Halted,          // HALT instruction was executed (Normal shutdown)
    Terminated,      // External termination signal was sent
    DivisionByZero,  // Attempted integer division by zero
    BadMemoryAccess, // Attempted to access main memory at invalid address
    BadAlignment,    // Jump address was not aligned to word boundaries
    BadJump,         // Jump address was out of instruction memory range
    InvalidOpcode,   // Opcode or funct was not recognized
    EmptyProgram,    // Loaded program is empty
    Unknown,         // Reason for shutdown unknown
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Error {
    InvalidProgram(usize),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::InvalidProgram(_) => f.write_str("InvalidProgram"),
        }
    }
}

impl StdError for Error {
    fn description(&self) -> &str {
        match *self {
            Error::InvalidProgram(_) => "Program has invalid size! (Must be multiple of 4).",
        }
    }
}

#[derive(Clone, Copy)]
pub union Register {
    i: i32,
    u: u32,
    f: f32,
}

impl Register {
    fn new() -> Register {
        Register { u: 0 }
    }

    pub fn i(self) -> i32 {
        unsafe { self.i }
    }

    pub fn u(self) -> u32 {
        unsafe { self.u }
    }

    pub fn f(self) -> f32 {
        unsafe { self.f }
    }

    fn set_i(&mut self, value: i32) {
        self.i = value;
    }

    fn set_u(&mut self, value: u32) {
        self.u = value;
    }

    fn set_f(&mut self, value: f32) {
        self.f = value;
    }
}

pub struct Processor {
    core: Core,
    instructions: Vec<u8>,
    program_counter: u32,
}

impl Processor {
    pub fn construct<S: StorageMut + 'static>(
        instructions: &[u8],
        storage: S,
    ) -> Result<Processor, Error> {
        if instructions.len() as u32 % constants::WORD_BYTES != 0 {
            Err(Error::InvalidProgram(instructions.len()))
        } else {
            Ok(Processor {
                core: Core::new(storage),
                instructions: Vec::from(instructions),
                program_counter: 0u32,
            })
        }
    }

    pub fn storage(&self) -> &dyn StorageMut {
        self.core.storage()
    }

    pub fn register(&self, id: RegisterId) -> &Register {
        self.core.register(id)
    }

    pub fn tick(&mut self) -> Option<ExitCode> {
        if self.instructions.is_empty() {
            Some(ExitCode::EmptyProgram)
        } else {
            let instr_len = self.instructions.len() as u32;

            let pc = self.program_counter as usize;
            let instruction =
                Endian::read_u32(&self.instructions[pc..(pc + constants::WORD_BYTES as usize)]);

            let tick_result = self.core.tick(instruction, self.program_counter);

            match tick_result {
                TickResult::Next => {
                    let new_pc = self.program_counter.wrapping_add(constants::WORD_BYTES);
                    self.program_counter = if new_pc < instr_len { new_pc } else { 0 };
                    None
                }
                TickResult::Jump(new_pc) => {
                    if new_pc >= instr_len {
                        Some(ExitCode::BadJump)
                    } else if (new_pc % (constants::WORD_BYTES as u32)) != 0 {
                        Some(ExitCode::BadAlignment)
                    } else {
                        self.program_counter = new_pc;
                        None
                    }
                }
                TickResult::Stop(exit_code) => Some(exit_code),
            }
        }
    }

    pub fn run(&mut self) -> ExitCode {
        loop {
            if let Some(exit_code) = self.tick() {
                return exit_code;
            }
        }
    }
}
