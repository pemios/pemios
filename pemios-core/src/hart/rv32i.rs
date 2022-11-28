// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// This Source Code Form is "Incompatible With Secondary Licenses", as
// defined by the Mozilla Public License, v. 2.0.
//
// Copyright © 2022 mumblingdrunkard

use crate::hart::instruction::ExceptionKind;

use super::{
    instruction::{Conclusion, Operation},
    Hart,
};

pub trait Rv32i {
    /// # Load upper immediate
    fn lui(&mut self, op: &Operation) -> Conclusion;

    /// # Add upper immediate to program counter
    fn auipc(&mut self, op: &Operation) -> Conclusion;

    /// # Jump and link
    fn jal(&mut self, op: &Operation) -> Conclusion;

    /// # Jump and link register
    fn jalr(&mut self, op: &Operation) -> Conclusion;

    /// # Branch equal
    fn beq(&mut self, op: &Operation) -> Conclusion;

    /// # Branch not equal
    fn bne(&mut self, op: &Operation) -> Conclusion;

    /// # Branch less than
    fn blt(&mut self, op: &Operation) -> Conclusion;

    /// # Branch greater than or equal
    fn bge(&mut self, op: &Operation) -> Conclusion;

    /// # Branch less than unsigned
    fn bltu(&mut self, op: &Operation) -> Conclusion;

    /// # Brach greater than or equal unsigned
    fn bgeu(&mut self, op: &Operation) -> Conclusion;

    /// # Load byte
    fn lb(&mut self, op: &Operation) -> Conclusion;

    /// # Load half-word
    fn lh(&mut self, op: &Operation) -> Conclusion;

    /// # Load word
    fn lw(&mut self, op: &Operation) -> Conclusion;

    /// # Load byte unsigned
    fn lbu(&mut self, op: &Operation) -> Conclusion;

    /// # Load half-word unsigned
    fn lhu(&mut self, op: &Operation) -> Conclusion;

    /// # Store byte
    fn sb(&mut self, op: &Operation) -> Conclusion;

    /// # Store half-word
    fn sh(&mut self, op: &Operation) -> Conclusion;

    /// # Store word
    fn sw(&mut self, op: &Operation) -> Conclusion;

    /// # Add immediate
    ///
    /// Adds the 12-bit sign-extended immediate value to the value in rs1 and stores the result in
    /// rd.
    fn addi(&mut self, op: &Operation) -> Conclusion;

    /// # Set less than immediate
    ///
    /// Interprets the value in rs1 as a signed number and compares it with the signed 12-bit
    /// sign-extended immediate value and sets rd to 1 if rs1 < immediate.
    fn slti(&mut self, op: &Operation) -> Conclusion;

    /// # Set less than immediate unsigned
    ///
    /// Interprets the value in rs1 as an unsigned number and compares it with the unsigned 12-bit
    /// sign-extended immediate value and sets rd to 1 if rs1 < immediate -- "(i.e., the immediate is
    /// first sign-extended to XLEN bits then treated as an unsigned number)".
    fn sltiu(&mut self, op: &Operation) -> Conclusion;

    /// # Exclusive or immediate
    ///
    /// Calculates the exclusive or of rs1 and the 12-bit sign-extended immediate value and stores
    /// the result in rd.
    fn xori(&mut self, op: &Operation) -> Conclusion;

    /// # Or immediate
    fn ori(&mut self, op: &Operation) -> Conclusion;

    /// # And immediate
    fn andi(&mut self, op: &Operation) -> Conclusion;

    /// # Shift left logical immediate
    fn slli(&mut self, op: &Operation) -> Conclusion;

    /// # Shift right logical immediate
    fn srli(&mut self, op: &Operation) -> Conclusion;

    /// # Shift right arithmetic immediate
    fn srai(&mut self, op: &Operation) -> Conclusion;

    /// # Add
    fn add(&mut self, op: &Operation) -> Conclusion;

    /// # Subtract
    fn sub(&mut self, op: &Operation) -> Conclusion;

    /// # Shift left logical
    fn sll(&mut self, op: &Operation) -> Conclusion;

    /// # Set less than
    fn slt(&mut self, op: &Operation) -> Conclusion;

    /// # Set less than unsigned
    fn sltu(&mut self, op: &Operation) -> Conclusion;

    /// # Exclusive or
    fn xor(&mut self, op: &Operation) -> Conclusion;

    /// # Shift right logical
    fn srl(&mut self, op: &Operation) -> Conclusion;

    /// # Shift right arithmetic
    fn sra(&mut self, op: &Operation) -> Conclusion;

    /// # Or
    fn or(&mut self, op: &Operation) -> Conclusion;

    /// # And
    fn and(&mut self, op: &Operation) -> Conclusion;

    /// # Fence
    fn fence(&mut self, op: &Operation) -> Conclusion;

    /// # Environment call
    fn ecall(&mut self, op: &Operation) -> Conclusion;

    /// # Environment break
    fn ebreak(&mut self, op: &Operation) -> Conclusion;
}

#[deny(clippy::integer_arithmetic)] // only use wrapping arithmetic
impl Rv32i for Hart {
    // Load upper immediate
    #[inline(always)]
    fn lui(&mut self, op: &Operation) -> Conclusion {
        self.reg.set(op.rd(), op.imm());
        Conclusion::None
    }

    // Add upper immediate to program counter
    #[inline(always)]
    fn auipc(&mut self, op: &Operation) -> Conclusion {
        self.reg.set(op.rd(), self.pc.wrapping_add(op.imm()));
        Conclusion::None
    }

    // Jump and link
    #[inline(always)]
    fn jal(&mut self, op: &Operation) -> Conclusion {
        let target = self.pc.wrapping_add(op.imm());

        if target & 0b11 != 0 {
            // "The JAL and JALR instructions will generate an
            // instruction-address-misaligned exception if the target
            // address is not aligned to a four-byte boundary."
            todo!();
        }

        let linked = self.pc.wrapping_add(4);

        self.pc = target;
        self.reg.set(op.rd(), linked);
        Conclusion::Jumped
    }

    // Jump and link register
    #[inline(always)]
    fn jalr(&mut self, op: &Operation) -> Conclusion {
        let target = self.reg[op.rs1()].wrapping_add(op.imm()) & 0xfffffffe;

        if target & 0b11 != 0 {
            // "The JAL and JALR instructions will generate an
            // instruction-address-misaligned exception if the target
            // address is not aligned to a four-byte boundary."
            todo!();
        }

        let linked = self.pc.wrapping_add(4);

        self.pc = target;
        self.reg.set(op.rd(), linked);
        Conclusion::Jumped
    }

    /// Branch equal
    #[inline(always)]
    fn beq(&mut self, op: &Operation) -> Conclusion {
        if self.reg[op.rs1()] != self.reg[op.rs2()] {
            return Conclusion::None;
        }

        let target = self.pc.wrapping_add(op.imm());

        if target & 0b11 != 0 {
            // "The conditional branch instructions will generate an instruction-address-misaligned
            // exception if the target address is not aligned to a four-byte boundary and the
            // branch condition evaluates to true."
            todo!();
        }

        self.pc = target;
        Conclusion::Jumped
    }

    /// Branch not equal
    #[inline(always)]
    fn bne(&mut self, op: &Operation) -> Conclusion {
        if self.reg[op.rs1()] == self.reg[op.rs2()] {
            return Conclusion::None;
        }

        let target = self.pc.wrapping_add(op.imm());

        if target & 0b11 != 0 {
            // "The conditional branch instructions will generate an instruction-address-misaligned
            // exception if the target address is not aligned to a four-byte boundary and the
            // branch condition evaluates to true."
            todo!();
        }

        self.pc = target;
        Conclusion::Jumped
    }

    /// Branch less than
    #[inline(always)]
    fn blt(&mut self, op: &Operation) -> Conclusion {
        if (self.reg[op.rs1()] as i32) >= (self.reg[op.rs2()] as i32) {
            return Conclusion::None;
        }

        let target = self.pc.wrapping_add(op.imm());

        if target & 0b11 != 0 {
            // "The conditional branch instructions will generate an instruction-address-misaligned
            // exception if the target address is not aligned to a four-byte boundary and the
            // branch condition evaluates to true."
            todo!();
        }

        self.pc = target;
        Conclusion::Jumped
    }

    /// Branch greater than or equal
    #[inline(always)]
    fn bge(&mut self, op: &Operation) -> Conclusion {
        if (self.reg[op.rs1()] as i32) < (self.reg[op.rs2()] as i32) {
            return Conclusion::None;
        }

        let target = self.pc.wrapping_add(op.imm());

        if target & 0b11 != 0 {
            // "The conditional branch instructions will generate an instruction-address-misaligned
            // exception if the target address is not aligned to a four-byte boundary and the
            // branch condition evaluates to true."
            todo!();
        }

        self.pc = target;
        Conclusion::Jumped
    }

    /// Branch less than unsigned
    #[inline(always)]
    fn bltu(&mut self, op: &Operation) -> Conclusion {
        if self.reg[op.rs1()] >= self.reg[op.rs2()] {
            return Conclusion::None;
        }

        let target = self.pc.wrapping_add(op.imm());

        if target & 0b11 != 0 {
            // "The conditional branch instructions will generate an instruction-address-misaligned
            // exception if the target address is not aligned to a four-byte boundary and the
            // branch condition evaluates to true."
            todo!();
        }

        self.pc = target;
        Conclusion::Jumped
    }

    /// Branch greater than or equal unsigned
    #[inline(always)]
    fn bgeu(&mut self, op: &Operation) -> Conclusion {
        if self.reg[op.rs1()] < self.reg[op.rs2()] {
            return Conclusion::None;
        }

        let target = self.pc.wrapping_add(op.imm());

        if target & 3 != 0 {
            // "The conditional branch instructions will generate an instruction-address-misaligned
            // exception if the target address is not aligned to a four-byte boundary and the
            // branch condition evaluates to true."
            todo!();
        }

        self.pc = target;
        Conclusion::Jumped
    }

    /// Load byte
    #[inline(always)]
    fn lb(&mut self, op: &Operation) -> Conclusion {
        let _addr = self.reg[op.rs1()].wrapping_add(op.imm());
        todo!()
    }

    /// Load half-word
    #[inline(always)]
    fn lh(&mut self, op: &Operation) -> Conclusion {
        let addr = self.reg[op.rs1()].wrapping_add(op.imm());

        if addr & 1 != 0 {
            todo!() // address-misaligned
        }

        todo!()
    }

    /// Load word
    #[inline(always)]
    fn lw(&mut self, op: &Operation) -> Conclusion {
        let addr = self.reg[op.rs1()].wrapping_add(op.imm());

        if addr & 3 != 0 {
            todo!() // address-misaligned
        }

        match self.mmu.load_word(addr) {
            Ok(w) => {
                self.reg.set(op.rd(), w);
                Conclusion::None
            }
            Err(_) => todo!(),
        }
    }

    /// Load byte unsigned
    #[inline(always)]
    fn lbu(&mut self, op: &Operation) -> Conclusion {
        let _addr = self.reg[op.rs1()].wrapping_add(op.imm());

        todo!()
    }

    /// Load half-word unsigned
    #[inline(always)]
    fn lhu(&mut self, op: &Operation) -> Conclusion {
        let addr = self.reg[op.rs1()].wrapping_add(op.imm());

        if addr & 1 != 0 {
            todo!() // address-misaligned
        }

        todo!()
    }

    /// Store byte
    #[inline(always)]
    fn sb(&mut self, op: &Operation) -> Conclusion {
        let _addr = self.reg[op.rs1()].wrapping_add(op.imm());

        todo!()
    }

    /// Store half-word
    #[inline(always)]
    fn sh(&mut self, op: &Operation) -> Conclusion {
        let addr = self.reg[op.rs1()].wrapping_add(op.imm());

        if addr & 1 != 0 {
            todo!() // address-misaligned
        }

        todo!()
    }

    /// Store word
    #[inline(always)]
    fn sw(&mut self, op: &Operation) -> Conclusion {
        let addr = self.reg[op.rs1()].wrapping_add(op.imm());

        if addr & 3 != 0 {
            todo!() // address-misaligned
        }

        match self.mmu.store_word(addr, self.reg[op.rs2()]) {
            Ok(_) => Conclusion::None,
            Err(e) => {
                println!("{:?}", e);
                todo!("Handle this")
            }
        }
    }

    #[inline(always)]
    fn addi(&mut self, op: &Operation) -> Conclusion {
        self.reg
            .set(op.rd(), self.reg[op.rs1()].wrapping_add(op.imm()));
        Conclusion::None
    }

    #[inline(always)]
    fn slti(&mut self, op: &Operation) -> Conclusion {
        self.reg.set(
            op.rd(),
            ((self.reg[op.rs1()] as i32) < (op.imm() as i32)) as u32,
        );
        Conclusion::None
    }

    #[inline(always)]
    fn sltiu(&mut self, op: &Operation) -> Conclusion {
        self.reg
            .set(op.rd(), (self.reg[op.rs1()] < op.imm()) as u32);
        Conclusion::None
    }

    #[inline(always)]
    fn xori(&mut self, op: &Operation) -> Conclusion {
        self.reg.set(op.rd(), self.reg[op.rs1()] ^ op.imm());
        Conclusion::None
    }

    /// Or immediate
    #[inline(always)]
    fn ori(&mut self, op: &Operation) -> Conclusion {
        self.reg.set(op.rd(), self.reg[op.rs1()] | op.imm());
        Conclusion::None
    }

    /// And immediate
    #[inline(always)]
    fn andi(&mut self, op: &Operation) -> Conclusion {
        self.reg.set(op.rd(), self.reg[op.rs1()] & op.imm());
        Conclusion::None
    }

    /// Shift left logical immediate
    #[inline(always)]
    fn slli(&mut self, op: &Operation) -> Conclusion {
        self.reg.set(op.rd(), self.reg[op.rs1()] << op.shamt());
        Conclusion::None
    }

    /// Shift right logical immediate
    #[inline(always)]
    fn srli(&mut self, op: &Operation) -> Conclusion {
        self.reg.set(op.rd(), self.reg[op.rs1()] >> op.shamt());
        Conclusion::None
    }

    /// Shift right arithmetic immediate
    #[inline(always)]
    fn srai(&mut self, op: &Operation) -> Conclusion {
        self.reg
            .set(op.rd(), (self.reg[op.rs1()] as i32 >> op.shamt()) as u32);
        Conclusion::None
    }

    /// Add
    #[inline(always)]
    fn add(&mut self, op: &Operation) -> Conclusion {
        self.reg
            .set(op.rd(), self.reg[op.rs1()].wrapping_add(self.reg[op.rs2()]));
        Conclusion::None
    }

    /// Sub
    #[inline(always)]
    fn sub(&mut self, op: &Operation) -> Conclusion {
        self.reg
            .set(op.rd(), self.reg[op.rs1()].wrapping_sub(self.reg[op.rs2()]));
        Conclusion::None
    }

    /// Shift left logical
    #[inline(always)]
    fn sll(&mut self, op: &Operation) -> Conclusion {
        self.reg
            .set(op.rd(), self.reg[op.rs1()] << (self.reg[op.rs2()] & 0x1f));
        Conclusion::None
    }

    /// Set less than
    #[inline(always)]
    fn slt(&mut self, op: &Operation) -> Conclusion {
        self.reg.set(
            op.rd(),
            ((self.reg[op.rs1()] as i32) < (self.reg[op.rs2()] as i32)) as u32,
        );
        Conclusion::None
    }

    /// Set less than unsigned
    #[inline(always)]
    fn sltu(&mut self, op: &Operation) -> Conclusion {
        self.reg
            .set(op.rd(), (self.reg[op.rs1()] < self.reg[op.rs2()]) as u32);
        Conclusion::None
    }

    /// Exclusive or
    #[inline(always)]
    fn xor(&mut self, op: &Operation) -> Conclusion {
        self.reg
            .set(op.rd(), self.reg[op.rs1()] ^ self.reg[op.rs2()]);
        Conclusion::None
    }

    /// Shift right logical
    #[inline(always)]
    fn srl(&mut self, op: &Operation) -> Conclusion {
        self.reg
            .set(op.rd(), self.reg[op.rs1()] >> (self.reg[op.rs2()] & 0x1f));
        Conclusion::None
    }

    /// Shift right arithmetic
    #[inline(always)]
    fn sra(&mut self, op: &Operation) -> Conclusion {
        self.reg.set(
            op.rd(),
            (self.reg[op.rs1()] as i32 >> (self.reg[op.rs2()] & 0x1f)) as u32,
        );
        Conclusion::None
    }

    /// Or
    #[inline(always)]
    fn or(&mut self, op: &Operation) -> Conclusion {
        self.reg
            .set(op.rd(), self.reg[op.rs1()] | self.reg[op.rs2()]);
        Conclusion::None
    }

    /// And
    #[inline(always)]
    fn and(&mut self, op: &Operation) -> Conclusion {
        self.reg
            .set(op.rd(), self.reg[op.rs1()] & self.reg[op.rs2()]);
        Conclusion::None
    }

    /// Fence
    #[inline(always)]
    fn fence(&mut self, _op: &Operation) -> Conclusion {
        todo!()
    }

    /// Environment call
    #[inline(always)]
    fn ecall(&mut self, _op: &Operation) -> Conclusion {
        println!("Made it!");
        Conclusion::Exception(ExceptionKind::Ecall)
    }

    /// Environment break
    #[inline(always)]
    fn ebreak(&mut self, _op: &Operation) -> Conclusion {
        todo!()
    }
}
