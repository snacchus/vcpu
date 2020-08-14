use crate::{constants, Address, Immediate, Word};
use num::traits::ToPrimitive;
use num_derive::{FromPrimitive, ToPrimitive};
use util::{EnumFromStr, InteropGetName};
use util_derive::{EnumFromStr, InteropGetName};

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

// TODO: write proper doc comments

#[derive(
    Clone, Copy, PartialEq, Eq, Debug, ToPrimitive, FromPrimitive, InteropGetName, EnumFromStr,
)]
pub enum OpCode {
    //  Mnemonic    | Name                            | Format | Effect
    //--------------+---------------------------------+--------+-------------------------------------------------
    // Misc         |                                 |        |
    NOP,  // | No-op                           | I      | Does nothing
    ALU,  // | ALU-instruction                 | R      | Executes ALU function denoted by <funct>
    HALT, // | Halt                            | I      | Stops the CPU
    CALL, // | System call                     | I      | Calls system function with code <immediate> and argument <Rs1>
    //--------------+---------------------------------+--------+-------------------------------------------------
    // I/O          |                                 |        |
    COPY, // | Copy                            | I      | Rd = Rs1
    LI,   // | Load immediate                  | I      | Rd = extend(immediate)
    LHI,  // | Load immediate                  | I      | Rd = immediate << 16
    SLO,  // | Set low bits                    | I      | Rd[0..15] = immediate
    SHI,  // | Set high bits                   | I      | Rd[16..31] = immediate
    LB,   // | Load byte                       | I      | Rd = MEM[Rs1 + extend(immediate)]
    LH,   // | Load half word                  | I      | Rd = MEM[Rs1 + extend(immediate)]
    LW,   // | Load word                       | I      | Rd = MEM[Rs1 + extend(immediate)]
    SB,   // | Store byte                      | I      | MEM[Rs1 + extend(immediate)] = Rd
    SH,   // | Store half word                 | I      | MEM[Rs1 + extend(immediate)] = Rd
    SW,   // | Store word                      | I      | MEM[Rs1 + extend(immediate)] = Rd
    //--------------+---------------------------------+--------+-------------------------------------------------
    // Arithmetic   |                                 |        |
    ADDI, // | Add immediate                   | I      | Rd = Rs1 + extend(immediate)
    SUBI, // | Subtract immediate              | I      | Rd = Rs1 - extend(immediate)
    MULI, // | Multiply immediate              | I      | Rd = Rs1 * extend(immediate); RM = high bits
    DIVI, // | Divide immediate                | I      | Rd = Rs1 / extend(immediate); RM = remainder
    //--------------+---------------------------------+--------+-------------------------------------------------
    // Logic        |                                 |        |
    ANDI, // | And immediate                   | I      | Rd = Rs1 & extend(immediate)
    ORI,  // | Or immediate                    | I      | Rd = Rs1 | extend(immediate)
    XORI, // | Exclusive-Or immediate          | I      | Rd = Rs1 ^ extend(immediate)
    FLIP, // | Flip                            | I      | Rd = ~Rs1
    //--------------+---------------------------------+--------+-------------------------------------------------
    // Shifts       |                                 |        |
    SLLI, // | Shift left logical immediate    | I      | Rd = Rs1 << immediate
    SRLI, // | Shift right logical immediate   | I      | Rd = Rs1 >> immediate (inserting zeros)
    SRAI, // | Shift right arithmetic immediate| I      | Rd = Rs1 >> immediate (inserting sign-bit)
    //--------------+---------------------------------+--------+-------------------------------------------------
    // Comparisons  |                                 |        |
    SEQI, // | Set if equal immediate          | I      | Rd = (Rs1 == extend(immediate)) ? 1 : 0
    SNEI, // | Set if not equal immediate      | I      | Rd = (Rs1 != extend(immediate)) ? 1 : 0
    SLTI, // | Set if less than immediate      | I      | Rd = (Rs1 < extend(immediate)) ? 1 : 0
    SGTI, // | Set if greater than immediate   | I      | Rd = (Rs1 > extend(immediate)) ? 1 : 0
    SLEI, // | Set if less equal immediate     | I      | Rd = (Rs1 <= extend(immediate)) ? 1 : 0
    SGEI, // | Set if greater equal immediate  | I      | Rd = (Rs1 >= extend(immediate)) ? 1 : 0
    SLTUI,
    SGTUI,
    SLEUI,
    SGEUI,
    //--------------+---------------------------------+--------+-------------------------------------------------
    // Branching    |                                 |        |
    BEZ, // | Branch if zero                  | I      | PC += ((Rs1 == 0) ? extend(immediate) : 0)
    BNZ, // | Branch if not zero              | I      | PC += ((Rs1 != 0) ? extend(immediate) : 0)
    //--------------+---------------------------------+--------+-------------------------------------------------
    // Jumping      |                                 |        |
    JMP, // | Jump                            | J      | PC += extend(address)
    JL,  // | Jump and link                   | J      | RA = PC + 4; PC += extend(address)
    JR,  // | Jump register                   | I      | PC = Rs1
    JLR, // | Jump and link register          | I      | RA = PC + 4; PC = Rs1
    //--------------+---------------------------------+--------+-------------------------------------------------
    // Float conv.  |                                 |        |
    ITOF, // | Int to float                    | I      | Rd = float(Rs1)
    FTOI, // | Float to int                    | I      | Rd = int(Rs1)
    //--------------+---------------------------------+--------+-------------------------------------------------
    FLOP,
}

#[derive(
    Clone, Copy, PartialEq, Eq, FromPrimitive, ToPrimitive, Debug, InteropGetName, EnumFromStr,
)]
pub enum ALUFunct {
    //--------------+---------------------------------+--------+-------------------------------------------------
    // Arithmetic   |                                 |        |
    ADD, // | Add                             | R      | Rd = Rs1 + Rs2
    SUB, // | Subtract                        | R      | Rd = Rs1 - Rs2
    MUL, // | Multiply                        | R      | Rd = Rs1 * Rs2; RM = high bits
    DIV, // | Divide                          | R      | Rd = Rs1 / Rs2; RM = remainder
    //--------------+---------------------------------+--------+-------------------------------------------------
    // Logic        |                                 |        |
    AND, // | And                             | R      | Rd = Rs1 & Rs2
    OR,  // | Or                              | R      | Rd = Rs1 | Rs2
    XOR, // | Exclusive-Or                    | R      | Rd = Rs1 ^ Rs2
    //--------------+---------------------------------+--------+-------------------------------------------------
    // Shifts       |                                 |        |
    SLL, // | Shift left logical              | R      | Rd = Rs1 << Rs2
    SRL, // | Shift right logical             | R      | Rd = Rs1 >> Rs2 (inserting zeros)
    SRA, // | Shift right arithmetic          | R      | Rd = Rs1 >> Rs2 (inserting sign-bit)
    //--------------+---------------------------------+--------+-------------------------------------------------
    // Comparisons  |                                 |        |
    SEQ, // | Set if equal                    | R      | Rd = (Rs1 == Rs2) ? 1 : 0
    SNE, // | Set if not equal                | R      | Rd = (Rs1 != Rs2) ? 1 : 0
    SLT, // | Set if less than                | R      | Rd = (Rs1 < Rs2) ? 1 : 0
    SGT, // | Set if greater than             | R      | Rd = (Rs1 > Rs2) ? 1 : 0
    SLE, // | Set if less equal               | R      | Rd = (Rs1 <= Rs2) ? 1 : 0
    SGE, // | Set if greater equal            | R      | Rd = (Rs1 >= Rs2) ? 1 : 0
    SLTU,
    SGTU,
    SLEU,
    SGEU, //--------------+---------------------------------+--------+-------------------------------------------------
}

// TODO: add more float operations
#[derive(
    Clone, Copy, PartialEq, Eq, FromPrimitive, ToPrimitive, Debug, InteropGetName, EnumFromStr,
)]
pub enum FLOPFunct {
    //--------------+---------------------------------+--------+-------------------------------------------------
    // Float Arithm.|                                 |        |
    FADD, // | Float add                       | R      | Rd = Rs1 + Rs2 (using IEEE 754 floats)
    FSUB, // | Float subtract                  | R      | Rd = Rs1 - Rs2 (using IEEE 754 floats)
    FMUL, // | Float multiply                  | R      | Rd = Rs1 * Rs2 (using IEEE 754 floats)
    FDIV, // | Float divide                    | R      | Rd = Rs1 / Rs2 (using IEEE 754 floats)
}

#[derive(
    Clone, Copy, PartialEq, Eq, FromPrimitive, ToPrimitive, Debug, InteropGetName, EnumFromStr,
)]
pub enum RegisterId {
    ZERO, // Always zero (read only)

    V0, // Value 0
    V1, // Value 1

    A0, // Argument 0
    A1, // Argument 1
    A2, // Argument 2
    A3, // Argument 3
    A4, // Argument 4

    T0, // Temporary 0
    T1, // Temporary 1
    T2, // Temporary 2
    T3, // Temporary 3
    T4, // Temporary 4
    T5, // Temporary 5
    T6, // Temporary 6
    T7, // Temporary 7
    T8, // Temporary 8
    T9, // Temporary 9

    S0, // Saved 0
    S1, // Saved 1
    S2, // Saved 2
    S3, // Saved 3
    S4, // Saved 4
    S5, // Saved 5
    S6, // Saved 6
    S7, // Saved 7
    S8, // Saved 8
    S9, // Saved 9

    SP, // Stack pointer
    FP, // Frame pointer

    RM, // Contains remainder after integer division or high bits after multiplication
    RA, // Contains return address after jump and link
}

#[inline]
pub fn enum_to_u32<T: ToPrimitive + Copy>(val: T) -> u32 {
    val.to_u32().unwrap()
}

macro_rules! impl_enum_display {
    ($e:ty) => {
        impl std::fmt::Display for $e {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                std::fmt::Debug::fmt(self, f)
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
pub fn make_r_instruction(
    oc: OpCode,
    rd: RegisterId,
    rs1: RegisterId,
    rs2: RegisterId,
    funct: u32,
) -> Word {
    ((enum_to_u32(oc) << constants::OPCODE_OFFSET) & constants::OPCODE_MASK)
        | ((enum_to_u32(rd) << constants::RD_OFFSET) & constants::RD_MASK)
        | ((enum_to_u32(rs1) << constants::RS1_OFFSET) & constants::RS1_MASK)
        | ((enum_to_u32(rs2) << constants::RS2_OFFSET) & constants::RS2_MASK)
        | ((funct << constants::FUNCT_OFFSET) & constants::FUNCT_MASK)
}

#[macro_export]
macro_rules! instr_r {
    ($opcode:ident, $rd:ident, $rs1:ident, $rs2:ident, $funct:expr) => {
        make_r_instruction(
            OpCode::$opcode,
            RegisterId::$rd,
            RegisterId::$rs1,
            RegisterId::$rs2,
            $funct,
        )
    };
}

pub fn make_alu_instruction(
    funct: ALUFunct,
    rd: RegisterId,
    rs1: RegisterId,
    rs2: RegisterId,
) -> Word {
    make_r_instruction(OpCode::ALU, rd, rs1, rs2, enum_to_u32(funct))
}

#[macro_export]
macro_rules! instr_alu {
    ($funct:ident, $rd:ident, $rs1:ident, $rs2:ident) => {
        make_alu_instruction(
            ALUFunct::$funct,
            RegisterId::$rd,
            RegisterId::$rs1,
            RegisterId::$rs2,
        )
    };
}

pub fn make_flop_instruction(
    funct: FLOPFunct,
    rd: RegisterId,
    rs1: RegisterId,
    rs2: RegisterId,
) -> Word {
    make_r_instruction(OpCode::FLOP, rd, rs1, rs2, enum_to_u32(funct))
}

#[macro_export]
macro_rules! instr_flop {
    ($funct:ident, $rd:ident, $rs1:ident, $rs2:ident) => {
        make_flop_instruction(
            FLOPFunct::$funct,
            RegisterId::$rd,
            RegisterId::$rs1,
            RegisterId::$rs2,
        )
    };
}

#[inline]
pub fn make_i_instruction(
    oc: OpCode,
    rd: RegisterId,
    rs1: RegisterId,
    immediate: Immediate,
) -> Word {
    ((enum_to_u32(oc) << constants::OPCODE_OFFSET) & constants::OPCODE_MASK)
        | ((enum_to_u32(rd) << constants::RD_OFFSET) & constants::RD_MASK)
        | ((enum_to_u32(rs1) << constants::RS1_OFFSET) & constants::RS1_MASK)
        | (((immediate as u32) << constants::IMMEDIATE_OFFSET) & constants::IMMEDIATE_MASK)
}

#[macro_export]
macro_rules! instr_i {
    ($opcode:ident, $rd:ident, $rs1:ident, $imm:expr) => {
        make_i_instruction(OpCode::$opcode, RegisterId::$rd, RegisterId::$rs1, $imm)
    };
}

#[inline]
pub fn make_j_instruction(oc: OpCode, address: Address) -> Word {
    ((enum_to_u32(oc) << constants::OPCODE_OFFSET) & constants::OPCODE_MASK)
        | (((address as u32) << constants::ADDRESS_OFFSET) & constants::ADDRESS_MASK)
}

#[macro_export]
macro_rules! instr_j {
    ($opcode:ident, $addr:expr) => {
        make_j_instruction(OpCode::$opcode, $addr)
    };
}

#[macro_export]
macro_rules! nop {
    () => {
        instr_i!(NOP, ZERO, ZERO, 0)
    };
}
