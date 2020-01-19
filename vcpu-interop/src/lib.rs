#![allow(clippy::not_unsafe_ptr_arg_deref)]

use vasm::assemble;
use vcpu::memory::composite::{CompositeMemory, MountError};
use vcpu::memory::io::{IOHandler, IOMemory};
use vcpu::{Processor, Storage, StorageMut};
use vexfile::Program;

use std::cell::{Cell, RefCell};
use std::ffi::{c_void, CStr, CString};
use std::ops::{Deref, DerefMut};
use std::os::raw::c_char;
use std::rc::Rc;
use std::slice;

#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum VCPUResult {
    Ok = 0,
    UnknownError,
    InvalidType,
    UTF8Error,
    AssemblerError,
    MemoryInUse,
    FragmentIntersection,
    KeyAlreadyExists,
    OutOfRange,
}

unsafe fn into_ptr<T>(t: T) -> *mut T {
    Box::into_raw(Box::new(t))
}

unsafe fn destroy<T>(ptr: *mut T) {
    drop(Box::from_raw(ptr))
}

pub type CanWriteCallback = *const extern "C" fn(
    data: *const u8,
    data_len: usize,
    address: u32,
    size: u32,
    user_data: *mut c_void,
) -> bool;

pub type OnWriteCallback = *const extern "C" fn(
    data: *const u8,
    data_len: usize,
    address: u32,
    size: u32,
    user_data: *mut c_void,
);

pub struct FunPtrIOHandler {
    can_write_fn: CanWriteCallback,
    on_write_fn: OnWriteCallback,
    user_data: *mut c_void,
}

impl IOHandler for FunPtrIOHandler {
    fn can_write(&self, memory: &[u8], address: u32, size: u32) -> bool {
        unsafe {
            (*self.can_write_fn)(memory.as_ptr(), memory.len(), address, size, self.user_data)
        }
    }

    fn on_write(&self, memory: &[u8], address: u32, size: u32) {
        unsafe { (*self.on_write_fn)(memory.as_ptr(), memory.len(), address, size, self.user_data) }
    }
}

enum MemoryVariant {
    Plain(Vec<u8>),
    IO(IOMemory<FunPtrIOHandler>),
    Composite(CompositeMemory),
}

pub struct Memory(Rc<RefCell<MemoryVariant>>);

impl Memory {
    fn new(variant: MemoryVariant) -> Memory {
        Memory(Rc::new(RefCell::new(variant)))
    }

    fn try_use<F: FnOnce(&MemoryVariant) -> VCPUResult>(&self, f: F) -> VCPUResult {
        match &self.0.try_borrow() {
            Ok(reference) => f(reference.deref()),
            Err(_) => VCPUResult::MemoryInUse,
        }
    }

    fn try_use_mut<F: FnOnce(&mut MemoryVariant) -> VCPUResult>(&mut self, f: F) -> VCPUResult {
        match &mut self.0.try_borrow_mut() {
            Ok(reference) => f(reference.deref_mut()),
            Err(_) => VCPUResult::MemoryInUse,
        }
    }
}

impl Clone for Memory {
    fn clone(&self) -> Memory {
        Memory(self.0.clone())
    }
}

impl Storage for Memory {
    fn length(&self) -> u32 {
        match self.0.try_borrow() {
            Ok(reference) => match reference.deref() {
                MemoryVariant::Plain(inner) => inner.length(),
                MemoryVariant::IO(inner) => inner.length(),
                MemoryVariant::Composite(inner) => inner.length(),
            },
            Err(_) => 0,
        }
    }

    fn check_range(&self, address: u32, length: u32) -> bool {
        match self.0.try_borrow() {
            Ok(reference) => match reference.deref() {
                MemoryVariant::Plain(inner) => inner.check_range(address, length),
                MemoryVariant::IO(inner) => inner.check_range(address, length),
                MemoryVariant::Composite(inner) => inner.check_range(address, length),
            },
            Err(_) => false,
        }
    }

    fn read(&self, address: u32, size: u32) -> Result<u32, ()> {
        match self.0.try_borrow().map_err(|_| ())?.deref() {
            MemoryVariant::Plain(inner) => inner.read(address, size),
            MemoryVariant::IO(inner) => inner.read(address, size),
            MemoryVariant::Composite(inner) => inner.read(address, size),
        }
    }
}

impl StorageMut for Memory {
    fn write(&mut self, address: u32, size: u32, value: u32) -> Result<(), ()> {
        match self.0.try_borrow_mut().map_err(|_| ())?.deref_mut() {
            MemoryVariant::Plain(inner) => inner.write(address, size, value),
            MemoryVariant::IO(inner) => inner.write(address, size, value),
            MemoryVariant::Composite(inner) => inner.write(address, size, value),
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn vcpu_memory_create_plain(size: u32) -> *mut Memory {
    into_ptr(Memory::new(MemoryVariant::Plain(vec![0; size as usize])))
}

#[no_mangle]
pub unsafe extern "C" fn vcpu_memory_create_io(
    size: u32,
    can_write: CanWriteCallback,
    on_write: OnWriteCallback,
    user_data: *mut c_void,
) -> *mut Memory {
    into_ptr(Memory::new(MemoryVariant::IO(IOMemory::new(
        size,
        FunPtrIOHandler {
            can_write_fn: can_write,
            on_write_fn: on_write,
            user_data,
        },
    ))))
}

#[no_mangle]
pub unsafe extern "C" fn vcpu_memory_get_ptr(
    memory: *mut Memory,
    ptr: *mut *mut u8,
    size: *mut u32,
) -> VCPUResult {
    (*memory).try_use_mut(|variant| {
        let slice = match variant {
            MemoryVariant::Plain(inner) => inner,
            MemoryVariant::IO(inner) => inner.data_mut(),
            _ => {
                return VCPUResult::InvalidType;
            }
        };

        *ptr = slice.as_mut_ptr();
        *size = slice.len() as u32;

        VCPUResult::Ok
    })
}

#[no_mangle]
pub unsafe extern "C" fn vcpu_memory_read(
    memory: *const Memory,
    dest: *mut u8,
    offset: u32,
    length: u32,
) -> VCPUResult {
    (*memory).try_use(|variant| {
        let slice = match variant {
            MemoryVariant::Plain(inner) => inner,
            MemoryVariant::IO(inner) => inner.data(),
            _ => {
                return VCPUResult::InvalidType;
            }
        };

        if slice.check_range(offset, length) {
            std::slice::from_raw_parts_mut(dest, length as usize)
                .copy_from_slice(&slice[offset as usize..(offset + length) as usize]);
            VCPUResult::Ok
        } else {
            VCPUResult::OutOfRange
        }
    })
}

#[no_mangle]
pub unsafe extern "C" fn vcpu_memory_write(
    memory: *mut Memory,
    src: *const u8,
    offset: u32,
    length: u32,
) -> VCPUResult {
    (*memory).try_use_mut(|variant| {
        let slice = match variant {
            MemoryVariant::Plain(inner) => inner,
            MemoryVariant::IO(inner) => inner.data_mut(),
            _ => {
                return VCPUResult::InvalidType;
            }
        };

        if slice.check_range(offset, length) {
            slice[offset as usize..(offset + length) as usize]
                .copy_from_slice(std::slice::from_raw_parts(src, length as usize));
            VCPUResult::Ok
        } else {
            VCPUResult::OutOfRange
        }
    })
}

#[no_mangle]
pub unsafe extern "C" fn vcpu_memcpy(
    dst: *mut c_void,
    src: *const c_void,
    length: usize,
) -> *mut c_void {
    std::ptr::copy_nonoverlapping(src, dst, length);
    dst
}

#[no_mangle]
pub unsafe extern "C" fn vcpu_memory_create_comp() -> *mut Memory {
    into_ptr(Memory::new(
        MemoryVariant::Composite(CompositeMemory::new()),
    ))
}

#[no_mangle]
pub unsafe extern "C" fn vcpu_memory_comp_mount(
    memory: *mut Memory,
    address: u32,
    key: *const c_char,
    fragment: *mut Memory,
) -> VCPUResult {
    (*memory).try_use_mut(|variant| match variant {
        MemoryVariant::Composite(inner) => match CStr::from_ptr(key).to_str() {
            Ok(key_str) => {
                let result = inner.mount(address, key_str, (*fragment).clone());

                match result {
                    Ok(_) => VCPUResult::Ok,
                    Err(MountError::FragmentIntersection) => VCPUResult::FragmentIntersection,
                    Err(MountError::KeyAlreadyExists) => VCPUResult::KeyAlreadyExists,
                }
            }
            Err(_) => VCPUResult::UTF8Error,
        },
        _ => VCPUResult::InvalidType,
    })
}

#[no_mangle]
pub unsafe extern "C" fn vcpu_memory_comp_unmount(
    memory: *mut Memory,
    key: *const c_char,
) -> VCPUResult {
    (*memory).try_use_mut(|variant| match variant {
        MemoryVariant::Composite(inner) => match CStr::from_ptr(key).to_str() {
            Ok(key_str) => {
                inner.unmount(key_str);
                VCPUResult::Ok
            }
            Err(_) => VCPUResult::UTF8Error,
        },
        _ => VCPUResult::InvalidType,
    })
}

#[no_mangle]
pub unsafe extern "C" fn vcpu_memory_destroy(memory: *mut Memory) {
    destroy(memory)
}

#[no_mangle]
pub unsafe extern "C" fn vcpu_processor_create() -> *mut Processor {
    into_ptr(Processor::new())
}

#[no_mangle]
pub unsafe extern "C" fn vcpu_processor_get_state(processor: *const Processor) -> i32 {
    match (*processor).state() {
        Some(code) => code as i32,
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
) -> VCPUResult {
    (*memory).try_use_mut(|variant| {
        (*processor).tick(
            slice::from_raw_parts(instr, instr_len),
            match variant {
                MemoryVariant::Plain(inner) => inner,
                MemoryVariant::IO(inner) => inner,
                MemoryVariant::Composite(inner) => inner,
            },
        );
        VCPUResult::Ok
    })
}

#[no_mangle]
pub unsafe extern "C" fn vcpu_processor_run(
    processor: *mut Processor,
    instr: *const u8,
    instr_len: usize,
    memory: *mut Memory,
) -> VCPUResult {
    (*memory).try_use_mut(|variant| {
        (*processor).run(
            slice::from_raw_parts(instr, instr_len),
            match variant {
                MemoryVariant::Plain(inner) => inner,
                MemoryVariant::IO(inner) => inner,
                MemoryVariant::Composite(inner) => inner,
            },
        );
        VCPUResult::Ok
    })
}

#[no_mangle]
pub unsafe extern "C" fn vcpu_program_assemble(
    source: *const c_char,
    program: *mut *mut Program,
    error: *mut *const c_char,
) -> VCPUResult {
    match CStr::from_ptr(source).to_str() {
        Ok(src) => match assemble(src) {
            Ok(result) => {
                *program = into_ptr(result);
                VCPUResult::Ok
            }
            Err(err) => {
                if !error.is_null() {
                    LAST_ERROR.with(|f| {
                        let err_str = CString::new(format!("{}", err)).unwrap_or_default();
                        *error = err_str.as_ptr();
                        f.set(err_str);
                    });
                }
                VCPUResult::AssemblerError
            }
        },

        Err(_) => VCPUResult::UTF8Error,
    }
}

thread_local! {
    static LAST_ERROR: Cell<CString> = Cell::new(Default::default());
}

#[no_mangle]
pub unsafe extern "C" fn vcpu_program_get_data(
    program: *const Program,
    data: *mut *const u8,
    data_len: *mut usize,
) {
    let prog_data = (*program).data();
    *data = prog_data.as_ptr();
    *data_len = prog_data.len();
}

#[no_mangle]
pub unsafe extern "C" fn vcpu_program_get_instructions(
    program: *const Program,
    instr: *mut *const u8,
    instr_len: *mut usize,
) {
    let prog_instr = (*program).instructions();
    *instr = prog_instr.as_ptr();
    *instr_len = prog_instr.len();
}

#[no_mangle]
pub unsafe extern "C" fn vcpu_program_destroy(program: *mut Program) {
    destroy(program);
}

#[cfg(test)]
mod test;
