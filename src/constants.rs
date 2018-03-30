use super::{Immediate, Word};
use std::mem;

pub const BYTE_BYTES        : usize = mem::size_of::<u8>();
pub const BYTE_WIDTH        : usize = 8;

pub const HALF_BYTES        : usize = mem::size_of::<u16>();
pub const HALF_WIDTH        : usize = HALF_BYTES * BYTE_WIDTH;

pub const WORD_BYTES        : usize = mem::size_of::<Word>();
pub const WORD_WIDTH        : usize = WORD_BYTES * BYTE_WIDTH;

pub const IMMEDIATE_BYTES   : usize = mem::size_of::<Immediate>();
pub const IMMEDIATE_WIDTH   : usize = IMMEDIATE_BYTES * BYTE_WIDTH;

pub const OPCODE_WIDTH      : usize = 6;
pub const REG_ID_WIDTH      : usize = 5;
pub const ADDRESS_WIDTH     : usize = 26;

pub const OPCODE_MASK       : u32 = 0b11111100000000000000000000000000;
pub const RD_MASK           : u32 = 0b00000011111000000000000000000000;
pub const RS1_MASK          : u32 = 0b00000000000111110000000000000000;
pub const RS2_MASK          : u32 = 0b00000000000000001111100000000000;
pub const OPCODE_R_MASK     : u32 = 0b00000000000000000000000000111111;
pub const IMMEDIATE_MASK    : u32 = 0b00000000000000001111111111111111;
pub const ADDRESS_MASK      : u32 = 0b00000011111111111111111111111111;
pub const ADDRESS_SIGN_MASK : u32 = 0b00000010000000000000000000000000;
pub const ADDRESS_EXTENSION : u32 = 0b11111100000000000000000000000000;

pub const OPCODE_OFFSET     : usize = 26;
pub const RD_OFFSET         : usize = 21;
pub const RS1_OFFSET        : usize = 16;
pub const RS2_OFFSET        : usize = 11;
pub const OPCODE_R_OFFSET   : usize = 0;
pub const IMMEDIATE_OFFSET  : usize = 0;
pub const ADDRESS_OFFSET    : usize = 0;

pub const REGISTER_COUNT    : usize = 32;
