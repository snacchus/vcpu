use crate::result::VcpuResult;
use crate::util::{destroy, into_ptr};
use std::cell::RefCell;
use std::ffi::{c_void, CStr};
use std::ops::{Deref, DerefMut};
use std::os::raw::c_char;
use std::rc::Rc;
use vcpu::{CompositeMemory, IOHandler, IOMemory, MountError, Storage, StorageMut};

pub type CanWriteCallback = extern "C" fn(
    data: *const u8,
    data_len: usize,
    address: u32,
    size: u32,
    user_data: *mut c_void,
) -> bool;

pub type OnWriteCallback = extern "C" fn(
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
        (self.can_write_fn)(memory.as_ptr(), memory.len(), address, size, self.user_data)
    }

    fn on_write(&self, memory: &[u8], address: u32, size: u32) {
        (self.on_write_fn)(memory.as_ptr(), memory.len(), address, size, self.user_data)
    }
}

pub enum MemoryVariant {
    Plain(Vec<u8>),
    IO(IOMemory<FunPtrIOHandler>),
    Composite(CompositeMemory),
}

pub struct Memory(Rc<RefCell<MemoryVariant>>);

impl Memory {
    pub fn new(variant: MemoryVariant) -> Memory {
        Memory(Rc::new(RefCell::new(variant)))
    }

    pub fn try_use<F: FnOnce(&MemoryVariant) -> VcpuResult>(&self, f: F) -> VcpuResult {
        match &self.0.try_borrow() {
            Ok(reference) => f(reference.deref()),
            Err(_) => VcpuResult::MemoryInUse,
        }
    }

    pub fn try_use_mut<F: FnOnce(&mut MemoryVariant) -> VcpuResult>(&mut self, f: F) -> VcpuResult {
        match &mut self.0.try_borrow_mut() {
            Ok(reference) => f(reference.deref_mut()),
            Err(_) => VcpuResult::MemoryInUse,
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
) -> VcpuResult {
    (*memory).try_use_mut(|variant| {
        let slice = match variant {
            MemoryVariant::Plain(inner) => inner,
            MemoryVariant::IO(inner) => inner.data_mut(),
            _ => {
                return VcpuResult::InvalidType;
            }
        };

        *ptr = slice.as_mut_ptr();
        *size = slice.len() as u32;

        VcpuResult::Ok
    })
}

#[no_mangle]
pub unsafe extern "C" fn vcpu_memory_read(
    memory: *const Memory,
    dest: *mut u8,
    offset: u32,
    length: u32,
) -> VcpuResult {
    (*memory).try_use(|variant| {
        let slice = match variant {
            MemoryVariant::Plain(inner) => inner,
            MemoryVariant::IO(inner) => inner.data(),
            _ => {
                return VcpuResult::InvalidType;
            }
        };

        if slice.check_range(offset, length) {
            std::slice::from_raw_parts_mut(dest, length as usize)
                .copy_from_slice(&slice[offset as usize..(offset + length) as usize]);
            VcpuResult::Ok
        } else {
            VcpuResult::OutOfRange
        }
    })
}

#[no_mangle]
pub unsafe extern "C" fn vcpu_memory_write(
    memory: *mut Memory,
    src: *const u8,
    offset: u32,
    length: u32,
) -> VcpuResult {
    (*memory).try_use_mut(|variant| {
        let slice = match variant {
            MemoryVariant::Plain(inner) => inner,
            MemoryVariant::IO(inner) => inner.data_mut(),
            _ => {
                return VcpuResult::InvalidType;
            }
        };

        if slice.check_range(offset, length) {
            slice[offset as usize..(offset + length) as usize]
                .copy_from_slice(std::slice::from_raw_parts(src, length as usize));
            VcpuResult::Ok
        } else {
            VcpuResult::OutOfRange
        }
    })
}

unsafe fn memory_get(
    memory: *const Memory,
    address: u32,
    size: u32,
    value: *mut u32,
) -> VcpuResult {
    (*memory).try_use(|variant| {
        let result = match variant {
            MemoryVariant::Plain(inner) => inner.read(address, size),
            MemoryVariant::IO(inner) => inner.read(address, size),
            MemoryVariant::Composite(inner) => inner.read(address, size),
        };

        match result {
            Ok(v) => {
                *value = v;
                VcpuResult::Ok
            }
            Err(_) => VcpuResult::OutOfRange,
        }
    })
}

#[no_mangle]
pub unsafe extern "C" fn vcpu_memory_get_word(
    memory: *const Memory,
    address: u32,
    value: *mut u32,
) -> VcpuResult {
    memory_get(memory, address, vcpu::WORD_BYTES, value)
}

#[no_mangle]
pub unsafe extern "C" fn vcpu_memory_get_half(
    memory: *const Memory,
    address: u32,
    value: *mut u16,
) -> VcpuResult {
    let mut v = 0u32;
    let result = memory_get(memory, address, vcpu::HALF_BYTES, &mut v);
    *value = v as u16;
    result
}

#[no_mangle]
pub unsafe extern "C" fn vcpu_memory_get_byte(
    memory: *const Memory,
    address: u32,
    value: *mut u8,
) -> VcpuResult {
    let mut v = 0u32;
    let result = memory_get(memory, address, vcpu::BYTE_BYTES, &mut v);
    *value = v as u8;
    result
}

unsafe fn memory_set(memory: *mut Memory, address: u32, size: u32, value: u32) -> VcpuResult {
    (*memory).try_use_mut(|variant| {
        let result = match variant {
            MemoryVariant::Plain(inner) => inner.write(address, size, value),
            MemoryVariant::IO(inner) => inner.write(address, size, value),
            MemoryVariant::Composite(inner) => inner.write(address, size, value),
        };

        match result {
            Ok(_) => VcpuResult::Ok,
            Err(_) => VcpuResult::OutOfRange,
        }
    })
}

#[no_mangle]
pub unsafe extern "C" fn vcpu_memory_set_word(
    memory: *mut Memory,
    address: u32,
    value: u32,
) -> VcpuResult {
    memory_set(memory, address, vcpu::WORD_BYTES, value)
}

#[no_mangle]
pub unsafe extern "C" fn vcpu_memory_set_half(
    memory: *mut Memory,
    address: u32,
    value: u16,
) -> VcpuResult {
    memory_set(memory, address, vcpu::HALF_BYTES, value.into())
}

#[no_mangle]
pub unsafe extern "C" fn vcpu_memory_set_byte(
    memory: *mut Memory,
    address: u32,
    value: u8,
) -> VcpuResult {
    memory_set(memory, address, vcpu::BYTE_BYTES, value.into())
}

#[no_mangle]
pub unsafe extern "C" fn vcpu_memory_resize(memory: *mut Memory, size: u32) -> VcpuResult {
    (*memory).try_use_mut(|variant| match variant {
        MemoryVariant::Plain(inner) => {
            inner.resize(size as usize, u8::default());
            VcpuResult::Ok
        }
        MemoryVariant::IO(inner) => {
            inner.resize(size);
            VcpuResult::Ok
        }
        _ => VcpuResult::InvalidType,
    })
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
) -> VcpuResult {
    (*memory).try_use_mut(|variant| match variant {
        MemoryVariant::Composite(inner) => match CStr::from_ptr(key).to_str() {
            Ok(key_str) => {
                let result = inner.mount(address, key_str, (*fragment).clone());

                match result {
                    Ok(_) => VcpuResult::Ok,
                    Err(MountError::FragmentIntersection) => VcpuResult::FragmentIntersection,
                    Err(MountError::KeyAlreadyExists) => VcpuResult::KeyAlreadyExists,
                }
            }
            Err(_) => VcpuResult::UTF8Error,
        },
        _ => VcpuResult::InvalidType,
    })
}

#[no_mangle]
pub unsafe extern "C" fn vcpu_memory_comp_unmount(
    memory: *mut Memory,
    key: *const c_char,
) -> VcpuResult {
    (*memory).try_use_mut(|variant| match variant {
        MemoryVariant::Composite(inner) => match CStr::from_ptr(key).to_str() {
            Ok(key_str) => {
                inner.unmount(key_str);
                VcpuResult::Ok
            }
            Err(_) => VcpuResult::UTF8Error,
        },
        _ => VcpuResult::InvalidType,
    })
}

#[no_mangle]
pub unsafe extern "C" fn vcpu_memory_destroy(memory: *mut Memory) {
    destroy(memory)
}
