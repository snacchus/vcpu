use crate::result::VcpuResult;
use num_traits::FromPrimitive;
use std::os::raw::c_char;
use util::InteropGetName;

#[no_mangle]
pub unsafe extern "C" fn vcpu_exit_code_get_description(
    code: i32,
    desc: *mut *const c_char,
) -> VcpuResult {
    if let Some(code) = vcpu::ExitCode::from_i32(code) {
        *desc = code.interop_name().as_ptr() as *const c_char;
        VcpuResult::Ok
    } else {
        VcpuResult::OutOfRange
    }
}
