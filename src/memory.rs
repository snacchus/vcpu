use super::{constants, Endian};
use byteorder::ByteOrder;

/// Represents a memory storage unit with basic read and write operations.
pub trait Storage {
    /// Returns the total length of the storage in bytes.
    ///
    /// It is not guaranteed that every address within the range `[0..length]` is accessible.
    /// Use [`check_range`] to test if a certain address range is addressable.
    ///
    /// # Examples
    /// ```
    /// use vcpu::{Storage, Memory};
    ///
    /// let memory = Memory::new(16);
    /// assert_eq!(memory.length(), 16);
    /// ```
    /// [`check_range`]: ./trait.Storage.html#tymethod.check_range
    fn length(&self) -> u32;

    /// Checks whether the range `[address..address+length]` is addressable within the storage.
    ///
    /// # Examples
    /// ```
    /// use vcpu::{Storage, Memory};
    ///
    /// let memory = Memory::new(16);
    /// assert_eq!(memory.check_range(4, 12), true);
    /// assert_eq!(memory.check_range(10, 20), false);
    /// ```
    fn check_range(&self, address: u32, length: u32) -> bool;

    /// Immutably borrows a slice from the address range specified by `address` and `length`.
    ///
    /// # Errors
    /// Returns an error if the range `[address..address+length]` is not addressable.
    ///
    /// # Examples
    /// ```
    /// use vcpu::{Storage, Memory};
    ///
    /// let memory = Memory::from(&[1, 2, 3, 4][..]);
    /// assert_eq!(memory.borrow_slice(1, 2), Ok(&[2, 3][..]));
    /// assert_eq!(memory.borrow_slice(2, 4), Err(()));
    /// ```
    fn borrow_slice(&self, address: u32, length: u32) -> Result<&[u8], ()>;

    /// Mutably borrows a slice from the address range specified by `address` and `length`.
    ///
    /// # Errors
    /// Returns an error if the range `[address..address+length]` is not addressable.
    ///
    /// # Examples
    /// ```
    /// use vcpu::{Storage, Memory};
    ///
    /// let mut memory = Memory::from(&[1, 2, 3, 4][..]);
    /// assert_eq!(memory.borrow_slice_mut(2, 2), Ok(&mut[3, 4][..]));
    /// assert_eq!(memory.borrow_slice_mut(0, 5), Err(()));
    /// ```
    fn borrow_slice_mut(&mut self, address: u32, length: u32) -> Result<&mut [u8], ()>;

    /// Reads the amount of bytes specified by `size` starting at the specified address, and converts the result to an unsigned integer.
    ///
    /// The conversion is always performed with the endianness defined by the [`Endian`] type alias.
    ///
    /// # Panics
    /// Panics if `size < 1` or `size > 4`.
    ///
    /// # Errors
    /// Returns an error if the range `[address..address+length]` is not addressable.
    ///
    /// # Examples
    /// ```
    /// use vcpu::{Storage, Memory};
    ///
    /// let memory = Memory::from(&[1, 2, 3, 4][..]);
    /// assert_eq!(memory.read(1, 3), Ok(262914));
    /// assert_eq!(memory.read(4, 4), Err(()));
    /// ```
    /// [`Endian`]: ../type.Endian.html
    fn read(&self, address: u32, size: u32) -> Result<u32, ()> {
        assert!(size >= 1 && size <= 4);
        Ok(Endian::read_uint(self.borrow_slice(address, size)?, size as usize) as u32)
    }

    /// Reads [`BYTE_BYTES`] bytes starting at the specified address and returns the result as `u8`.
    ///
    /// # Errors
    /// Returns an error if the range `[address..address+BYTE_BYTES]` is not addressable.
    ///
    /// # Examples
    /// ```
    /// use vcpu::{Storage, Memory};
    ///
    /// let memory = Memory::from(&[5, 23, 0, 206][..]);
    /// assert_eq!(memory.read_byte(3), Ok(206));
    /// assert_eq!(memory.read_byte(15), Err(()));
    /// ```
    /// [`BYTE_BYTES`]: ../constants/constant.BYTE_BYTES.html
    fn read_byte(&self, address: u32) -> Result<u8, ()> {
        Ok(self.borrow_slice(address, constants::BYTE_BYTES)?[0])
    }

    /// Reads [`HALF_BYTES`] bytes starting at the specified address and converts the result to `u16`.
    ///
    /// The conversion is always performed with the endianness defined by the [`Endian`] type alias.
    ///
    /// # Errors
    /// Returns an error if the range `[address..address+HALF_BYTES]` is not addressable.
    ///
    /// # Examples
    /// ```
    /// use vcpu::{Storage, Memory};
    ///
    /// let memory = Memory::from(&[5, 23, 0, 206][..]);
    /// assert_eq!(memory.read_half(0), Ok(5893));
    /// assert_eq!(memory.read_half(3), Err(()));
    /// ```
    /// [`HALF_BYTES`]: ../constants/constant.HALF_BYTES.html
    /// [`Endian`]: ../type.Endian.html
    fn read_half(&self, address: u32) -> Result<u16, ()> {
        Ok(Endian::read_u16(
            self.borrow_slice(address, constants::HALF_BYTES)?,
        ))
    }

    /// Reads [`WORD_BYTES`] bytes starting at the specified address and converts the result to `u32`.
    ///
    /// The conversion is always performed with the endianness defined by the [`Endian`] type alias.
    ///
    /// # Errors
    /// Returns an error if the range `[address..address+WORD_BYTES]` is not addressable.
    ///
    /// # Examples
    /// ```
    /// use vcpu::{Storage, Memory};
    ///
    /// let memory = Memory::from(&[5, 23, 0, 206][..]);
    /// assert_eq!(memory.read_word(0), Ok(3456112389));
    /// assert_eq!(memory.read_word(1), Err(()));
    /// ```
    /// [`WORD_BYTES`]: ../constants/constant.WORD_BYTES.html
    /// [`Endian`]: ../type.Endian.html
    fn read_word(&self, address: u32) -> Result<u32, ()> {
        Ok(Endian::read_u32(
            self.borrow_slice(address, constants::WORD_BYTES)?,
        ))
    }

    /// Takes `size` bytes from `value` (starting at the least significant byte) and writes them to the specified `address`.
    ///
    /// # Errors
    /// Returns an error if the range `[address..address+size]` is not addressable.
    ///
    /// # Panics
    /// Panics if `size < 1` or `size > 4`, or if `value` is not representable with `size` bytes.
    ///
    /// # Examples
    /// ```
    /// use vcpu::{Storage, Memory};
    ///
    /// let mut memory = Memory::new(4);
    ///
    /// assert_eq!(memory.write(0, 2, 32938), Ok(()));
    /// assert_eq!(memory.borrow_slice(0, 4), Ok(&[170, 128, 0, 0][..]));
    ///
    /// assert_eq!(memory.write(0, 4, 587226975), Ok(()));
    /// assert_eq!(memory.borrow_slice(0, 4), Ok(&[95, 95, 0, 35][..]));
    /// ```
    fn write(&mut self, address: u32, size: u32, value: u32) -> Result<(), ()> {
        assert!(size >= 1 && size <= 4);
        Endian::write_uint(
            self.borrow_slice_mut(address, size)?,
            u64::from(value),
            size as usize,
        );
        Ok(())
    }

    /// Writes `value` to the specified `address`.
    ///
    /// # Errors
    /// Returns an error if the range `[address..address+BYTE_BYTES]` is not addressable.
    ///
    /// # Examples
    /// ```
    /// use vcpu::{Storage, Memory};
    ///
    /// let mut memory = Memory::new(4);
    /// assert_eq!(memory.write_byte(2, 102), Ok(()));
    /// assert_eq!(memory.borrow_slice(0, 4), Ok(&[0, 0, 102, 0][..]));
    /// assert_eq!(memory.write_byte(4, 224), Err(()));
    /// ```
    /// [`BYTE_BYTES`]: ../constants/constant.BYTE_BYTES.html
    fn write_byte(&mut self, address: u32, value: u8) -> Result<(), ()> {
        self.borrow_slice_mut(address, constants::BYTE_BYTES)?[0] = value;
        Ok(())
    }

    /// Converts `value` to individual bytes and writes them to the specified `address`.
    ///
    /// The conversion is always performed with the endianness defined by the [`Endian`] type alias.
    ///
    /// # Errors
    /// Returns an error if the range `[address..address+HALF_BYTES]` is not addressable.
    ///
    /// # Examples
    /// ```
    /// use vcpu::{Storage, Memory};
    ///
    /// let mut memory = Memory::new(4);
    /// assert_eq!(memory.write_half(1, 5871), Ok(()));
    /// assert_eq!(memory.borrow_slice(0, 4), Ok(&[0, 239, 22, 0][..]));
    /// assert_eq!(memory.write_half(3, 8922), Err(()));
    /// ```
    /// [`HALF_BYTES`]: ../constants/constant.HALF_BYTES.html
    /// [`Endian`]: ../type.Endian.html
    fn write_half(&mut self, address: u32, value: u16) -> Result<(), ()> {
        Endian::write_u16(
            self.borrow_slice_mut(address, constants::HALF_BYTES)?,
            value,
        );
        Ok(())
    }

    /// Converts `value` to individual bytes and writes them to the specified `address`.
    ///
    /// The conversion is always performed with the endianness defined by the [`Endian`] type alias.
    ///
    /// # Errors
    /// Returns an error if the range `[address..address+WORD_BYTES]` is not addressable.
    ///
    /// # Examples
    /// ```
    /// use vcpu::{Storage, Memory};
    ///
    /// let mut memory = Memory::new(4);
    /// assert_eq!(memory.write_word(0, 2568242499), Ok(()));
    /// assert_eq!(memory.borrow_slice(0, 4), Ok(&[67, 69, 20, 153][..]));
    /// assert_eq!(memory.write_word(1, 2220885), Err(()));
    /// ```
    /// [`WORD_BYTES`]: ../constants/constant.WORD_BYTES.html
    /// [`Endian`]: ../type.Endian.html
    fn write_word(&mut self, address: u32, value: u32) -> Result<(), ()> {
        Endian::write_u32(
            self.borrow_slice_mut(address, constants::WORD_BYTES)?,
            value,
        );
        Ok(())
    }
}

/// A basic default implementation of the `Storage` trait, which provides a contiguous block of memory.
pub struct Memory {
    data: Vec<u8>,
}

impl Memory {
    /// Constructs a new `Memory` object with the specified `length`.
    ///
    /// # Examples
    /// ```
    /// use vcpu::{Storage, Memory};
    ///
    /// let memory = Memory::new(16);
    /// assert_eq!(memory.length(), 16);
    /// ```
    pub fn new(length: u32) -> Memory {
        Memory {
            data: vec![0; length as usize],
        }
    }

    /// Returns the internal block of memory as an immutable slice.
    ///
    /// # Examples
    /// ```
    /// use vcpu::{Storage, Memory};
    ///
    /// let memory = Memory::from(&[3, 10, 4, 2][..]);
    /// assert_eq!(memory.data(), &[3, 10, 4, 2][..]);
    /// ```
    pub fn data(&self) -> &[u8] {
        &self.data[..]
    }
}

impl Storage for Memory {
    fn length(&self) -> u32 {
        self.data.len() as u32
    }

    fn check_range(&self, address: u32, length: u32) -> bool {
        let len = self.data.len() as u32;
        address <= len && (address + length) <= len
    }

    fn borrow_slice(&self, address: u32, length: u32) -> Result<&[u8], ()> {
        if self.check_range(address, length) {
            Ok(&self.data[address as usize..(address + length) as usize])
        } else {
            Err(())
        }
    }

    fn borrow_slice_mut(&mut self, address: u32, length: u32) -> Result<&mut [u8], ()> {
        if self.check_range(address, length) {
            Ok(&mut self.data[address as usize..(address + length) as usize])
        } else {
            Err(())
        }
    }
}

impl<'a> From<&'a [u8]> for Memory {
    fn from(s: &'a [u8]) -> Memory {
        Memory { data: Vec::from(s) }
    }
}

pub mod composite;
