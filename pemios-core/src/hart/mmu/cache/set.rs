// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// This Source Code Form is "Incompatible With Secondary Licenses", as
// defined by the Mozilla Public License, v. 2.0.
//
// Copyright © 2022 mumblingdrunkard

use super::{block::Block, types::Tag};

#[derive(Clone, Copy)]
pub struct Set<T, U, const S: usize, const A: usize, const B: usize>
where
    [(); 1 << B]:,
    T: Copy,
    U: Copy + Default,
{
    blocks: [Block<T, U, B>; A],
    tags: [Tag<S, B>; A],
    dirty: [bool; A],
    victim: usize,
}

impl<T, U, const S: usize, const A: usize, const B: usize> Set<T, U, S, A, B>
where
    [(); 1 << B]:,
    T: Copy + Default,
    U: Copy + Default,
{
    pub fn new() -> Self {
        Self {
            blocks: [Block::new(); A],
            tags: [Tag::INV; A],
            dirty: [false; A],
            victim: 0,
        }
    }

    #[inline(always)]
    pub fn get_block(&self, tag: Tag<S, B>) -> Option<&Block<T, U, B>> {
        self.tags
            .iter()
            .position(|&t| t == tag)
            .and_then(|i| self.blocks.get(i))
    }

    #[inline(always)]
    pub fn get_block_mut(&mut self, tag: Tag<S, B>) -> Option<&mut Block<T, U, B>> {
        self.tags.iter().position(|&t| t == tag).and_then(|i| {
            self.dirty[i] = true;
            self.blocks.get_mut(i)
        })
    }

    #[inline(always)]
    pub fn get_block_or_insert_with<F, O, E>(
        &mut self,
        tag: Tag<S, B>,
        f: F,
    ) -> Result<(&Block<T, U, B>, Option<(Tag<S, B>, Block<T, U, B>)>), E>
    where
        F: Fn(&mut [T; 1 << B]) -> Result<O, E>,
    {
        if let Some(i) = self.tags.iter().position(|&t| t == tag) {
            Ok((&self.blocks[i], None))
        } else {
            let (inserted, victim) = self.insert_with(tag, f)?;
            Ok((inserted, victim))
        }
    }

    #[inline(always)]
    pub fn get_block_mut_or_insert_with<F, O, E>(
        &mut self,
        tag: Tag<S, B>,
        f: F,
    ) -> Result<(&mut Block<T, U, B>, Option<(Tag<S, B>, Block<T, U, B>)>), E>
    where
        F: Fn(&mut [T; 1 << B]) -> Result<O, E>,
    {
        if let Some(i) = self.tags.iter().position(|&t| t == tag) {
            return Ok((&mut self.blocks[i], None));
        } else {
            let (inserted, victim) = self.insert_with(tag, f)?;
            Ok((inserted, victim))
        }
    }

    #[allow(unused)]
    #[inline(always)]
    pub fn insert(
        &mut self,
        tag: Tag<S, B>,
        block: Block<T, U, B>,
    ) -> (&mut Block<T, U, B>, Option<(Tag<S, B>, Block<T, U, B>)>) {
        // search for empty slot
        let idx = self
            .tags
            .iter()
            .position(|&t| t.is_invalid())
            // or select victim and increment
            .unwrap_or_else(|| {
                let res = self.victim;
                self.victim += 1;
                self.victim %= A;
                res
            });

        let victim_tag = self.tags[idx];
        let victim_block = self.blocks[idx];
        self.tags[idx] = tag;
        self.blocks[idx] = block;

        (
            &mut self.blocks[idx],
            (victim_tag.is_valid() && self.dirty[idx]).then_some((victim_tag, victim_block)),
        )
    }

    #[inline(always)]
    pub fn insert_with<F, O, E>(
        &mut self,
        tag: Tag<S, B>,
        f: F,
    ) -> Result<(&mut Block<T, U, B>, Option<(Tag<S, B>, Block<T, U, B>)>), E>
    where
        F: Fn(&mut [T; 1 << B]) -> Result<O, E>,
    {
        let idx = self
            .tags
            .iter()
            .position(|&t| t.is_invalid() || t == tag)
            // or select victim and increment
            .unwrap_or_else(|| {
                let res = self.victim;
                self.victim += 1;
                self.victim %= A;
                res
            });

        let victim_tag = self.tags[idx];
        let victim_block = self.blocks[idx];

        self.tags[idx] = tag;
        f(&mut self.blocks[idx].internal_mut().0)?;

        Ok((
            &mut self.blocks[idx],
            (victim_tag.is_valid() && self.dirty[idx]).then_some((victim_tag, victim_block)),
        ))
    }
}
