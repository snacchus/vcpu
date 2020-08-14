pub mod constants;
mod enums;
pub mod memory;
mod processor;
mod register;

// TODO: Refactor this library, reorganize types

pub type Word = u32;
pub type Immediate = i16;
pub type Address = i32;

pub type Endian = util::Endian;

pub use crate::enums::*;
pub use crate::memory::*;
pub use crate::processor::*;
pub use crate::register::*;

#[cfg(test)]
mod test;
