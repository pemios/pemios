// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// This Source Code Form is "Incompatible With Secondary Licenses", as
// defined by the Mozilla Public License, v. 2.0.
//
// Copyright © 2022 mumblingdrunkard

use crate::hart::{rv32i::Rv32i, Hart};

#[cfg(feature = "rv32a")]
use crate::hart::rv32a::Rv32a;
#[cfg(feature = "rv32m")]
use crate::hart::rv32m::Rv32m;
#[cfg(feature = "zicsr")]
use crate::hart::zicsr::Zicsr;
#[cfg(feature = "zifencei")]
use crate::hart::zifencei::Zifencei;

use super::{Conclusion, Operation};

pub trait Execute {
    fn execute(&mut self) -> Conclusion;
}

impl<'a> Hart<'a> {
    #[inline(always)]
    fn execute_op(&mut self, op: &Operation) -> Conclusion {
        use super::InstructionKind::*;
        let conclusion = match op.kind {
            Lui => self.lui(op),
            Auipc => self.auipc(op),

            Jal => self.jal(op),
            Jalr => self.jalr(op),

            Beq => self.beq(op),
            Bne => self.bne(op),
            Blt => self.blt(op),
            Bge => self.bge(op),
            Bltu => self.bltu(op),
            Bgeu => self.bgeu(op),

            Lb => self.lb(op),
            Lh => self.lh(op),
            Lw => self.lw(op),
            Lbu => self.lbu(op),
            Lhu => self.lhu(op),
            Sb => self.sb(op),
            Sh => self.sh(op),
            Sw => self.sw(op),

            Addi => self.addi(op),
            Slti => self.slti(op),
            Sltiu => self.sltiu(op),
            Xori => self.xori(op),
            Ori => self.ori(op),
            Andi => self.andi(op),
            Slli => self.slli(op),
            Srli => self.srli(op),
            Srai => self.srai(op),

            Add => self.add(op),
            Sub => self.sub(op),
            Sll => self.sll(op),
            Slt => self.slt(op),
            Sltu => self.sltu(op),
            Xor => self.xor(op),
            Srl => self.srl(op),
            Sra => self.sra(op),
            Or => self.or(op),
            And => self.and(op),

            Fence => self.fence(op),

            Ecall => self.ecall(op),
            Ebreak => self.ebreak(op),

            #[cfg(feature = "zifencei")]
            Fencei => self.fencei(op),

            #[cfg(feature = "zicsr")]
            CsrRw => self.csrrw(op),
            #[cfg(feature = "zicsr")]
            CsrRs => self.csrrs(op),
            #[cfg(feature = "zicsr")]
            CsrRc => self.csrrc(op),
            #[cfg(feature = "zicsr")]
            CsrRwi => self.csrrwi(op),
            #[cfg(feature = "zicsr")]
            CsrRsi => self.csrrsi(op),
            #[cfg(feature = "zicsr")]
            CsrRci => self.csrrci(op),

            #[cfg(feature = "rv32m")]
            Mul => self.mul(op),
            #[cfg(feature = "rv32m")]
            Mulh => self.mulh(op),
            #[cfg(feature = "rv32m")]
            Mulhsu => self.mulhsu(op),
            #[cfg(feature = "rv32m")]
            Mulhu => self.mulhu(op),
            #[cfg(feature = "rv32m")]
            Div => self.div(op),
            #[cfg(feature = "rv32m")]
            Divu => self.divu(op),
            #[cfg(feature = "rv32m")]
            Rem => self.rem(op),
            #[cfg(feature = "rv32m")]
            Remu => self.remu(op),

            #[cfg(feature = "rv32a")]
            Lrw => self.lr_w(op),
            #[cfg(feature = "rv32a")]
            Scw => self.sc_w(op),
            #[cfg(feature = "rv32a")]
            AmoSwapw => self.amoswap_w(op),
            #[cfg(feature = "rv32a")]
            AmoAddw => self.amoadd_w(op),
            #[cfg(feature = "rv32a")]
            AmoXorw => self.amoxor_w(op),
            #[cfg(feature = "rv32a")]
            AmoAndw => self.amoand_w(op),
            #[cfg(feature = "rv32a")]
            AmoOrw => self.amoor_w(op),
            #[cfg(feature = "rv32a")]
            AmoMinw => self.amomin_w(op),
            #[cfg(feature = "rv32a")]
            AmoMaxw => self.amomax_w(op),
            #[cfg(feature = "rv32a")]
            AmoMinuw => self.amominu_w(op),
            #[cfg(feature = "rv32a")]
            AmoMaxuw => self.amomaxu_w(op),

            Invalid => self.inst_invalid(op),
        };

        if let Conclusion::None = conclusion {
            self.pc = self.pc.wrapping_add(4)
        };

        conclusion
    }

    #[cfg(test)]
    pub fn test_execute_op(&mut self, op: &Operation) -> Conclusion {
        self.execute_op(op)
    }
}

impl<'a> Execute for Hart<'a> {
    #[allow(unused)]
    #[inline(always)]
    fn execute(&mut self) -> Conclusion {
        use super::InstructionKind::*;

        let op = match self.mmu.load_instruction(self.pc) {
            Ok(op) => op,
            Err(_) => todo!(),
        };

        self.execute_op(&op)
    }
}
