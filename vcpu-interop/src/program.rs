use crate::result::VCPUResult;
use crate::source_map::SourceMap;
use crate::util::{destroy, into_ptr};
use std::os::raw::c_char;
use vasm::assemble_addressed;
use vexfile::{get_required_size, Program, ReadVexExt, WriteVexExt};

use std::cell::Cell;
use std::ffi::{CStr, CString};
use std::slice;

#[no_mangle]
pub unsafe extern "C" fn vcpu_program_assemble(
    source: *const c_char,
    data_offset: u32,
    program: *mut *mut Program,
    source_map: *mut *mut SourceMap,
    error: *mut *const c_char,
) -> VCPUResult {
    match CStr::from_ptr(source).to_str() {
        Ok(src) => match assemble_addressed(src, data_offset) {
            Ok((result, result_map)) => {
                *program = into_ptr(result);
                if !source_map.is_null() {
                    let map_data = result_map
                        .into_iter()
                        .flat_map(|item| vec![item.start_line, item.line_count])
                        .collect();
                    let map_copy = SourceMap { data: map_data };
                    *source_map = into_ptr(map_copy);
                }
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
pub unsafe extern "C" fn vcpu_program_load_vex(
    vex_data: *const u8,
    vex_data_len: usize,
    program: *mut *mut Program,
) -> VCPUResult {
    match slice::from_raw_parts(vex_data, vex_data_len).read_vex() {
        Ok(result) => {
            *program = into_ptr(result);
            VCPUResult::Ok
        }
        Err(_) => VCPUResult::ProgramLoadFailed,
    }
}

#[no_mangle]
pub unsafe extern "C" fn vcpu_program_get_data_offset(program: *const Program) -> u32 {
    (*program).data_offset()
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
pub unsafe extern "C" fn vcpu_program_destroy(program: *mut Program) {
    destroy(program);
}

#[no_mangle]
pub unsafe extern "C" fn vcpu_program_get_vex_size(program: *const Program) -> usize {
    get_required_size(&*program)
}

#[no_mangle]
pub unsafe extern "C" fn vcpu_program_save_vex(
    program: *const Program,
    vex_data: *mut u8,
    vex_data_len: usize,
) -> VCPUResult {
    let mut output = slice::from_raw_parts_mut(vex_data, vex_data_len);
    match output.write_vex(&*program) {
        Ok(_) => VCPUResult::Ok,
        Err(_) => VCPUResult::ProgramSaveFailed,
    }
}
