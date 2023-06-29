use bus::*;

pub struct Cpu {
    pub regs: [u64; 32],
    pub pc: u64,
    pub bus: Bus,
    dram: Vec<u8>,
}

impl Cpu {
    pub fn new(code: Vec<u8>) -> Self {
        let mut regs = [0; 32];
        regs[2] = DRAM_SIZE;

        Self {
            regs,
            pc: 0,
            dram: code
        }
    }

    pub fn fetch(&self) -> u32 {
        let index = self.pc as usize;
        return (self.dram[index] as u32)
            | ((self.dram[index + 1] as u32) << 8)
            | ((self.dram[index + 2] as u32) << 16)
            | ((self.dram[index + 3] as u32) << 24);
    }

    pub fn fetch(&mut self) -> Result<u64, ()> {
        match self.bus.load(self.pc, 32) {
            Ok(inst) => Ok(inst),
            Err(_) => Err(()),
        }
    }

    pub fn execute(&mut self, instruction: u32) {
        let opcode = instruction & 0x7f;
        let rd = ((instruction >> 7) & 0x1f) as usize;
        let rs1 = ((instruction >> 15) & 0x1f) as usize;
        let rs2 = ((instruction >> 20) & 0x1f) as usize;
        let funct3 = (instruction >> 12) & 0x7;
        let funct7 = (instruction >> 25) & 0x7f

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
            0x23 => {
                // Store instructions
                // imm[11:5 | 4:0]
                let imm = (((instruction & 0xfe00_0000) as i32 as i64 >> 20) as u64) | ((instruction >> 7) & 0x1f);
                let addr = self.regs[rs1].wrapping_add(imm);

                match funct3 {
                    0x0 => self.store(addr, 8, self.regs[rs2])?,    // SB
                    0x1 => self.store(addr, 16, self.regs[rs2])?,   // SH
                    0x2 => self.store(addr, 32, self.regs[rs2])?,   // SW
                    0x3 => self.store(addr, 64, self.regs[rs2])?,   // SD
                    _ => {}
                }
            }
            0x13 => {
                // addi
                let imm = ((instruction >> 20) & 0xfff) as u64;
                self.regs[rd] = self.regs[rs1].wrapping_add(imm);
            }
            0x33 => {
                // add
                self.regs[rd] = self.regs[rs1].wrapping_add(self.regs[rs2]);
            }
            _ => {
                dbg!(format!("Not implemented opcode: {:#x}", opcode));
            }
        }
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