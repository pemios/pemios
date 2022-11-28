// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// This Source Code Form is "Incompatible With Secondary Licenses", as
// defined by the Mozilla Public License, v. 2.0.
//
// Copyright © 2022 mumblingdrunkard

use super::OpCode;

#[allow(unused)]
#[derive(Copy, Clone, Debug, PartialOrd, Ord, PartialEq, Eq)]
#[repr(u8)]
pub enum InstructionKind {
    Lui = 0,
    Auipc,
    Jal,
    Jalr,
    Beq,
    Bne,
    Blt,
    Bge,
    Bltu,
    Bgeu,
    Lb,
    Lh,
    Lw,
    Lbu,
    Lhu,
    Sb,
    Sh,
    Sw,
    Addi,
    Slti,
    Sltiu,
    Xori,
    Ori,
    Andi,
    Slli,
    Srli,
    Srai,
    Add,
    Sub,
    Sll,
    Slt,
    Sltu,
    Xor,
    Srl,
    Sra,
    Or,
    And,
    Fence,
    Ecall,
    Ebreak,

    #[cfg(feature = "zifencei")]
    Fencei,

    #[cfg(feature = "zicsr")]
    CsrRw,
    #[cfg(feature = "zicsr")]
    CsrRs,
    #[cfg(feature = "zicsr")]
    CsrRc,
    #[cfg(feature = "zicsr")]
    CsrRwi,
    #[cfg(feature = "zicsr")]
    CsrRsi,
    #[cfg(feature = "zicsr")]
    CsrRci,

    #[cfg(feature = "rv32m")]
    Mul,
    #[cfg(feature = "rv32m")]
    Mulh,
    #[cfg(feature = "rv32m")]
    Mulhsu,
    #[cfg(feature = "rv32m")]
    Mulhu,
    #[cfg(feature = "rv32m")]
    Div,
    #[cfg(feature = "rv32m")]
    Divu,
    #[cfg(feature = "rv32m")]
    Rem,
    #[cfg(feature = "rv32m")]
    Remu,

    #[cfg(feature = "rv32a")]
    Lrw,
    #[cfg(feature = "rv32a")]
    Scw,
    #[cfg(feature = "rv32a")]
    AmoSwapw,
    #[cfg(feature = "rv32a")]
    AmoAddw,
    #[cfg(feature = "rv32a")]
    AmoXorw,
    #[cfg(feature = "rv32a")]
    AmoAndw,
    #[cfg(feature = "rv32a")]
    AmoOrw,
    #[cfg(feature = "rv32a")]
    AmoMinw,
    #[cfg(feature = "rv32a")]
    AmoMaxw,
    #[cfg(feature = "rv32a")]
    AmoMinuw,
    #[cfg(feature = "rv32a")]
    AmoMaxuw,

    /// Invalid is always the last variant of this enum, ensuring that all
    /// other variants compare less than Invalid. It is safe to construct an
    /// array of size `Invalid as usize + 1` and index it with all variants.
    Invalid,
}

#[allow(unused)]
impl InstructionKind {
    pub fn opcode(&self) -> OpCode {
        use InstructionKind::*;

        match self {
            Lui => OpCode::Lui,
            Auipc => OpCode::Auipc,
            Jal => OpCode::Jal,
            Jalr => OpCode::Jalr,

            Beq | Bne | Blt | Bge | Bltu | Bgeu => OpCode::Branch,

            Lb | Lh | Lw | Lbu | Lhu => OpCode::Load,

            Sb | Sh | Sw => OpCode::Store,

            Addi | Slti | Sltiu | Xori | Ori | Andi | Slli | Srli | Srai => OpCode::OpImm,

            Add | Sub | Sll | Slt | Sltu | Xor | Srl | Sra | Or | And => OpCode::Op,

            Fence => OpCode::MiscMem,

            Ecall | Ebreak => OpCode::System,

            #[cfg(feature = "zifencei")]
            Fencei => OpCode::MiscMem,

            #[cfg(feature = "zicsr")]
            CsrRw | CsrRs | CsrRc | CsrRwi | CsrRsi | CsrRci => OpCode::System,

            #[cfg(feature = "rv32m")]
            Mul | Mulh | Mulhsu | Mulhu | Div | Divu | Rem | Remu => OpCode::Op,

            #[cfg(feature = "rv32a")]
            Lrw | Scw | AmoSwapw | AmoAddw | AmoXorw | AmoAndw | AmoOrw | AmoMinw | AmoMaxw
            | AmoMinuw | AmoMaxuw => OpCode::Amo,

            _ => OpCode::Invalid,
        }
    }

    #[rustfmt::skip]
    pub fn template(&self) -> u32 {
        use InstructionKind::*;
        #[allow(clippy::unusual_byte_groupings)]
        match self {
            //         imm                  rd    opcode
            Lui   => 0b00000000000000000000_00000_0110111,
            Auipc => 0b00000000000000000000_00000_0010111,

            //        20|10:1|11|19:12     rd    opcode
            Jal  => 0b00000000000000000000_00000_1101111,

            //        imm          rs1   0   rd    opcode
            Jalr => 0b000000000000_00000_000_00000_1100111,

            //        12|10:5 rs2   rs1   f3  4:1|11 opcode
            Beq  => 0b0000000_00000_00000_000__00000_1100011,
            Bne  => 0b0000000_00000_00000_001__00000_1100011,
            Blt  => 0b0000000_00000_00000_100__00000_1100011,
            Bge  => 0b0000000_00000_00000_101__00000_1100011,
            Bltu => 0b0000000_00000_00000_110__00000_1100011,
            Bgeu => 0b0000000_00000_00000_111__00000_1100011,

            //       imm          rs1   f3  rd    opcode
            Lb  => 0b000000000000_00000_000_00000_0000011,
            Lh  => 0b000000000000_00000_001_00000_0000011,
            Lw  => 0b000000000000_00000_010_00000_0000011,
            Lbu => 0b000000000000_00000_100_00000_0000011,
            Lhu => 0b000000000000_00000_101_00000_0000011,

            //      i11:5   rs2   rs1   f3  i4:0  opcode
            Sb => 0b0000000_00000_00000_000_00000_0100011,
            Sh => 0b0000000_00000_00000_001_00000_0100011,
            Sw => 0b0000000_00000_00000_010_00000_0100011,

            //         f7      rs2   rs1   f3  rd    opcode
            Addi  => 0b0000000_00000_00000_000_00000_0010011,
            Slti  => 0b0000000_00000_00000_010_00000_0010011,
            Sltiu => 0b0000000_00000_00000_011_00000_0010011,
            Xori  => 0b0000000_00000_00000_100_00000_0010011,
            Ori   => 0b0000000_00000_00000_110_00000_0010011,
            Andi  => 0b0000000_00000_00000_111_00000_0010011,
            //         f7      shamt rs1   f3  rd    opcode
            Slli  => 0b0000000_00000_00000_001_00000_0010011,
            Srli  => 0b0000000_00000_00000_101_00000_0010011,
            Srai  => 0b0100000_00000_00000_101_00000_0010011,

            //        f7      rs2   rs1   f3  rd    opcode
            Add  => 0b0000000_00000_00000_000_00000_0110011,
            Sub  => 0b0100000_00000_00000_000_00000_0110011,
            Sll  => 0b0000000_00000_00000_001_00000_0110011,
            Slt  => 0b0000000_00000_00000_010_00000_0110011,
            Sltu => 0b0000000_00000_00000_011_00000_0110011,
            Xor  => 0b0000000_00000_00000_100_00000_0110011,
            Srl  => 0b0000000_00000_00000_101_00000_0110011,
            Sra  => 0b0100000_00000_00000_101_00000_0110011,
            Or   => 0b0000000_00000_00000_110_00000_0110011,
            And  => 0b0000000_00000_00000_111_00000_0110011,

            //         fm   pred succ rs1   f3  rd    opcode
            Fence => 0b0000_0000_0000_00000_000_00000_0001111,

            //          funct12      0             opcode
            Ecall  => 0b000000000000_0000000000000_1110011,
            Ebreak => 0b000000000001_0000000000000_1110011,

            //          imm          rs1   f3  rd    opcode
            #[cfg(feature = "zifencei")]
            Fencei => 0b000000000000_00000_001_00000_0001111,

            //         csr          rs1   f3  rd    opcode
            #[cfg(feature = "zicsr")]
            CsrRw => 0b000000000000_00000_001_00000_1110011,
            #[cfg(feature = "zicsr")]
            CsrRs => 0b000000000000_00000_010_00000_1110011,
            #[cfg(feature = "zicsr")]
            CsrRc => 0b000000000000_00000_011_00000_1110011,
            //          csr          uimm  f3  rd    opcode
            #[cfg(feature = "zicsr")]
            CsrRwi => 0b000000000000_00000_101_00000_1110011,
            #[cfg(feature = "zicsr")]
            CsrRsi => 0b000000000000_00000_110_00000_1110011,
            #[cfg(feature = "zicsr")]
            CsrRci => 0b000000000000_00000_111_00000_1110011,

            //          f7      rs2   rs1   f3  rd    opcode
            #[cfg(feature = "rv32m")]
            Mul    => 0b0000001_00000_00000_000_00000_0110011,
            #[cfg(feature = "rv32m")]
            Mulh   => 0b0000001_00000_00000_001_00000_0110011,
            #[cfg(feature = "rv32m")]
            Mulhsu => 0b0000001_00000_00000_010_00000_0110011,
            #[cfg(feature = "rv32m")]
            Mulhu  => 0b0000001_00000_00000_011_00000_0110011,
            #[cfg(feature = "rv32m")]
            Div    => 0b0000001_00000_00000_100_00000_0110011,
            #[cfg(feature = "rv32m")]
            Divu   => 0b0000001_00000_00000_101_00000_0110011,
            #[cfg(feature = "rv32m")]
            Rem    => 0b0000001_00000_00000_110_00000_0110011,
            #[cfg(feature = "rv32m")]
            Remu   => 0b0000001_00000_00000_111_00000_0110011,

            //            f5    aq rl 0     rs1   f3  rd    opcode
            #[cfg(feature = "rv32a")]
            Lrw      => 0b00010_0__0__00000_00000_010_00000_0101111,
            //            f5    aq rl rs2   rs1   f3  rd    opcode
            #[cfg(feature = "rv32a")]
            Scw      => 0b00011_0__0__00000_00000_010_00000_0101111,
            //            f5    aq rl rs2   rs1   f3  rd    opcode
            #[cfg(feature = "rv32a")]
            AmoSwapw => 0b00001_0__0__00000_00000_010_00000_0101111,
            #[cfg(feature = "rv32a")]
            AmoAddw  => 0b00000_0__0__00000_00000_010_00000_0101111,
            #[cfg(feature = "rv32a")]
            AmoXorw  => 0b00100_0__0__00000_00000_010_00000_0101111,
            #[cfg(feature = "rv32a")]
            AmoAndw  => 0b01100_0__0__00000_00000_010_00000_0101111,
            #[cfg(feature = "rv32a")]
            AmoOrw   => 0b01000_0__0__00000_00000_010_00000_0101111,
            #[cfg(feature = "rv32a")]
            AmoMinw  => 0b10000_0__0__00000_00000_010_00000_0101111,
            #[cfg(feature = "rv32a")]
            AmoMaxw  => 0b10100_0__0__00000_00000_010_00000_0101111,
            #[cfg(feature = "rv32a")]
            AmoMinuw => 0b11000_0__0__00000_00000_010_00000_0101111,
            #[cfg(feature = "rv32a")]
            AmoMaxuw => 0b11100_0__0__00000_00000_010_00000_0101111,

            Invalid => 0,
        }
    }
}
