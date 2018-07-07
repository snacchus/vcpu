use byteorder::ByteOrder;

use super::{constants, Endian};

pub struct Memory {
    data: Vec<u8>
}

impl Memory {
    pub fn new(size: u32) -> Memory {
        Memory {
            data: vec![0; size as usize]
        }
    }

    fn slice(&self, address: u32, size: u32) -> &[u8] {
        &self.data[address as usize..(address + size) as usize]
    }

    fn mut_slice(&mut self, address: u32, size: u32) -> &mut[u8] {
        &mut self.data[address as usize..(address + size) as usize]
    }

    pub fn check_address(&self, address: u32, size: u32) -> bool {
        address <= (self.data.len() as u32 - size)
    }

    pub fn read(&self, address: u32, size: u32) -> Option<u32> {
        assert!(size >= 1 && size <= 4);
        if self.check_address(address, size) {
            Some(Endian::read_uint(self.slice(address, size), size as usize) as u32)
        } else {
            None
        }
    }

    pub fn read_byte(&self, address: u32) -> Option<u8> {
        match self.read(address, constants::BYTE_BYTES) {
            Some(value) => Some(value as u8),
            None => None,
        }
    }

    pub fn read_half(&self, address: u32) -> Option<u16> {
        match self.read(address, constants::HALF_BYTES) {
            Some(value) => Some(value as u16),
            None => None,
        }
    }

    pub fn read_word(&self, address: u32) -> Option<u32> {
        self.read(address, constants::WORD_BYTES)
    }

    pub fn write(&mut self, address: u32, size: u32, value: u32) -> bool {
        assert!(size >= 1 && size <= 4);
        if self.check_address(address, size) {
            Endian::write_uint(self.mut_slice(address, size), value as u64, size as usize);
            true
        } else {
            false
        }
    }
}
