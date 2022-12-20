pub mod decode;
pub mod execute;
mod types;

pub use types::Conclusion;

use super::{csr::Csr, Reg};
use types::*;

#[rustfmt::skip]
#[allow(unused)]
#[derive(Debug, Clone, Copy)]
pub enum Instruction {
    Lui   { rd: Reg, imm: Int32Trunc12 },
    Auipc { rd: Reg, imm: Int32Trunc12 },

    Jal  { rd: Reg, imm: Int21Trunc1 },

    Jalr { rd: Reg, rs1: Reg, imm: Int12 },

    Beq  { rs1: Reg, rs2: Reg, imm: Int13Trunc1 },
    Bne  { rs1: Reg, rs2: Reg, imm: Int13Trunc1 },
    Blt  { rs1: Reg, rs2: Reg, imm: Int13Trunc1 },
    Bge  { rs1: Reg, rs2: Reg, imm: Int13Trunc1 },
    Bltu { rs1: Reg, rs2: Reg, imm: Int13Trunc1 },
    Bgeu { rs1: Reg, rs2: Reg, imm: Int13Trunc1 },

    Lb  { rd: Reg, rs1: Reg, imm: Int12 },
    Lh  { rd: Reg, rs1: Reg, imm: Int12 },
    Lw  { rd: Reg, rs1: Reg, imm: Int12 },
    Lbu { rd: Reg, rs1: Reg, imm: Int12 },
    Lhu { rd: Reg, rs1: Reg, imm: Int12 },

    Sb { rs1: Reg, rs2: Reg, imm: Int12 },
    Sh { rs1: Reg, rs2: Reg, imm: Int12 },
    Sw { rs1: Reg, rs2: Reg, imm: Int12 },

    Addi  { rd: Reg, rs1: Reg, imm: Int12 },
    Slti  { rd: Reg, rs1: Reg, imm: Int12 },
    Sltiu { rd: Reg, rs1: Reg, imm: Int12 },
    Xori  { rd: Reg, rs1: Reg, imm: Int12 },
    Ori   { rd: Reg, rs1: Reg, imm: Int12 },
    Andi  { rd: Reg, rs1: Reg, imm: Int12 },

    Slli { rd: Reg, rs1: Reg, shamt: UInt5 },
    Srli { rd: Reg, rs1: Reg, shamt: UInt5 },
    Srai { rd: Reg, rs1: Reg, shamt: UInt5 },

    Add  { rd: Reg, rs1: Reg, rs2: Reg },
    Sub  { rd: Reg, rs1: Reg, rs2: Reg },
    Sll  { rd: Reg, rs1: Reg, rs2: Reg },
    Slt  { rd: Reg, rs1: Reg, rs2: Reg },
    Sltu { rd: Reg, rs1: Reg, rs2: Reg },
    Xor  { rd: Reg, rs1: Reg, rs2: Reg },
    Srl  { rd: Reg, rs1: Reg, rs2: Reg },
    Sra  { rd: Reg, rs1: Reg, rs2: Reg },
    Or   { rd: Reg, rs1: Reg, rs2: Reg },
    And  { rd: Reg, rs1: Reg, rs2: Reg },

    Fence { rd: Reg, rs1: Reg, pred: FenceSet, succ: FenceSet, mode: FenceMode },

    Ecall,
    Ebreak,

    Fencei { rd: Reg, rs1: Reg, imm: Int12 },

    CsrRw  { rd: Reg, rs1: Reg,    csr: Csr },
    CsrRs  { rd: Reg, rs1: Reg,    csr: Csr },
    CsrRc  { rd: Reg, rs1: Reg,    csr: Csr },

    CsrRwi { rd: Reg, uimm: UInt5, csr: Csr },
    CsrRsi { rd: Reg, uimm: UInt5, csr: Csr },
    CsrRci { rd: Reg, uimm: UInt5, csr: Csr },

    Mul    { rd: Reg, rs1: Reg, rs2: Reg },
    Mulh   { rd: Reg, rs1: Reg, rs2: Reg },
    Mulhsu { rd: Reg, rs1: Reg, rs2: Reg },
    Mulhu  { rd: Reg, rs1: Reg, rs2: Reg },
    Div    { rd: Reg, rs1: Reg, rs2: Reg },
    Divu   { rd: Reg, rs1: Reg, rs2: Reg },
    Rem    { rd: Reg, rs1: Reg, rs2: Reg },
    Remu   { rd: Reg, rs1: Reg, rs2: Reg },

    Lrw      { rd: Reg, rs1: Reg,           aq: bool, rl: bool },
    Scw      { rd: Reg, rs1: Reg, rs2: Reg, aq: bool, rl: bool },
    AmoSwapw { rd: Reg, rs1: Reg, rs2: Reg, aq: bool, rl: bool },
    AmoAddw  { rd: Reg, rs1: Reg, rs2: Reg, aq: bool, rl: bool },
    AmoXorw  { rd: Reg, rs1: Reg, rs2: Reg, aq: bool, rl: bool },
    AmoAndw  { rd: Reg, rs1: Reg, rs2: Reg, aq: bool, rl: bool },
    AmoOrw   { rd: Reg, rs1: Reg, rs2: Reg, aq: bool, rl: bool },
    AmoMinw  { rd: Reg, rs1: Reg, rs2: Reg, aq: bool, rl: bool },
    AmoMaxw  { rd: Reg, rs1: Reg, rs2: Reg, aq: bool, rl: bool },
    AmoMinuw { rd: Reg, rs1: Reg, rs2: Reg, aq: bool, rl: bool },
    AmoMaxuw { rd: Reg, rs1: Reg, rs2: Reg, aq: bool, rl: bool },

    Invalid { raw: u32 }
}

impl Default for Instruction {
    fn default() -> Self {
        Self::Invalid { raw: 0 }
    }
}
