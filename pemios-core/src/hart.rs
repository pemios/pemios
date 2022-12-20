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

use std::sync::atomic::AtomicU32;

pub use register::Reg;

use register::RegisterFile;

use crate::bus::Bus;

use self::mmu::Mmu;

pub struct Hart<'a> {
    pub pc: u32,
    pub reg: RegisterFile,
    mmu: Mmu<'a>,
    // csr: [u32; 4096],
}

impl<'a> Hart<'a> {
    pub fn new(bus: &'a Bus<'a>, reservation: &'a AtomicU32) -> Self {
        let hart = Self {
            pc: 0,
            reg: RegisterFile::new(),
            mmu: Mmu::new(bus, reservation),
        };

        // can't register here because hart gets moved at the end
        // bus.register_reservation_invalidation(0, hart.mmu.reservation());

        hart
    }

    pub fn reservation(&self) -> &AtomicU32 {
        self.mmu.reservation()
    }
}
