// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// This Source Code Form is "Incompatible With Secondary Licenses", as
// defined by the Mozilla Public License, v. 2.0.
//
// Copyright © 2022 mumblingdrunkard

use std::sync::Arc;

use crate::{
    bus::{Bus, BusError},
    memory::{
        self,
        memory_region::{Mapping, MemoryError, PmaPacked},
    },
};

use self::cache::Cache;

use super::{instruction::Operation, sv32::Pte};

mod cache;

#[derive(Debug)]
pub enum MmuError {
    LoadMisaligned { addr: u32, alignment: u32 },
    StoreMisaligned { addr: u32, alignment: u32 },
    OutOfBoundsAccess { addr: u32 },
    BusError { e: BusError },
}

impl From<BusError> for MmuError {
    fn from(e: BusError) -> Self {
        Self::BusError { e }
    }
}

impl From<MemoryError> for MmuError {
    fn from(e: MemoryError) -> Self {
        Self::BusError { e: e.into() }
    }
}

type MmuResult<T> = std::result::Result<T, MmuError>;

#[allow(unused)]
pub struct Mmu {
    d_cache: cache::Cache<u32, u64, 8, 3, 4>,
    i_cache: cache::Cache<Operation, (), 8, 3, 4>,
    // only one element per cache line as it makes little sense to block-fetch memory attributes
    attr: cache::Cache<PmaPacked, (), 12, 3, 0>,
    // only one element per cache line as block-fetching translations also makes no sense
    tlb: cache::Cache<Pte, (), 12, 3, 0>,
    bus: Arc<Bus>,
}

impl Mmu {
    pub fn new(bus: Arc<Bus>) -> Self {
        Self {
            d_cache: Cache::new(),
            i_cache: Cache::new(),
            attr: Cache::new(),
            tlb: Cache::new(),
            bus,
        }
    }

    #[allow(unused)]
    #[inline(always)]
    fn cacheable(&self, addr: u32) -> bool {
        addr & 0x80000000 == 0 || { todo!("Check attribute cache, or get attributes from bus") }
    }

    #[allow(unused)]
    #[inline(always)]
    fn translate(&self, addr: u32) -> u32 {
        todo!("Check tlb, or walk page table")
    }

    #[inline(always)]
    pub fn load_byte(&mut self, _addr: u32) -> MmuResult<u32> {
        todo!()
    }

    #[inline(always)]
    pub fn load_half_word(&mut self, _addr: u32) -> MmuResult<u32> {
        todo!()
    }

    #[inline(always)]
    fn load_word_physical(&mut self, addr: u32) -> MmuResult<u32> {
        if addr & 3 != 0 {
            return Err(MmuError::LoadMisaligned { addr, alignment: 4 });
        }

        if self.cacheable(addr) {
            // fast path
            if let Some(&w) = self.d_cache.get(addr >> 2) {
                return Ok(u32::from_le(w));
            }

            // closure to be executed when cache line is missing
            let missing = |x: &mut [u32; 16]| {
                let (_, dst, _) = unsafe { x.align_to_mut::<u8>() };
                self.bus.block_read(addr & 0xffffffc0, dst)
            };

            let (&w, evicted) = self.d_cache.get_or_insert_with(addr >> 2, missing)?;

            if let Some((evicted_addr, evicted_data, evicted_tracker)) = evicted {
                let (_, src, _) = unsafe { evicted_data.align_to::<u8>() };
                self.bus.block_write(evicted_addr << 2, src)?;
                todo!("Use bus.block_write_masked()")
            }

            Ok(w)
        } else {
            todo!()
        }
    }

    #[inline(always)]
    pub fn load_word(&mut self, addr: u32) -> MmuResult<u32> {
        // TODO Address translation
        // TODO Check user mode
        // TODO Check read permissions
        self.load_word_physical(addr)
    }

    #[inline(always)]
    pub fn load_instruction(&mut self, addr: u32) -> MmuResult<Operation> {
        // TODO Address translation
        // TODO Check user mode
        // TODO Check read permissions

        if addr & 3 != 0 {
            return Err(MmuError::LoadMisaligned { addr, alignment: 4 });
        }

        if let Some(&op) = self.i_cache.get(addr >> 2) {
            return Ok(op);
        }

        let missing = |x: &mut [Operation; 16]| -> memory::memory_region::MemoryResult<()> {
            let mut raw = [0u32; 16];
            let (_, dst, _) = unsafe { raw.align_to_mut::<u8>() };
            self.bus.block_read(addr & 0xffffffc0, dst)?;

            x.iter_mut()
                .zip(raw.into_iter())
                .for_each(|(d, s)| *d = u32::from_le(s).into());

            Ok(())
        };

        let (&op, _) = self.i_cache.get_or_insert_with(addr >> 2, missing)?;

        Ok(op)
    }

    #[inline(always)]
    pub fn store_byte(&mut self, _addr: u32) -> MmuResult<()> {
        todo!()
    }

    #[inline(always)]
    pub fn store_half_word(&mut self, _addr: u32) -> MmuResult<()> {
        todo!()
    }

    #[inline(always)]
    pub fn store_word(&mut self, addr: u32, w: u32) -> MmuResult<()> {
        // fast path
        if let Some(target) = self.d_cache.get_mut(addr >> 2) {
            *target.0 = w;
            // todo!("Update tracker");
            return Ok(());
        }

        let missing = |x: &mut [u32; 16]| {
            let (_, dst, _) = unsafe { x.align_to_mut::<u8>() };
            self.bus.block_read(addr & 0xffffffc0, dst)
        };

        let (target, evicted) = self.d_cache.get_mut_or_insert_with(addr >> 2, missing)?;
        *target.0 = w;

        // todo!("Update tracker");

        #[allow(unreachable_code)]
        if let Some((addr, data, mask)) = evicted {
            let (_, src, _) = unsafe { data.align_to::<u8>() };
            self.bus.block_write(addr << 2, src)?;
            todo!("Use bus.block_write_masked");
        }

        Ok(())
    }

    #[inline(always)]
    pub fn load_word_atomic(&mut self, _addr: u32) -> MmuResult<u32> {
        todo!()
    }

    #[inline(always)]
    pub fn store_word_atomic(&mut self, _addr: u32) -> MmuResult<()> {
        todo!()
    }

    #[inline(always)]
    pub fn swap_word_atomic(&mut self, _addr: u32) -> MmuResult<u32> {
        todo!()
    }

    #[inline(always)]
    pub fn add_word_atomic(&mut self, _addr: u32) -> MmuResult<u32> {
        todo!()
    }

    #[inline(always)]
    pub fn and_word_atomic(&mut self, _addr: u32) -> MmuResult<u32> {
        todo!()
    }

    #[inline(always)]
    pub fn or_word_atomic(&mut self, _addr: u32) -> MmuResult<u32> {
        todo!()
    }

    #[inline(always)]
    pub fn xor_word_atomic(&mut self, _addr: u32) -> MmuResult<u32> {
        todo!()
    }

    #[inline(always)]
    pub fn max_word_atomic(&mut self, _addr: u32) -> MmuResult<u32> {
        todo!()
    }

    #[inline(always)]
    pub fn min_word_atomic(&mut self, _addr: u32) -> MmuResult<u32> {
        todo!()
    }

    #[inline(always)]
    pub fn maxu_word_atomic(&mut self, _addr: u32) -> MmuResult<u32> {
        todo!()
    }

    #[inline(always)]
    pub fn minu_word_atomic(&mut self, _addr: u32) -> MmuResult<u32> {
        todo!()
    }
}
