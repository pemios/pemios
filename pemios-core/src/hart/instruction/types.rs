// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// This Source Code Form is "Incompatible With Secondary Licenses", as
// defined by the Mozilla Public License, v. 2.0.
//
// Copyright © 2022 mumblingdrunkard

#[derive(Clone, Copy, Debug)]
/// A Conclusion is used to indicate the status of the executed instruction.
pub enum Conclusion {
    /// Conclusion::None indicates nothing special should hoppen
    None,
    /// Conclusion::Jumped indicates that the program counter has been set in the instruction and
    /// we should not manually update it
    Jumped,
    /// Conclusion::Exception indicates an exception occured and we should raise this to the OS
    Exception(u8),
}

#[derive(Clone, Copy, Debug)]
/// Unsigned, 5-bit integer
/// Can be cast to a u32
pub struct UInt5(u8);

impl From<u32> for UInt5 {
    fn from(value: u32) -> Self {
        assert!((0..32).contains(&value), "UInt5 must be in the range 0..32");
        Self(value as u8)
    }
}

impl From<UInt5> for u32 {
    fn from(value: UInt5) -> Self {
        value.0 as u32
    }
}

#[derive(Clone, Copy, Debug)]
/// Signed, 12-bit integer
/// Can be cast to an i32
pub struct Int12(i16);

impl From<i32> for Int12 {
    fn from(val: i32) -> Self {
        debug_assert!(
            (-2048..2048).contains(&val),
            "Int12 must be in the range -2048..2048"
        );
        Self(val as i16)
    }
}

impl From<Int12> for i32 {
    fn from(imm: Int12) -> Self {
        imm.0 as i32
    }
}

#[derive(Clone, Copy, Debug)]
/// Signed, 32-bit integer with the 12 least significant bits set to 0
/// Can be cast to an i32
pub struct Int32Trunc12([u8; 3]);

impl From<i32> for Int32Trunc12 {
    fn from(val: i32) -> Self {
        debug_assert!(
            val & 0xfffff000u32 as i32 == val,
            "Int32Trunc12 must have the 12 lower bits set to 0"
        );
        Self(unsafe {
            (val & 0xfffff000u32 as i32).to_le_bytes()[1..]
                .try_into()
                .unwrap_unchecked()
        })
    }
}

impl From<Int32Trunc12> for i32 {
    fn from(imm: Int32Trunc12) -> Self {
        let mut val = [0; 4];
        val[1..].copy_from_slice(&imm.0[..]);
        i32::from_le_bytes(val)
    }
}

#[derive(Clone, Copy, Debug)]
/// Signed, 21-bit integer with the least significant bit set to 0
/// Can be cast to an i32
pub struct Int21Trunc1([u8; 3]);

impl From<i32> for Int21Trunc1 {
    fn from(val: i32) -> Self {
        assert!((val << 11) >> 11 == val && val & 1 == 0, "");
        Self(
            (val & 0xfffff000u32 as i32).to_le_bytes()[..3]
                .try_into()
                .unwrap(),
        )
    }
}

impl From<Int21Trunc1> for i32 {
    fn from(imm: Int21Trunc1) -> Self {
        let mut val = [0; 4];
        val[1..].copy_from_slice(&imm.0[..]);
        i32::from_le_bytes(val) >> 11
    }
}

#[derive(Clone, Copy, Debug)]
/// Signed, 13-bit integer with the least significant bit set to 0
/// Can be cast to an i32
pub struct Int13Trunc1(i16);

impl From<i32> for Int13Trunc1 {
    fn from(val: i32) -> Self {
        assert!((val << 19) >> 19 == val && val & 1 == 0, "");
        Self(val as i16)
    }
}

impl From<Int13Trunc1> for i32 {
    fn from(imm: Int13Trunc1) -> Self {
        imm.0 as i32
    }
}

#[derive(Clone, Copy, Debug)]
pub struct FenceSet(u8);

impl FenceSet {
    pub fn new(set: u8) -> Self {
        assert!(
            set & 0xf0 == 0,
            "set should not have any of the 4 upper bits set"
        );

        Self(set)
    }

    pub fn input(&self) -> bool {
        self.0 & 8 != 0
    }

    pub fn output(&self) -> bool {
        self.0 & 4 != 0
    }

    pub fn read(&self) -> bool {
        self.0 & 2 != 0
    }

    pub fn write(&self) -> bool {
        self.0 & 1 != 0
    }
}

#[derive(Clone, Copy, Debug)]
/// The fence mode.
/// Fence will never raise an exception meaning we can store this in a lossy format
pub enum FenceMode {
    None,
    Tso,
    Other,
}

impl FenceMode {
    pub fn new(mode: u8) -> Self {
        match mode {
            0 => Self::None,
            8 => Self::Tso,
            _ => Self::Other,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum OpCode {
    Load,
    MiscMem,
    OpImm,
    Auipc,
    Store,
    Amo,
    Op,
    Lui,
    Branch,
    Jalr,
    Jal,
    System,
    Invalid,
}

impl From<u32> for OpCode {
    fn from(op: u32) -> Self {
        use OpCode::*;
        match op {
            0b0000011 => Load,
            0b0001111 => MiscMem,
            0b0010011 => OpImm,
            0b0010111 => Auipc,
            0b0100011 => Store,
            0b0101111 => Amo,
            0b0110011 => Op,
            0b0110111 => Lui,
            0b1100011 => Branch,
            0b1100111 => Jalr,
            0b1101111 => Jal,
            0b1110011 => System,
            _ => Invalid,
        }
    }
}
