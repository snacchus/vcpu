mod core;
mod enums;

pub use self::enums::*;

use std::rc::Rc;
use std::cell::RefCell;
use std::fmt;
use std::error::Error as StdError;

use byteorder::ByteOrder;

use super::{constants, Address, Immediate, Endian};
use super::memory::Memory;
use self::core::{Core, TickResult};

#[inline]
pub fn jmp_addr_i16(offset: i16) -> Immediate {
    offset * (constants::WORD_BYTES as i16)
}

#[inline]
pub fn jmp_addr_i32(offset: i32) -> Address {
    offset * (constants::WORD_BYTES as i32)
}

#[derive(PartialEq, Eq, Debug)]
pub enum ExitCode {
    Unknown,         // Reason for shutdown unknown
    Halted,          // HALT instruction was executed (Normal shutdown)
    Terminated,      // External termination signal was sent
    DivisionByZero,  // Attempted integer division by zero
    BadMemoryAccess, // Attempted to access main memory at invalid address
    BadAlignment,    // Jump address was not aligned to word boundaries
    BadJump,         // Jump address was out of instruction memory range
    InvalidOpcode,   // Opcode or funct was not recognized
    EmptyProgram,    // Loaded program is empty
}

#[derive(Debug)]
pub enum Error {
    InvalidProgram(usize)
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
    i : i32,
    u : u32,
    f : f32,
}

impl Register {
    fn new() -> Register {
        Register { u: 0 }
    }

    pub fn i(&self) -> i32 {
        unsafe {
            self.i
        }
    }

    pub fn u(&self) -> u32 {
        unsafe {
            self.u
        }
    }

    pub fn f(&self) -> f32 {
        unsafe {
            self.f
        }
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
    data: Vec<u8>,
}

impl Processor {
    pub fn new(memory: Rc<RefCell<Memory>>) -> Processor {
        Processor{ 
            core: Core::new(memory),
            data: Vec::new(),
        }
    }

    pub fn load_program(&mut self, data: &[u8]) -> Result<(), Error> {
        if data.len() as u32 % constants::WORD_BYTES != 0 {
            Err(Error::InvalidProgram(data.len()))
        } else {
            self.data = Vec::from(data);
            Ok(())
        }
    }

    pub fn register(&self, id: RegisterId) -> &Register {
        self.core.register(id)
    }

    pub fn run(&mut self) -> ExitCode {
        self.core.zero_registers();

        if self.data.is_empty() {
            return ExitCode::EmptyProgram;
        }

        let program_bytes = self.data.len() as u32;

        let mut program_counter = 0u32;

        loop {
            let pc = program_counter as usize;
            let instruction = Endian::read_u32(&self.data[pc..(pc + constants::WORD_BYTES as usize)]);
            
            let tick_result = self.core.tick(instruction, program_counter);

            match tick_result {
                TickResult::Next => {
                    let new_pc = program_counter.wrapping_add(constants::WORD_BYTES);
                    program_counter = if new_pc < program_bytes { new_pc } else { 0 };
                },
                TickResult::Jump(new_pc) => {
                    if new_pc >= program_bytes {
                        return ExitCode::BadJump;
                    } else if (new_pc % (constants::WORD_BYTES as u32)) != 0 {
                        return ExitCode::BadAlignment;
                    } else {
                        program_counter = new_pc;
                    }
                },
                TickResult::Stop(exit_code) => {
                    return exit_code;
                }
            }
        }
    }
}
