use num::FromPrimitive;
use std::num::Wrapping;

use crate::memory::StorageMut;
use crate::{
    constants, register_index, ALUFunct, ExitCode, FLOPFunct, OpCode, Register, RegisterId, Word,
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
            OpCode::NOP => {}

            OpCode::ALU => {
                let funct_value = (instruction & constants::FUNCT_MASK) >> constants::FUNCT_OFFSET;
                let funct = ALUFunct::from_u32(funct_value);

                if let Some(funct) = funct {
                    match funct {
                        ALUFunct::ADD => {
                            write_i(registers, rdid, rs1i + rs2i);
                        }

                        ALUFunct::SUB => {
                            write_i(registers, rdid, rs1i - rs2i);
                        }

                        ALUFunct::MUL => {
                            mul(registers, rdid, rs1i, rs2i);
                        }

                        ALUFunct::DIV => {
                            if !div(registers, rdid, rs1i, rs2i) {
                                return TickResult::Stop(ExitCode::DivisionByZero);
                            }
                        }

                        ALUFunct::AND => {
                            write_i(registers, rdid, rs1i & rs2i);
                        }

                        ALUFunct::OR => {
                            write_i(registers, rdid, rs1i | rs2i);
                        }

                        ALUFunct::XOR => {
                            write_i(registers, rdid, rs1i ^ rs2i);
                        }

                        ALUFunct::SLL => {
                            write_i(registers, rdid, rs1i << rs2u.0 as usize);
                        }

                        ALUFunct::SRL => {
                            write_u(registers, rdid, rs1u >> rs2u.0 as usize);
                        }

                        ALUFunct::SRA => {
                            write_i(registers, rdid, rs1i >> rs2u.0 as usize);
                        }

                        ALUFunct::SEQ => {
                            set_if(registers, rdid, rs1i == rs2i);
                        }

                        ALUFunct::SNE => {
                            set_if(registers, rdid, rs1i != rs2i);
                        }

                        ALUFunct::SLT => {
                            set_if(registers, rdid, rs1i < rs2i);
                        }

                        ALUFunct::SGT => {
                            set_if(registers, rdid, rs1i > rs2i);
                        }

                        ALUFunct::SLE => {
                            set_if(registers, rdid, rs1i <= rs2i);
                        }

                        ALUFunct::SGE => {
                            set_if(registers, rdid, rs1i >= rs2i);
                        }

                        ALUFunct::SLTU => {
                            set_if(registers, rdid, rs1u < rs2u);
                        }

                        ALUFunct::SGTU => {
                            set_if(registers, rdid, rs1u > rs2u);
                        }

                        ALUFunct::SLEU => {
                            set_if(registers, rdid, rs1u <= rs2u);
                        }

                        ALUFunct::SGEU => {
                            set_if(registers, rdid, rs1u >= rs2u);
                        }
                    }
                } else {
                    return TickResult::Stop(ExitCode::InvalidOpcode);
                }
            }

            OpCode::HALT => {
                return TickResult::Stop(ExitCode::Halted);
            }

            OpCode::CALL => {
                // TODO: define an interface for arbitrary syscalls (or figure out if they are necessary at all)
            }

            OpCode::COPY => {
                write_i(registers, rdid, rs1i);
            }

            OpCode::LI => {
                write_i(registers, rdid, imm_i);
            }

            OpCode::LHI => {
                write_i(registers, rdid, imm_i << 16);
            }

            OpCode::SLO => {
                let high = Wrapping(rd.u() & !constants::LOW_BITS_MASK);
                write_u(registers, rdid, imm_u | high);
            }

            OpCode::SHI => {
                let low = Wrapping(rd.u() & !constants::HIGH_BITS_MASK);
                write_u(registers, rdid, (imm_u << 16) | low);
            }

            OpCode::LB => {
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

            OpCode::LH => {
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

            OpCode::LW => {
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

            OpCode::SB => {
                if storage
                    .write_byte((rs1u + imm_u_ex).0, rd.u() as u8)
                    .is_err()
                {
                    return TickResult::Stop(ExitCode::BadMemoryAccess);
                }
            }

            OpCode::SH => {
                if storage
                    .write_half((rs1u + imm_u_ex).0, rd.u() as u16)
                    .is_err()
                {
                    return TickResult::Stop(ExitCode::BadMemoryAccess);
                }
            }

            OpCode::SW => {
                if storage
                    .write_word((rs1u + imm_u_ex).0, rd.u() as u32)
                    .is_err()
                {
                    return TickResult::Stop(ExitCode::BadMemoryAccess);
                }
            }

            OpCode::ADDI => {
                write_i(registers, rdid, rs1i + imm_i);
            }

            OpCode::SUBI => {
                write_i(registers, rdid, rs1i - imm_i);
            }

            OpCode::MULI => {
                mul(registers, rdid, rs1i, imm_i);
            }

            OpCode::DIVI => {
                if !div(registers, rdid, rs1i, imm_i) {
                    return TickResult::Stop(ExitCode::DivisionByZero);
                }
            }

            OpCode::ANDI => {
                write_i(registers, rdid, rs1i & imm_i);
            }

            OpCode::ORI => {
                write_i(registers, rdid, rs1i | imm_i);
            }

            OpCode::XORI => {
                write_i(registers, rdid, rs1i ^ imm_i);
            }

            OpCode::FLIP => {
                write_i(registers, rdid, !rs1i);
            }

            OpCode::SLLI => {
                write_i(registers, rdid, rs1i << imm_u_ex.0 as usize);
            }

            OpCode::SRLI => {
                write_u(registers, rdid, rs1u >> imm_u_ex.0 as usize);
            }

            OpCode::SRAI => {
                write_i(registers, rdid, rs1i >> imm_u_ex.0 as usize);
            }

            OpCode::SEQI => {
                set_if(registers, rdid, rs1i == imm_i);
            }

            OpCode::SNEI => {
                set_if(registers, rdid, rs1i != imm_i);
            }

            OpCode::SLTI => {
                set_if(registers, rdid, rs1i < imm_i);
            }

            OpCode::SGTI => {
                set_if(registers, rdid, rs1i > imm_i);
            }

            OpCode::SLEI => {
                set_if(registers, rdid, rs1i <= imm_i);
            }

            OpCode::SGEI => {
                set_if(registers, rdid, rs1i >= imm_i);
            }

            OpCode::SLTUI => {
                set_if(registers, rdid, rs1u < imm_u);
            }

            OpCode::SGTUI => {
                set_if(registers, rdid, rs1u > imm_u);
            }

            OpCode::SLEUI => {
                set_if(registers, rdid, rs1u <= imm_u);
            }

            OpCode::SGEUI => {
                set_if(registers, rdid, rs1u >= imm_u);
            }

            OpCode::BEZ => {
                if rs1i.0 == 0 {
                    return jump(program_counter + imm_u_ex, false);
                }
            }

            OpCode::BNZ => {
                if rs1i.0 != 0 {
                    return jump(program_counter + imm_u_ex, false);
                }
            }

            OpCode::JMP => {
                return jump(program_counter + address, false);
            }

            OpCode::JL => {
                return jump(program_counter + address, true);
            }

            OpCode::JR => {
                return jump(rs1u, false);
            }

            OpCode::JLR => {
                return jump(rs1u, true);
            }

            OpCode::ITOF => write_f(registers, rdid, rs1i.0 as f32),

            OpCode::FTOI => {
                write_i(registers, rdid, Wrapping(rs1f as i32));
            }

            OpCode::FLOP => {
                let funct_value = (instruction & constants::FUNCT_MASK) >> constants::FUNCT_OFFSET;
                let funct = FLOPFunct::from_u32(funct_value);
                if let Some(funct) = funct {
                    match funct {
                        FLOPFunct::FADD => {
                            write_f(registers, rdid, rs1f + rs2f);
                        }

                        FLOPFunct::FSUB => {
                            write_f(registers, rdid, rs1f - rs2f);
                        }

                        FLOPFunct::FMUL => {
                            write_f(registers, rdid, rs1f * rs2f);
                        }

                        FLOPFunct::FDIV => {
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
