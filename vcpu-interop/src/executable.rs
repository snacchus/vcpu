use crate::result::VcpuResult;
use crate::source_map::SourceMap;
use crate::util::{destroy, into_ptr};
use std::os::raw::c_char;
use vasm::assemble_addressed;
use vex::{Executable, ReadVexExt, WriteVexExt};

use std::cell::Cell;
use std::ffi::{CStr, CString};
use std::slice;

#[no_mangle]
pub unsafe extern "C" fn vcpu_executable_assemble(
    source: *const c_char,
    data_offset: u32,
    executable: *mut *mut Executable,
    source_map: *mut *mut SourceMap,
    error: *mut *const c_char,
) -> VcpuResult {
    match CStr::from_ptr(source).to_str() {
        Ok(src) => match assemble_addressed(src, data_offset) {
            Ok((result, result_map)) => {
                *executable = into_ptr(result);
                if !source_map.is_null() {
                    let map_data = result_map
                        .into_iter()
                        .flat_map(|item| vec![item.start_line, item.line_count])
                        .collect();
                    let map_copy = SourceMap { data: map_data };
                    *source_map = into_ptr(map_copy);
                }
                VcpuResult::Ok
            }
            Err(err) => {
                if !error.is_null() {
                    LAST_ERROR.with(|f| {
                        let err_str = CString::new(format!("{}", err)).unwrap_or_default();
                        *error = err_str.as_ptr();
                        f.set(err_str);
                    });
                }
                VcpuResult::AssemblerError
            }
        },

        Err(_) => VcpuResult::UTF8Error,
    }
}

thread_local! {
    static LAST_ERROR: Cell<CString> = Cell::new(Default::default());
}

#[no_mangle]
pub unsafe extern "C" fn vcpu_executable_load_vex(
    vex_data: *const u8,
    vex_data_len: usize,
    executable: *mut *mut Executable,
) -> VcpuResult {
    match slice::from_raw_parts(vex_data, vex_data_len).read_vex() {
        Ok(result) => {
            *executable = into_ptr(result);
            VcpuResult::Ok
        }
        Err(_) => VcpuResult::ExecutableLoadFailed,
    }
}

#[no_mangle]
pub unsafe extern "C" fn vcpu_executable_get_data_offset(executable: *const Executable) -> u32 {
    (*executable).data_offset()
}

#[no_mangle]
pub unsafe extern "C" fn vcpu_executable_get_instructions(
    executable: *const Executable,
    instr: *mut *const u8,
    instr_len: *mut usize,
) {
    let prog_instr = (*executable).instructions();
    *instr = prog_instr.as_ptr();
    *instr_len = prog_instr.len();
}

#[no_mangle]
pub unsafe extern "C" fn vcpu_executable_get_data(
    executable: *const Executable,
    data: *mut *const u8,
    data_len: *mut usize,
) {
    let prog_data = (*executable).data();
    *data = prog_data.as_ptr();
    *data_len = prog_data.len();
}

#[no_mangle]
pub unsafe extern "C" fn vcpu_executable_destroy(executable: *mut Executable) {
    destroy(executable);
}

#[no_mangle]
pub unsafe extern "C" fn vcpu_executable_get_vex_size(executable: *const Executable) -> usize {
    (&*executable).required_size()
}

#[no_mangle]
pub unsafe extern "C" fn vcpu_executable_save_vex(
    executable: *const Executable,
    vex_data: *mut u8,
    vex_data_len: usize,
) -> VcpuResult {
    let mut output = slice::from_raw_parts_mut(vex_data, vex_data_len);
    match output.write_vex(&*executable) {
        Ok(_) => VcpuResult::Ok,
        Err(_) => VcpuResult::ExecutableSaveFailed,
    }
}
