use num::FromPrimitive;
use std::num::Wrapping;

use crate::memory::StorageMut;
use crate::{constants, register_index, ALUFunct, ExitCode, OpCode, Register, RegisterId, Word};

pub enum TickResult {
    Next,
    Jump(u32),
    Stop(ExitCode),
}

fn write_i(registers: &mut [Register], id: usize, value: Wrapping<i32>) {
    registers[id].set_i(value.0);
}

fn write_u(registers: &mut [Register], id: usize, value: Wrapping<u32>) {
    registers[id].set_u(value.0);
}

fn write_f(registers: &mut [Register], id: usize, value: f32) {
    registers[id].set_f(value);
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

fn link(registers: &mut [Register], program_counter: Wrapping<u32>) {
    let addr = program_counter + Wrapping(constants::WORD_BYTES as u32);
    write_u(registers, register_index(RegisterId::RA), addr);
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

fn store(
    registers: &mut [Register],
    storage: &mut dyn StorageMut,
    id: usize,
    address: Wrapping<u32>,
    size: u32,
) -> bool {
    let value = registers[id].u();
    storage.write(address.0, size, value).is_ok()
}

fn jump(new_addr: Wrapping<u32>) -> TickResult {
    TickResult::Jump(new_addr.0)
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

        let rs1 = registers[rs1id];
        let rs2 = registers[rs2id];

        let rs1i = Wrapping(rs1.i());
        let rs2i = Wrapping(rs2.i());
        let rs1u = Wrapping(rs1.u());
        let rs2u = Wrapping(rs2.u());
        let rs1f = rs1.f();
        let rs2f = rs2.f();

        let immediate = Wrapping(
            ((instruction & constants::IMMEDIATE_MASK) >> constants::IMMEDIATE_OFFSET) as i32,
        );
        let immediateu = Wrapping(immediate.0 as u32);

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
                            write_i(registers, rdid, rs1i * rs2i);
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

                        ALUFunct::FADD => {
                            write_f(registers, rdid, rs1f + rs2f);
                        }

                        ALUFunct::FSUB => {
                            write_f(registers, rdid, rs1f - rs2f);
                        }

                        ALUFunct::FMUL => {
                            write_f(registers, rdid, rs1f * rs2f);
                        }

                        ALUFunct::FDIV => {
                            write_f(registers, rdid, rs1f / rs2f);
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
                // TODO
            }

            OpCode::COPY => {
                write_i(registers, rdid, rs1i);
            }

            OpCode::LI => {
                write_i(registers, rdid, immediate);
            }

            OpCode::LHI => {
                write_i(registers, rdid, immediate << 16);
            }

            OpCode::LB => {
                if !load(
                    registers,
                    storage,
                    rdid,
                    rs1u + immediateu,
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
                    rs1u + immediateu,
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
                    rs1u + immediateu,
                    constants::WORD_BYTES,
                ) {
                    return TickResult::Stop(ExitCode::BadMemoryAccess);
                }
            }

            OpCode::SB => {
                if !store(
                    registers,
                    storage,
                    rdid,
                    rs1u + immediateu,
                    constants::BYTE_BYTES,
                ) {
                    return TickResult::Stop(ExitCode::BadMemoryAccess);
                }
            }

            OpCode::SH => {
                if !store(
                    registers,
                    storage,
                    rdid,
                    rs1u + immediateu,
                    constants::HALF_BYTES,
                ) {
                    return TickResult::Stop(ExitCode::BadMemoryAccess);
                }
            }

            OpCode::SW => {
                if !store(
                    registers,
                    storage,
                    rdid,
                    rs1u + immediateu,
                    constants::WORD_BYTES,
                ) {
                    return TickResult::Stop(ExitCode::BadMemoryAccess);
                }
            }

            OpCode::ADDI => {
                write_i(registers, rdid, rs1i + immediate);
            }

            OpCode::SUBI => {
                write_i(registers, rdid, rs1i - immediate);
            }

            OpCode::MULI => {
                write_i(registers, rdid, rs1i * immediate);
            }

            OpCode::DIVI => {
                if !div(registers, rdid, rs1i, immediate) {
                    return TickResult::Stop(ExitCode::DivisionByZero);
                }
            }

            OpCode::ANDI => {
                write_i(registers, rdid, rs1i & immediate);
            }

            OpCode::ORI => {
                write_i(registers, rdid, rs1i | immediate);
            }

            OpCode::XORI => {
                write_i(registers, rdid, rs1i ^ immediate);
            }

            OpCode::FLIP => {
                write_i(registers, rdid, !rs1i);
            }

            OpCode::SLLI => {
                write_i(registers, rdid, rs1i << immediateu.0 as usize);
            }

            OpCode::SRLI => {
                write_u(registers, rdid, rs1u >> immediateu.0 as usize);
            }

            OpCode::SRAI => {
                write_i(registers, rdid, rs1i >> immediateu.0 as usize);
            }

            OpCode::SEQI => {
                set_if(registers, rdid, rs1i == immediate);
            }

            OpCode::SNEI => {
                set_if(registers, rdid, rs1i != immediate);
            }

            OpCode::SLTI => {
                set_if(registers, rdid, rs1i < immediate);
            }

            OpCode::SGTI => {
                set_if(registers, rdid, rs1i > immediate);
            }

            OpCode::SLEI => {
                set_if(registers, rdid, rs1i <= immediate);
            }

            OpCode::SGEI => {
                set_if(registers, rdid, rs1i >= immediate);
            }

            OpCode::BEZ => {
                if rs1i.0 == 0 {
                    return jump(program_counter + immediateu);
                }
            }

            OpCode::BNZ => {
                if rs1i.0 != 0 {
                    return jump(program_counter + immediateu);
                }
            }

            OpCode::JMP => {
                return jump(program_counter + address);
            }

            OpCode::JL => {
                link(registers, program_counter);
                return jump(program_counter + address);
            }

            OpCode::JR => {
                return jump(rs1u);
            }

            OpCode::JLR => {
                link(registers, program_counter);
                return jump(rs1u);
            }

            OpCode::ITOF => write_f(registers, rdid, rs1i.0 as f32),

            OpCode::FTOI => {
                write_i(registers, rdid, Wrapping(rs1f as i32));
            }
        }
    } else {
        return TickResult::Stop(ExitCode::InvalidOpcode);
    }

    TickResult::Next
}
