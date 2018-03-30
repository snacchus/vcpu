mod core;

use std::rc::Rc;
use std::cell::RefCell;
use std::fmt;
use std::error::Error as StdError;
use num::traits::ToPrimitive;

use byteorder::ByteOrder;

use super::{constants, Address, Immediate, Word, Endian};
use super::memory::Memory;
use self::core::{Core, TickResult};

// Instruction set based on the DLX processor

// Instruction Formats
//                                                           
//          +------+-----+-----+-----+-----+------+
//          |31    |     |     |     |     |     0|
//          +------+-----+-----+-----+-----+------+
// R-Format |  RIN | Rd  | Rs1 | Rs2 |  -  |opcode|
//          +------+-----+-----+-----+-----+------+
// I-Format |opcode| Rd  | Rs1 |    immediate     |
//          +------+-----+-----+-----+-----+------+
// J-Format |opcode|           address            |
//          +------+-----+-----+-----+-----+------+

#[derive(FromPrimitive, ToPrimitive)]
pub enum OpCode {
    //  Mnemonic    | Name                            | Format | Effect
    //--------------+---------------------------------+--------+-------------------------------------------------
    // Misc         |                                 |        |
    NOP,         // | No-op                           | I      | Does nothing
    RIN,         // | R-instruction                   | R      | Special opcode, looks for actual opcode in lowest 6 bits
    HALT,        // | Halt                            | I      | Stops the CPU
    CALL,        // | System call                     | I      | Calls system function with code <immediate> and argument <Rs1>
    //--------------+---------------------------------+--------+-------------------------------------------------
    // I/O          |                                 |        |
    COPY,        // | Copy                            | I      | Rd = Rs1
    LI,          // | Load immediate                  | I      | Rd = extend(immediate)
    LHI,         // | Load high bits                  | I      | Rd = immediate << 16
    LB,          // | Load byte                       | I      | Rd = MEM[Rs1 + extend(immediate)]
    LH,          // | Load half word                  | I      | Rd = MEM[Rs1 + extend(immediate)]
    LW,          // | Load word                       | I      | Rd = MEM[Rs1 + extend(immediate)]
    SB,          // | Store byte                      | I      | MEM[Rs1 + extend(immediate)] = Rd
    SH,          // | Store half word                 | I      | MEM[Rs1 + extend(immediate)] = Rd
    SW,          // | Store word                      | I      | MEM[Rs1 + extend(immediate)] = Rd
    //--------------+---------------------------------+--------+-------------------------------------------------
    // Arithmetic   |                                 |        |
    ADDI,        // | Add immediate                   | I      | Rd = Rs1 + extend(immediate)
    SUBI,        // | Subtract immediate              | I      | Rd = Rs1 - extend(immediate)
    MULI,        // | Multiply immediate              | I      | Rd = Rs1 * extend(immediate)
    DIVI,        // | Divide immediate                | I      | Rd = Rs1 / extend(immediate); REM = remainder
    //--------------+---------------------------------+--------+-------------------------------------------------
    // Logic        |                                 |        |
    ANDI,        // | And immediate                   | I      | Rd = Rs1 & extend(immediate)
    ORI,         // | Or immediate                    | I      | Rd = Rs1 | extend(immediate)
    XORI,        // | Exclusive-Or immediate          | I      | Rd = Rs1 ^ extend(immediate)
    FLIP,        // | Flip                            | I      | Rd = ~Rs1
    //--------------+---------------------------------+--------+-------------------------------------------------
    // Shifts       |                                 |        |
    SLLI,        // | Shift left logical immediate    | I      | Rd = Rs1 << immediate
    SRLI,        // | Shift right logical immediate   | I      | Rd = Rs1 >> immediate (inserting zeros)
    SRAI,        // | Shift right arithmetic immediate| I      | Rd = Rs1 >> immediate (inserting sign-bit)
    //--------------+---------------------------------+--------+-------------------------------------------------
    // Comparisons  |                                 |        |
    SEQI,        // | Set if equal immediate          | I      | Rd = (Rs1 == extend(immediate)) ? 1 : 0
    SNEI,        // | Set if not equal immediate      | I      | Rd = (Rs1 != extend(immediate)) ? 1 : 0
    SLTI,        // | Set if less than immediate      | I      | Rd = (Rs1 < extend(immediate)) ? 1 : 0
    SGTI,        // | Set if greater than immediate   | I      | Rd = (Rs1 > extend(immediate)) ? 1 : 0
    SLEI,        // | Set if less equal immediate     | I      | Rd = (Rs1 <= extend(immediate)) ? 1 : 0
    SGEI,        // | Set if greater equal immediate  | I      | Rd = (Rs1 >= extend(immediate)) ? 1 : 0
    //--------------+---------------------------------+--------+-------------------------------------------------
    // Branching    |                                 |        |
    BEZ,         // | Branch if zero                  | I      | PC += ((Rs1 == 0) ? extend(immediate) : 0)
    BNZ,         // | Branch if not zero              | I      | PC += ((Rs1 != 0) ? extend(immediate) : 0)
    //--------------+---------------------------------+--------+-------------------------------------------------
    // Jumping      |                                 |        |
    JMP,         // | Jump                            | J      | PC += extend(address)
    JL,          // | Jump and link                   | J      | RET = PC + 4; PC += extend(address)
    JR,          // | Jump register                   | I      | PC = Rs1
    JLR,         // | Jump and link register          | I      | RET = PC + 4; PC = Rs1
    //--------------+---------------------------------+--------+-------------------------------------------------
    // Float conv.  |                                 |        |
    ITOF,        // | Int to float                    | I      | Rd = float(Rs1)
    FTOI,        // | Float to int                    | I      | Rd = int(Rs1)
    //--------------+---------------------------------+--------+-------------------------------------------------
}

#[derive(FromPrimitive, ToPrimitive)]
pub enum OpCodeR {
    //--------------+---------------------------------+--------+-------------------------------------------------
    // Arithmetic   |                                 |        |
    ADD,         // | Add                             | R      | Rd = Rs1 + Rs2
    SUB,         // | Subtract                        | R      | Rd = Rs1 - Rs2
    MUL,         // | Multiply                        | R      | Rd = Rs1 * Rs2
    DIV,         // | Divide                          | R      | Rd = Rs1 / Rs2; REM = remainder
    //--------------+---------------------------------+--------+-------------------------------------------------
    // Logic        |                                 |        |
    AND,         // | And                             | R      | Rd = Rs1 & Rs2
    OR,          // | Or                              | R      | Rd = Rs1 | Rs2
    XOR,         // | Exclusive-Or                    | R      | Rd = Rs1 ^ Rs2
    //--------------+---------------------------------+--------+-------------------------------------------------
    // Shifts       |                                 |        |
    SLL,         // | Shift left logical              | R      | Rd = Rs1 << Rs2
    SRL,         // | Shift right logical             | R      | Rd = Rs1 >> Rs2 (inserting zeros)
    SRA,         // | Shift right arithmetic          | R      | Rd = Rs1 >> Rs2 (inserting sign-bit)
    //--------------+---------------------------------+--------+-------------------------------------------------
    // Comparisons  |                                 |        |
    SEQ,         // | Set if equal                    | R      | Rd = (Rs1 == Rs2) ? 1 : 0
    SNE,         // | Set if not equal                | R      | Rd = (Rs1 != Rs2) ? 1 : 0
    SLT,         // | Set if less than                | R      | Rd = (Rs1 < Rs2) ? 1 : 0
    SGT,         // | Set if greater than             | R      | Rd = (Rs1 > Rs2) ? 1 : 0
    SLE,         // | Set if less equal               | R      | Rd = (Rs1 <= Rs2) ? 1 : 0
    SGE,         // | Set if greater equal            | R      | Rd = (Rs1 >= Rs2) ? 1 : 0
    //--------------+---------------------------------+--------+-------------------------------------------------
    // Float Arithm.|                                 |        |
    FADD,        // | Float add                       | R      | Rd = Rs1 + Rs2 (using IEEE 754 floats)
    FSUB,        // | Float subtract                  | R      | Rd = Rs1 - Rs2 (using IEEE 754 floats) 
    FMUL,        // | Float multiply                  | R      | Rd = Rs1 * Rs2 (using IEEE 754 floats)
    FDIV,        // | Float divide                    | R      | Rd = Rs1 / Rs2 (using IEEE 754 floats)
    //--------------+---------------------------------+--------+-------------------------------------------------
}

#[derive(FromPrimitive, ToPrimitive)]
pub enum RegisterId {
    ZERO,       // Always zero (read only)

    V0,         // Value 0
    V1,         // Value 1

    A0,         // Argument 0
    A1,         // Argument 1
    A2,         // Argument 2 
    A3,         // Argument 3 
    A4,         // Argument 4 

    T0,         // Temporary 0
    T1,         // Temporary 1
    T2,         // Temporary 2 
    T3,         // Temporary 3
    T4,         // Temporary 4
    T5,         // Temporary 5
    T6,         // Temporary 6
    T7,         // Temporary 7
    T8,         // Temporary 8
    T9,         // Temporary 9

    S0,         // Saved 0
    S1,         // Saved 1
    S2,         // Saved 2 
    S3,         // Saved 3 
    S4,         // Saved 4
    S5,         // Saved 5
    S6,         // Saved 6
    S7,         // Saved 7
    S8,         // Saved 8
    S9,         // Saved 9

    SP,         // Stack pointer
    FP,         // Frame pointer

    RM,         // Contains remainder after integer division
    RA,         // Contains return address after jump and link
}

#[inline]
fn enum_to_u32<T: ToPrimitive>(val: T) -> u32 {
    ToPrimitive::to_u32(&val).unwrap()
}

#[inline]
pub fn register_index(id: RegisterId) -> usize {
    enum_to_u32(id) as usize
}

#[inline]
pub fn instr_r(oc: OpCodeR, rd: RegisterId, rs1: RegisterId, rs2: RegisterId) -> Word {
    ((enum_to_u32(OpCode::RIN) << constants::OPCODE_OFFSET)   & constants::OPCODE_MASK)   |
    ((enum_to_u32(rd)          << constants::RD_OFFSET)       & constants::RD_MASK)       |
    ((enum_to_u32(rs1)         << constants::RS1_OFFSET)      & constants::RS1_MASK)      |
    ((enum_to_u32(rs2)         << constants::RS2_OFFSET)      & constants::RS2_MASK)      |
    ((enum_to_u32(oc)          << constants::OPCODE_R_OFFSET) & constants::OPCODE_R_MASK) 
}

#[inline]
pub fn instr_i(oc: OpCode, rd: RegisterId, rs1: RegisterId, immediate: Immediate) -> Word {
    ((enum_to_u32(oc)    << constants::OPCODE_OFFSET)    & constants::OPCODE_MASK)    |
    ((enum_to_u32(rd)    << constants::RD_OFFSET)        & constants::RD_MASK)        |
    ((enum_to_u32(rs1)   << constants::RS1_OFFSET)       & constants::RS1_MASK)       |
    (((immediate as u32) << constants::IMMEDIATE_OFFSET) & constants::IMMEDIATE_MASK)
}

#[inline]
pub fn instr_j(oc: OpCode, address: Address) -> Word {
    ((enum_to_u32(oc)    << constants::OPCODE_OFFSET) & constants::OPCODE_MASK) |
    (((address as u32) << constants::ADDRESS_OFFSET)  & constants::ADDRESS_MASK)
}

#[inline]
pub fn jmp_addr_i16(offset: i16) -> Immediate {
    offset * (constants::WORD_BYTES as i16)
}

#[inline]
pub fn jmp_addr_i32(offset: i32) -> Address {
    offset * (constants::WORD_BYTES as i32)
}

#[derive(PartialEq, Debug)]
pub enum ExitCode {
    Unknown,         // Reason for shutdown unknown
    Halted,          // HALT instruction was executed (Normal shutdown)
    Terminated,      // External termination signal was sent
    DivisionByZero,  // Attempted integer division by zero
    BadMemoryAccess, // Attempted to access main memory at invalid address
    BadAlignment,    // Jump address was not aligned to word boundaries
    BadJump,         // Jump address was out of instruction memory range
    InvalidOpcode,   // Opcode was not recognized
    EmptyProgram,    // Loaded program is empty
}

#[derive(Clone, Copy)]
pub union Register {
    i : i32,
    u : u32,
    f : f32,
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
        if data.len() % constants::WORD_BYTES != 0 {
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
            let instruction = Endian::read_u32(&self.data[pc..(pc + constants::WORD_BYTES)]);
            
            let tick_result = self.core.tick(instruction, program_counter);

            match tick_result {
                TickResult::Next => {
                    let new_pc = program_counter.wrapping_add(constants::WORD_BYTES as u32);
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
