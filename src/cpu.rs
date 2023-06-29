use crate::bus::*;
use crate::dram::*;

pub struct Cpu {
    pub regs: [u64; 32],
    pub pc: u64,
    // pub csrs: [u64; 4096],
    pub bus: Bus,
}

impl Cpu {
    pub fn new(binary: Vec<u8>) -> Self {
        let mut regs = [0; 32];
        regs[2] = DRAM_BASE + DRAM_SIZE;

        Self {
            regs,
            pc: DRAM_BASE,
            bus: Bus::new(binary)
        }
    }

    // pub fn fetch(&self) -> u32 {
    //     let index = self.pc as usize;
    //     return (self.dram[index] as u32)
    //         | ((self.dram[index + 1] as u32) << 8)
    //         | ((self.dram[index + 2] as u32) << 16)
    //         | ((self.dram[index + 3] as u32) << 24);
    // }

    pub fn load(&mut self, addr: u64, size: u64) -> Result<u64, ()> {
        self.bus.load(addr, size)
    }

    pub fn store(&mut self, addr: u64, size: u64, value: u64) -> Result<(), ()> {
        self.bus.store(addr, size, value)
    }

    pub fn fetch(&mut self) -> Result<u64, ()> {
        match self.bus.load(self.pc, 32) {
            Ok(inst) => Ok(inst),
            Err(_) => Err(()),
        }
    }

    // fn load_csr(&self, addr: usize) -> u64 {
    //     match addr {
    //         SIE => self.csrs[MIE] & self.csrs[MIDELEG],
    //         _ => self.csrs[addr],
    //     }
    // }

    // fn store_csr(&mut self, addr: usize, value: u64)  {
    //     match addr {
    //         SIE => {
    //             self.csrs[MIE] = (self.csrs[MIE] & !self.csrs[MIDELEG]) | (value & self.csrs[MIDELEG]);
    //         }
    //         _ => self.csrs[addr] = value,
    //     }
    // }

    pub fn execute(&mut self, instruction: u32) -> Result<(), ()> {
        let opcode = instruction & 0x7f;
        let rd = ((instruction >> 7) & 0x1f) as usize;
        let rs1 = ((instruction >> 15) & 0x1f) as usize;
        let rs2 = ((instruction >> 20) & 0x1f) as usize;
        let funct3 = (instruction >> 12) & 0x7;
        let funct7 = (instruction >> 25) & 0x7f;

        self.regs[0] = 0;

        match opcode {
            0x03 => {
                // Load instructions
                // imm[11:0] = inst[31:20]
                let imm = ((instruction as i32 as i64) >> 20) as u64;
                let addr = self.regs[rs1].wrapping_add(imm);

                match funct3 {
                    0x0 => {
                        // LB
                        let val = self.load(addr, 8)?;
                        self.regs[rd] = val as i8 as i64 as u64;
                    }
                    0x1 => {
                        // LH
                        let val = self.load(addr, 16)?;
                        self.regs[rd] = val as i16 as i64 as u64;
                    }
                    0x2 => {
                        // LW
                        let val = self.load(addr, 32)?;
                        self.regs[rd] = val as i32 as i64 as u64;
                    }
                    0x3 => {
                        // LD
                        let val = self.load(addr, 64)?;
                        self.regs[rd] = val;
                    }
                    0x4 => {
                        // LBU
                        let val = self.load(addr, 8)?;
                        self.regs[rd] = val;
                    }
                    0x5 => {
                        // LHU
                        let val = self.load(addr, 16)?;
                        self.regs[rd] = val;
                    }
                    0x6 => {
                        // LWU
                        let val = self.load(addr, 32)?;
                        self.regs[rd] = val;
                    }
                    _ => {
                        println!(
                            "not implemented yet, opcode: {:#x} funct3 {:#x}",
                            opcode, funct3
                        )
                    }
                }
            }
            0x13 => {
                let imm = ((instruction & 0xffff_0000) as i32 as i64 >> 20) as u64;
                // Shift amount is in the lower 6 bits of the I-immediate field
                let shift_amount = (imm & 0x3f) as u32;

                match funct3 {
                    0x0 => {
                        // Addi
                        self.regs[rd] = self.regs[rs1].wrapping_add(imm);
                    }
                    0x1 => {
                        // Slli
                        self.regs[rd] = self.regs[rs1] << shift_amount;
                    }
                    0x2 => {
                        // Slti
                        self.regs[rd] = if (self.regs[rs1] as i64) < (imm as i64) {
                            1
                        } else {
                            0
                        };
                    }
                    0x3 => {
                        // Sltiu
                        self.regs[rd] = if self.regs[rs1] < imm { 1 } else { 0 };
                    }
                    0x4 => {
                        // xori
                        self.regs[rd] = self.regs[rs1] ^ imm;
                    }
                    0x5 => {
                        match funct7 >> 1 {
                            // Srli
                            0x00 => self.regs[rd] = self.regs[rs1].wrapping_shr(shift_amount),
                            0x10 => self.regs[rd] = (self.regs[rs1] as i64).wrapping_shr(shift_amount) as u64,
                            _ => {}
                        }
                    }
                    0x6 => self.regs[rd] = self.regs[rs1] | imm, // Ori
                    0x7 => self.regs[rd] = self.regs[rs1] & imm, // Andi
                    _ => {}
                }
            }
            0x17 => {
                // Auipc
                let imm = (instruction & 0xffff_f000) as i32 as i64 as u64;
                self.regs[rd] = self.pc.wrapping_add(imm).wrapping_sub(4);
            }
            0x1b => {
                let imm = ((instruction as i32 as i64) >> 20) as u64;
                let shift_amount = (imm & 0x1f) as u32;

                match funct3 {
                    0x0 => {
                        // Addiw
                        self.regs[rd] = self.regs[rs1].wrapping_add(imm) as i32 as i64 as u64;
                    }
                    0x1 => {
                        // Slliw
                        self.regs[rd] = self.regs[rs1].wrapping_shl(shift_amount) as i32 as i64 as u64;
                    }
                    0x5 => {
                        match funct7 {
                            0x00 => {
                                // Srliw
                                self.regs[rd] = (self.regs[rs1] as u32).wrapping_shr(shift_amount) as i32 as i64 as u64;
                            }
                            0x20 => {
                                // Sraiw
                                self.regs[rd] = (self.regs[rs1] as i32).wrapping_shr(shift_amount) as i64 as u64;
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
            0x23 => {
                // Store instructions
                // imm[11:5 | 4:0]
                let imm = (((instruction & 0xfe00_0000) as i32 as i64 >> 20) as u64) | ((instruction >> 7) & 0x1f) as u64;
                let addr = self.regs[rs1].wrapping_add(imm);

                match funct3 {
                    0x0 => self.store(addr, 8, self.regs[rs2])?,    // SB
                    0x1 => self.store(addr, 16, self.regs[rs2])?,   // SH
                    0x2 => self.store(addr, 32, self.regs[rs2])?,   // SW
                    0x3 => self.store(addr, 64, self.regs[rs2])?,   // SD
                    _ => {}
                }
            }
            0x33 => {
                let shift_amount = ((self.regs[rs2] & 0x3f) as u64) as u32;

                match (funct3, funct7) {
                    (0x0, 0x00) => {
                        // add
                        self.regs[rd] = self.regs[rs1].wrapping_add(self.regs[rs2]);
                    }
                    (0x0, 0x01) => {
                        // mul
                        self.regs[rd] = self.regs[rs1].wrapping_mul(self.regs[rs2]);
                    }
                    (0x0, 0x20) => {
                        // sub
                        self.regs[rd] = self.regs[rs1].wrapping_sub(self.regs[rs2]);
                    }
                    (0x1, 0x00) => {
                        // sll
                        self.regs[rd] = self.regs[rs1].wrapping_shl(shift_amount);
                    }
                    (0x2, 0x00) => {
                        // slt
                        self.regs[rd] = if (self.regs[rs1] as i64) < (self.regs[rs2] as i64) {
                            1
                        } else {
                            0
                        };
                    }
                    (0x3, 0x00) => {
                        // sltu
                        self.regs[rd] = if self.regs[rs1] < self.regs[rs2] {
                            1
                        } else {
                            0
                        }
                    }
                    (0x4, 0x00) => {
                        // xor
                        self.regs[rd] = self.regs[rs1] ^ self.regs[rs2];
                    }
                    (0x5, 0x00) => {
                        // srl
                        self.regs[rd] = self.regs[rs1].wrapping_shr(shift_amount);
                    }
                    (0x5, 0x20) => {
                        // sra
                        self.regs[rd] = (self.regs[rs1] as i64).wrapping_shr(shift_amount) as u64;
                    }
                    (0x6, 0x00) => {
                        // or
                        self.regs[rd] = self.regs[rs1] | self.regs[rs2];
                    }
                    (0x7, 0x00) => {
                        // and
                        self.regs[rd] = self.regs[rs1] & self.regs[rs2];
                    }
                    _ => {}
                }
            }
            0x37 => {
                // lui
                self.regs[rd] = (instruction & 0xffff_f000) as i32 as i64 as u64;
            }
            0x3b => {
                let shift_amount = (self.regs[rs2] & 0x1f) as u32;
                match (funct3, funct7) {
                    (0x0, 0x00) => {
                        // addw
                        self.regs[rd] = self.regs[rs1].wrapping_add(self.regs[rs2]) as i32 as i64 as u64;
                    }
                    (0x0, 0x20) => {
                        // subw
                        self.regs[rd] = ((self.regs[rs1].wrapping_sub(self.regs[rs2])) as i32) as u64;
                    }
                    (0x1, 0x00) => {
                        // sllw
                        self.regs[rd] = (self.regs[rs1] as u32).wrapping_shl(shift_amount) as i32 as u64;
                    }
                    (0x5, 0x00) => {
                        // srlw
                        self.regs[rd] = (self.regs[rs1] as u32).wrapping_shr(shift_amount) as i32 as u64;
                    }
                    (0x5, 0x20) => {
                        // sraw
                        self.regs[rd] = ((self.regs[rs1] as i32) >> (shift_amount as i32)) as u64;
                    }
                    _ => {}
                }
            }
            0x63 => {
                let imm = (((instruction & 0x8000_0000) as i32 as i64 >> 19) as u64)
                    | ((instruction & 0x80) << 4) as u64
                    | ((instruction >> 20) & 0x7e0) as u64
                    | ((instruction >> 7) & 0x1e) as u64;

                match funct3 {
                    0x0 => {
                        // beq
                        if self.regs[rs1] == self.regs[rs2] {
                            self.pc = self.pc.wrapping_add(imm).wrapping_sub(4);
                        }
                    }
                    0x1 => {
                        // bne
                        if self.regs[rs1] != self.regs[rs2] {
                            self.pc = self.pc.wrapping_add(imm).wrapping_sub(4);
                        }
                    }
                    0x4 => {
                        // blt
                        if (self.regs[rs1] as i64) < (self.regs[rs2] as i64) {
                            self.pc = self.pc.wrapping_add(imm).wrapping_sub(4);
                        }
                    }
                    0x5 => {
                        // bge
                        if (self.regs[rs1] as i64) >= (self.regs[rs2] as i64) {
                            self.pc = self.pc.wrapping_add(imm).wrapping_sub(4);
                        }
                    }
                    0x6 => {
                        // bltu
                        if self.regs[rs1] < self.regs[rs2] {
                            self.pc = self.pc.wrapping_add(imm).wrapping_sub(4);
                        }
                    }
                    0x7 => {
                        // bgeu
                        if self.regs[rs1] >= self.regs[rs2] {
                            self.pc = self.pc.wrapping_add(imm).wrapping_sub(4);
                        }
                    }
                    _ => {}
                }
            }
            0x67 => {
                // jalr
                let t = self.pc;

                let imm = ((((instruction & 0xfff0_0000) as i32) as i64) >> 20) as u64;
                self.pc = (self.regs[rs1].wrapping_add(imm)) & !1;

                self.regs[rd] = t;
            }
            0x6f => {
                // jal
                self.regs[rd] = self.pc;

                let imm = (((instruction & 0x8000_0000) as i32 as i64 >> 11) as u64)
                    | (instruction & 0xff000) as u64
                    | ((instruction >> 9) & 0x800) as u64
                    | ((instruction >> 20) & 0x7fe) as u64;

                self.pc = self.pc.wrapping_add(imm).wrapping_sub(4);
            }
            _ => {
                dbg!(format!("Not implemented opcode: {:#x}", opcode));
            }
        }

        Ok(())
    }

    pub fn dump_registers(&self) {
        let mut output = String::from("");
        let abi = [
            "zero", " ra ", " sp ", " gp ", " tp ", " t0 ", " t1 ", " t2 ", " s0 ", " s1 ", " a0 ",
            " a1 ", " a2 ", " a3 ", " a4 ", " a5 ", " a6 ", " a7 ", " s2 ", " s3 ", " s4 ", " s5 ",
            " s6 ", " s7 ", " s8 ", " s9 ", " s10", " s11", " t3 ", " t4 ", " t5 ", " t6 ",
        ];
        for i in (0..32).step_by(4) {
            output = format!(
                "{}\n{}",
                output,
                format!(
                    "x{:02}({})={:>#18x} x{:02}({})={:>#18x} x{:02}({})={:>#18x} x{:02}({})={:>#18x}",
                    i,
                    abi[i],
                    self.regs[i],
                    i + 1,
                    abi[i + 1],
                    self.regs[i + 1],
                    i + 2,
                    abi[i + 2],
                    self.regs[i + 2],
                    i + 3,
                    abi[i + 3],
                    self.regs[i + 3],
                )
            );
        }
        println!("{}", output);
    }

}