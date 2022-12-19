// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// This Source Code Form is "Incompatible With Secondary Licenses", as
// defined by the Mozilla Public License, v. 2.0.
//
// Copyright © 2022 mumblingdrunkard

use std::sync::atomic::{AtomicU32, Ordering};

use crate::{
    bus::{Bus, BusError},
    memory::{
        self,
        mapping::{Mapping, MemoryError, PmaPacked},
    },
};

use self::cache::Cache;

use super::{instruction::Instruction, sv32::Pte};

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

pub type MmuResult<T> = std::result::Result<T, MmuError>;

pub fn addr_to_reservation_set(addr: u32) -> u32 {
    addr >> 6
}

pub fn helper_invalidate_reservations(
    reservations: &[&std::sync::atomic::AtomicU32],
    should_be: u32,
) {
    reservations.iter().for_each(|r| {
        let _ = r.compare_exchange(should_be, 0xffffffff, Ordering::Relaxed, Ordering::Relaxed);
    });
}

pub fn helper_check_reservation(reservation: &AtomicU32, should_be: u32) -> u32 {
    match reservation.compare_exchange(should_be, 0xffffffff, Ordering::Relaxed, Ordering::Relaxed)
    {
        Ok(_) => 0,
        Err(_) => 1,
    }
}

pub struct Mmu<'a> {
    reservation: &'a AtomicU32,
    d_cache: Box<cache::Cache<u32, u64, 8, 2, 4>>,
    i_cache: Box<cache::Cache<Instruction, (), 8, 2, 4>>,
    // only one element per cache line as it makes little sense to block-fetch memory attributes
    #[allow(unused)]
    attr: Box<cache::Cache<PmaPacked, (), 12, 3, 0>>,
    // only one element per cache line as block-fetching translations also makes no sense
    #[allow(unused)]
    tlb: Box<cache::Cache<Pte, (), 12, 3, 0>>,
    bus: &'a Bus<'a>,
}

trait AsU8Array<const W: usize> {
    fn as_u8_array(&self) -> &[u8; W];
    fn as_u8_array_mut(&mut self) -> &mut [u8; W];
}

impl AsU8Array<4> for u32 {
    #[inline(always)]
    fn as_u8_array(&self) -> &[u8; 4] {
        unsafe { std::mem::transmute::<&Self, &[u8; 4]>(self) }
    }

    #[inline(always)]
    fn as_u8_array_mut(&mut self) -> &mut [u8; 4] {
        unsafe { std::mem::transmute::<&mut Self, &mut [u8; 4]>(self) }
    }
}

impl AsU8Array<8> for u64 {
    #[inline(always)]
    fn as_u8_array(&self) -> &[u8; 8] {
        unsafe { std::mem::transmute::<&Self, &[u8; 8]>(self) }
    }

    #[inline(always)]
    fn as_u8_array_mut(&mut self) -> &mut [u8; 8] {
        unsafe { std::mem::transmute::<&mut Self, &mut [u8; 8]>(self) }
    }
}

trait AsU16Array<const W: usize> {
    fn as_u16_array(&self) -> &[u16; W];
    fn as_u16_array_mut(&mut self) -> &mut [u16; W];
}

impl AsU16Array<2> for u32 {
    #[inline(always)]
    fn as_u16_array(&self) -> &[u16; 2] {
        unsafe { std::mem::transmute::<&Self, &[u16; 2]>(self) }
    }

    #[inline(always)]
    fn as_u16_array_mut(&mut self) -> &mut [u16; 2] {
        unsafe { std::mem::transmute::<&mut Self, &mut [u16; 2]>(self) }
    }
}

impl<'a> Mmu<'a> {
    pub fn new(bus: &'a Bus<'a>, reservation: &'a AtomicU32) -> Self {
        Self {
            reservation,
            d_cache: Box::new(Cache::new()),
            i_cache: Box::new(Cache::new()),
            attr: Box::new(Cache::new()),
            tlb: Box::new(Cache::new()),
            bus,
        }
    }

    pub fn reservation(&self) -> &AtomicU32 {
        self.reservation
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
    fn load_physical<const W: u8>(&mut self, addr: u32) -> MmuResult<u32> {
        assert!(matches!(W, 1 | 2 | 4), "Load width must be 1, 2, or 4");

        if W == 4 && addr & 3 != 0 {
            return Err(MmuError::LoadMisaligned { addr, alignment: 4 });
        } else if W == 2 && addr & 1 != 0 {
            return Err(MmuError::LoadMisaligned { addr, alignment: 2 });
        }

        // fast path, if the value is in cache, it's cacheable
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

        if self.cacheable(addr) {
            // if the address is cacheable, cache it

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
            // check if address supports streamed operations
            todo!()
        }
    }

    #[inline(always)]
    fn load<const W: u8>(&mut self, addr: u32) -> MmuResult<u32> {
        assert!(matches!(W, 1 | 2 | 4), "Load width must be 1, 2, or 4!");

        // TODO Address translation
        // TODO Check user mode
        // TODO Check read permissions

        self.load_physical::<W>(addr)
    }

    #[inline(always)]
    pub fn load_byte(&mut self, addr: u32) -> MmuResult<u32> {
        self.load::<1>(addr)
    }

    #[inline(always)]
    pub fn load_half_word(&mut self, addr: u32) -> MmuResult<u32> {
        self.load::<2>(addr)
    }

    #[inline(always)]
    pub fn load_word(&mut self, addr: u32) -> MmuResult<u32> {
        self.load::<4>(addr)
    }

    #[inline(always)]
    pub fn load_instruction(&mut self, addr: u32) -> MmuResult<Instruction> {
        // TODO Address translation
        // TODO Check user mode
        // TODO Check read permissions

        if addr & 3 != 0 {
            return Err(MmuError::LoadMisaligned { addr, alignment: 4 });
        }

        if let Some(&op) = self.i_cache.get(addr >> 2) {
            return Ok(op);
        }

        let missing = |x: &mut [Instruction; 16]| -> memory::mapping::MemoryResult<()> {
            let mut raw = [0u32; 16];
            let (_, dst, _) = unsafe { raw.align_to_mut::<u8>() };
            match self.bus.block_read(addr & 0xffffffc0, dst) {
                Ok(_) => {}
                Err(e) => return Err(e),
            };

            x.iter_mut()
                .zip(raw.into_iter())
                .for_each(|(d, s)| *d = u32::from_le(s).into());

            Ok(())
        };

        let (&op, _) = self.i_cache.get_or_insert_with(addr >> 2, missing)?;

        Ok(op)
    }

    #[inline(always)]
    fn store_physical<const W: u8>(&mut self, addr: u32, val: u32) -> MmuResult<()> {
        assert!(matches!(W, 1 | 2 | 4), "Load width must be 1, 2, or 4");

        if W == 4 && addr & 3 != 0 {
            return Err(MmuError::LoadMisaligned { addr, alignment: 4 });
        } else if W == 2 && addr & 1 != 0 {
            return Err(MmuError::LoadMisaligned { addr, alignment: 2 });
        }

        // fast path, if it is in cache, it's cacheable
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

        if self.cacheable(addr) {
            // fast path
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
    fn store<const W: u8>(&mut self, addr: u32, val: u32) -> MmuResult<()> {
        assert!(matches!(W, 1 | 2 | 4), "Load width must be 1, 2, or 4");

        if false {
            todo!("Address translation");
        }

        self.store_physical::<W>(addr, val)
    }

    #[inline(always)]
    pub fn store_byte(&mut self, addr: u32, b: u8) -> MmuResult<()> {
        self.store::<1>(addr, b as u32)
    }

    #[inline(always)]
    pub fn store_half_word(&mut self, addr: u32, hw: u16) -> MmuResult<()> {
        self.store::<2>(addr, hw as u32)
    }

    #[inline(always)]
    pub fn store_word(&mut self, addr: u32, w: u32) -> MmuResult<()> {
        self.store::<4>(addr, w)
    }

    #[inline(always)]
    pub fn load_reserved(&mut self, _addr: u32) -> MmuResult<u32> {
        // TODO address translation
        // TODO check physical address attributes about reservability

        let reservation_set = addr_to_reservation_set(_addr);

        // register reservation
        self.reservation.store(reservation_set, Ordering::Relaxed);
        Ok(self.bus.load_word(_addr)?) // load directly from bus
    }

    #[inline(always)]
    pub fn store_conditional(&mut self, _addr: u32, _val: u32) -> MmuResult<u32> {
        let reservation_set = addr_to_reservation_set(_addr);
        if self.reservation.load(Ordering::Relaxed) != addr_to_reservation_set(_addr) {
            Ok(1) // indicates failure
        } else {
            Ok(self
                .bus
                .store_conditional(_addr, _val, self.reservation, reservation_set)?)
        }
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
