use { Address, Immediate, Word, constants };
use num::traits::ToPrimitive;
use std::error::Error;
use std::fmt;
use std::str::FromStr;

// Instruction set based on the DLX processor

// Instruction Formats
//                                                           
//          +------+-----+-----+-----+-----+------+
//          |31    |     |     |     |     |     0|
//          +------+-----+-----+-----+-----+------+
// R-Format |opcode| Rd  | Rs1 | Rs2 |  -  |funct |
//          +------+-----+-----+-----+-----+------+
// I-Format |opcode| Rd  | Rs1 |    immediate     |
//          +------+-----+-----+-----+-----+------+
// J-Format |opcode|           address            |
//          +------+-----+-----+-----+-----+------+

#[derive(Clone, Copy, PartialEq, Eq, Debug, ToPrimitive, FromPrimitive)]
pub enum OpCode {
    //  Mnemonic    | Name                            | Format | Effect
    //--------------+---------------------------------+--------+-------------------------------------------------
    // Misc         |                                 |        |
    NOP,         // | No-op                           | I      | Does nothing
    ALU,         // | ALU-instruction                 | R      | Executes ALU function denoted by <funct>
    HALT,        // | Halt                            | I      | Stops the CPU
    CALL,        // | System call                     | I      | Calls system function with code <immediate> and argument <Rs1>
    //--------------+---------------------------------+--------+-------------------------------------------------
    // I/O          |                                 |        |
    COPY,        // | Copy                            | I      | Rd = Rs1
    LI,          // | Load immediate                  | I      | Rd = extend(immediate)
    LHI,         // | Load immediate high bits        | I      | Rd = immediate << 16
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

#[derive(Clone, Copy, PartialEq, Eq, FromPrimitive, ToPrimitive, Debug)]
pub enum ALUFunct {
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

#[derive(Clone, Copy, PartialEq, Eq, FromPrimitive, ToPrimitive, Debug)]
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

macro_rules! impl_enum_display {
    ($e:ty) => {
        impl fmt::Display for $e {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                fmt::Debug::fmt(self, f)
            }
        }
    };
}

impl_enum_display!(OpCode);
impl_enum_display!(ALUFunct);
impl_enum_display!(RegisterId);

#[inline]
pub fn register_index(id: RegisterId) -> usize {
    enum_to_u32(id) as usize
}

#[inline]
fn instr_r(oc: OpCode, rd: RegisterId, rs1: RegisterId, rs2: RegisterId, funct: u32) -> Word {
    ((enum_to_u32(oc)  << constants::OPCODE_OFFSET) & constants::OPCODE_MASK) |
    ((enum_to_u32(rd)  << constants::RD_OFFSET)     & constants::RD_MASK)     |
    ((enum_to_u32(rs1) << constants::RS1_OFFSET)    & constants::RS1_MASK)    |
    ((enum_to_u32(rs2) << constants::RS2_OFFSET)    & constants::RS2_MASK)    |
    ((funct            << constants::FUNCT_OFFSET)  & constants::FUNCT_MASK) 
}

#[inline]
pub fn instr_alu(funct: ALUFunct, rd: RegisterId, rs1: RegisterId, rs2: RegisterId) -> Word {
    instr_r(OpCode::ALU, rd, rs1, rs2, enum_to_u32(funct))
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
    ((enum_to_u32(oc)  << constants::OPCODE_OFFSET)  & constants::OPCODE_MASK) |
    (((address as u32) << constants::ADDRESS_OFFSET) & constants::ADDRESS_MASK)
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct ParseEnumError {
    value: String,
    enum_name: &'static str
}

impl fmt::Display for ParseEnumError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Failed to parse \"{}\" as {}.", &self.value, &self.enum_name)
    }
}

impl Error for ParseEnumError { 
    fn description(&self) -> &str {
        "Failed to parse enum."
    }
}

impl FromStr for OpCode {
    type Err = ParseEnumError;

    fn from_str(s: &str) -> Result<OpCode, ParseEnumError> {
        match s {
            "NOP" => Ok(OpCode::NOP), 
            "ALU" => Ok(OpCode::ALU), 
            "HALT" => Ok(OpCode::HALT),
            "CALL" => Ok(OpCode::CALL),
            "COPY" => Ok(OpCode::COPY),
            "LI" => Ok(OpCode::LI),  
            "LHI" => Ok(OpCode::LHI), 
            "LB" => Ok(OpCode::LB),  
            "LH" => Ok(OpCode::LH),  
            "LW" => Ok(OpCode::LW),  
            "SB" => Ok(OpCode::SB),  
            "SH" => Ok(OpCode::SH),  
            "SW" => Ok(OpCode::SW),  
            "ADDI" => Ok(OpCode::ADDI),
            "SUBI" => Ok(OpCode::SUBI),
            "MULI" => Ok(OpCode::MULI),
            "DIVI" => Ok(OpCode::DIVI),
            "ANDI" => Ok(OpCode::ANDI),
            "ORI" => Ok(OpCode::ORI), 
            "XORI" => Ok(OpCode::XORI),
            "FLIP" => Ok(OpCode::FLIP),
            "SLLI" => Ok(OpCode::SLLI),
            "SRLI" => Ok(OpCode::SRLI),
            "SRAI" => Ok(OpCode::SRAI),
            "SEQI" => Ok(OpCode::SEQI),
            "SNEI" => Ok(OpCode::SNEI),
            "SLTI" => Ok(OpCode::SLTI),
            "SGTI" => Ok(OpCode::SGTI),
            "SLEI" => Ok(OpCode::SLEI),
            "SGEI" => Ok(OpCode::SGEI),
            "BEZ" => Ok(OpCode::BEZ), 
            "BNZ" => Ok(OpCode::BNZ), 
            "JMP" => Ok(OpCode::JMP), 
            "JL" => Ok(OpCode::JL),  
            "JR" => Ok(OpCode::JR),  
            "JLR" => Ok(OpCode::JLR), 
            "ITOF" => Ok(OpCode::ITOF),
            "FTOI" => Ok(OpCode::FTOI),
            _ => Err(ParseEnumError { value: s.to_string(), enum_name: "OpCode" })
        }
    }
}

impl FromStr for ALUFunct {
    type Err = ParseEnumError;

    fn from_str(s: &str) -> Result<ALUFunct, ParseEnumError> {
        match s {
            "ADD" => Ok(ALUFunct::ADD), 
            "SUB" => Ok(ALUFunct::SUB), 
            "MUL" => Ok(ALUFunct::MUL), 
            "DIV" => Ok(ALUFunct::DIV), 
            "AND" => Ok(ALUFunct::AND), 
            "OR" => Ok(ALUFunct::OR),  
            "XOR" => Ok(ALUFunct::XOR), 
            "SLL" => Ok(ALUFunct::SLL), 
            "SRL" => Ok(ALUFunct::SRL), 
            "SRA" => Ok(ALUFunct::SRA), 
            "SEQ" => Ok(ALUFunct::SEQ), 
            "SNE" => Ok(ALUFunct::SNE), 
            "SLT" => Ok(ALUFunct::SLT), 
            "SGT" => Ok(ALUFunct::SGT), 
            "SLE" => Ok(ALUFunct::SLE), 
            "SGE" => Ok(ALUFunct::SGE), 
            "FADD" => Ok(ALUFunct::FADD),
            "FSUB" => Ok(ALUFunct::FSUB),
            "FMUL" => Ok(ALUFunct::FMUL),
            "FDIV" => Ok(ALUFunct::FDIV),
            _ => Err(ParseEnumError{ value: s.to_string(), enum_name: "ALUFunct" })
        }
    }
}

impl FromStr for RegisterId {
    type Err = ParseEnumError;

    fn from_str(s: &str) -> Result<RegisterId, ParseEnumError> {
        match s {
            "ZERO" => Ok(RegisterId::ZERO),
            "V0" => Ok(RegisterId::V0),
            "V1" => Ok(RegisterId::V1),
            "A0" => Ok(RegisterId::A0),
            "A1" => Ok(RegisterId::A1),
            "A2" => Ok(RegisterId::A2),
            "A3" => Ok(RegisterId::A3),
            "A4" => Ok(RegisterId::A4),
            "T0" => Ok(RegisterId::T0),
            "T1" => Ok(RegisterId::T1),
            "T2" => Ok(RegisterId::T2),
            "T3" => Ok(RegisterId::T3),
            "T4" => Ok(RegisterId::T4),
            "T5" => Ok(RegisterId::T5),
            "T6" => Ok(RegisterId::T6),
            "T7" => Ok(RegisterId::T7),
            "T8" => Ok(RegisterId::T8),
            "T9" => Ok(RegisterId::T9),
            "S0" => Ok(RegisterId::S0),
            "S1" => Ok(RegisterId::S1),
            "S2" => Ok(RegisterId::S2),
            "S3" => Ok(RegisterId::S3),
            "S4" => Ok(RegisterId::S4),
            "S5" => Ok(RegisterId::S5),
            "S6" => Ok(RegisterId::S6),
            "S7" => Ok(RegisterId::S7),
            "S8" => Ok(RegisterId::S8),
            "S9" => Ok(RegisterId::S9),
            "SP" => Ok(RegisterId::SP),
            "FP" => Ok(RegisterId::FP),
            "RM" => Ok(RegisterId::RM),
            "RA" => Ok(RegisterId::RA),
            _ => Err(ParseEnumError { value: s.to_string(), enum_name: "RegisterId" })
        }
    }
}
