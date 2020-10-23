mod constants;
mod instructions;
mod memory;
mod processor;
mod register;
mod storage;

pub type Word = u32;
pub type Immediate = i16;
pub type Address = i32;

pub type Endian = util::Endian;

pub use crate::constants::*;
pub use crate::instructions::*;
pub use crate::memory::*;
pub use crate::processor::*;
pub use crate::register::*;
pub use crate::storage::*;

#[cfg(test)]
mod test;
