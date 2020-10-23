use crate::executable::*;
use crate::exit_code::*;
use crate::memory::*;
use crate::processor::*;
use crate::register::*;
use crate::result::*;
use std::ffi::{c_void, CStr, CString};
use std::os::raw::c_char;
use std::ptr::{null, null_mut};
use vcpu::*;
use vex::Executable;

fn get_c_str(value: &str) -> CString {
    CString::new(value).expect("CString::new failed.")
}

#[test]
fn composite_memory() {
    unsafe {
        let composite = vcpu_memory_create_comp();
        let plain = vcpu_memory_create_plain(16);

        let key = get_c_str("f");

        vcpu_memory_comp_mount(composite, 0, key.as_ptr(), plain);
        vcpu_memory_comp_unmount(composite, key.as_ptr());

        vcpu_memory_destroy(plain);
        vcpu_memory_destroy(composite);
    }
}

#[test]
fn run_simple() {
    unsafe {
        let memory = vcpu_memory_create_plain(128);
        let processor = vcpu_processor_create();

        let iterations = 32i32;

        let instructions = instructions_from_words(&[
            instr_i!(SLTI, T2, T0, iterations as i16),
            instr_i!(BEZ, ZERO, T2, jmp_addr_i16(5)),
            instr_i!(SLLI, T1, T0, 2),
            instr_i!(SW, T0, T1, 0),
            instr_i!(ADDI, T0, T0, 1),
            instr_j!(JMP, jmp_addr_i32(-5)),
            instr_i!(HALT, ZERO, ZERO, 0),
        ]);

        assert_eq!(
            vcpu_processor_run(processor, instructions.as_ptr(), instructions.len(), memory),
            VcpuResult::Ok
        );

        assert_eq!(vcpu_processor_get_state(processor), ExitCode::Halted as i32);

        let result = (*memory).try_use(|v| {
            if let MemoryVariant::Plain(vec) = v {
                for i in 0..iterations {
                    let value = vec.read_word(i as u32 * vcpu::WORD_BYTES).unwrap() as i32;
                    assert_eq!(value, i);
                }
            }
            VcpuResult::Ok
        });

        assert_eq!(VcpuResult::Ok, result);

        vcpu_processor_destroy(processor);
        vcpu_memory_destroy(memory);
    }
}

#[test]
fn run_assembled() {
    unsafe {
        let memory = vcpu_memory_create_plain(128);
        let processor = vcpu_processor_create();

        let source_str = ".data
.instructions
loop: SLTI $t2, $t0, 32
      BEZ  $t2, end
      SLLI $t1, $t0, 2
      SW   $t0, 0($t1)
      ADDI $t0, $t0, 1
      JMP loop
end:  HALT";

        let source = get_c_str(source_str);

        let mut executable: *mut Executable = null_mut();

        assert_eq!(
            vcpu_executable_assemble(source.as_ptr(), 0, &mut executable, null_mut(), null_mut()),
            VcpuResult::Ok
        );

        assert_ne!(executable, null_mut());

        let mut instr: *const u8 = null();
        let mut instr_len: usize = 0;

        vcpu_executable_get_instructions(executable, &mut instr, &mut instr_len);

        assert_ne!(instr, null());
        assert_ne!(instr_len, 0);

        assert_eq!(
            vcpu_processor_run(processor, instr, instr_len, memory),
            VcpuResult::Ok
        );

        let result = (*memory).try_use(|v| {
            if let MemoryVariant::Plain(vec) = v {
                for i in 0..32 {
                    let value = vec.read_word(i as u32 * vcpu::WORD_BYTES).unwrap() as i32;
                    assert_eq!(value, i);
                }
            }
            VcpuResult::Ok
        });
        assert_eq!(VcpuResult::Ok, result);

        vcpu_executable_destroy(executable);
        vcpu_processor_destroy(processor);
        vcpu_memory_destroy(memory);
    }
}

#[test]
fn assemble_with_error() {
    unsafe {
        let source_str = ".data
.instructions
STUFF
HALT";

        let source = get_c_str(source_str);
        let mut executable: *mut Executable = null_mut();
        let mut error: *const c_char = null();

        assert_eq!(
            vcpu_executable_assemble(source.as_ptr(), 0, &mut executable, null_mut(), &mut error),
            VcpuResult::AssemblerError
        );

        assert_eq!(executable, null_mut());
        assert_ne!(error, null());
    }
}

#[test]
fn get_register_name_valid() {
    unsafe {
        let mut name: *const c_char = null();

        assert_eq!(vcpu_register_get_name(0, &mut name), VcpuResult::Ok);
        assert_ne!(name, null());
        assert_eq!(CStr::from_ptr(name).to_str(), Ok("ZERO"));

        assert_eq!(vcpu_register_get_name(1, &mut name), VcpuResult::Ok);
        assert_ne!(name, null());
        assert_eq!(CStr::from_ptr(name).to_str(), Ok("V0"));

        assert_eq!(vcpu_register_get_name(31, &mut name), VcpuResult::Ok);
        assert_ne!(name, null());
        assert_eq!(CStr::from_ptr(name).to_str(), Ok("RA"));
    }
}

#[test]
fn get_register_name_invalid() {
    unsafe {
        let mut name: *const c_char = null();

        assert_eq!(
            vcpu_register_get_name(32, &mut name),
            VcpuResult::OutOfRange
        );
        assert_eq!(
            vcpu_register_get_name(u32::max_value(), &mut name),
            VcpuResult::OutOfRange
        );
    }
}

extern "C" fn can_write_dummy(
    _data: *const u8,
    _data_len: usize,
    _address: u32,
    _size: u32,
    _user_data: *mut c_void,
) -> bool {
    true
}

extern "C" fn on_write_dummy(
    _data: *const u8,
    _data_len: usize,
    _address: u32,
    _size: u32,
    _user_data: *mut c_void,
) {
}

#[test]
fn composite_mem_with_io() {
    unsafe {
        let source_str = ".data
.instructions
li $t0, 1
lhi $t1, 0xf1ed
sb $t0, 0($t1)
halt";

        let source = get_c_str(source_str);

        let mut executable: *mut Executable = null_mut();
        let mut error: *const c_char = null();

        let assemble_result =
            vcpu_executable_assemble(source.as_ptr(), 0, &mut executable, null_mut(), &mut error);
        if let VcpuResult::AssemblerError = assemble_result {
            let error_str = CStr::from_ptr(error).to_str().unwrap();
            panic!(error_str);
        }
        assert_eq!(assemble_result, VcpuResult::Ok);

        let mut instr: *const u8 = null();
        let mut instr_len: usize = 0;

        vcpu_executable_get_instructions(executable, &mut instr, &mut instr_len);

        let plain_mem = vcpu_memory_create_plain(1024);
        let io_mem = vcpu_memory_create_io(1, can_write_dummy, on_write_dummy, null_mut());
        let comp_mem = vcpu_memory_create_comp();

        let main_key = get_c_str("main");
        let io_key = get_c_str("io");

        assert_eq!(
            vcpu_memory_comp_mount(comp_mem, 0, main_key.as_ptr(), plain_mem),
            VcpuResult::Ok
        );

        assert_eq!(
            vcpu_memory_comp_mount(comp_mem, 0xF1ED_0000, io_key.as_ptr(), io_mem),
            VcpuResult::Ok
        );

        let processor = vcpu_processor_create();

        assert_eq!(
            vcpu_processor_run(processor, instr, instr_len, comp_mem),
            VcpuResult::Ok
        );

        let result = (*io_mem).try_use(|v| {
            if let MemoryVariant::IO(io) = v {
                assert_eq!(io.data(), &[1]);
            }
            VcpuResult::Ok
        });

        assert_eq!(VcpuResult::Ok, result);

        vcpu_processor_destroy(processor);
        vcpu_memory_destroy(comp_mem);
        vcpu_memory_destroy(plain_mem);
        vcpu_memory_destroy(io_mem);
        vcpu_executable_destroy(executable);
    }
}

#[test]
fn access_comp_mem() {
    unsafe {
        let plain_mem = vcpu_memory_create_plain(1024);
        let io_mem = vcpu_memory_create_io(1, can_write_dummy, on_write_dummy, null_mut());
        let comp_mem = vcpu_memory_create_comp();

        let main_key = get_c_str("main");
        let io_key = get_c_str("io");

        assert_eq!(
            vcpu_memory_comp_mount(comp_mem, 0, main_key.as_ptr(), plain_mem),
            VcpuResult::Ok
        );

        assert_eq!(
            vcpu_memory_comp_mount(comp_mem, 0xF1ED_0000, io_key.as_ptr(), io_mem),
            VcpuResult::Ok
        );

        assert_eq!((*comp_mem).write_byte(0, 1), Ok(()));
        assert_eq!((*plain_mem).read_byte(0), Ok(1));

        assert_eq!((*comp_mem).write_byte(0xF1ED_0000, 1), Ok(()));
        assert_eq!((*io_mem).read_byte(0), Ok(1));

        vcpu_memory_destroy(comp_mem);
        vcpu_memory_destroy(plain_mem);
        vcpu_memory_destroy(io_mem);
    }
}

#[test]
fn write_io_memory() {
    unsafe {
        let io_mem = vcpu_memory_create_io(1, can_write_dummy, on_write_dummy, null_mut());

        assert_eq!((*io_mem).write_byte(0, 1), Ok(()));
        assert_eq!((*io_mem).read_byte(0), Ok(1));

        vcpu_memory_destroy(io_mem);
    }
}

#[test]
fn get_exit_code_desc() {
    unsafe {
        let mut name: *const c_char = null();

        assert_eq!(vcpu_exit_code_get_description(0, &mut name), VcpuResult::Ok);
        assert_ne!(name, null());
        assert_eq!(CStr::from_ptr(name).to_str(), Ok("Halted"));

        assert_eq!(vcpu_exit_code_get_description(1, &mut name), VcpuResult::Ok);
        assert_ne!(name, null());
        assert_eq!(CStr::from_ptr(name).to_str(), Ok("DivisionByZero"));

        assert_eq!(vcpu_exit_code_get_description(6, &mut name), VcpuResult::Ok);
        assert_ne!(name, null());
        assert_eq!(CStr::from_ptr(name).to_str(), Ok("BadProgramCounter"));
    }
}
