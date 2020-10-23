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

pub const OPCODE_OFFSET: u32 = 26;
pub const RD_OFFSET: u32 = 21;
pub const RS1_OFFSET: u32 = 16;
pub const RS2_OFFSET: u32 = 11;
pub const FUNCT_OFFSET: u32 = 0;
pub const IMMEDIATE_OFFSET: u32 = 0;
pub const ADDRESS_OFFSET: u32 = 0;

pub const OPCODE_MASK: u32 = 0b1111_1100_0000_0000_0000_0000_0000_0000;
pub const RD_MASK: u32 = 0b0000_0011_1110_0000_0000_0000_0000_0000;
pub const RS1_MASK: u32 = 0b0000_0000_0001_1111_0000_0000_0000_0000;
pub const RS2_MASK: u32 = 0b0000_0000_0000_0000_1111_1000_0000_0000;
pub const FUNCT_MASK: u32 = 0b0000_0000_0000_0000_0000_0000_0011_1111;
pub const IMMEDIATE_MASK: u32 = 0b0000_0000_0000_0000_1111_1111_1111_1111;
pub const ADDRESS_MASK: u32 = 0b0000_0011_1111_1111_1111_1111_1111_1111;
pub const ADDRESS_SIGN_MASK: u32 = 0b0000_0010_0000_0000_0000_0000_0000_0000;
pub const ADDRESS_EXTENSION: u32 = 0b1111_1100_0000_0000_0000_0000_0000_0000;

pub const REGISTER_COUNT: usize = 32;

pub const LOW_BITS_MASK: u32 = 0x0000_FFFF;
pub const HIGH_BITS_MASK: u32 = 0xFFFF_0000;

#[cfg(test)]
mod test {
    use super::*;
    use crate::RegisterId;
    use num::traits::FromPrimitive;

    const R_FORMAT_UNUSED_WIDTH: u32 = 5;
    const R_FORMAT_UNUSED_MASK: u32 = 0b0000_0000_0000_0000_0000_0111_1100_0000;

    #[test]
    fn opcode_mask_width_matches() {
        assert_eq!(OPCODE_WIDTH, OPCODE_MASK.count_ones());
    }

    #[test]
    fn rd_mask_width_matches() {
        assert_eq!(REG_ID_WIDTH, RD_MASK.count_ones());
    }

    #[test]
    fn rs1_mask_width_matches() {
        assert_eq!(REG_ID_WIDTH, RS1_MASK.count_ones());
    }

    #[test]
    fn rs2_mask_width_matches() {
        assert_eq!(REG_ID_WIDTH, RS2_MASK.count_ones());
    }

    #[test]
    fn funct_mask_width_matches() {
        assert_eq!(FUNCT_WIDTH, FUNCT_MASK.count_ones());
    }

    #[test]
    fn immediate_mask_width_matches() {
        assert_eq!(IMMEDIATE_WIDTH, IMMEDIATE_MASK.count_ones());
    }

    #[test]
    fn address_mask_width_matches() {
        assert_eq!(ADDRESS_WIDTH, ADDRESS_MASK.count_ones());
    }

    #[test]
    fn r_format_width_adds_up() {
        assert_eq!(
            32,
            OPCODE_WIDTH + REG_ID_WIDTH * 3 + FUNCT_WIDTH + R_FORMAT_UNUSED_WIDTH
        );
    }

    #[test]
    fn i_format_width_adds_up() {
        assert_eq!(32, OPCODE_WIDTH + REG_ID_WIDTH * 2 + IMMEDIATE_WIDTH);
    }

    #[test]
    fn j_format_width_adds_up() {
        assert_eq!(32, OPCODE_WIDTH + ADDRESS_WIDTH);
    }

    #[test]
    fn r_format_bitmasks_dont_overlap() {
        assert_eq!(
            0xFFFF_FFFF,
            OPCODE_MASK ^ RD_MASK ^ RS1_MASK ^ RS2_MASK ^ FUNCT_MASK ^ R_FORMAT_UNUSED_MASK
        );
    }

    #[test]
    fn i_format_bitmasks_dont_overlap() {
        assert_eq!(
            0xFFFF_FFFF,
            OPCODE_MASK ^ RD_MASK ^ RS1_MASK ^ IMMEDIATE_MASK
        );
    }

    #[test]
    fn j_format_bitmasks_dont_overlap() {
        assert_eq!(0xFFFF_FFFF, OPCODE_MASK ^ ADDRESS_MASK);
    }

    #[test]
    fn address_sign_mask_and_extension() {
        assert_eq!(0xFFFF_FFFF, ADDRESS_MASK ^ ADDRESS_EXTENSION);
        assert_eq!(0, ADDRESS_SIGN_MASK & ADDRESS_EXTENSION);
        assert_eq!(1, ADDRESS_SIGN_MASK.count_ones());
    }

    #[test]
    fn register_count_matches_enum() {
        for i in 0..REGISTER_COUNT {
            assert!(RegisterId::from_usize(i).is_some());
        }

        assert!(RegisterId::from_usize(REGISTER_COUNT + 1).is_none());
    }
}
