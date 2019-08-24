use num::FromPrimitive;
use std::num::Wrapping;
use std::ops::{Deref, DerefMut};

use super::super::memory::StorageMut;
use super::super::{constants, Word};
use super::{register_index, ALUFunct, ExitCode, OpCode, Register, RegisterId};

pub enum TickResult {
    Next,
    Jump(u32),
    Stop(ExitCode),
}

pub struct Core {
    registers: [Register; constants::REGISTER_COUNT],
    storage: Box<dyn StorageMut>,
}

impl Core {
    /// Constructs a new Core object.
    pub fn new<S: StorageMut + 'static>(storage: S) -> Core {
        Core {
            registers: [Register::new(); constants::REGISTER_COUNT],
            storage: Box::new(storage),
        }
    }

    pub fn storage(&self) -> &dyn StorageMut {
        self.storage.deref()
    }

    pub fn register(&self, id: RegisterId) -> &Register {
        &self.registers[register_index(id)]
    }

    pub fn tick(&mut self, instruction: Word, program_counter: u32) -> TickResult {
        let op_code = (instruction & constants::OPCODE_MASK) >> constants::OPCODE_OFFSET;
        let op_code = FromPrimitive::from_u32(op_code);

        let program_counter = Wrapping(program_counter);

        if let Some(op_code) = op_code {
            let rdid = ((instruction & constants::RD_MASK) >> constants::RD_OFFSET) as usize;
            let rs1id = ((instruction & constants::RS1_MASK) >> constants::RS1_OFFSET) as usize;
            let rs2id = ((instruction & constants::RS2_MASK) >> constants::RS2_OFFSET) as usize;

            let rs1 = self.registers[rs1id];
            let rs2 = self.registers[rs2id];

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
                    let funct_value =
                        (instruction & constants::FUNCT_MASK) >> constants::FUNCT_OFFSET;
                    let funct = ALUFunct::from_u32(funct_value);

                    if let Some(funct) = funct {
                        match funct {
                            ALUFunct::ADD => {
                                self.write_i(rdid, rs1i + rs2i);
                            }

                            ALUFunct::SUB => {
                                self.write_i(rdid, rs1i - rs2i);
                            }

                            ALUFunct::MUL => {
                                self.write_i(rdid, rs1i * rs2i);
                            }

                            ALUFunct::DIV => {
                                if !self.div(rdid, rs1i, rs2i) {
                                    return TickResult::Stop(ExitCode::DivisionByZero);
                                }
                            }

                            ALUFunct::AND => {
                                self.write_i(rdid, rs1i & rs2i);
                            }

                            ALUFunct::OR => {
                                self.write_i(rdid, rs1i | rs2i);
                            }

                            ALUFunct::XOR => {
                                self.write_i(rdid, rs1i ^ rs2i);
                            }

                            ALUFunct::SLL => {
                                self.write_i(rdid, rs1i << rs2u.0 as usize);
                            }

                            ALUFunct::SRL => {
                                self.write_u(rdid, rs1u >> rs2u.0 as usize);
                            }

                            ALUFunct::SRA => {
                                self.write_i(rdid, rs1i >> rs2u.0 as usize);
                            }

                            ALUFunct::SEQ => {
                                self.set_if(rdid, rs1i == rs2i);
                            }

                            ALUFunct::SNE => {
                                self.set_if(rdid, rs1i != rs2i);
                            }

                            ALUFunct::SLT => {
                                self.set_if(rdid, rs1i < rs2i);
                            }

                            ALUFunct::SGT => {
                                self.set_if(rdid, rs1i > rs2i);
                            }

                            ALUFunct::SLE => {
                                self.set_if(rdid, rs1i <= rs2i);
                            }

                            ALUFunct::SGE => {
                                self.set_if(rdid, rs1i >= rs2i);
                            }

                            ALUFunct::FADD => {
                                self.write_f(rdid, rs1f + rs2f);
                            }

                            ALUFunct::FSUB => {
                                self.write_f(rdid, rs1f - rs2f);
                            }

                            ALUFunct::FMUL => {
                                self.write_f(rdid, rs1f * rs2f);
                            }

                            ALUFunct::FDIV => {
                                self.write_f(rdid, rs1f / rs2f);
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
                    self.write_i(rdid, rs1i);
                }

                OpCode::LI => {
                    self.write_i(rdid, immediate);
                }

                OpCode::LHI => {
                    self.write_i(rdid, immediate << 16);
                }

                OpCode::LB => {
                    if !self.load(rdid, rs1u + immediateu, constants::BYTE_BYTES) {
                        return TickResult::Stop(ExitCode::BadMemoryAccess);
                    }
                }

                OpCode::LH => {
                    if !self.load(rdid, rs1u + immediateu, constants::HALF_BYTES) {
                        return TickResult::Stop(ExitCode::BadMemoryAccess);
                    }
                }

                OpCode::LW => {
                    if !self.load(rdid, rs1u + immediateu, constants::WORD_BYTES) {
                        return TickResult::Stop(ExitCode::BadMemoryAccess);
                    }
                }

                OpCode::SB => {
                    if !self.store(rdid, rs1u + immediateu, constants::BYTE_BYTES) {
                        return TickResult::Stop(ExitCode::BadMemoryAccess);
                    }
                }

                OpCode::SH => {
                    if !self.store(rdid, rs1u + immediateu, constants::HALF_BYTES) {
                        return TickResult::Stop(ExitCode::BadMemoryAccess);
                    }
                }

                OpCode::SW => {
                    if !self.store(rdid, rs1u + immediateu, constants::WORD_BYTES) {
                        return TickResult::Stop(ExitCode::BadMemoryAccess);
                    }
                }

                OpCode::ADDI => {
                    self.write_i(rdid, rs1i + immediate);
                }

                OpCode::SUBI => {
                    self.write_i(rdid, rs1i - immediate);
                }

                OpCode::MULI => {
                    self.write_i(rdid, rs1i * immediate);
                }

                OpCode::DIVI => {
                    if !self.div(rdid, rs1i, immediate) {
                        return TickResult::Stop(ExitCode::DivisionByZero);
                    }
                }

                OpCode::ANDI => {
                    self.write_i(rdid, rs1i & immediate);
                }

                OpCode::ORI => {
                    self.write_i(rdid, rs1i | immediate);
                }

                OpCode::XORI => {
                    self.write_i(rdid, rs1i ^ immediate);
                }

                OpCode::FLIP => {
                    self.write_i(rdid, !rs1i);
                }

                OpCode::SLLI => {
                    self.write_i(rdid, rs1i << immediateu.0 as usize);
                }

                OpCode::SRLI => {
                    self.write_u(rdid, rs1u >> immediateu.0 as usize);
                }

                OpCode::SRAI => {
                    self.write_i(rdid, rs1i >> immediateu.0 as usize);
                }

                OpCode::SEQI => {
                    self.set_if(rdid, rs1i == immediate);
                }

                OpCode::SNEI => {
                    self.set_if(rdid, rs1i != immediate);
                }

                OpCode::SLTI => {
                    self.set_if(rdid, rs1i < immediate);
                }

                OpCode::SGTI => {
                    self.set_if(rdid, rs1i > immediate);
                }

                OpCode::SLEI => {
                    self.set_if(rdid, rs1i <= immediate);
                }

                OpCode::SGEI => {
                    self.set_if(rdid, rs1i >= immediate);
                }

                OpCode::BEZ => {
                    if rs1i.0 == 0 {
                        return Self::jump(program_counter + immediateu);
                    }
                }

                OpCode::BNZ => {
                    if rs1i.0 != 0 {
                        return Self::jump(program_counter + immediateu);
                    }
                }

                OpCode::JMP => {
                    return Self::jump(program_counter + address);
                }

                OpCode::JL => {
                    self.link(program_counter);
                    return Self::jump(program_counter + address);
                }

                OpCode::JR => {
                    return Self::jump(rs1u);
                }

                OpCode::JLR => {
                    self.link(program_counter);
                    return Self::jump(rs1u);
                }

                OpCode::ITOF => self.write_f(rdid, rs1i.0 as f32),

                OpCode::FTOI => {
                    self.write_i(rdid, Wrapping(rs1f as i32));
                }
            }
        } else {
            return TickResult::Stop(ExitCode::InvalidOpcode);
        }

        TickResult::Next
    }

    fn write_i(&mut self, id: usize, value: Wrapping<i32>) {
        self.registers[id].set_i(value.0);
    }

    fn write_u(&mut self, id: usize, value: Wrapping<u32>) {
        self.registers[id].set_u(value.0);
    }

    fn write_f(&mut self, id: usize, value: f32) {
        self.registers[id].set_f(value);
    }

    fn div(&mut self, id: usize, dividend: Wrapping<i32>, divisor: Wrapping<i32>) -> bool {
        if divisor.0 == 0 {
            return false;
        }

        self.write_i(id, dividend / divisor);
        self.write_i(register_index(RegisterId::RM), dividend % divisor);

        true
    }

    fn set_if(&mut self, id: usize, condition: bool) {
        self.write_u(id, if condition { Wrapping(1) } else { Wrapping(0) });
    }

    fn link(&mut self, program_counter: Wrapping<u32>) {
        let addr = program_counter + Wrapping(constants::WORD_BYTES as u32);
        self.write_u(register_index(RegisterId::RA), addr);
    }

    fn load(&mut self, id: usize, address: Wrapping<u32>, size: u32) -> bool {
        self.storage
            .deref()
            .read(address.0, size)
            .map(|v| self.write_u(id, Wrapping(v)))
            .is_ok()
    }

    fn store(&mut self, id: usize, address: Wrapping<u32>, size: u32) -> bool {
        let value = self.registers[id].u();
        self.storage
            .deref_mut()
            .write(address.0, size, value)
            .is_ok()
    }

    fn jump(new_addr: Wrapping<u32>) -> TickResult {
        TickResult::Jump(new_addr.0)
    }
}
