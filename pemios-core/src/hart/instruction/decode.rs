// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// This Source Code Form is "Incompatible With Secondary Licenses", as
// defined by the Mozilla Public License, v. 2.0.
//
// Copyright © 2022 mumblingdrunkard

use crate::hart::{csr::Csr, instruction::Instruction, Reg};

use super::types::{
    FenceMode, FenceSet, Int12, Int13Trunc1, Int21Trunc1, Int32Trunc12, OpCode, UInt5,
};

/// An adapter for u32 that lets us extract fields from a RISC-V instruction
pub struct Decoder<'a>(&'a u32);

impl<'a> Decoder<'a> {
    fn new(raw: &'a u32) -> Self {
        Self(raw)
    }
}

impl Decoder<'_> {
    fn opcode(&self) -> OpCode {
        (self.0 & 0x7f).into()
    }

    fn rd(&self) -> Reg {
        match ((self.0 >> 7) & 0x1f).into() {
            Reg::X0 => Reg::Ignore,
            rd => rd,
        }
    }

    fn funct3(&self) -> u32 {
        (self.0 >> 12) & 7
    }

    fn funct5(&self) -> u32 {
        (self.0 >> 27) & 0x1f
    }

    fn funct7(&self) -> u32 {
        (self.0 >> 25) & 0x7f
    }

    fn funct12(&self) -> u32 {
        (self.0 >> 20) & 0xfff
    }

    fn rs1(&self) -> Reg {
        ((self.0 >> 15) & 0x1f).into()
    }

    fn rs2(&self) -> Reg {
        ((self.0 >> 20) & 0x1f).into()
    }

    fn shamt(&self) -> UInt5 {
        ((self.0 >> 15) & 0x1f).into()
    }

    fn imm_i(&self) -> Int12 {
        (*self.0 as i32 >> 20).into()
    }

    fn imm_s(&self) -> Int12 {
        let imm11_5at25 = self.0 & 0xfe000000;
        let imm4_0at7 = self.0 & 0xf80;

        ((imm11_5at25 as i32 >> 20) | (imm4_0at7 >> 7) as i32).into()
    }

    fn imm_j(&self) -> Int21Trunc1 {
        let imm20at31 = self.0 & 0x80000000;
        let imm10_1at21 = self.0 & 0x7fe00000;
        let imm11at20 = self.0 & 0x00100000;
        let imm19_12at12 = self.0 & 0x000ff000;
        ((imm20at31 as i32 >> 11)
            | imm19_12at12 as i32
            | (imm11at20 >> 9) as i32
            | (imm10_1at21 >> 20) as i32)
            .into()
    }

    fn imm_b(&self) -> Int13Trunc1 {
        let imm12at31 = self.0 & 0x80000000;
        let imm10_5at25 = self.0 & 0x7e000000;
        let imm4_1at8 = self.0 & 0xf00;
        let imm11at7 = self.0 & 0x80;

        ((imm12at31 as i32 >> 19)
            | ((imm11at7 as i32) << 4)
            | (imm10_5at25 as i32 >> 20)
            | (imm4_1at8 as i32 >> 7))
            .into()
    }

    fn imm_u(&self) -> Int32Trunc12 {
        ((self.0 & 0xfffff000) as i32).into()
    }

    // Csr fields
    fn csr(&self) -> Csr {
        (self.0 >> 20).into()
    }

    fn uimm(&self) -> UInt5 {
        ((self.0 >> 15) & 0x1f).into()
    }

    // Atomics fields
    fn aq(&self) -> bool {
        self.0 & (1 << 26) != 0
    }

    fn rl(&self) -> bool {
        self.0 & (1 << 25) != 0
    }

    // Fence fields
    fn pred(&self) -> FenceSet {
        FenceSet::new((self.0 >> 24) as u8 & 15)
    }

    fn succ(&self) -> FenceSet {
        FenceSet::new((self.0 >> 20) as u8 & 15)
    }

    fn mode(&self) -> FenceMode {
        todo!()
    }
}

pub trait Decode {
    fn decode(&self) -> Instruction;
}

impl Decode for u32 {
    fn decode(&self) -> Instruction {
        use Instruction::*;
        let decoder = Decoder::new(self);

        let raw = *self;
        let rd = decoder.rd();
        let rs1 = decoder.rs1();
        let rs2 = decoder.rs2();
        let funct3 = decoder.funct3();
        let funct7 = decoder.funct7();

        match decoder.opcode() {
            OpCode::Load => {
                let imm = decoder.imm_i();
                match funct3 {
                    0 => Lb { rd, rs1, imm },
                    1 => Lh { rd, rs1, imm },
                    2 => Lw { rd, rs1, imm },
                    4 => Lbu { rd, rs1, imm },
                    5 => Lhu { rd, rs1, imm },
                    _ => Invalid { raw },
                }
            }

            OpCode::MiscMem => match funct3 {
                0 => Fence {
                    rd,
                    rs1,
                    mode: decoder.mode(),
                    pred: decoder.pred(),
                    succ: decoder.succ(),
                },
                1 => Fencei {
                    rd: Reg::Ignore,
                    rs1: Reg::Ignore,
                    imm: 0.into(),
                },
                _ => Invalid { raw },
            },

            OpCode::OpImm => {
                let imm = decoder.imm_i();
                let shamt = decoder.shamt();
                match funct3 {
                    0b000 => Addi { rd, rs1, imm },
                    0b010 => Slti { rd, rs1, imm },
                    0b011 => Sltiu { rd, rs1, imm },
                    0b100 => Xori { rd, rs1, imm },
                    0b110 => Ori { rd, rs1, imm },
                    0b111 => Andi { rd, rs1, imm },
                    0b001 => Slli { rd, rs1, shamt },
                    0b101 if funct7 == 0 => Srli { rd, rs1, shamt },
                    0b101 if funct7 == 0x20 => Srai { rd, rs1, shamt },
                    _ => Invalid { raw },
                }
            }

            OpCode::Auipc => Auipc {
                rd,
                imm: decoder.imm_u(),
            },

            OpCode::Store => {
                let imm = decoder.imm_s();
                match funct3 {
                    0 => Sb { rs1, rs2, imm },
                    1 => Sh { rs1, rs2, imm },
                    2 => Sw { rs1, rs2, imm },
                    _ => Invalid { raw },
                }
            }

            #[rustfmt::skip]
            OpCode::Amo => {
                let aq = decoder.aq();
                let rl = decoder.rl();
                match decoder.funct5() {
                    0b00010 => Lrw { rd, rs1, aq, rl },
                    0b00011 => Scw { rd, rs1, rs2, aq, rl, },
                    0b00001 => AmoSwapw { rd, rs1, rs2, aq, rl, },
                    0b00000 => AmoAddw { rd, rs1, rs2, aq, rl, },
                    0b00100 => AmoXorw { rd, rs1, rs2, aq, rl, },
                    0b01100 => AmoAndw { rd, rs1, rs2, aq, rl, },
                    0b01000 => AmoOrw { rd, rs1, rs2, aq, rl, },
                    0b10000 => AmoMinw { rd, rs1, rs2, aq, rl, },
                    0b10100 => AmoMaxw { rd, rs1, rs2, aq, rl, },
                    0b11000 => AmoMinuw { rd, rs1, rs2, aq, rl, },
                    0b11100 => AmoMaxuw { rd, rs1, rs2, aq, rl, },
                    _ => Invalid { raw },
                }
            }

            OpCode::Op => match funct3 {
                0 if funct7 == 0 => Add { rd, rs1, rs2 },
                0 if funct7 == 0x20 => Sub { rd, rs1, rs2 },
                1 => Sll { rd, rs1, rs2 },
                2 => Slt { rd, rs1, rs2 },
                3 => Sltu { rd, rs1, rs2 },
                4 => Xor { rd, rs1, rs2 },
                5 if funct7 == 0 => Srl { rd, rs1, rs2 },
                5 if funct7 == 0x20 => Srl { rd, rs1, rs2 },
                6 => Or { rd, rs1, rs2 },
                7 => And { rd, rs1, rs2 },
                _ => Invalid { raw },
            },

            OpCode::Lui => Lui {
                rd,
                imm: decoder.imm_u(),
            },

            OpCode::Branch => {
                let imm = decoder.imm_b();
                match funct3 {
                    0 => Bne { rs1, rs2, imm },
                    1 => Beq { rs1, rs2, imm },
                    4 => Blt { rs1, rs2, imm },
                    5 => Bge { rs1, rs2, imm },
                    6 => Bltu { rs1, rs2, imm },
                    7 => Bgeu { rs1, rs2, imm },
                    _ => Invalid { raw },
                }
            }

            OpCode::Jalr => Jalr {
                rd,
                rs1,
                imm: decoder.imm_i(),
            },

            OpCode::Jal => Jal {
                rd,
                imm: decoder.imm_j(),
            },

            OpCode::System if funct3 == 0 => match decoder.funct12() {
                0 => Ecall,
                1 => Ebreak,
                _ => Invalid { raw },
            },

            OpCode::System if funct3 != 4 => {
                let csr = decoder.csr();
                let uimm = decoder.uimm();
                match funct3 {
                    1 => CsrRw { rd, rs1, csr },
                    2 => CsrRs { rd, rs1, csr },
                    3 => CsrRc { rd, rs1, csr },
                    5 => CsrRwi { rd, uimm, csr },
                    6 => CsrRsi { rd, uimm, csr },
                    7 => CsrRci { rd, uimm, csr },
                    _ => unreachable!(),
                }
            }

            _ => Invalid { raw },
        }
    }
}

impl From<u32> for Instruction {
    fn from(value: u32) -> Self {
        value.decode()
    }
}
