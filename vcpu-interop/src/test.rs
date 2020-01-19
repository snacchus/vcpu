use super::*;
use vcpu::*;

use std::ffi::CString;
use std::ptr::{null, null_mut};

#[test]
fn composite_memory() {
    unsafe {
        let composite = vcpu_memory_create_comp();
        let plain = vcpu_memory_create_plain(16);

        let key = CString::new("f").expect("CString::new failed.");

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

        let program = program_from_words(&[
            instr_i(
                OpCode::SLTI,
                RegisterId::T2,
                RegisterId::T0,
                iterations as i16,
            ),
            instr_i(
                OpCode::BEZ,
                RegisterId::ZERO,
                RegisterId::T2,
                jmp_addr_i16(5),
            ),
            instr_i(OpCode::SLLI, RegisterId::T1, RegisterId::T0, 2),
            instr_i(OpCode::SW, RegisterId::T0, RegisterId::T1, 0),
            instr_i(OpCode::ADDI, RegisterId::T0, RegisterId::T0, 1),
            instr_j(OpCode::JMP, jmp_addr_i32(-5)),
            instr_i(OpCode::HALT, RegisterId::ZERO, RegisterId::ZERO, 0),
        ]);

        assert_eq!(
            vcpu_processor_run(processor, program.as_ptr(), program.len(), memory),
            VCPUResult::Ok
        );

        assert_eq!(vcpu_processor_get_state(processor), ExitCode::Halted as i32);

        if let MemoryVariant::Plain(vec) = (*memory).0.borrow().deref() {
            for i in 0..iterations {
                let value = vec.read_word(i as u32 * constants::WORD_BYTES).unwrap() as i32;
                assert_eq!(value, i);
            }
        }

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

        let source = CString::new(source_str).expect("CString::new failed.");

        let mut program: *mut Program = null_mut();

        assert_eq!(
            vcpu_program_assemble(source.as_ptr(), &mut program, null_mut()),
            VCPUResult::Ok
        );

        assert_ne!(program, null_mut());

        let mut instr: *const u8 = null();
        let mut instr_len: usize = 0;

        vcpu_program_get_instructions(program, &mut instr, &mut instr_len);

        assert_ne!(instr, null());
        assert_ne!(instr_len, 0);

        assert_eq!(
            vcpu_processor_run(processor, instr, instr_len, memory),
            VCPUResult::Ok
        );

        if let MemoryVariant::Plain(vec) = (*memory).0.borrow().deref() {
            for i in 0..32 {
                let value = vec.read_word(i as u32 * constants::WORD_BYTES).unwrap() as i32;
                assert_eq!(value, i);
            }
        }

        vcpu_program_destroy(program);
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

        let source = CString::new(source_str).expect("CString::new failed.");
        let mut program: *mut Program = null_mut();
        let mut error: *const c_char = null();

        assert_eq!(
            vcpu_program_assemble(source.as_ptr(), &mut program, &mut error),
            VCPUResult::AssemblerError
        );

        assert_eq!(program, null_mut());
        assert_ne!(error, null());
    }
}
