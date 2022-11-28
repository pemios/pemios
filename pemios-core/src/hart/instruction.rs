// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// This Source Code Form is "Incompatible With Secondary Licenses", as
// defined by the Mozilla Public License, v. 2.0.
//
// Copyright © 2022 mumblingdrunkard

mod decode;
mod execute;
mod instruction_type;

use std::ops::RangeInclusive;

use super::{
    csr::Csr,
    register::Reg,
    utils::{Bit, BitRange, Bits, SignExtend},
};

pub use instruction_type::InstructionKind;

pub use execute::*;

pub enum ExceptionKind {
    Ecall,
}

#[allow(unused)]
pub enum Conclusion {
    None,
    Jumped,
    Todo,
    Exception(ExceptionKind),
}

#[allow(unused)]
#[derive(Copy, Clone, Debug)]
#[repr(u8)]
pub enum OpCode {
    Load,
    MiscMem,
    OpImm,
    Auipc,
    Store,
    Amo,
    Op,
    Lui,
    Branch,
    Jalr,
    Jal,
    System,
    Invalid,
}

impl From<u32> for OpCode {
    fn from(op: u32) -> Self {
        use OpCode::*;
        match op {
            0b0000011 => Load,
            0b0001111 => MiscMem,
            0b0010011 => OpImm,
            0b0010111 => Auipc,
            0b0100011 => Store,
            0b0101111 => Amo,
            0b0110011 => Op,
            0b0110111 => Lui,
            0b1100011 => Branch,
            0b1100111 => Jalr,
            0b1101111 => Jal,
            0b1110011 => System,
            _ => Invalid,
        }
    }
}

#[allow(unused)]
pub enum Format {
    RType,
    IType,
    SType,
    BType,
    UType,
    JType,
    Invalid,
}

#[allow(unused)]
impl OpCode {
    pub fn format(&self) -> Format {
        use Format::*;
        use OpCode::*;
        match *self {
            Load | MiscMem | OpImm | Jalr | System => IType,
            Lui | Auipc => UType,
            Jal => JType,
            Store => SType,
            Amo | Op => RType,
            Branch => BType,
            _ => Format::Invalid,
        }
    }
}

impl Instruction {
    #[allow(unused)]
    pub fn format(&self) -> Format {
        self.opcode().format()
    }
}

#[derive(Copy, Clone)]
pub struct Instruction(u32);

impl From<u32> for Instruction {
    fn from(i: u32) -> Self {
        Self(i)
    }
}

impl From<u32> for Operation {
    fn from(i: u32) -> Self {
        Instruction::from(i).into()
    }
}

#[allow(unused)]
#[repr(u8)]
pub enum FenceMode {
    None = 0b0000,
    Tso = 0b1000,
    Other,
}

impl From<u32> for FenceMode {
    fn from(v: u32) -> Self {
        match v {
            0b0000 => Self::None,
            0b1000 => Self::Tso,
            _ => Self::Other,
        }
    }
}

#[derive(Copy, Clone)]
pub struct FenceSet(u8);

#[allow(unused)]
impl FenceSet {
    pub fn i(&self) -> bool {
        self.0 & 0b1000 != 0
    }

    pub fn o(&self) -> bool {
        self.0 & 0b0100 != 0
    }

    pub fn r(&self) -> bool {
        self.0 & 0b0010 != 0
    }

    pub fn w(&self) -> bool {
        self.0 & 0b0001 != 0
    }
}

pub trait CommonFields {
    fn opcode(&self) -> OpCode;
    fn rd(&self) -> Reg;
    fn rs1(&self) -> Reg;
    fn rs2(&self) -> Reg;
    fn funct3(&self) -> u8;
    fn funct7(&self) -> u8;
}

pub trait ImmFields {
    fn imm_i(&self) -> u32;
    fn imm_s(&self) -> u32;
    fn imm_b(&self) -> u32;
    fn imm_u(&self) -> u32;
    fn imm_j(&self) -> u32;
}

#[cfg(feature = "rv32a")]
pub trait AmoFields {
    fn aq(&self) -> bool;
    fn rl(&self) -> bool;
    fn funct5(&self) -> u8;
}

pub trait FenceFields {
    fn fm(&self) -> FenceMode;
    fn pred(&self) -> FenceSet;
    fn succ(&self) -> FenceSet;
}

pub trait SystemFields {
    #[cfg(feature = "zicsr")]
    fn uimm(&self) -> u8;
    fn funct12(&self) -> u16;
}

impl Bits for Instruction {
    #[inline(always)]
    fn bits(&self, r: RangeInclusive<usize>) -> BitRange {
        BitRange::new(
            (self.0 >> r.start()) & (u32::MAX >> (32 - (r.end() - r.start() + 1))),
            r.end() - r.start() + 1,
        )
    }
}

impl Bit for Instruction {
    #[inline(always)]
    fn bit(&self, i: usize) -> BitRange {
        BitRange::new((self.0 >> i) & 1, 1)
    }
}

#[allow(unused)]
impl Instruction {
    #[inline(always)]
    pub fn raw(&self) -> u32 {
        self.0
    }
}

impl FenceFields for Instruction {
    #[inline(always)]
    fn fm(&self) -> FenceMode {
        self.bits(28..=31).get().into()
    }

    #[inline(always)]
    fn pred(&self) -> FenceSet {
        FenceSet(self.bits(24..=27).get() as u8)
    }

    #[inline(always)]
    fn succ(&self) -> FenceSet {
        FenceSet(self.bits(20..=23).get() as u8)
    }
}

#[cfg(feature = "rv32a")]
impl AmoFields for Instruction {
    #[inline(always)]
    fn aq(&self) -> bool {
        self.bit(26).get() == 1
    }

    #[inline(always)]
    fn rl(&self) -> bool {
        self.bit(25).get() == 1
    }

    #[inline(always)]
    fn funct5(&self) -> u8 {
        self.bits(27..=31).get() as u8
    }
}

impl CommonFields for Instruction {
    /// Interprets the bit range 0..=6 as an OpCode.
    /// See OpCode::from() for behaviour.
    #[inline(always)]
    fn opcode(&self) -> OpCode {
        self.bits(0..=6).get().into()
    }

    /// Gets the bit range 12..=14 as a u8.
    /// The top 5 bits are always 0.
    #[inline(always)]
    fn funct3(&self) -> u8 {
        self.bits(12..=14).get() as u8
    }

    /// Gets the bit range 25..=31 as a u8.
    /// The top bit is always 0.
    #[inline(always)]
    fn funct7(&self) -> u8 {
        self.bits(25..=31).get() as u8
    }

    /// Interprets the bit range 7..=11 as a Reg.
    /// Returns Reg::Discard if the actual target was 0.
    #[inline(always)]
    fn rd(&self) -> Reg {
        self.bits(7..=11).get().into()
    }

    /// Interprets the bit range 15..=19 as a Reg.
    #[inline(always)]
    fn rs1(&self) -> Reg {
        self.bits(15..=19).get().into()
    }

    /// Interprets the bit range 20..=24 as a Reg.
    #[inline(always)]
    fn rs2(&self) -> Reg {
        self.bits(20..=24).get().into()
    }
}

impl ImmFields for Instruction {
    /// Extract an i-format immediate from this instruction.
    #[inline(always)]
    fn imm_i(&self) -> u32 {
        self.bits(20..=31).get().sign_extend(11)
    }

    /// Extract an s-format immediate from this instruction.
    #[inline(always)]
    fn imm_s(&self) -> u32 {
        self.bits(25..=31)
            .cat(self.bits(7..=11))
            .get()
            .sign_extend(11)
    }

    /// Extract a b-format immediate from this instruction.
    #[inline(always)]
    fn imm_b(&self) -> u32 {
        self.bit(31)
            .cat(self.bit(7))
            .cat(self.bits(25..=30))
            .cat(self.bits(8..=11))
            .cat(BitRange::new(0, 1))
            .get()
            .sign_extend(12)
    }

    /// Extract a u-format immediate from this instruction.
    #[inline(always)]
    fn imm_u(&self) -> u32 {
        self.bits(12..=31).cat(BitRange::new(0, 12)).get()
    }

    /// Extract a j-format immediate from this instruction.
    #[inline(always)]
    fn imm_j(&self) -> u32 {
        self.bit(31)
            .cat(self.bits(12..=19))
            .cat(self.bit(20))
            .cat(self.bits(21..=30))
            .cat(BitRange::new(0, 1))
            .get()
            .sign_extend(20)
    }
}

impl SystemFields for Instruction {
    /// Gets the bit range 20..=31 as a u16.
    /// The top 4 bits are always 0.
    fn funct12(&self) -> u16 {
        self.bits(20..=31).get() as u16
    }

    /// Gets the bit range 15..=19 as a u8.
    /// The top 3 bits are always 0.
    #[cfg(feature = "zicsr")]
    fn uimm(&self) -> u8 {
        self.bits(15..=19).get() as u8
    }
}

impl From<Instruction> for Operation {
    fn from(inst: Instruction) -> Self {
        use decode::Decode;
        use OpCode::*;

        let value = match inst.decode() {
            InstructionKind::Invalid => inst.raw(),

            _ => match inst.opcode() {
                System => match inst.decode() {
                    InstructionKind::CsrRw
                    | InstructionKind::CsrRwi
                    | InstructionKind::CsrRs
                    | InstructionKind::CsrRsi
                    | InstructionKind::CsrRc
                    | InstructionKind::CsrRci => match Csr::from(inst.imm_i()) {
                        Csr::Invalid => inst.imm_i(),
                        csr => csr as u32,
                    },
                    _ => inst.imm_i(),
                },
                Load | OpImm | Jalr | MiscMem => inst.imm_i(),
                Jal => inst.imm_j(),
                Lui | Auipc => inst.imm_u(),
                Store => inst.imm_s(),
                Branch => inst.imm_b(),
                Amo => inst.bits(25..=26).get(),
                _ => inst.raw(),
            },
        };

        // Set reg to 0 if unused
        let (rd, rs1, rs2) = match inst.opcode() {
            Lui | Auipc | Jal => (inst.rd(), Reg::X0, Reg::X0),
            Jalr | OpImm | Load | MiscMem => (inst.rd(), inst.rs1(), Reg::X0),
            Branch | Store => (Reg::X0, inst.rs1(), inst.rs2()),
            Amo | Op => (inst.rd(), inst.rs1(), inst.rs2()),
            System | Invalid => (Reg::X0, Reg::X0, Reg::X0),
        };

        Self {
            kind: inst.decode(),
            rd,
            rs1,
            rs2,
            value,
        }
    }
}

#[allow(unused)]
#[derive(Debug, Copy, Clone)]
// Operation should contain all the information necessary to convert it back to
// its source instruction without any loss.
pub struct Operation {
    kind: InstructionKind,
    rd: Reg,
    rs1: Reg,
    rs2: Reg,
    value: u32,
}

impl Default for Operation {
    fn default() -> Self {
        Self {
            kind: InstructionKind::Invalid,
            rd: Reg::X0,
            rs1: Reg::X0,
            rs2: Reg::X0,
            value: 0,
        }
    }
}

#[allow(unused)]
impl Operation {
    pub fn new(kind: InstructionKind, rd: Reg, rs1: Reg, rs2: Reg, value: u32) -> Self {
        Self {
            kind,
            rd,
            rs1,
            rs2,
            value,
        }
    }

    #[inline(always)]
    pub fn kind(&self) -> InstructionKind {
        self.kind
    }

    #[inline(always)]
    pub fn rd(&self) -> Reg {
        self.rd
    }

    #[inline(always)]
    pub fn rs1(&self) -> Reg {
        self.rs1
    }

    #[inline(always)]
    pub fn rs2(&self) -> Reg {
        self.rs2
    }

    #[inline(always)]
    pub fn shamt(&self) -> u32 {
        self.value & 0x1f
    }

    #[inline(always)]
    pub fn imm(&self) -> u32 {
        self.value
    }

    #[inline(always)]
    #[cfg(feature = "zicsr")]
    pub fn uimm(&self) -> u32 {
        self.rs1 as u32
    }

    #[inline(always)]
    #[cfg(feature = "zicsr")]
    pub fn csr(&self) -> Csr {
        unsafe { std::mem::transmute::<u16, Csr>(self.value as u16) }
    }

    #[inline(always)]
    #[cfg(feature = "rv32a")]
    pub fn aq(&self) -> bool {
        self.value & 0x04000000 != 0
    }

    #[inline(always)]
    #[cfg(feature = "rv32a")]
    pub fn rl(&self) -> bool {
        self.value & 0x02000000 != 0
    }

    #[inline(always)]
    #[cfg(feature = "rv32a")]
    pub fn fm(&self) -> FenceMode {
        ((self.value >> 8) & 0xf).into()
    }
}
