// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// This Source Code Form is "Incompatible With Secondary Licenses", as
// defined by the Mozilla Public License, v. 2.0.
//
// Copyright © 2022 mumblingdrunkard

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct BlockOffset<const B: usize>(u32);

impl<const B: usize> BlockOffset<B> {
    pub const fn raw(&self) -> u32 {
        self.0
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Tag<const S: usize, const B: usize>(u32);

impl<const S: usize, const B: usize> Tag<S, B> {
    pub const INV: Self = Self(u32::MAX);

    pub fn is_invalid(&self) -> bool {
        *self == Self::INV
    }

    pub fn is_valid(&self) -> bool {
        *self != Self::INV
    }

    pub const fn raw(&self) -> u32 {
        self.0
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct TagSet<const S: usize, const B: usize>(u32);

impl<const S: usize, const B: usize> TagSet<S, B> {
    pub const fn tag(&self) -> Tag<S, B> {
        Tag(self.0 >> (S + B))
    }

    pub const fn set(&self) -> SetIndex<S, B> {
        SetIndex((self.0 << (32 - S - B)) >> (32 - S))
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct SetIndex<const S: usize, const B: usize>(u32);

impl<const S: usize, const B: usize> From<SetIndex<S, B>> for usize {
    fn from(csi: SetIndex<S, B>) -> Self {
        csi.0 as usize
    }
}

impl<const S: usize, const B: usize> SetIndex<S, B> {
    pub const fn raw(&self) -> u32 {
        self.0
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Addr<const S: usize, const B: usize>(u32);

impl<const S: usize, const B: usize> Addr<S, B> {
    pub const fn raw(&self) -> u32 {
        self.0
    }

    pub const fn tag(&self) -> Tag<S, B> {
        Tag(self.0 >> (S + B))
    }

    pub const fn set(&self) -> SetIndex<S, B> {
        SetIndex((self.0 << (32 - S - B)) >> (32 - S))
    }

    pub const fn offset(&self) -> BlockOffset<B> {
        BlockOffset((self.0 << (32 - B)) >> (32 - B))
    }

    pub const fn tag_set(&self) -> TagSet<S, B> {
        TagSet(self.0)
    }
}

impl<const S: usize, const B: usize> From<u32> for Addr<S, B> {
    fn from(addr: u32) -> Self {
        Self(addr)
    }
}
