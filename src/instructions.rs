use crate::{constants, Address, Immediate, Word};
use num::traits::ToPrimitive;
use num_derive::{FromPrimitive, ToPrimitive};
use util::{EnumFromStr, InteropGetName};
use util_derive::{EnumFromStr, InteropGetName};

/// Processor instruction set.
///
/// Instruction Formats:
///
/// | Format   | Bits 31-26 | Bits 25-21 | Bits 20-16 | Bits 15-11 | Bits 10-6 | Bits 5-0  |
/// |----------|------------|------------|------------|------------|-----------|-----------|
/// | R-Format | opcode     | Rd         | Rs1        | Rs2        | unused    | funct     |
/// | I-Format | opcode     | Rd         | Rs1        | immediate  | immediate | immediate |
/// | J-Format | opcode     | address    | address    | address    | address   | address   |
#[derive(
    Clone, Copy, PartialEq, Eq, Debug, ToPrimitive, FromPrimitive, InteropGetName, EnumFromStr,
)]
pub enum Opcode {
    /// No-op.
    ///
    /// Format: `I`.
    /// Has no effect.
    NOP,
    /// Arithmetic logic unit.
    ///
    /// Format: `R`.
    /// Performs arithmetic or logic operation specified by `funct` (see [`AluFunct`](enum.AluFunct.html)).
    ALU,
    /// Halt.
    ///
    /// Format: `I`.
    /// Stops execution of the current program.
    HALT,
    /// Currently not used.
    CALL,
    /// Copy.
    ///
    /// Format: `I`.
    /// Copies register value from `Rs1` to `Rd`.
    COPY,
    /// Load immediate.
    ///
    /// Format: `I`.
    /// Sets `Rd` to the value of `immediate` (sign bit is extended).
    LI,
    /// Load high immediate.
    /// Format: `I`.
    /// Sets `Rd` to the value of `immediate << 16`.
    LHI,
    /// Set low bits.
    ///
    /// Format: `I`.
    /// Sets (only) the low 16 bits of `Rd` to `immediate`.
    SLO,
    /// Set high bits.
    ///
    /// Format: `I`.
    /// Sets (only) the high 16 bits of `Rd` to `immediate`.
    SHI,
    /// Load byte.
    ///
    /// Format: `I`.
    /// Sets `Rd` to an 8 bit value loaded from memory at address `Rs1 + immedate`.
    LB,
    /// Load half.
    ///
    /// Format: `I`.
    /// Sets `Rd` to an 16 bit value loaded from memory at address `Rs1 + immedate`.
    LH,
    /// Load word.
    ///
    /// Format: `I`.
    /// Sets `Rd` to an 32 bit value loaded from memory at address `Rs1 + immedate`.
    LW,
    /// Store byte.
    ///
    /// Format: `I`.
    /// Writes `Rd` truncated to 8 bits to memory at address `Rs1 + immedate`.
    SB,
    /// Store half.
    ///
    /// Format: `I`.
    /// Writes `Rd` truncated to 16 bits to memory at address `Rs1 + immedate`.
    SH,
    /// Store word.
    ///
    /// Format: `I`.
    /// Writes `Rd` to memory at address `Rs1 + immedate`.
    SW,
    /// Add immediate.
    ///
    /// Format: `I`.
    /// Sets `Rd` to `Rs1 + immediate`.
    ADDI,
    /// Subtract immediate.
    ///
    /// Format: `I`.
    /// Sets `Rd` to `Rs1 - immediate`.
    SUBI,
    /// Multiply immediate.
    ///
    /// Format: `I`.
    /// Sets `Rd` to `Rs1 * immediate`, sets register `RM` to high 32 bits of the product.
    MULI,
    /// Divide immediate.
    ///
    /// Format: `I`.
    /// Sets `Rd` to `Rs1 / immediate`, sets register `RM` to remainder.
    DIVI,
    /// And immediate.
    ///
    /// Format: `I`.
    /// Sets `Rd` to `Rs1 & immediate`.
    ANDI,
    /// Or immediate.
    ///
    /// Format: `I`.
    /// Sets `Rd` to `Rs1 | immediate`.
    ORI,
    /// Exclusive-or immediate.
    ///
    /// Format: `I`.
    /// Sets `Rd` to `Rs1 ^ immediate`.
    XORI,
    /// Flip.
    ///
    /// Format: `I`.
    /// Sets `Rd` to `~Rs1`.
    FLIP,
    /// Shift left logical immediate.
    ///
    /// Format: `I`.
    /// Sets `Rd` to `Rs1 << immediate`.
    SLLI,
    /// Shift right logical immediate.
    ///
    /// Format: `I`.
    /// Sets `Rd` to `Rs1 >> immediate` (inserting zeros).
    SRLI,
    /// Shift right arithmetic immediate.
    ///
    /// Format: `I`.
    /// Sets `Rd` to `Rs1 >> immediate` (inserting sign bit).
    SRAI,
    /// Set if equal immediate.
    ///
    /// Format: `I`.
    /// Sets `Rd` to `1` if `Rs1 == immediate` and to `0` otherwise.
    SEQI,
    /// Set if not equal immediate.
    ///
    /// Format: `I`.
    /// Sets `Rd` to `1` if `Rs1 != immediate` and to `0` otherwise.
    SNEI,
    /// Set if less than immediate.
    ///
    /// Format: `I`.
    /// Sets `Rd` to `1` if `Rs1 < immediate` and to `0` otherwise.
    SLTI,
    /// Set if greater than immediate.
    ///
    /// Format: `I`.
    /// Sets `Rd` to `1` if `Rs1 > immediate` and to `0` otherwise.
    SGTI,
    /// Set if less equal immediate.
    ///
    /// Format: `I`.
    /// Sets `Rd` to `1` if `Rs1 <= immediate` and to `0` otherwise.
    SLEI,
    /// Set if greater equal immediate.
    ///
    /// Format: `I`.
    /// Sets `Rd` to `1` if `Rs1 >= immediate` and to `0` otherwise.
    SGEI,
    /// Set if less than unsigned immediate.
    ///
    /// Format: `I`.
    /// Sets `Rd` to `1` if `Rs1 < immediate` and to `0` otherwise (using unsigned arithmetic).
    SLTUI,
    /// Set if greater than unsigned immediate.
    ///
    /// Format: `I`.
    /// Sets `Rd` to `1` if `Rs1 > immediate` and to `0` otherwise (using unsigned arithmetic).
    SGTUI,
    /// Set if less or equal unsigned immediate.
    ///
    /// Format: `I`.
    /// Sets `Rd` to `1` if `Rs1 <= immediate` and to `0` otherwise (using unsigned arithmetic).
    SLEUI,
    /// Set if greater or equal unsigned immediate.
    ///
    /// Format: `I`.
    /// Sets `Rd` to `1` if `Rs1 >= immediate` and to `0` otherwise (using unsigned arithmetic).
    SGEUI,
    /// Branch if zero.
    ///
    /// Format: `I`.
    /// If `Rs1 == 0`, adds `immediate` to program counter.
    BEZ,
    /// Branch if not zero.
    ///
    /// Format: `I`.
    /// If `Rs1 != 0`, adds `immediate` to program counter.
    BNZ,
    /// Jump.
    ///
    /// Format: `J`.
    /// Adds `address` to program counter.
    JMP,
    /// Jump and link.
    ///
    /// Format: `J`.
    /// Adds `address` to program counter and sets register `RA` to the next instructions' address.
    JL,
    /// Jump register.
    ///
    /// Format: `I`.
    /// Sets program counter to value of `Rs1`.
    JR,
    /// Jump and link register.
    ///
    /// Format: `I`.
    /// Sets program counter to value of `Rs1` and sets register `RA` to the next instructions' address.
    JLR,
    /// Int to float.
    ///
    /// Format: `I`.
    /// Sets `Rd` to `Rs1` converted from integer to a IEEE 754 floating point value.
    ITOF,
    /// Float to int.
    ///
    /// Format: `I`.
    /// Sets `Rd` to `Rs1` converted to from IEEE 754 floating point value to integer.
    FTOI,
    /// Floating point operation.
    ///
    /// Format: `I`.
    /// Performs floating point operation specified by `funct` (see [`FlopFunct`](enum.FlopFunct.html)).
    FLOP,
}

/// List of functions used by the [`Opcode::ALU`](enum.Opcode.html#variant.ALU) instruction.
#[derive(
    Clone, Copy, PartialEq, Eq, FromPrimitive, ToPrimitive, Debug, InteropGetName, EnumFromStr,
)]
pub enum AluFunct {
    /// Add.
    ///
    /// Sets `Rd` to `Rs1 + Rs2`.
    ADD,
    /// Subtract.
    ///
    /// Sets `Rd` to `Rs1 - Rs2`.
    SUB,
    /// Multiply.
    ///
    /// Sets `Rd` to `Rs1 * Rs2` and sets register `RM` to high 32 bits of the product.
    MUL,
    /// Subtract.
    ///
    /// Sets `Rd` to `Rs1 / Rs2` and sets register `RM` to the remainder.
    DIV,
    /// And.
    ///
    /// Sets `Rd` to `Rs1 & Rs2`.
    AND,
    /// Or.
    ///
    /// Sets `Rd` to `Rs1 | Rs2`.
    OR,
    /// Exclusive-Or.
    ///
    /// Sets `Rd` to `Rs1 ^ Rs2`.
    XOR,
    /// Shift left logical.
    ///
    /// Sets `Rd` to `Rs1 << Rs2`.
    SLL,
    /// Shift right logical.
    ///
    /// Sets `Rd` to `Rs1 >> Rs2` (inserting zeros).
    SRL,
    /// Shift right arithmetic.
    ///
    /// Sets `Rd` to `Rs1 >> Rs2` (inserting sign bit).
    SRA,
    /// Set if equal.
    ///
    /// Sets `Rd` to `1` if `Rs1 == Rs1` and to `0` otherwise.
    SEQ,
    /// Set if not equal.
    ///
    /// Sets `Rd` to `1` if `Rs1 != Rs1` and to `0` otherwise.
    SNE,
    /// Set if less than.
    ///
    /// Sets `Rd` to `1` if `Rs1 < Rs1` and to `0` otherwise.
    SLT,
    /// Set if greater than.
    ///
    /// Sets `Rd` to `1` if `Rs1 > Rs1` and to `0` otherwise.
    SGT,
    /// Set if less equal.
    ///
    /// Sets `Rd` to `1` if `Rs1 <= Rs1` and to `0` otherwise.
    SLE,
    /// Set if greater equal.
    ///
    /// Sets `Rd` to `1` if `Rs1 >= Rs1` and to `0` otherwise.
    SGE,
    /// Set if less than unsigned.
    ///
    /// Sets `Rd` to `1` if `Rs1 < Rs1` and to `0` otherwise (using unsigned arithmetic).
    SLTU,
    /// Set if greater than unsigned.
    ///
    /// Sets `Rd` to `1` if `Rs1 > Rs1` and to `0` otherwise (using unsigned arithmetic).
    SGTU,
    /// Set if less equal unsigned.
    ///
    /// Sets `Rd` to `1` if `Rs1 <= Rs1` and to `0` otherwise (using unsigned arithmetic).
    SLEU,
    /// Set if greater equal unsigned.
    ///
    /// Sets `Rd` to `1` if `Rs1 >= Rs1` and to `0` otherwise (using unsigned arithmetic).
    SGEU,
}

// TODO: add more float operations

/// List of functions used by the [`Opcode::FLOP`](enum.Opcode.html#variant.FLOP) instruction.
#[derive(
    Clone, Copy, PartialEq, Eq, FromPrimitive, ToPrimitive, Debug, InteropGetName, EnumFromStr,
)]
pub enum FlopFunct {
    /// Float add.
    ///
    /// Sets `Rd` to `Rs1 + Rs2` using IEEE 754 floats.
    FADD,
    /// Float subtract.
    ///
    /// Sets `Rd` to `Rs1 - Rs2` using IEEE 754 floats.
    FSUB,
    /// Float multiply.
    ///
    /// Sets `Rd` to `Rs1 * Rs2` using IEEE 754 floats.
    FMUL,
    /// Float divide.
    ///
    /// Sets `Rd` to `Rs1 / Rs2` using IEEE 754 floats.
    FDIV,
}

/// List of available registers.
#[derive(
    Clone, Copy, PartialEq, Eq, FromPrimitive, ToPrimitive, Debug, InteropGetName, EnumFromStr,
)]
pub enum RegisterId {
    /// Always zero (read only).
    ZERO,
    /// Value 0.
    V0,
    /// Value 1.
    V1,
    /// Argument 0.
    A0,
    /// Argument 1.
    A1,
    /// Argument 2.
    A2,
    /// Argument 3.
    A3,
    /// Argument 4.
    A4,
    /// Temporary 0.
    T0,
    /// Temporary 1.
    T1,
    /// Temporary 2.
    T2,
    /// Temporary 3.
    T3,
    /// Temporary 4.
    T4,
    /// Temporary 5.
    T5,
    /// Temporary 6.
    T6,
    /// Temporary 7.
    T7,
    /// Temporary 8.
    T8,
    /// Temporary 9.
    T9,
    /// Saved 0.
    S0,
    /// Saved 1.
    S1,
    /// Saved 2.
    S2,
    /// Saved 3.
    S3,
    /// Saved 4.
    S4,
    /// Saved 5.
    S5,
    /// Saved 6.
    S6,
    /// Saved 7.
    S7,
    /// Saved 8.
    S8,
    /// Saved 9.
    S9,
    /// Stack pointer.
    SP,
    /// Frame pointer.
    FP,
    /// Remainder.
    RM,
    /// Return address.
    RA,
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

impl_enum_display!(Opcode);
impl_enum_display!(AluFunct);
impl_enum_display!(FlopFunct);
impl_enum_display!(RegisterId);

#[inline]
pub fn register_index(id: RegisterId) -> usize {
    enum_to_u32(id) as usize
}

/// Constructs a R-format instruction.
#[inline]
pub fn make_r_instruction(
    oc: Opcode,
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

/// Constructs a R-format instruction.
#[macro_export]
macro_rules! instr_r {
    ($opcode:ident, $rd:ident, $rs1:ident, $rs2:ident, $funct:expr) => {
        make_r_instruction(
            Opcode::$opcode,
            RegisterId::$rd,
            RegisterId::$rs1,
            RegisterId::$rs2,
            $funct,
        )
    };
}

/// Constructs an ALU instruction.
pub fn make_alu_instruction(
    funct: AluFunct,
    rd: RegisterId,
    rs1: RegisterId,
    rs2: RegisterId,
) -> Word {
    make_r_instruction(Opcode::ALU, rd, rs1, rs2, enum_to_u32(funct))
}

/// Constructs an ALU instruction.
#[macro_export]
macro_rules! instr_alu {
    ($funct:ident, $rd:ident, $rs1:ident, $rs2:ident) => {
        make_alu_instruction(
            AluFunct::$funct,
            RegisterId::$rd,
            RegisterId::$rs1,
            RegisterId::$rs2,
        )
    };
}

/// Constructs a FLOP instruction.
pub fn make_flop_instruction(
    funct: FlopFunct,
    rd: RegisterId,
    rs1: RegisterId,
    rs2: RegisterId,
) -> Word {
    make_r_instruction(Opcode::FLOP, rd, rs1, rs2, enum_to_u32(funct))
}

/// Constructs a FLOP instruction.
#[macro_export]
macro_rules! instr_flop {
    ($funct:ident, $rd:ident, $rs1:ident, $rs2:ident) => {
        make_flop_instruction(
            FlopFunct::$funct,
            RegisterId::$rd,
            RegisterId::$rs1,
            RegisterId::$rs2,
        )
    };
}

/// Constructs an I-format instruction.
#[inline]
pub fn make_i_instruction(
    oc: Opcode,
    rd: RegisterId,
    rs1: RegisterId,
    immediate: Immediate,
) -> Word {
    ((enum_to_u32(oc) << constants::OPCODE_OFFSET) & constants::OPCODE_MASK)
        | ((enum_to_u32(rd) << constants::RD_OFFSET) & constants::RD_MASK)
        | ((enum_to_u32(rs1) << constants::RS1_OFFSET) & constants::RS1_MASK)
        | (((immediate as u32) << constants::IMMEDIATE_OFFSET) & constants::IMMEDIATE_MASK)
}

/// Constructs an I-format instruction.
#[macro_export]
macro_rules! instr_i {
    ($opcode:ident, $rd:ident, $rs1:ident, $imm:expr) => {
        make_i_instruction(Opcode::$opcode, RegisterId::$rd, RegisterId::$rs1, $imm)
    };
}

/// Constructs a J-format instruction.
#[inline]
pub fn make_j_instruction(oc: Opcode, address: Address) -> Word {
    ((enum_to_u32(oc) << constants::OPCODE_OFFSET) & constants::OPCODE_MASK)
        | (((address as u32) << constants::ADDRESS_OFFSET) & constants::ADDRESS_MASK)
}

/// Constructs a J-format instruction.
#[macro_export]
macro_rules! instr_j {
    ($opcode:ident, $addr:expr) => {
        make_j_instruction(Opcode::$opcode, $addr)
    };
}

/// Constructs a [`Opcode::NOP`](enum.Opcode.html#variant.NOP) instruction.
#[macro_export]
macro_rules! nop {
    () => {
        instr_i!(NOP, ZERO, ZERO, 0)
    };
}
