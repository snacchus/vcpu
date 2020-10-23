use num::FromPrimitive;
use std::num::Wrapping;

use crate::{
    constants, register_index, AluFunct, ExitCode, FlopFunct, Opcode, Register, RegisterId,
    StorageMut, Word,
};

pub enum TickResult {
    Next,
    Jump(u32, bool),
    Stop(ExitCode),
}

fn write_i(registers: &mut [Register], id: usize, value: Wrapping<i32>) {
    if id != 0 {
        registers[id].set_i(value.0);
    }
}

fn write_u(registers: &mut [Register], id: usize, value: Wrapping<u32>) {
    if id != 0 {
        registers[id].set_u(value.0);
    }
}

fn write_f(registers: &mut [Register], id: usize, value: f32) {
    if id != 0 {
        registers[id].set_f(value);
    }
}

fn mul(registers: &mut [Register], id: usize, factor1: Wrapping<i32>, factor2: Wrapping<i32>) {
    let product = factor1.0 as i64 * factor2.0 as i64;
    registers[id].set_i(product as i32);
    registers[register_index(RegisterId::RM)]
        .set_i((product >> (std::mem::size_of::<i32>() * 8)) as i32);
}

fn div(
    registers: &mut [Register],
    id: usize,
    dividend: Wrapping<i32>,
    divisor: Wrapping<i32>,
) -> bool {
    if divisor.0 == 0 {
        return false;
    }

    write_i(registers, id, dividend / divisor);
    write_i(
        registers,
        register_index(RegisterId::RM),
        dividend % divisor,
    );

    true
}

fn set_if(registers: &mut [Register], id: usize, condition: bool) {
    write_u(
        registers,
        id,
        if condition { Wrapping(1) } else { Wrapping(0) },
    );
}

fn load(
    registers: &mut [Register],
    storage: &dyn StorageMut,
    id: usize,
    address: Wrapping<u32>,
    size: u32,
) -> bool {
    storage
        .read(address.0, size)
        .map(|v| write_u(registers, id, Wrapping(v)))
        .is_ok()
}

fn jump(new_addr: Wrapping<u32>, link: bool) -> TickResult {
    TickResult::Jump(new_addr.0, link)
}

pub fn tick(
    registers: &mut [Register],
    storage: &mut dyn StorageMut,
    instruction: Word,
    program_counter: u32,
) -> TickResult {
    let op_code = (instruction & constants::OPCODE_MASK) >> constants::OPCODE_OFFSET;
    let op_code = FromPrimitive::from_u32(op_code);

    let program_counter = Wrapping(program_counter);

    if let Some(op_code) = op_code {
        let rdid = ((instruction & constants::RD_MASK) >> constants::RD_OFFSET) as usize;
        let rs1id = ((instruction & constants::RS1_MASK) >> constants::RS1_OFFSET) as usize;
        let rs2id = ((instruction & constants::RS2_MASK) >> constants::RS2_OFFSET) as usize;

        let rd = &registers[rdid];
        let rs1 = &registers[rs1id];
        let rs2 = &registers[rs2id];

        let rs1i = Wrapping(rs1.i());
        let rs2i = Wrapping(rs2.i());
        let rs1u = Wrapping(rs1.u());
        let rs2u = Wrapping(rs2.u());
        let rs1f = rs1.f();
        let rs2f = rs2.f();

        let imm_i16 =
            ((instruction & constants::IMMEDIATE_MASK) >> constants::IMMEDIATE_OFFSET) as i16;
        let imm_u16 = imm_i16 as u16;
        let imm_u = Wrapping(imm_u16 as u32);
        let imm_i = Wrapping(imm_i16 as i32);
        let imm_u_ex = Wrapping(imm_i.0 as u32);

        let mut address = (instruction & constants::ADDRESS_MASK) >> constants::ADDRESS_OFFSET;

        if (address & constants::ADDRESS_SIGN_MASK) != 0 {
            address |= constants::ADDRESS_EXTENSION;
        }

        let address = Wrapping(address as u32);

        match op_code {
            Opcode::NOP => {}

            Opcode::ALU => {
                let funct_value = (instruction & constants::FUNCT_MASK) >> constants::FUNCT_OFFSET;
                let funct = AluFunct::from_u32(funct_value);

                if let Some(funct) = funct {
                    match funct {
                        AluFunct::ADD => {
                            write_i(registers, rdid, rs1i + rs2i);
                        }

                        AluFunct::SUB => {
                            write_i(registers, rdid, rs1i - rs2i);
                        }

                        AluFunct::MUL => {
                            mul(registers, rdid, rs1i, rs2i);
                        }

                        AluFunct::DIV => {
                            if !div(registers, rdid, rs1i, rs2i) {
                                return TickResult::Stop(ExitCode::DivisionByZero);
                            }
                        }

                        AluFunct::AND => {
                            write_i(registers, rdid, rs1i & rs2i);
                        }

                        AluFunct::OR => {
                            write_i(registers, rdid, rs1i | rs2i);
                        }

                        AluFunct::XOR => {
                            write_i(registers, rdid, rs1i ^ rs2i);
                        }

                        AluFunct::SLL => {
                            write_i(registers, rdid, rs1i << rs2u.0 as usize);
                        }

                        AluFunct::SRL => {
                            write_u(registers, rdid, rs1u >> rs2u.0 as usize);
                        }

                        AluFunct::SRA => {
                            write_i(registers, rdid, rs1i >> rs2u.0 as usize);
                        }

                        AluFunct::SEQ => {
                            set_if(registers, rdid, rs1i == rs2i);
                        }

                        AluFunct::SNE => {
                            set_if(registers, rdid, rs1i != rs2i);
                        }

                        AluFunct::SLT => {
                            set_if(registers, rdid, rs1i < rs2i);
                        }

                        AluFunct::SGT => {
                            set_if(registers, rdid, rs1i > rs2i);
                        }

                        AluFunct::SLE => {
                            set_if(registers, rdid, rs1i <= rs2i);
                        }

                        AluFunct::SGE => {
                            set_if(registers, rdid, rs1i >= rs2i);
                        }

                        AluFunct::SLTU => {
                            set_if(registers, rdid, rs1u < rs2u);
                        }

                        AluFunct::SGTU => {
                            set_if(registers, rdid, rs1u > rs2u);
                        }

                        AluFunct::SLEU => {
                            set_if(registers, rdid, rs1u <= rs2u);
                        }

                        AluFunct::SGEU => {
                            set_if(registers, rdid, rs1u >= rs2u);
                        }
                    }
                } else {
                    return TickResult::Stop(ExitCode::InvalidOpcode);
                }
            }

            Opcode::HALT => {
                return TickResult::Stop(ExitCode::Halted);
            }

            Opcode::CALL => {
                // TODO: define an interface for arbitrary syscalls (or figure out if they are necessary at all)
            }

            Opcode::COPY => {
                write_i(registers, rdid, rs1i);
            }

            Opcode::LI => {
                write_i(registers, rdid, imm_i);
            }

            Opcode::LHI => {
                write_i(registers, rdid, imm_i << 16);
            }

            Opcode::SLO => {
                let high = Wrapping(rd.u() & !constants::LOW_BITS_MASK);
                write_u(registers, rdid, imm_u | high);
            }

            Opcode::SHI => {
                let low = Wrapping(rd.u() & !constants::HIGH_BITS_MASK);
                write_u(registers, rdid, (imm_u << 16) | low);
            }

            Opcode::LB => {
                if !load(
                    registers,
                    storage,
                    rdid,
                    rs1u + imm_u_ex,
                    constants::BYTE_BYTES,
                ) {
                    return TickResult::Stop(ExitCode::BadMemoryAccess);
                }
            }

            Opcode::LH => {
                if !load(
                    registers,
                    storage,
                    rdid,
                    rs1u + imm_u_ex,
                    constants::HALF_BYTES,
                ) {
                    return TickResult::Stop(ExitCode::BadMemoryAccess);
                }
            }

            Opcode::LW => {
                if !load(
                    registers,
                    storage,
                    rdid,
                    rs1u + imm_u_ex,
                    constants::WORD_BYTES,
                ) {
                    return TickResult::Stop(ExitCode::BadMemoryAccess);
                }
            }

            Opcode::SB => {
                if storage
                    .write_byte((rs1u + imm_u_ex).0, rd.u() as u8)
                    .is_err()
                {
                    return TickResult::Stop(ExitCode::BadMemoryAccess);
                }
            }

            Opcode::SH => {
                if storage
                    .write_half((rs1u + imm_u_ex).0, rd.u() as u16)
                    .is_err()
                {
                    return TickResult::Stop(ExitCode::BadMemoryAccess);
                }
            }

            Opcode::SW => {
                if storage
                    .write_word((rs1u + imm_u_ex).0, rd.u() as u32)
                    .is_err()
                {
                    return TickResult::Stop(ExitCode::BadMemoryAccess);
                }
            }

            Opcode::ADDI => {
                write_i(registers, rdid, rs1i + imm_i);
            }

            Opcode::SUBI => {
                write_i(registers, rdid, rs1i - imm_i);
            }

            Opcode::MULI => {
                mul(registers, rdid, rs1i, imm_i);
            }

            Opcode::DIVI => {
                if !div(registers, rdid, rs1i, imm_i) {
                    return TickResult::Stop(ExitCode::DivisionByZero);
                }
            }

            Opcode::ANDI => {
                write_i(registers, rdid, rs1i & imm_i);
            }

            Opcode::ORI => {
                write_i(registers, rdid, rs1i | imm_i);
            }

            Opcode::XORI => {
                write_i(registers, rdid, rs1i ^ imm_i);
            }

            Opcode::FLIP => {
                write_i(registers, rdid, !rs1i);
            }

            Opcode::SLLI => {
                write_i(registers, rdid, rs1i << imm_u_ex.0 as usize);
            }

            Opcode::SRLI => {
                write_u(registers, rdid, rs1u >> imm_u_ex.0 as usize);
            }

            Opcode::SRAI => {
                write_i(registers, rdid, rs1i >> imm_u_ex.0 as usize);
            }

            Opcode::SEQI => {
                set_if(registers, rdid, rs1i == imm_i);
            }

            Opcode::SNEI => {
                set_if(registers, rdid, rs1i != imm_i);
            }

            Opcode::SLTI => {
                set_if(registers, rdid, rs1i < imm_i);
            }

            Opcode::SGTI => {
                set_if(registers, rdid, rs1i > imm_i);
            }

            Opcode::SLEI => {
                set_if(registers, rdid, rs1i <= imm_i);
            }

            Opcode::SGEI => {
                set_if(registers, rdid, rs1i >= imm_i);
            }

            Opcode::SLTUI => {
                set_if(registers, rdid, rs1u < imm_u);
            }

            Opcode::SGTUI => {
                set_if(registers, rdid, rs1u > imm_u);
            }

            Opcode::SLEUI => {
                set_if(registers, rdid, rs1u <= imm_u);
            }

            Opcode::SGEUI => {
                set_if(registers, rdid, rs1u >= imm_u);
            }

            Opcode::BEZ => {
                if rs1i.0 == 0 {
                    return jump(program_counter + imm_u_ex, false);
                }
            }

            Opcode::BNZ => {
                if rs1i.0 != 0 {
                    return jump(program_counter + imm_u_ex, false);
                }
            }

            Opcode::JMP => {
                return jump(program_counter + address, false);
            }

            Opcode::JL => {
                return jump(program_counter + address, true);
            }

            Opcode::JR => {
                return jump(rs1u, false);
            }

            Opcode::JLR => {
                return jump(rs1u, true);
            }

            Opcode::ITOF => write_f(registers, rdid, rs1i.0 as f32),

            Opcode::FTOI => {
                let i = if rs1f.is_finite() {
                    rs1f as i32
                } else {
                    i32::MIN
                };
                write_i(registers, rdid, Wrapping(i));
            }

            Opcode::FLOP => {
                let funct_value = (instruction & constants::FUNCT_MASK) >> constants::FUNCT_OFFSET;
                let funct = FlopFunct::from_u32(funct_value);
                if let Some(funct) = funct {
                    match funct {
                        FlopFunct::FADD => {
                            write_f(registers, rdid, rs1f + rs2f);
                        }

                        FlopFunct::FSUB => {
                            write_f(registers, rdid, rs1f - rs2f);
                        }

                        FlopFunct::FMUL => {
                            write_f(registers, rdid, rs1f * rs2f);
                        }

                        FlopFunct::FDIV => {
                            write_f(registers, rdid, rs1f / rs2f);
                        }
                    }
                } else {
                    return TickResult::Stop(ExitCode::InvalidOpcode);
                }
            }
        }
    } else {
        return TickResult::Stop(ExitCode::InvalidOpcode);
    }

    TickResult::Next
}
