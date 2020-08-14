use super::{Immediate, Word};
use std::mem;

pub const BYTE_BYTES: u32 = mem::size_of::<u8>() as u32;
pub const BYTE_WIDTH: u32 = 8;

pub const HALF_BYTES: u32 = mem::size_of::<u16>() as u32;
pub const HALF_WIDTH: u32 = HALF_BYTES * BYTE_WIDTH;

pub const WORD_BYTES: u32 = mem::size_of::<Word>() as u32;
pub const WORD_WIDTH: u32 = WORD_BYTES * BYTE_WIDTH;

pub const IMMEDIATE_BYTES: u32 = mem::size_of::<Immediate>() as u32;
pub const IMMEDIATE_WIDTH: u32 = IMMEDIATE_BYTES * BYTE_WIDTH;

pub const OPCODE_WIDTH: u32 = 6;
pub const REG_ID_WIDTH: u32 = 5;
pub const FUNCT_WIDTH: u32 = 6;
pub const ADDRESS_WIDTH: u32 = 26;

pub const OPCODE_MASK: u32 = 0b1111_1100_0000_0000_0000_0000_0000_0000;
pub const RD_MASK: u32 = 0b0000_0011_1110_0000_0000_0000_0000_0000;
pub const RS1_MASK: u32 = 0b0000_0000_0001_1111_0000_0000_0000_0000;
pub const RS2_MASK: u32 = 0b0000_0000_0000_0000_1111_1000_0000_0000;
pub const FUNCT_MASK: u32 = 0b0000_0000_0000_0000_0000_0000_0011_1111;
pub const IMMEDIATE_MASK: u32 = 0b0000_0000_0000_0000_1111_1111_1111_1111;
pub const ADDRESS_MASK: u32 = 0b0000_0011_1111_1111_1111_1111_1111_1111;
pub const ADDRESS_SIGN_MASK: u32 = 0b0000_0010_0000_0000_0000_0000_0000_0000;
pub const ADDRESS_EXTENSION: u32 = 0b1111_1100_0000_0000_0000_0000_0000_0000;

pub const OPCODE_OFFSET: u32 = 26;
pub const RD_OFFSET: u32 = 21;
pub const RS1_OFFSET: u32 = 16;
pub const RS2_OFFSET: u32 = 11;
pub const FUNCT_OFFSET: u32 = 0;
pub const IMMEDIATE_OFFSET: u32 = 0;
pub const ADDRESS_OFFSET: u32 = 0;

// TODO: generate this automatically using derive macro (enum variant count)
pub const REGISTER_COUNT: usize = 32;

pub const LOW_BITS_MASK: u32 = 0x0000_FFFF;
pub const HIGH_BITS_MASK: u32 = 0xFFFF_0000;
