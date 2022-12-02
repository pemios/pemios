// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// This Source Code Form is "Incompatible With Secondary Licenses", as
// defined by the Mozilla Public License, v. 2.0.
//
// Copyright © 2022 mumblingdrunkard

use super::types::BlockOffset;

#[derive(Clone, Copy)]
pub struct Block<T, U, const B: usize>
where
    [(); 1 << B]:,
    T: Copy,
    U: Copy,
{
    data: [T; 1 << B],
    tracker: U,
}

impl<T, U, const B: usize> From<[T; 1 << B]> for Block<T, U, B>
where
    T: Copy,
    U: Copy + Default,
    [(); 1 << B]:,
{
    fn from(data: [T; 1 << B]) -> Self {
        Self::with_data(data)
    }
}

impl<T, U, const B: usize> Block<T, U, B>
where
    [(); 1 << B]:,
    T: Copy + Default,
    U: Copy + Default,
{
    pub fn new() -> Self {
        Self {
            tracker: U::default(),
            data: [T::default(); 1 << B],
        }
    }
}

impl<T, U, const B: usize> Block<T, U, B>
where
    [(); 1 << B]:,
    T: Copy,
    U: Copy + Default,
{
    pub fn with_data(data: [T; 1 << B]) -> Self {
        Self {
            tracker: U::default(),
            data,
        }
    }

    #[inline(always)]
    pub fn get(&self, cbo: BlockOffset<B>) -> &T {
        unsafe { self.data.get_unchecked(cbo.raw() as usize) }
    }

    #[inline(always)]
    pub fn get_mut(&mut self, cbo: BlockOffset<B>) -> (&mut T, &mut U) {
        unsafe {
            (
                self.data.get_unchecked_mut(cbo.raw() as usize),
                &mut self.tracker,
            )
        }
    }

    #[inline(always)]
    pub fn internal(&self) -> (&[T; 1 << B], &U) {
        (&self.data, &self.tracker)
    }

    #[inline(always)]
    pub fn internal_mut(&mut self) -> (&mut [T; 1 << B], &mut U) {
        (&mut self.data, &mut self.tracker)
    }
}
