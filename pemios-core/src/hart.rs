// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// This Source Code Form is "Incompatible With Secondary Licenses", as
// defined by the Mozilla Public License, v. 2.0.
//
// Copyright © 2022 mumblingdrunkard

pub mod csr;
pub mod instruction;
pub mod mmu;
pub mod register;
pub mod sv32;
mod utils;

mod rv32i;

#[cfg(feature = "rv32a")]
mod rv32a;
#[cfg(feature = "rv32m")]
mod rv32m;
#[cfg(feature = "zicsr")]
mod zicsr;
#[cfg(feature = "zifencei")]
mod zifencei;

pub use register::Reg;

use register::RegisterFile;
use std::sync::Arc;

use crate::bus::Bus;

use self::{
    instruction::{Conclusion, Operation},
    mmu::Mmu,
};

pub struct Hart {
    pc: u32,
    pub reg: RegisterFile,
    mmu: Mmu,
    // csr: [u32; 4096],
}

impl Hart {
    pub fn new(bus: Arc<Bus>) -> Self {
        Self {
            pc: 0,
            reg: RegisterFile::new(),
            mmu: Mmu::new(bus),
        }
    }

    /// Invalid instruction
    /// Not a part of spec, but included for cases when decoding does not recognise the
    /// instruction.
    #[inline(always)]
    fn inst_invalid(&mut self, op: &Operation) -> Conclusion {
        panic!("Executed invalid instruction! {:?}", op.kind());
    }
}
