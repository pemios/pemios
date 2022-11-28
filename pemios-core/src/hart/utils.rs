// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// This Source Code Form is "Incompatible With Secondary Licenses", as
// defined by the Mozilla Public License, v. 2.0.
//
// Copyright © 2022 mumblingdrunkard

use std::ops::RangeInclusive;

pub trait SignExtend<T> {
    fn sign_extend(&self, s: usize) -> T;
}

impl SignExtend<u32> for u32 {
    fn sign_extend(&self, s: usize) -> u32 {
        (((self << (32 - s - 1)) as i32) >> (32 - s - 1)) as u32
    }
}

pub struct BitRange(u32, usize);

impl BitRange {
    #[inline]
    pub fn cat(&self, next: BitRange) -> BitRange {
        Self(self.0 << next.1 | next.0, self.1 + next.1)
    }

    pub fn new(n: u32, width: usize) -> Self {
        Self((n << (32 - width)) >> (32 - width), width)
    }

    #[inline]
    pub fn get(&self) -> u32 {
        self.0
    }
}

pub trait Bits {
    fn bits(&self, r: RangeInclusive<usize>) -> BitRange;
}

pub trait Bit {
    fn bit(&self, i: usize) -> BitRange;
}

impl Bits for u32 {
    #[inline]
    fn bits(&self, r: RangeInclusive<usize>) -> BitRange {
        BitRange::new(
            (self >> r.start()) & (u32::MAX >> (32 - (r.end() - r.start() + 1))),
            r.end() - r.start() + 1,
        )
    }
}

impl Bit for u32 {
    #[inline]
    fn bit(&self, i: usize) -> BitRange {
        BitRange::new((self >> i) & 1, 1)
    }
}
