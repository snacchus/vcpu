//! This crate procudes a DLL with C-Bindings for crates [vcpu](../vcpu/index.html),
//! [vasm](../vasm/index.html) and [vex](../vex/index.html)

#![allow(clippy::not_unsafe_ptr_arg_deref)]
#![allow(clippy::missing_safety_doc)]

mod executable;
mod exit_code;
mod memory;
mod processor;
mod register;
mod result;
mod source_map;
mod util;

// TODO: unit tests for all functions

#[cfg(test)]
mod test;
