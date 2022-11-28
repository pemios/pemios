// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// This Source Code Form is "Incompatible With Secondary Licenses", as
// defined by the Mozilla Public License, v. 2.0.
//
// Copyright © 2022 mumblingdrunkard

#[allow(unused)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum Reg {
    /// Description: "Hard-wired zero"
    /// ABI Name: `zero`
    /// Aliases: `Reg::ZERO`
    /// Saver: ---
    X0 = 0,

    /// Description: "Return address"
    /// ABI Name: `ra`
    /// Aliases: `Reg::RA`
    /// Saver: Caller
    X1,

    /// Description: "Stack pointer"
    /// ABI Name: `sp`
    /// Aliases: `Reg::RA`
    /// Saver: Callee
    X2,

    /// Description: "Global pointer"
    /// ABI Name: `gp`
    /// Aliases: `Reg::GP`
    /// Saver: ---
    X3,

    /// Description: "Thread pointer"
    /// ABI Name: `tp`
    /// Aliases: `Reg::TP`
    /// Saver: ---
    X4,

    /// Description: "Temporary/alternate link register"
    /// ABI Name: `t0`
    /// Aliases: `Reg::T0`
    /// Saver: Caller
    X5,

    /// Description: "Temporaries"
    /// ABI Name: `t1`
    /// Aliases: `Reg::T1`
    /// Saver: Caller
    X6,

    /// Description: "Temporaries"
    /// ABI Name: `t2`
    /// Aliases: `Reg::T2`
    /// Saver: Caller
    X7,

    /// Description: "Saved register/frame pointer"
    /// ABI Name: `s0`/`fp`
    /// Aliases: `Reg::S0`, `Reg::FP`
    /// Saver: Caller
    X8,

    /// Description: "Saved register/frame pointer"
    /// ABI Name: `s0`/`fp`
    /// Aliases: `Reg::S0`, `Reg::FP`
    /// Saver: Caller
    X9,

    /// Description: "Function arguments/return values"
    /// ABI Name: `a0`
    /// Aliases: `Reg::A0`
    /// Saver: Caller
    X10,

    /// Description: "Function arguments/return values"
    /// ABI Name: `a1`
    /// Aliases: `Reg::A1`
    /// Saver: Caller
    X11,

    /// Description: "Function arguments"
    /// ABI Name: `a2`
    /// Aliases: `Reg::A2`
    /// Saver: Caller
    X12,

    /// Description: "Function arguments"
    /// ABI Name: `a3`
    /// Aliases: `Reg::A3`
    /// Saver: Caller
    X13,

    /// Description: "Function arguments"
    /// ABI Name: `a4`
    /// Aliases: `Reg::A4`
    /// Saver: Caller
    X14,

    /// Description: "Function arguments"
    /// ABI Name: `a5`
    /// Aliases: `Reg::A5`
    /// Saver: Caller
    X15,

    /// Description: "Function arguments"
    /// ABI Name: `a6`
    /// Aliases: `Reg::A6`
    /// Saver: Caller
    X16,

    /// Description: "Function arguments"
    /// ABI Name: `a7`
    /// Aliases: `Reg::A7`
    /// Saver: Caller
    X17,

    /// Description: "Saved registers"
    /// ABI Name: `s2`
    /// Aliases: `Reg::S2`
    /// Saver: Callee
    X18,

    /// Description: "Saved registers"
    /// ABI Name: `s3`
    /// Aliases: `Reg::S3`
    /// Saver: Callee
    X19,

    /// Description: "Saved registers"
    /// ABI Name: `s4`
    /// Aliases: `Reg::S4`
    /// Saver: Callee
    X20,

    /// Description: "Saved registers"
    /// ABI Name: `s5`
    /// Aliases: `Reg::S5`
    /// Saver: Callee
    X21,

    /// Description: "Saved registers"
    /// ABI Name: `s6`
    /// Aliases: `Reg::S6`
    /// Saver: Callee
    X22,

    /// Description: "Saved registers"
    /// ABI Name: `s7`
    /// Aliases: `Reg::S7`
    /// Saver: Callee
    X23,

    /// Description: "Saved registers"
    /// ABI Name: `s8`
    /// Aliases: `Reg::S8`
    /// Saver: Callee
    X24,

    /// Description: "Saved registers"
    /// ABI Name: `s9`
    /// Aliases: `Reg::S9`
    /// Saver: Callee
    X25,

    /// Description: "Saved registers"
    /// ABI Name: `s10`
    /// Aliases: `Reg::S10`
    /// Saver: Callee
    X26,

    /// Description: "Saved registers"
    /// ABI Name: `s11`
    /// Aliases: `Reg::S11`
    /// Saver: Callee
    X27,

    /// Description: "Temporaries"
    /// ABI Name: `t3`
    /// Aliases: `Reg::T3`
    /// Saver: Caller
    X28,

    /// Description: "Temporaries"
    /// ABI Name: `t4`
    /// Aliases: `Reg::T4`
    /// Saver: Caller
    X29,

    /// Description: "Temporaries"
    /// ABI Name: `t5`
    /// Aliases: `Reg::T5`
    /// Saver: Caller
    X30,

    /// Description: "Temporaries"
    /// ABI Name: `t6`
    /// Aliases: `Reg::T6`
    /// Saver: Caller
    X31,
}

// Register ABI names
#[allow(unused)]
impl Reg {
    /// Description: "Hard-wired zero"
    /// Saver: ---
    pub const ZERO: Self = Self::X0;

    /// Description: "Return address"
    /// Saver: Caller
    pub const RA: Self = Self::X1;

    /// Description: "Stack pointer"
    /// Saver: Callee
    pub const SP: Self = Self::X2;

    /// Description: "Global pointer"
    /// Saver: ---
    pub const GP: Self = Self::X3;

    /// Description: "Thread pointer"
    /// Saver: ---
    pub const TP: Self = Self::X4;

    /// Description: "Temporary/alternate link register"
    /// Saver: Caller
    pub const T0: Self = Self::X5;

    /// Description: "Temporaries"
    /// Saver: Caller
    pub const T1: Self = Self::X6;

    /// Description: "Temporaries"
    /// Saver: ---
    pub const T2: Self = Self::X7;

    /// Description: "Saved register"
    /// Saver: Callee
    pub const S0: Self = Self::X8;

    /// Description: "Frame pointer"
    /// Saver: Callee
    pub const FP: Self = Self::X8;

    /// Description: "Saved register"
    /// Saver: Callee
    pub const S1: Self = Self::X9;

    /// Description: "Function arguments/return values"
    /// Saver: Caller
    pub const A0: Self = Self::X10;

    /// Description: "Function arguments/return values"
    /// Saver: Caller
    pub const A1: Self = Self::X11;

    /// Description: "Function arguments"
    /// Saver: Caller
    pub const A2: Self = Self::X12;

    /// Description: "Function arguments"
    /// Saver: Caller
    pub const A3: Self = Self::X13;

    /// Description: "Function arguments"
    /// Saver: Caller
    pub const A4: Self = Self::X14;

    /// Description: "Function arguments"
    /// Saver: Caller
    pub const A5: Self = Self::X15;

    /// Description: "Function arguments"
    /// Saver: Caller
    pub const A6: Self = Self::X16;

    /// Description: "Function arguments"
    /// Saver: Caller
    pub const A7: Self = Self::X17;

    /// Description: "Saved registers"
    /// Saver: Callee
    pub const S2: Self = Self::X18;

    /// Description: "Saved registers"
    /// Saver: Callee
    pub const S3: Self = Self::X19;

    /// Description: "Saved registers"
    /// Saver: Callee
    pub const S4: Self = Self::X20;

    /// Description: "Saved registers"
    /// Saver: Callee
    pub const S5: Self = Self::X21;

    /// Description: "Saved registers"
    /// Saver: Callee
    pub const S6: Self = Self::X22;

    /// Description: "Saved registers"
    /// Saver: Callee
    pub const S7: Self = Self::X23;

    /// Description: "Saved registers"
    /// Saver: Callee
    pub const S8: Self = Self::X24;

    /// Description: "Saved registers"
    /// Saver: Callee
    pub const S9: Self = Self::X25;

    /// Description: "Saved registers"
    /// Saver: Callee
    pub const S10: Self = Self::X26;

    /// Description: "Saved registers"
    /// Saver: Callee
    pub const S11: Self = Self::X27;

    /// Description: "Temporaries"
    /// Saver: Caller
    pub const T3: Self = Self::X28;

    /// Description: "Temporaries"
    /// Saver: Caller
    pub const T4: Self = Self::X29;

    /// Description: "Temporaries"
    /// Saver: Caller
    pub const T5: Self = Self::X30;

    /// Description: "Temporaries"
    /// Saver: Caller
    pub const T6: Self = Self::X31;
}

impl From<u32> for Reg {
    fn from(r: u32) -> Self {
        #[rustfmt::skip]
        const REGISTERS: [Reg; 32] = [
            Reg::X0, Reg::X1, Reg::X2, Reg::X3, Reg::X4, Reg::X5, Reg::X6, Reg::X7, Reg::X8,
            Reg::X9, Reg::X10, Reg::X11, Reg::X12, Reg::X13, Reg::X14, Reg::X15, Reg::X16,
            Reg::X17, Reg::X18, Reg::X19, Reg::X20, Reg::X21, Reg::X22, Reg::X23, Reg::X24,
            Reg::X25, Reg::X26, Reg::X27, Reg::X28, Reg::X29, Reg::X30, Reg::X31,
        ];

        match r {
            0..=31 => REGISTERS[r as usize],
            _ => panic!(),
        }
    }
}

#[derive(Debug)]
pub struct RegisterFile {
    reg: [u32; 33],
}

impl RegisterFile {
    pub fn new() -> Self {
        Self { reg: [0; 33] }
    }

    #[inline]
    pub fn set(&mut self, rd: Reg, val: u32) {
        const LUT: [u8; 32] = {
            let mut lut = [32; 32];
            let mut i = 1;
            while i < 32 {
                lut[i] = i as u8;
                i += 1;
            }
            lut
        };

        unsafe {
            *self
                .reg
                .get_unchecked_mut(*LUT.get_unchecked(rd as usize) as usize) = val
        };
    }
}

impl std::ops::Index<Reg> for RegisterFile {
    type Output = u32;

    fn index(&self, index: Reg) -> &Self::Output {
        unsafe { self.reg.get_unchecked(index as usize) }
    }
}
