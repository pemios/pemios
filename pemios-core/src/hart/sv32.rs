// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// This Source Code Form is "Incompatible With Secondary Licenses", as
// defined by the Mozilla Public License, v. 2.0.
//
// Copyright © 2022 mumblingdrunkard

#[allow(unused)]
pub struct VirtualAddress(u32);

#[allow(unused)]
impl VirtualAddress {
    #[inline]
    pub fn raw(&self) -> u32 {
        self.0
    }

    #[inline]
    pub fn vpn1(&self) -> u32 {
        self.0 >> 22
    }

    #[inline]
    pub fn vpn0(&self) -> u32 {
        (self.0 >> 12) & 0x3ff
    }

    #[inline]
    pub fn offset(&self) -> u32 {
        self.0 & 0xfff
    }
}

#[allow(unused)]
pub struct PhysicalAddress(u32);

#[allow(unused)]
impl PhysicalAddress {
    pub fn raw(&self) -> u32 {
        self.0
    }

    pub fn ppn1(&self) -> u32 {
        self.0 >> 22
    }

    pub fn ppn0(&self) -> u32 {
        (self.0 >> 12) & 0x3ff
    }

    pub fn offset(&self) -> u32 {
        self.0 & 0xfff
    }
}

#[allow(unused)]
#[derive(Copy, Clone, Default)]
pub struct Pte(u32);

#[allow(unused)]
#[repr(u8)]
pub enum PteRsw {
    Rsw0,
    Rsw1,
    Rsw2,
    Rsw3,
}

pub enum PteKind {
    Pointer,
    Read,
    ReadWrite,
    Execute,
    ReadExecute,
    ReadWriteExecute,
    Reserved,
}

#[allow(unused)]
impl Pte {
    pub fn raw(&self) -> u32 {
        self.0
    }

    pub fn ppn1(&self) -> u32 {
        self.0 >> 20
    }

    pub fn ppn0(&self) -> u32 {
        (self.0 >> 10) & 0x3ff
    }

    pub fn base(&self) -> u32 {
        (self.0 << 2) & 0xfffff000
    }

    pub fn rsw(&self) -> PteRsw {
        use PteRsw::*;
        match (self.0 >> 8) & 3 {
            0 => Rsw0,
            1 => Rsw1,
            2 => Rsw2,
            3 => Rsw3,
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }

    pub fn dirty(&self) -> bool {
        self.0 & 0b10000000 != 0
    }

    pub fn accessed(&self) -> bool {
        self.0 & 0b01000000 != 0
    }

    pub fn global(&self) -> bool {
        self.0 & 0b00100000 != 0
    }

    pub fn user(&self) -> bool {
        self.0 & 0b00010000 != 0
    }

    pub fn executable(&self) -> bool {
        self.0 & 0b00001000 != 0
    }

    pub fn writable(&self) -> bool {
        self.0 & 0b00000100 != 0
    }

    pub fn readable(&self) -> bool {
        self.0 & 0b00000010 != 0
    }

    pub fn valid(&self) -> bool {
        self.0 & 0b00000001 != 0
    }

    pub fn kind(&self) -> PteKind {
        match (self.executable(), self.writable(), self.readable()) {
            (false, false, false) => PteKind::Pointer,
            (false, false, true) => PteKind::Read,
            (false, true, false) => PteKind::Reserved,
            (false, true, true) => PteKind::ReadWrite,
            (true, false, false) => PteKind::Execute,
            (true, false, true) => PteKind::ReadExecute,
            (true, true, false) => PteKind::Reserved,
            (true, true, true) => PteKind::ReadWriteExecute,
        }
    }
}
