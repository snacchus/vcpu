#![allow(clippy::not_unsafe_ptr_arg_deref)]
#![allow(clippy::missing_safety_doc)]

mod exit_code;
mod memory;
mod processor;
mod program;
mod register;
mod result;
mod source_map;
mod util;

// TODO: unit tests for all functions

#[cfg(test)]
mod test;
