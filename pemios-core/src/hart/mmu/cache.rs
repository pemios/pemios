// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// This Source Code Form is "Incompatible With Secondary Licenses", as
// defined by the Mozilla Public License, v. 2.0.
//
// Copyright © 2022 mumblingdrunkard

use self::{
    block::Block,
    set::Set,
    types::{Addr, SetIndex, TagSet},
};

mod block;
mod set;
mod types;

pub struct Cache<T, U, const S: usize, const A: usize, const B: usize>
where
    [(); 1 << B]:,
    [(); 1 << S]:,
    T: Copy,
    U: Copy + Default,
{
    sets: [Set<T, U, S, A, B>; 1 << S],
}

impl<T, U, const S: usize, const A: usize, const B: usize> Cache<T, U, S, A, B>
where
    [(); 1 << B]:,
    [(); 1 << S]:,
    T: Copy + Default,
    U: Copy + Default,
{
    pub fn new() -> Self {
        Self {
            sets: [Set::<T, U, S, A, B>::new(); 1 << S],
        }
    }

    #[inline(always)]
    pub fn get(&self, addr: u32) -> Option<&T> {
        let addr = Self::addr_from_u32(addr);
        self.get_block(addr.tag_set()).map(|b| b.get(addr.offset()))
    }

    #[inline(always)]
    pub fn get_mut(&mut self, addr: u32) -> Option<(&mut T, &mut U)> {
        let addr = Self::addr_from_u32(addr);
        self.get_block_mut(addr.tag_set())
            .map(|b| b.get_mut(addr.offset()))
    }

    #[inline(always)]
    pub fn get_or_insert_with<F, O, E>(
        &mut self,
        addr: u32,
        f: F,
    ) -> Result<(&T, Option<(u32, [T; 1 << B], U)>), E>
    where
        F: Fn(&mut [T; 1 << B]) -> Result<O, E>,
    {
        let addr = Self::addr_from_u32(addr);

        let (block, victim) = self
            .get_set_mut(addr.set())
            .get_block_or_insert_with(addr.tag(), f)?;

        Ok((
            block.get(addr.offset()),
            victim.map(|(tag, block)| {
                let set_offset = (addr.raw() << (32 - S - B)) >> (32 - S - B);
                let tag = (tag.raw() >> (S + B)) << (S + B);
                let block_addr = tag | set_offset;

                let (data, tracker) = block.internal();

                (block_addr, *data, *tracker)
            }),
        ))
    }

    #[inline(always)]
    pub fn get_mut_or_insert_with<F, O, E>(
        &mut self,
        addr: u32,
        f: F,
    ) -> Result<((&mut T, &mut U), Option<(u32, [T; 1 << B], U)>), E>
    where
        F: Fn(&mut [T; 1 << B]) -> Result<O, E>,
    {
        let addr = Self::addr_from_u32(addr);
        let (block, victim) = self
            .get_set_mut(addr.set())
            .get_block_mut_or_insert_with(addr.tag(), f)?;

        Ok((
            block.get_mut(addr.offset()),
            victim.map(|(tag, block)| {
                let set_offset = (addr.raw() << (32 - S - B)) >> (32 - S - B);
                let tag = (tag.raw() >> (S + B)) << (S + B);
                let block_addr = tag | set_offset;

                let (data, tracker) = block.internal();

                (block_addr, *data, *tracker)
            }),
        ))
    }

    #[inline(always)]
    fn get_set(&self, csi: SetIndex<S, B>) -> &Set<T, U, S, A, B> {
        unsafe { self.sets.get_unchecked(csi.raw() as usize) }
    }

    #[inline(always)]
    fn get_set_mut(&mut self, csi: SetIndex<S, B>) -> &mut Set<T, U, S, A, B> {
        unsafe { self.sets.get_unchecked_mut(csi.raw() as usize) }
    }

    #[inline(always)]
    fn get_block(&self, cts: TagSet<S, B>) -> Option<&Block<T, U, B>> {
        self.get_set(cts.set()).get_block(cts.tag())
    }

    #[inline(always)]
    fn get_block_mut(&mut self, cts: TagSet<S, B>) -> Option<&mut Block<T, U, B>> {
        self.get_set_mut(cts.set()).get_block_mut(cts.tag())
    }

    #[inline(always)]
    fn addr_from_u32(addr: u32) -> Addr<S, B> {
        addr.into()
    }

    #[allow(unused)]
    #[inline(always)]
    pub fn insert(&mut self, addr: u32, block: [T; 1 << B]) -> Option<(u32, [T; 1 << B], U)> {
        let addr = Self::addr_from_u32(addr);
        if let Some((tag, block)) = self
            .get_set_mut(addr.set())
            .insert(addr.tag(), block.into())
            .1
        {
            let set_offset = (addr.raw() << (32 - S - B)) >> (32 - S - B);
            let tag = (tag.raw() >> (S + B)) << (S + B);
            let block_addr = tag | set_offset;

            let (data, tracker) = block.internal();

            Some((block_addr, *data, *tracker))
        } else {
            None
        }
    }

    #[allow(unused)]
    #[inline(always)]
    pub fn insert_with<F, O, E>(
        &mut self,
        addr: u32,
        f: F,
    ) -> Result<Option<(u32, [T; 1 << B], U)>, E>
    where
        F: Fn(&mut [T; 1 << B]) -> Result<O, E>,
    {
        let addr = Self::addr_from_u32(addr);
        if let Some((tag, block)) = self.get_set_mut(addr.set()).insert_with(addr.tag(), f)?.1 {
            let set_offset = (addr.raw() << (32 - S - B)) >> (32 - S - B);
            let tag = (tag.raw() >> (S + B)) << (S + B);
            let block_addr = tag | set_offset;
            let (data, tracker) = block.internal();

            Ok(Some((block_addr, *data, *tracker)))
        } else {
            Ok(None)
        }
    }
}
