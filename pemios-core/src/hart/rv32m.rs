// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// This Source Code Form is "Incompatible With Secondary Licenses", as
// defined by the Mozilla Public License, v. 2.0.
//
// Copyright © 2022 mumblingdrunkard

use super::{
    instruction::{Conclusion, Operation},
    Hart,
};

pub trait Rv32m {
    fn mul(&mut self, op: &Operation) -> Conclusion;
    fn mulh(&mut self, op: &Operation) -> Conclusion;
    fn mulhsu(&mut self, op: &Operation) -> Conclusion;
    fn mulhu(&mut self, op: &Operation) -> Conclusion;
    fn div(&mut self, op: &Operation) -> Conclusion;
    fn divu(&mut self, op: &Operation) -> Conclusion;
    fn rem(&mut self, op: &Operation) -> Conclusion;
    fn remu(&mut self, op: &Operation) -> Conclusion;
}

#[deny(clippy::integer_arithmetic)]
impl Rv32m for Hart {
    #[inline(always)]
    fn mul(&mut self, op: &Operation) -> Conclusion {
        self.reg
            .set(op.rd(), self.reg[op.rs1()].wrapping_mul(self.reg[op.rs2()]));
        Conclusion::None
    }

    #[inline(always)]
    fn mulh(&mut self, op: &Operation) -> Conclusion {
        self.reg.set(
            op.rd(),
            ((self.reg[op.rs1()] as i32 as i64).wrapping_mul(self.reg[op.rs2()] as i32 as i64)
                >> 32) as u32,
        );
        Conclusion::None
    }

    #[inline(always)]
    fn mulhsu(&mut self, op: &Operation) -> Conclusion {
        self.reg.set(
            op.rd(),
            ((self.reg[op.rs1()] as i32 as i64).wrapping_mul(self.reg[op.rs2()] as u64 as i64)
                >> 32) as u32,
        );
        Conclusion::None
    }

    #[inline(always)]
    fn mulhu(&mut self, op: &Operation) -> Conclusion {
        self.reg.set(
            op.rd(),
            ((self.reg[op.rs1()] as u64).wrapping_mul(self.reg[op.rs2()] as u64) >> 32) as u32,
        );
        Conclusion::None
    }

    #[inline(always)]
    fn div(&mut self, op: &Operation) -> Conclusion {
        self.reg.set(
            op.rd(),
            (self.reg[op.rs1()] as i32 as i64).wrapping_div(self.reg[op.rs2()] as i32 as i64)
                as u32,
        );
        Conclusion::None
    }

    #[inline(always)]
    fn divu(&mut self, op: &Operation) -> Conclusion {
        self.reg
            .set(op.rd(), self.reg[op.rs1()].wrapping_div(self.reg[op.rs2()]));
        Conclusion::None
    }

    #[inline(always)]
    fn rem(&mut self, op: &Operation) -> Conclusion {
        self.reg.set(
            op.rd(),
            (self.reg[op.rs1()] as i32).wrapping_rem(self.reg[op.rs2()] as i32) as u32,
        );
        Conclusion::None
    }

    #[inline(always)]
    fn remu(&mut self, op: &Operation) -> Conclusion {
        self.reg
            .set(op.rd(), self.reg[op.rs1()].wrapping_rem(self.reg[op.rs2()]));
        Conclusion::None
    }
}

#[cfg(all(test, feature = "rv32m"))]
mod tests {
    use std::sync::Arc;

    use crate::{
        bus::Bus,
        hart::{
            instruction::{InstructionKind, Operation},
            Hart, Reg,
        },
    };

    fn test_hart() -> Hart {
        let b = Arc::new(Bus::new(0));
        Hart::new(b)
    }

    #[test]
    fn mul() {
        let mut h = test_hart();
        h.reg.set(Reg::X1, 4);
        h.reg.set(Reg::X2, 4);
        let op = Operation::new(InstructionKind::Mul, Reg::X1, Reg::X1, Reg::X2, 0);
        h.test_execute_op(&op);
        assert_eq!(h.reg[Reg::X1], 16);
    }
}
