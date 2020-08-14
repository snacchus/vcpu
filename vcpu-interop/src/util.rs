use std::ffi::c_void;

pub unsafe fn into_ptr<T>(t: T) -> *mut T {
    Box::into_raw(Box::new(t))
}

pub unsafe fn destroy<T>(ptr: *mut T) {
    drop(Box::from_raw(ptr))
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
