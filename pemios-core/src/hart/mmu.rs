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
        mapping::{Mapping, MemoryError, PmaPacked},
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

pub struct Mmu {
    d_cache: cache::Cache<u32, u64, 8, 2, 4>,
    i_cache: cache::Cache<Operation, (), 8, 2, 4>,
    // only one element per cache line as it makes little sense to block-fetch memory attributes
    #[allow(unused)]
    attr: cache::Cache<PmaPacked, (), 12, 3, 0>,
    // only one element per cache line as block-fetching translations also makes no sense
    #[allow(unused)]
    tlb: cache::Cache<Pte, (), 12, 3, 0>,
    bus: Arc<Bus>,
}

trait AsU8Array<const W: usize> {
    fn as_u8_array(&self) -> &[u8; W];
    fn as_u8_array_mut(&mut self) -> &mut [u8; W];
}

impl AsU8Array<4> for u32 {
    fn as_u8_array(&self) -> &[u8; 4] {
        unsafe { std::mem::transmute::<&Self, &[u8; 4]>(self) }
    }

    fn as_u8_array_mut(&mut self) -> &mut [u8; 4] {
        unsafe { std::mem::transmute::<&mut Self, &mut [u8; 4]>(self) }
    }
}

impl AsU8Array<8> for u64 {
    fn as_u8_array(&self) -> &[u8; 8] {
        unsafe { std::mem::transmute::<&Self, &[u8; 8]>(self) }
    }

    fn as_u8_array_mut(&mut self) -> &mut [u8; 8] {
        unsafe { std::mem::transmute::<&mut Self, &mut [u8; 8]>(self) }
    }
}

trait AsU16Array<const W: usize> {
    fn as_u16_array(&self) -> &[u16; W];
    fn as_u16_array_mut(&mut self) -> &mut [u16; W];
}

impl AsU16Array<2> for u32 {
    fn as_u16_array(&self) -> &[u16; 2] {
        unsafe { std::mem::transmute::<&Self, &[u16; 2]>(self) }
    }

    fn as_u16_array_mut(&mut self) -> &mut [u16; 2] {
        unsafe { std::mem::transmute::<&mut Self, &mut [u16; 2]>(self) }
    }
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

    #[inline(always)]
    fn cacheable(&self, addr: u32) -> bool {
        addr & 0x80000000 == 0 || { todo!("Check attribute cache, or get attributes from bus") }
    }

    #[allow(unused)]
    #[inline(always)]
    fn translate(&self, addr: u32) -> u32 {
        todo!("Determine translation and protection, check tlb, walk page table")
    }

    #[inline(always)]
    fn load_physical<const W: usize>(&mut self, addr: u32) -> MmuResult<u32> {
        assert!(matches!(W, 1 | 2 | 4), "Load width must be 1, 2, or 4");

        if W == 4 && addr & 3 != 0 {
            return Err(MmuError::LoadMisaligned { addr, alignment: 4 });
        } else if W == 2 && addr & 1 != 0 {
            return Err(MmuError::LoadMisaligned { addr, alignment: 2 });
        }

        if self.cacheable(addr) {
            // fast path
            if let Some(&w) = self.d_cache.get(addr >> 2) {
                if W == 4 {
                    return Ok(u32::from_le(w));
                } else if W == 2 {
                    let a = w.as_u16_array();
                    return Ok(u16::from_le(a[(addr as usize >> 1) & 1]) as u32);
                } else {
                    let a = w.as_u8_array();
                    return Ok((a[addr as usize & 3]) as u32);
                }
            }

            // closure to be executed when cache line is missing
            let missing = |x: &mut [u32; 16]| {
                let (_, dst, _) = unsafe { x.align_to_mut::<u8>() };
                self.bus.block_read(addr & 0xffffffc0, dst)
            };

            let (&w, evicted) = self.d_cache.get_or_insert_with(addr >> 2, missing)?;

            if let Some((addr, data, mask)) = evicted {
                let mask = mask.to_le(); // ensures mask.as_u8_array()[0] & 1 is the first bit
                let mask = mask.as_u8_array();
                let (_, src, _) = unsafe { data.align_to::<u8>() };
                self.bus.block_write_masked(addr << 2, src, &mask[..])?;
            }

            if W == 4 {
                Ok(u32::from_le(w))
            } else if W == 2 {
                let a = w.as_u16_array();
                Ok(u16::from_le(a[(addr as usize >> 1) & 1]) as u32)
            } else {
                let a = w.as_u8_array();
                Ok((a[addr as usize & 3]) as u32)
            }
        } else {
            todo!()
        }
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
    pub fn load_word(&mut self, addr: u32) -> MmuResult<u32> {
        // TODO Address translation
        // TODO Check user mode
        // TODO Check read permissions
        self.load_physical::<4>(addr)
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

        let missing = |x: &mut [Operation; 16]| -> memory::mapping::MemoryResult<()> {
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
    fn store_physical<const W: usize>(&mut self, addr: u32, val: u32) -> MmuResult<()> {
        assert!(matches!(W, 1 | 2 | 4), "Load width must be 1, 2, or 4");

        if W == 4 && addr & 3 != 0 {
            return Err(MmuError::LoadMisaligned { addr, alignment: 4 });
        } else if W == 2 && addr & 1 != 0 {
            return Err(MmuError::LoadMisaligned { addr, alignment: 2 });
        }

        if self.cacheable(addr) {
            // fast path
            if let Some((target, tracker)) = self.d_cache.get_mut(addr >> 2) {
                if W == 4 {
                    *target = val.to_le();
                    *tracker |= 15 << (addr & 0x3f);
                } else if W == 2 {
                    let a = target.as_u16_array_mut();
                    a[(addr as usize >> 1) & 1] = (val as u16).to_le();
                    *tracker |= 3 << (addr & 0x3f);
                } else {
                    let a = target.as_u8_array_mut();
                    a[addr as usize & 3] = val as u8;
                    *tracker |= 1 << (addr & 0x3f);
                }
                return Ok(());
            }

            // closure to be executed when cache line is missing
            let missing = |x: &mut [u32; 16]| {
                let (_, dst, _) = unsafe { x.align_to_mut::<u8>() };
                self.bus.block_read(addr & 0xffffffc0, dst)
            };

            let ((target, tracker), evicted) =
                self.d_cache.get_mut_or_insert_with(addr >> 2, missing)?;

            if let Some((addr, data, mask)) = evicted {
                let mask = mask.to_le(); // ensures mask.as_u8_array()[0] & 1 is the first bit
                let mask = mask.as_u8_array();
                let (_, src, _) = unsafe { data.align_to::<u8>() };
                self.bus.block_write_masked(addr << 2, src, &mask[..])?;
            }

            if W == 4 {
                *target = val.to_le();
                *tracker |= 15 << (addr & 0x3f);
            } else if W == 2 {
                let a = target.as_u16_array_mut();
                a[(addr as usize >> 1) & 1] = (val as u16).to_le();
                *tracker |= 3 << (addr & 0x3f);
            } else {
                let a = target.as_u8_array_mut();
                a[addr as usize & 3] = val as u8;
                *tracker |= 1 << (addr & 0x3f);
            }
            Ok(())
        } else {
            todo!()
        }
    }

    #[inline(always)]
    pub fn store_byte(&mut self, addr: u32, b: u8) -> MmuResult<()> {
        self.store_physical::<1>(addr, b as u32)
    }

    #[inline(always)]
    pub fn store_half_word(&mut self, addr: u32, hw: u16) -> MmuResult<()> {
        self.store_physical::<2>(addr, hw as u32)
    }

    #[inline(always)]
    pub fn store_word(&mut self, addr: u32, w: u32) -> MmuResult<()> {
        self.store_physical::<4>(addr, w)
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
