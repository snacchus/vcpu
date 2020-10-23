use crate::result::VcpuResult;
use num_traits::FromPrimitive;
use std::os::raw::c_char;
use util::InteropGetName;

#[no_mangle]
pub unsafe extern "C" fn vcpu_register_get_count() -> u32 {
    vcpu::REGISTER_COUNT as u32
}

#[no_mangle]
pub unsafe extern "C" fn vcpu_register_get_name(
    index: u32,
    name: *mut *const c_char,
) -> VcpuResult {
    if let Some(id) = vcpu::RegisterId::from_u32(index) {
        *name = id.interop_name().as_ptr() as *const c_char;
        VcpuResult::Ok
    } else {
        VcpuResult::OutOfRange
    }
}
