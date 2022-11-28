// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// This Source Code Form is "Incompatible With Secondary Licenses", as
// defined by the Mozilla Public License, v. 2.0.
//
// Copyright © 2022 mumblingdrunkard

use crate::hart::Reg;

use super::{CommonFields, Instruction, InstructionKind, OpCode, SystemFields};

#[cfg(feature = "rv32a")]
use super::AmoFields;

pub trait Decode {
    fn decode(&self) -> InstructionKind;
}

impl Decode for Instruction {
    fn decode(&self) -> InstructionKind {
        match self.opcode() {
            // == Rv32i ==
            OpCode::Lui => InstructionKind::Lui,
            OpCode::Auipc => InstructionKind::Auipc,
            OpCode::Jal => InstructionKind::Jal,
            OpCode::Jalr => InstructionKind::Jalr,

            OpCode::Branch => match self.funct3() {
                0b000 => InstructionKind::Beq,
                0b001 => InstructionKind::Bne,
                0b100 => InstructionKind::Blt,
                0b101 => InstructionKind::Bge,
                0b110 => InstructionKind::Bltu,
                0b111 => InstructionKind::Bgeu,
                _ => InstructionKind::Invalid,
            },

            OpCode::Load => match self.funct3() {
                0b000 => InstructionKind::Lb,
                0b001 => InstructionKind::Lh,
                0b010 => InstructionKind::Lw,
                0b100 => InstructionKind::Lbu,
                0b101 => InstructionKind::Lhu,
                _ => InstructionKind::Invalid,
            },

            OpCode::Store => match self.funct3() {
                0b000 => InstructionKind::Sb,
                0b001 => InstructionKind::Sh,
                0b010 => InstructionKind::Sw,
                _ => InstructionKind::Invalid,
            },

            OpCode::OpImm => match (self.funct3(), self.funct7()) {
                (0b000, _) => InstructionKind::Addi,
                (0b010, _) => InstructionKind::Slti,
                (0b011, _) => InstructionKind::Sltiu,
                (0b100, _) => InstructionKind::Xori,
                (0b110, _) => InstructionKind::Ori,
                (0b111, _) => InstructionKind::Andi,
                (0b001, 0b0000000) => InstructionKind::Slli,
                (0b101, 0b0000000) => InstructionKind::Srli,
                (0b101, 0b0100000) => InstructionKind::Srai,
                _ => InstructionKind::Invalid,
            },

            OpCode::Op if self.funct7() == 0 || self.funct7() == 0b0100000 => {
                match (self.funct3(), self.funct7()) {
                    (0b000, 0b0000000) => InstructionKind::Add,
                    (0b000, 0b0100000) => InstructionKind::Sub,
                    (0b001, 0b0000000) => InstructionKind::Sll,
                    (0b010, 0b0000000) => InstructionKind::Slt,
                    (0b011, 0b0000000) => InstructionKind::Sltu,
                    (0b100, 0b0000000) => InstructionKind::Xor,
                    (0b101, 0b0000000) => InstructionKind::Srl,
                    (0b101, 0b0100000) => InstructionKind::Sra,
                    (0b110, 0b0000000) => InstructionKind::Or,
                    (0b111, 0b0000000) => InstructionKind::And,
                    _ => InstructionKind::Invalid,
                }
            }

            OpCode::MiscMem if self.funct3() == 0b000 => InstructionKind::Fence,

            OpCode::System if (self.rd(), self.funct3(), self.rs1()) == (Reg::X0, 0, Reg::X0) => {
                match self.funct12() {
                    0 => InstructionKind::Ecall,
                    1 => InstructionKind::Ebreak,
                    _ => InstructionKind::Invalid,
                }
            }

            // == Zifencei ==
            #[cfg(feature = "zifencei")]
            OpCode::MiscMem if self.funct3() == 0b001 => InstructionKind::Fencei,

            // == Zicsr ==
            #[cfg(feature = "zicsr")]
            OpCode::System => match self.funct3() {
                0b001 => InstructionKind::CsrRw,
                0b010 => InstructionKind::CsrRs,
                0b011 => InstructionKind::CsrRc,
                0b101 => InstructionKind::CsrRwi,
                0b110 => InstructionKind::CsrRsi,
                0b111 => InstructionKind::CsrRci,
                _ => InstructionKind::Invalid,
            },

            // == Rv32m ==
            #[cfg(feature = "rv32m")]
            OpCode::Op if self.funct7() == 1 => match self.funct3() {
                0b000 => InstructionKind::Mul,
                0b001 => InstructionKind::Mulh,
                0b010 => InstructionKind::Mulhsu,
                0b011 => InstructionKind::Mulhu,
                0b100 => InstructionKind::Div,
                0b101 => InstructionKind::Divu,
                0b110 => InstructionKind::Rem,
                0b111 => InstructionKind::Remu,
                _ => unsafe { std::hint::unreachable_unchecked() },
            },

            // == Rv32a ==
            #[cfg(feature = "rv32a")]
            OpCode::Amo if self.funct3() == 0b010 => match self.funct5() {
                0b00010 if self.rs2() == Reg::X0 => InstructionKind::Lrw,
                0b00011 => InstructionKind::Scw,
                0b00001 => InstructionKind::AmoSwapw,
                0b00000 => InstructionKind::AmoAddw,
                0b00100 => InstructionKind::AmoXorw,
                0b01100 => InstructionKind::AmoAndw,
                0b01000 => InstructionKind::AmoOrw,
                0b10000 => InstructionKind::AmoMinw,
                0b10100 => InstructionKind::AmoMaxw,
                0b11000 => InstructionKind::AmoMinuw,
                0b11100 => InstructionKind::AmoMaxuw,
                _ => InstructionKind::Invalid,
            },

            _ => InstructionKind::Invalid,
        }
    }
}
