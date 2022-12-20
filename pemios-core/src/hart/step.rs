// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// This Source Code Form is "Incompatible With Secondary Licenses", as
// defined by the Mozilla Public License, v. 2.0.
//
// Copyright © 2022 mumblingdrunkard

use std::ops::{BitAnd, BitOr, BitXor};

use crate::hart::{instruction::Instruction, Hart};

use super::instruction::Conclusion;

pub trait Step {
    fn step(&mut self) -> Conclusion;
}

impl Step for Hart<'_> {
    fn step(&mut self) -> Conclusion {
        use Instruction::*;

        let inst = match self.mmu.load_instruction(self.pc) {
            Ok(op) => op,
            Err(_) => todo!(),
        };

        let conclusion = match inst {
            Lui { rd, imm } => {
                self.reg[rd] = i32::from(imm) as u32;
                Conclusion::None
            }
            Auipc { rd, imm } => {
                self.reg[rd] = self.pc.wrapping_add_signed(imm.into());
                Conclusion::None
            }
            Jal { rd, imm } => {
                let target = self.pc.wrapping_add_signed(imm.into());
                if target & 3 != 0 {
                    todo!("Add misaligned jump exception");
                }
                self.reg[rd] = self.pc.wrapping_add(4);
                self.pc = target;
                Conclusion::Jumped
            }
            Jalr { rd, rs1, imm } => {
                let target = self.reg[rs1].wrapping_add_signed(imm.into()) & 0xfffffffe;
                if target & 3 != 0 {
                    todo!("Add misaligned jump exception");
                }
                self.reg[rd] = self.pc.wrapping_add(4);
                self.pc = target;
                Conclusion::Jumped
            }
            Beq { rs1, rs2, imm } => {
                if self.reg[rs1] != self.reg[rs2] {
                    Conclusion::None
                } else {
                    let target = self.pc.wrapping_add_signed(imm.into());
                    if target & 3 != 0 {
                        todo!("Add misaligned jump exception");
                    }

                    self.pc = target;
                    Conclusion::Jumped
                }
            }
            Bne { rs1, rs2, imm } => {
                if self.reg[rs1] == self.reg[rs2] {
                    Conclusion::None
                } else {
                    let target = self.pc.wrapping_add_signed(imm.into());
                    if target & 3 != 0 {
                        todo!("Add misaligned jump exception");
                    }

                    self.pc = target;
                    Conclusion::Jumped
                }
            }
            Blt { rs1, rs2, imm } => {
                if (self.reg[rs1] as i32) >= (self.reg[rs2] as i32) {
                    Conclusion::None
                } else {
                    let target = self.pc.wrapping_add_signed(imm.into());
                    if target & 3 != 0 {
                        todo!("Add misaligned jump exception");
                    }

                    self.pc = target;
                    Conclusion::Jumped
                }
            }
            Bge { rs1, rs2, imm } => {
                if (self.reg[rs1] as i32) < (self.reg[rs2] as i32) {
                    Conclusion::None
                } else {
                    let target = self.pc.wrapping_add_signed(imm.into());
                    if target & 3 != 0 {
                        todo!("Add misaligned jump exception");
                    }

                    self.pc = target;
                    Conclusion::Jumped
                }
            }
            Bltu { rs1, rs2, imm } => {
                if self.reg[rs1] >= self.reg[rs2] {
                    Conclusion::None
                } else {
                    let target = self.pc.wrapping_add_signed(imm.into());
                    if target & 3 != 0 {
                        todo!("Add misaligned jump exception");
                    }

                    self.pc = target;
                    Conclusion::Jumped
                }
            }
            Bgeu { rs1, rs2, imm } => {
                if self.reg[rs1] < self.reg[rs2] {
                    Conclusion::None
                } else {
                    let target = self.pc.wrapping_add_signed(imm.into());
                    if target & 3 != 0 {
                        todo!("Add misaligned jump exception");
                    }

                    self.pc = target;
                    Conclusion::Jumped
                }
            }

            Lb { rd, rs1, imm } => todo!("{rd:?}, {rs1:?}, {imm:?}"),
            Lh { rd, rs1, imm } => todo!("{rd:?}, {rs1:?}, {imm:?}"),
            Lw { rd, rs1, imm } => {
                let addr = self.reg[rs1].wrapping_add_signed(imm.into());
                match self.mmu.load_word(addr) {
                    Ok(val) => {
                        self.reg[rd] = val;
                        Conclusion::None
                    }
                    Err(e) => todo!("{:?}", e),
                }
            }
            Lbu { rd, rs1, imm } => todo!(),
            Lhu { rd, rs1, imm } => todo!(),

            Sb { rs1, rs2, imm } => {
                let addr = self.reg[rs1].wrapping_add_signed(imm.into());
                match self.mmu.store_byte(addr, self.reg[rs2] as u8) {
                    Ok(_) => Conclusion::None,
                    Err(e) => todo!("{:?}", e),
                }
            }
            Sh { rs1, rs2, imm } => {
                let addr = self.reg[rs1].wrapping_add_signed(imm.into());
                match self.mmu.store_half_word(addr, self.reg[rs2] as u16) {
                    Ok(_) => Conclusion::None,
                    Err(e) => todo!("{:?}", e),
                }
            }
            Sw { rs1, rs2, imm } => {
                let addr = self.reg[rs1].wrapping_add_signed(imm.into());
                match self.mmu.store_word(addr, self.reg[rs2]) {
                    Ok(_) => Conclusion::None,
                    Err(e) => todo!("{:?}", e),
                }
            }

            Addi { rd, rs1, imm } => {
                self.reg[rd] = self.reg[rs1].wrapping_add_signed(imm.into());
                Conclusion::None
            }
            Slti { rd, rs1, imm } => {
                self.reg[rd] = ((self.reg[rs1] as i32) < imm.into()) as u32;
                Conclusion::None
            }
            Sltiu { rd, rs1, imm } => {
                self.reg[rd] = (self.reg[rs1] < i32::from(imm) as u32) as u32;
                Conclusion::None
            }
            Xori { rd, rs1, imm } => {
                self.reg[rd] = self.reg[rs1].bitxor(i32::from(imm) as u32);
                Conclusion::None
            }
            Ori { rd, rs1, imm } => {
                self.reg[rd] = self.reg[rs1].bitor(i32::from(imm) as u32);
                Conclusion::None
            }
            Andi { rd, rs1, imm } => {
                self.reg[rd] = self.reg[rs1].bitand(i32::from(imm) as u32);
                Conclusion::None
            }
            Slli { rd, rs1, shamt } => {
                self.reg[rd] = self.reg[rs1] << u32::from(shamt);
                Conclusion::None
            }
            Srli { rd, rs1, shamt } => {
                self.reg[rd] = self.reg[rs1] >> u32::from(shamt);
                Conclusion::None
            }
            Srai { rd, rs1, shamt } => {
                self.reg[rd] = (self.reg[rs1] as i32 >> u32::from(shamt)) as u32;
                Conclusion::None
            }

            Add { rd, rs1, rs2 } => {
                self.reg[rd] = self.reg[rs1].wrapping_add(self.reg[rs2]);
                Conclusion::None
            }
            Sub { rd, rs1, rs2 } => {
                self.reg[rd] = self.reg[rs1].wrapping_sub(self.reg[rs2]);
                Conclusion::None
            }
            Sll { rd, rs1, rs2 } => {
                self.reg[rd] = self.reg[rs1].wrapping_shl(self.reg[rs2]);
                Conclusion::None
            }
            Slt { rd, rs1, rs2 } => {
                self.reg[rd] = ((self.reg[rs1] as i32) < (self.reg[rs2] as i32)) as u32;
                Conclusion::None
            }
            Sltu { rd, rs1, rs2 } => {
                self.reg[rd] = (self.reg[rs1] < self.reg[rs2]) as u32;
                Conclusion::None
            }
            Xor { rd, rs1, rs2 } => {
                self.reg[rd] = self.reg[rs1].bitxor(self.reg[rs2]);
                Conclusion::None
            }
            Srl { rd, rs1, rs2 } => {
                self.reg[rd] = self.reg[rs1].wrapping_shr(self.reg[rs2]);
                Conclusion::None
            }
            Sra { rd, rs1, rs2 } => {
                self.reg[rd] = (self.reg[rs1] as i32).wrapping_shr(self.reg[rs2]) as u32;
                Conclusion::None
            }
            Or { rd, rs1, rs2 } => {
                self.reg[rd] = self.reg[rs1].bitor(self.reg[rs2]);
                Conclusion::None
            }
            And { rd, rs1, rs2 } => {
                self.reg[rd] = self.reg[rs1].bitand(self.reg[rs2]);
                Conclusion::None
            }

            #[rustfmt::skip]
            Fence { rd, rs1, pred, succ, mode } => todo!(),
            Ecall => {
                println!("Executed ebreak which is unimplemented!");
                Conclusion::Exception(2)
            }
            Ebreak => todo!("Implement ebreak"),
            Fencei { rd, rs1, imm } => todo!("Implement fencei"),
            CsrRw { rd, rs1, csr } => todo!(),
            CsrRs { rd, rs1, csr } => todo!(),
            CsrRc { rd, rs1, csr } => todo!(),
            CsrRwi { rd, uimm, csr } => todo!(),
            CsrRsi { rd, uimm, csr } => todo!(),
            CsrRci { rd, uimm, csr } => todo!(),
            Mul { rd, rs1, rs2 } => todo!(),
            Mulh { rd, rs1, rs2 } => todo!(),
            Mulhsu { rd, rs1, rs2 } => todo!(),
            Mulhu { rd, rs1, rs2 } => todo!(),
            Div { rd, rs1, rs2 } => todo!(),
            Divu { rd, rs1, rs2 } => todo!(),
            Rem { rd, rs1, rs2 } => todo!(),
            Remu { rd, rs1, rs2 } => todo!(),
            Lrw { rd, rs1, aq, rl } => todo!(),
            #[rustfmt::skip]
            Scw { rd, rs1, rs2, aq, rl, } => todo!(),
            #[rustfmt::skip]
            AmoSwapw { rd, rs1, rs2, aq, rl, } => todo!(),
            #[rustfmt::skip]
            AmoAddw { rd, rs1, rs2, aq, rl, } => todo!(),
            #[rustfmt::skip]
            AmoXorw { rd, rs1, rs2, aq, rl, } => todo!(),
            #[rustfmt::skip]
            AmoAndw { rd, rs1, rs2, aq, rl, } => todo!(),
            #[rustfmt::skip]
            AmoOrw { rd, rs1, rs2, aq, rl, } => todo!(),
            #[rustfmt::skip]
            AmoMinw { rd, rs1, rs2, aq, rl, } => todo!(),
            #[rustfmt::skip]
            AmoMaxw { rd, rs1, rs2, aq, rl, } => todo!(),
            #[rustfmt::skip]
            AmoMinuw { rd, rs1, rs2, aq, rl, } => todo!(),
            #[rustfmt::skip]
            AmoMaxuw { rd, rs1, rs2, aq, rl, } => todo!(),
            Invalid { raw } => todo!("Invalid: {raw:b}"),
        };

        if let Conclusion::None = conclusion {
            self.pc = self.pc.wrapping_add(4);
        }

        conclusion
    }
}
