use super::*;
use num_traits::FromPrimitive;

macro_rules! instructions_execute {
    (
        $instr:expr,
        [$($id:ident = $v:expr),*] => [$($eid:ident = $ev:expr),*],
        $stor:expr => $estor:expr,
        $ticks:expr,
        $state:expr,
        $pc:expr
    ) => {{
        // ----- PREPARE -----
        // 1. Get instructions
        let instructions = super::program_from_words(&$instr);

        // 2. Initialize objects
        let storage = &mut $stor;
        let mut processor = Processor::default();

        // 3. Set initial register values (unspecified values stay as 0)
        $(
            *processor.register_mut(RegisterId::$id) = From::from($v);
        )*

        // 4. Get array of expected register values.
        //    The final register values are expected to be the same as the initial values,
        //    unless explicitly specified otherwise via the expected_registers parameter.

        // 4.a Copy initial values
        let mut expected_registers = [Register::default(); constants::REGISTER_COUNT];
        expected_registers.copy_from_slice(processor.registers());

        // 4.b Overwrite with specified values
        $(
            expected_registers[register_index(RegisterId::$eid)] = From::from($ev);
        )*

        // ----- ACT -----
        // Tick up to max_ticks times
        let max_ticks = $ticks;
        for _ in 0..max_ticks {
            if processor.tick(&instructions, storage).is_some() {
                break;
            }
        }

        // ----- ASSERT -----
        // 1. Check final state of processor
        assert_eq!($state, processor.state(), "Final state does not match!");

        // 2. Check final program counter
        assert_eq!($pc, processor.program_counter(), "Final program counter does not match!");

        // 3. Check final register values
        for i in 0..expected_registers.len() {
            assert_eq!(expected_registers[i], processor.registers()[i], "Final value of register {0} does not match!", RegisterId::from_usize(i).unwrap());
        }

        // 4. Check storage
        assert_eq!(&$estor, storage);
    }};
}

macro_rules! instruction_executes {
    (
        $instr:expr,
        [$($id:ident = $v:expr),*] => [$($eid:ident = $ev:expr),*],
        $stor:expr => $estor:expr,
        $state:expr,
        $pc:expr
    ) => {
        instructions_execute! {
            [$instr, nop!()],
            [$($id = $v),*] => [$($eid = $ev),*],
            $stor => $estor,
            1,
            $state,
            $pc
        }
    };
    (
        $instr:expr,
        [$($id:ident = $v:expr),*] => [$($eid:ident = $ev:expr),*]
        $state:expr,
        $pc:expr
    ) => {
        instruction_executes! {
            $instr,
            [$($id = $v),*] => [$($eid = $ev),*],
            empty_storage!() => empty_storage!(),
            $state,
            $pc
        }
    };
}

macro_rules! instruction_exits {
    (
        $instr:expr,
        [$($id:ident = $v:expr),*] => [$($eid:ident = $ev:expr),*],
        $stor:expr => $estor:expr,
        $code:ident
    ) => {
        instruction_executes! {
            $instr,
            [$($id = $v),*] => [$($eid = $ev),*],
            $stor => $estor,
            Some(ExitCode::$code),
            0
        }
    };
    (
        $instr:expr,
        [$($id:ident = $v:expr),*] => [$($eid:ident = $ev:expr),*],
        $code:ident
    ) => {
        instruction_exits! {
            $instr,
            [$($id = $v),*] => [$($eid = $ev),*],
            empty_storage!() => empty_storage!(),
            $code
        }
    };
}

macro_rules! instruction_runs {
    (
        $instr:expr,
        [$($id:ident = $v:expr),*] => [$($eid:ident = $ev:expr),*],
        $stor:expr => $estor:expr
    ) => {
        instruction_executes! {
            $instr,
            [$($id = $v),*] => [$($eid = $ev),*],
            $stor => $estor,
            None,
            4
        }
    };
    (
        $instr:expr,
        [$($id:ident = $v:expr),*] => [$($eid:ident = $ev:expr),*]
    ) => {
        instruction_runs! {
            $instr,
            [$($id = $v),*] => [$($eid = $ev),*],
            empty_storage!() => empty_storage!()
        }
    };
}

#[test]
fn nop() {
    instruction_runs! {
        nop!(), [] => []
    };
}

#[test]
fn halt() {
    instruction_exits! {
        instr_i!(HALT, ZERO, ZERO, 0),
        [] => [],
        Halted
    };
}

#[test]
fn call() {
    instruction_runs! {
        instr_i!(CALL, ZERO, ZERO, 0),
        [] => []
    };
}

#[test]
fn zero_register_read_only() {
    instruction_runs! {
        instr_i!(COPY, ZERO, T0, 0),
        [T0 = 0x1234_5678] => [ZERO = 0]
    }
}

mod add;
mod addi;
mod and;
mod andi;
mod bez;
mod bnz;
mod copy;
mod div;
mod divi;
mod fadd;
mod fdiv;
mod flip;
mod fmul;
mod fsub;
mod ftoi;
mod invalid;
mod itof;
mod jl;
mod jlr;
mod jmp;
mod jr;
mod lb;
mod lh;
mod lhi;
mod li;
mod lw;
mod mul;
mod muli;
mod or;
mod ori;
mod sb;
mod seq;
mod seqi;
mod sge;
mod sgei;
mod sgeu;
mod sgeui;
mod sgt;
mod sgti;
mod sgtu;
mod sgtui;
mod sh;
mod sle;
mod slei;
mod sleu;
mod sleui;
mod sll;
mod slli;
mod slo;
mod slt;
mod slti;
mod sltu;
mod sltui;
mod sne;
mod snei;
mod sra;
mod srai;
mod srl;
mod srli;
mod sub;
mod subi;
mod sw;
mod xor;
mod xori;
