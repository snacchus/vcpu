use byteorder::ByteOrder;

use super::Endian;

pub struct Memory {
    data: Vec<u8>
}

impl Memory {
    pub fn new(size: usize) -> Memory {
        Memory {
            data: vec![0; size]
        }
    }

    fn slice(&self, address: usize, size: usize) -> &[u8] {
        &self.data[address..(address + size)]
    }

    fn mut_slice(&mut self, address: usize, size: usize) -> &mut[u8] {
        &mut self.data[address..(address + size)]
    }

    pub fn check_address(&self, address: usize, size: usize) -> bool {
        address <= (self.data.len() - size)
    }

    pub fn read(&self, address: usize, size: usize) -> Option<u32> {
        assert!(size >= 1 && size <= 4);
        if self.check_address(address, size) {
            Some(Endian::read_uint(self.slice(address, size), size) as u32)
        } else {
            None
        }
    }

    pub fn write(&mut self, address: usize, size: usize, value: u32) -> bool {
        assert!(size >= 1 && size <= 4);
        if self.check_address(address, size) {
            Endian::write_uint(self.mut_slice(address, size), value as u64, size);
            true
        } else {
            false
        }
    }
}
