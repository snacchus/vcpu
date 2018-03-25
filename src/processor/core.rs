extern crate num_integer;

use std::num::Wrapping;
use num::FromPrimitive;

use super::super::{constants, Address, Word};
use super::{register_index, OpCodeR, ExitCode, RegisterId, Register, OpCode};

pub enum TickResult {
    Running,
    Exit(ExitCode),
}

pub struct Core {
    registers: [Register; constants::REGISTER_COUNT],
    instructions: Vec<Word>,
    program_counter: Wrapping<usize>,
}

impl Core {
    /// Constructs a new Core object.
    pub fn new() -> Core {
        Core {
            registers: [Register::new(); constants::REGISTER_COUNT],
            instructions: Vec::new(),
            program_counter: Wrapping(0),
        }
    }

    pub fn zero_registers(&mut self) {
        for i in 0..constants::REGISTER_COUNT {
            self.registers[i] = Register::new();
        }
    }

    pub fn register(&self, id: RegisterId) -> &Register {
        &self.registers[register_index(id)]
    }

    pub fn load_program(&mut self, data: &[Word]) {
        self.instructions = Vec::from(data);
    }

    pub fn tick(&mut self) -> TickResult {
        let i = (self.program_counter / Wrapping(constants::WORD_BYTES)).0;
        let instr = self.instructions[i];

        let op_code_value = (instr & constants::OPCODE_MASK) >> constants::OPCODE_OFFSET;
        let op_code = FromPrimitive::from_u32(op_code_value);

        let mut jumped = false;

        if let Some(op_code) = op_code {
            let rdid  = ((instr & constants::RD_MASK)  >> constants::RD_OFFSET) as usize;
            let rs1id = ((instr & constants::RS1_MASK) >> constants::RS1_OFFSET) as usize;
            let rs2id = ((instr & constants::RS2_MASK) >> constants::RS2_OFFSET) as usize;

            let rs1 = self.registers[rs1id];
            let rs2 = self.registers[rs2id];

            let rs1i = Wrapping(rs1.i());
            let rs2i = Wrapping(rs2.i());
            let rs1u = Wrapping(rs1.u());
            let rs2u = Wrapping(rs2.u());
            let rs1f = rs1.f();
            let rs2f = rs2.f();

            let immediate = Wrapping(((instr & constants::IMMEDIATE_MASK) >> constants::IMMEDIATE_OFFSET) as i32);
            let immediateu = Wrapping(immediate.0 as u32);

            let mut addr   = (instr & constants::ADDRESS_MASK) >> constants::ADDRESS_OFFSET;

            if (addr & constants::ADDRESS_SIGN_MASK) != 0
            {
                addr |= constants::ADDRESS_EXTENSION;
            }

            let address = Wrapping(addr as Address);
            let addressu = Wrapping(address.0 as u32);

            match op_code {
                OpCode::NOP => { },

                OpCode::RIN => {
                    let op_code_r_value = (instr & constants::OPCODE_R_MASK) >> constants::OPCODE_R_OFFSET;
                    let op_code_r = OpCodeR::from_u32(op_code_r_value);

                    if let Some(op_code_r) = op_code_r {
                        match op_code_r {
                            OpCodeR::ADD => {
                                self.write_i(rdid, rs1i + rs2i);
                            },

                            OpCodeR::SUB => {
                                self.write_i(rdid, rs1i - rs2i);
                            },

                            OpCodeR::MUL => {
                                self.write_i(rdid, rs1i * rs2i);
                            },

                            OpCodeR::DIV => {
                                if !self.div(rdid, rs1i, rs2i) {
                                    return TickResult::Exit(ExitCode::DivisionByZero);
                                }
                            },

                            OpCodeR::AND => {
                                self.write_i(rdid, rs1i & rs2i);
                            },

                            OpCodeR::OR => {
                                self.write_i(rdid, rs1i | rs2i);
                            },

                            OpCodeR::XOR => {
                                self.write_i(rdid, rs1i ^ rs2i);
                            },

                            OpCodeR::SLL => {
                                self.write_i(rdid, rs1i << rs2u.0 as usize);
                            },

                            OpCodeR::SRL => {
                                self.write_u(rdid, rs1u >> rs2u.0 as usize);
                            },

                            OpCodeR::SRA => {
                                self.write_i(rdid, rs1i >> rs2u.0 as usize);
                            },

                            OpCodeR::SEQ => {
                                self.set_if(rdid, rs1i == rs2i);
                            },

                            OpCodeR::SNE => {
                                self.set_if(rdid, rs1i != rs2i);
                            },

                            OpCodeR::SLT => {
                                self.set_if(rdid, rs1i < rs2i);
                            },

                            OpCodeR::SGT => {
                                self.set_if(rdid, rs1i > rs2i);
                            },

                            OpCodeR::SLE => {
                                self.set_if(rdid, rs1i <= rs2i);
                            },

                            OpCodeR::SGE => {
                                self.set_if(rdid, rs1i >= rs2i);
                            },

                            OpCodeR::FADD => {
                                self.write_f(rdid, rs1f + rs2f);
                            },

                            OpCodeR::FSUB => {
                                self.write_f(rdid, rs1f - rs2f);
                            },

                            OpCodeR::FMUL => {
                                self.write_f(rdid, rs1f * rs2f);
                            },

                            OpCodeR::FDIV => {
                                self.write_f(rdid, rs1f / rs2f);
                            }
                        }
                    } else {
                        return TickResult::Exit(ExitCode::InvalidOpcode);
                    }
                },

                OpCode::HALT => {
                    return TickResult::Exit(ExitCode::Halted);
                },

                OpCode::CALL => {
                    // TODO
                },

                OpCode::COPY => {
                    self.write_i(rdid, rs1i);
                },

                OpCode::LI => {
                    self.write_i(rdid, immediate);
                },

                OpCode::LHI => {
                    self.write_i(rdid, immediate << 16);
                },

                OpCode::LOAD => {
                    //let mem_addr = rs1u + immediateu;
                    /*if (!m_pMemory::Read(memAddr, &rd.u))
                    {
                        exitCode = EC_BAD_MEMORY_ACCESS;
                    }*/
                },

                OpCode::STOR => {
                    //let mem_addr = rs1u + immediateu;
                    /*if (!m_pMemory::Write(memAddr, rd.u))
                    {
                        exitCode = EC_BAD_MEMORY_ACCESS;
                    }*/
                },

                OpCode::ADDI => {
                    self.write_i(rdid, rs1i + immediate);
                },

                OpCode::SUBI => {
                    self.write_i(rdid, rs1i - immediate);
                },

                OpCode::MULI => {
                    self.write_i(rdid, rs1i * immediate);
                },

                OpCode::DIVI => {
                    if !self.div(rdid, rs1i, immediate) {
                        return TickResult::Exit(ExitCode::DivisionByZero);
                    }
                },

                OpCode::ANDI => {
                    self.write_i(rdid, rs1i & immediate);
                },

                OpCode::ORI => {
                    self.write_i(rdid, rs1i | immediate);
                },

                OpCode::XORI => {
                    self.write_i(rdid, rs1i ^ immediate);
                },

                OpCode::FLIP => {
                    self.write_i(rdid, !rs1i);
                },

                OpCode::SLLI => {
                    self.write_i(rdid, rs1i << immediateu.0 as usize);
                },

                OpCode::SRLI => {
                    self.write_u(rdid, rs1u >> immediateu.0 as usize);
                },

                OpCode::SRAI => {
                    self.write_i(rdid, rs1i >> immediateu.0 as usize);
                },

                OpCode::SEQI => {
                    self.set_if(rdid, rs1i == immediate);
                },

                OpCode::SNEI => {
                    self.set_if(rdid, rs1i != immediate);
                },

                OpCode::SLTI => {
                    self.set_if(rdid, rs1i < immediate);
                },

                OpCode::SGTI => {
                    self.set_if(rdid, rs1i > immediate);
                },

                OpCode::SLEI => {
                    self.set_if(rdid, rs1i <= immediate);
                },

                OpCode::SGEI => {
                    self.set_if(rdid, rs1i >= immediate);
                },

                OpCode::BEZ => {
                    if rs1i.0 == 0 {
                        self.program_counter += Wrapping(immediateu.0 as usize);
                        jumped = true;
                    }
                },

                OpCode::BNZ => {
                    if rs1i.0 != 0 {
                        self.program_counter += Wrapping(immediateu.0 as usize);
                        jumped = true;
                    }
                },

                OpCode::JMP => {
                    self.program_counter += Wrapping(addressu.0 as usize);
                    jumped = true;
                },

                OpCode::JL => {
                    let new_addr = self.program_counter + Wrapping(addressu.0 as usize);
                    self.jump_and_link(new_addr);
                    jumped = true;
                },

                OpCode::JR => {
                    self.program_counter = Wrapping(rs1u.0 as usize);
                    jumped = true;
                },

                OpCode::JLR => {
                    self.jump_and_link(Wrapping(rs1u.0 as usize));
                    jumped = true;
                },

                OpCode::ITOF => {
                    self.write_f(rdid, rs1i.0 as f32)
                },

                OpCode::FTOI => {
                    self.write_i(rdid, Wrapping(rs1f as i32));
                },
            }
        } else {
            return TickResult::Exit(ExitCode::InvalidOpcode);
        }

        if !jumped {
            self.program_counter += Wrapping(constants::WORD_BYTES);
        } else {
            if self.program_counter.0 > self.instructions.len() {
                return TickResult::Exit(ExitCode::BadJump);
            } else if (self.program_counter.0 % constants::WORD_BYTES) != 0 {
                return TickResult::Exit(ExitCode::BadAlignment);
            }
        }

        TickResult::Running
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
        self.write_u(id, set_if(condition));
    }

    fn jump_and_link(&mut self, new_addr: Wrapping<usize>) {
        let addr = self.program_counter + Wrapping(constants::WORD_BYTES);
        self.write_u(register_index(RegisterId::RA), Wrapping(addr.0 as u32));
        self.program_counter = new_addr;
    }
}

fn set_if(condition: bool) -> Wrapping<u32> {
    if condition { Wrapping(1) } else { Wrapping(0) }
}