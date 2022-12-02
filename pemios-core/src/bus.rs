// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// This Source Code Form is "Incompatible With Secondary Licenses", as
// defined by the Mozilla Public License, v. 2.0.
//
// Copyright © 2022 mumblingdrunkard

use std::{collections::HashMap, sync::Arc};

use fnv::FnvHashMap;

use crate::memory::{
    self,
    main::Main,
    mapping::{Mapping, MemoryError, MemoryResult, Properties},
};

#[derive(Debug)]
pub enum BusError {
    MemoryError { e: MemoryError },
}

impl From<MemoryError> for BusError {
    fn from(e: MemoryError) -> Self {
        Self::MemoryError { e }
    }
}

pub struct Bus {
    /// The main memory segment starts at address 0 and has some size.
    main: Main,

    #[allow(unused)]
    /// map[fnum] should contain (base_frame_number, mapping).
    /// base_frame_number is used to calculate the offset used for operations on
    /// the mapping.
    ///
    /// A mapping may appear multiple times in this map if it has a size larger
    /// than a single frame (4096 bytes).
    /// E.g. when adding a mapping that reports a frame count of 4, at frame
    /// number 2, the result should be that map[2], map[3], map[4], and map[5]
    /// all return the same thing.
    ///
    /// It is possible for a device to provide several mappings that may all be
    /// mapped at different locations in memory.
    /// E.g. control registers could be mapped at some location, while buffers
    /// may be located at a very different offset.
    ///
    /// This map does not have exclusive ownership of the mapping, and mappings
    /// at different frame numbers may have different implementations, hence the
    /// Arc<Box<dyn ...>>
    map: FnvHashMap<u32, (u32, Arc<Box<dyn Mapping>>)>,
}

impl Bus {
    pub fn new(pages: usize) -> Self {
        Self {
            main: Main::new(pages),
            map: HashMap::default(),
        }
    }

    pub fn with_mapping(self, _starting_frame_number: u32, _region: Arc<Box<dyn Mapping>>) -> Self {
        todo!("Add mapping to map")
    }

    pub fn set_mm(&self, data: &[u8]) -> MemoryResult<usize> {
        self.main.block_write(0, data)
    }
}

impl Mapping for Bus {
    fn block_write(&self, offset: u32, src: &[u8]) -> MemoryResult<usize> {
        if offset & 0x80000000 == 0 {
            self.main.block_write(offset, src)
        } else {
            todo!("Block write to a mapping")
        }
    }

    fn block_write_masked(&self, _offset: u32, _src: &[u8], _mask: &[u8]) -> MemoryResult<usize> {
        todo!()
    }

    fn block_read(&self, offset: u32, dst: &mut [u8]) -> MemoryResult<usize> {
        if offset & 0x80000000 == 0 {
            self.main.block_read(offset, dst)
        } else {
            todo!("Block read from a mapping")
        }
    }

    fn block_read_masked(
        &self,
        _offset: u32,
        _dst: &mut [u8],
        _mask: &[u8],
    ) -> MemoryResult<usize> {
        todo!()
    }

    fn stream_write(&self, frame_number: u32, writes: &[(u16, u8, u32)]) -> MemoryResult<usize> {
        if frame_number & 0x00080000 == 0 {
            self.main.stream_write(frame_number, writes)
        } else {
            todo!("Stream write to a mapping")
        }
    }

    fn stream_read(
        &self,
        frame_number: u32,
        reads: &[(u16 /* offset */, u8 /* width */)],
        dst: &mut [u32],
    ) -> MemoryResult<usize> {
        if frame_number & 0x00080000 == 0 {
            self.main.stream_read(frame_number, reads, dst)
        } else {
            todo!("Stream read from a mapping")
        }
    }

    fn store_byte(&self, _offset: u32, _byte: u8) -> MemoryResult<()> {
        todo!()
    }

    fn store_half_word(&self, _offset: u32, _half_word: u16) -> MemoryResult<()> {
        todo!()
    }

    fn store_word(&self, _offset: u32, _word: u32) -> MemoryResult<()> {
        todo!()
    }

    fn load_byte(&self, _offset: u32) -> MemoryResult<u8> {
        todo!()
    }

    fn load_half_word(&self, _offset: u32) -> MemoryResult<u16> {
        todo!()
    }

    fn load_word(&self, _offset: u32) -> MemoryResult<u32> {
        todo!()
    }

    fn load_reserved(&self, _offset: u32, _src: u32) -> MemoryResult<u32> {
        todo!()
    }

    fn store_conditional(&self, _offset: u32, _src: u32) -> MemoryResult<u32> {
        todo!()
    }

    fn amoswap_w(&self, _offset: u32, _src: u32) -> MemoryResult<u32> {
        todo!()
    }

    fn amoadd_w(&self, _offset: u32, _src: u32) -> MemoryResult<u32> {
        todo!()
    }

    fn amoand_w(&self, _offset: u32, _src: u32) -> MemoryResult<u32> {
        todo!()
    }

    fn amoor_w(&self, _offset: u32, _src: u32) -> MemoryResult<u32> {
        todo!()
    }

    fn amoxor_w(&self, _offset: u32, _src: u32) -> MemoryResult<u32> {
        todo!()
    }

    fn amomax_w(&self, _offset: u32, _src: u32) -> MemoryResult<u32> {
        todo!()
    }

    fn amomaxu_w(&self, _offset: u32, _src: u32) -> MemoryResult<u32> {
        todo!()
    }

    fn amomin_w(&self, _offset: u32, _src: u32) -> MemoryResult<u32> {
        todo!()
    }

    fn amominu_w(&self, _offset: u32, _src: u32) -> MemoryResult<u32> {
        todo!()
    }

    fn attributes(&self) -> memory::mapping::Pma {
        todo!()
    }

    fn properties(&self) -> Properties {
        Properties::new(0xfffff)
    }

    fn register_store_callback(&self, _f: Box<dyn Fn(u32)>) {
        todo!()
    }
}
