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
    main_memory::MainMemory,
    memory_region::{Mapping, MemoryError, MemoryResult, Properties},
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
    main: MainMemory,
    #[allow(unused)]
    /// map[fnum] should contain (base, mapping).
    /// base is used to calculate the offset used for operations on mapping.
    /// A mapping that provides 4 pages of memory should appear 4 times in this map.
    map: FnvHashMap<u32, (u32, Arc<Box<dyn Mapping>>)>,
}

impl Bus {
    pub fn new(pages: usize) -> Self {
        Self {
            main: MainMemory::new(pages),
            map: HashMap::default(),
        }
    }

    pub fn with_mapping(self, _offset: u32, _region: Arc<Box<dyn Mapping>>) -> Self {
        todo!("Add mapping to map")
    }

    pub fn set_mm(&self, data: &[u32]) -> MemoryResult<usize> {
        let (_, data, _) = unsafe { data.align_to::<u8>() };
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

    fn stream_write(&self, frame: u32, writes: &[(u16, u8, u32)]) -> MemoryResult<usize> {
        if frame & 0x00080000 == 0 {
            self.main.stream_write(frame, writes)
        } else {
            todo!("Stream write to a mapping")
        }
    }

    fn stream_read(
        &self,
        frame: u32,
        reads: &[(u16 /* Offset */, u8 /* width */)],
        dst: &mut [u32],
    ) {
        if frame & 0x00080000 == 0 {
            self.main.stream_read(frame, reads, dst)
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

    fn attributes(&self) -> memory::memory_region::Pma {
        todo!()
    }

    fn properties(&self) -> Properties {
        Properties::new(0xfffff)
    }
}
