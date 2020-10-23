use crate::memory::{Memory, MemoryVariant};
use crate::result::VcpuResult;
use crate::util::{destroy, into_ptr};
use num_traits::{FromPrimitive, ToPrimitive};
use std::slice;
use vcpu::Processor;

#[no_mangle]
pub unsafe extern "C" fn vcpu_processor_create() -> *mut Processor {
    into_ptr(Processor::new())
}

#[no_mangle]
pub unsafe extern "C" fn vcpu_processor_get_register(
    processor: *const Processor,
    index: u32,
    value: *mut i32,
) -> VcpuResult {
    match FromPrimitive::from_u32(index) {
        Some(rid) => {
            *value = (*processor).register(rid).i();
            VcpuResult::Ok
        }
        None => VcpuResult::OutOfRange,
    }
}

#[no_mangle]
pub unsafe extern "C" fn vcpu_processor_set_register(
    processor: *mut Processor,
    index: u32,
    value: i32,
) -> VcpuResult {
    match FromPrimitive::from_u32(index) {
        Some(rid) => {
            (*processor).register_mut(rid).set_i(value);
            VcpuResult::Ok
        }
        None => VcpuResult::OutOfRange,
    }
}

#[no_mangle]
pub unsafe extern "C" fn vcpu_processor_get_program_counter(processor: *const Processor) -> u32 {
    (*processor).program_counter()
}

#[no_mangle]
pub unsafe extern "C" fn vcpu_processor_get_state(processor: *const Processor) -> i32 {
    match (*processor).state() {
        Some(code) => code.to_i32().unwrap(),
        None => -1,
    }
}

#[no_mangle]
pub unsafe extern "C" fn vcpu_processor_destroy(processor: *mut Processor) {
    destroy(processor)
}

#[no_mangle]
pub unsafe extern "C" fn vcpu_processor_tick(
    processor: *mut Processor,
    instr: *const u8,
    instr_len: usize,
    memory: *mut Memory,
) -> VcpuResult {
    (*memory).try_use_mut(|variant| {
        (*processor).tick(
            slice::from_raw_parts(instr, instr_len),
            match variant {
                MemoryVariant::Plain(inner) => inner,
                MemoryVariant::IO(inner) => inner,
                MemoryVariant::Composite(inner) => inner,
            },
        );
        VcpuResult::Ok
    })
}

#[no_mangle]
pub unsafe extern "C" fn vcpu_processor_run(
    processor: *mut Processor,
    instr: *const u8,
    instr_len: usize,
    memory: *mut Memory,
) -> VcpuResult {
    (*memory).try_use_mut(|variant| {
        (*processor).run(
            slice::from_raw_parts(instr, instr_len),
            match variant {
                MemoryVariant::Plain(inner) => inner,
                MemoryVariant::IO(inner) => inner,
                MemoryVariant::Composite(inner) => inner,
            },
        );
        VcpuResult::Ok
    })
}

#[no_mangle]
pub unsafe extern "C" fn vcpu_processor_reset(processor: *mut Processor) {
    (*processor).reset()
}
