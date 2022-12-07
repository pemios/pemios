// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// This Source Code Form is "Incompatible With Secondary Licenses", as
// defined by the Mozilla Public License, v. 2.0.
//
// Copyright © 2022 mumblingdrunkard

use std::{collections::HashMap, sync::atomic::AtomicU32};

use fnv::{FnvHashMap, FnvHashSet};

use crate::memory::{
    self,
    main::Main,
    mapping::{Mapping, MemoryError, MemoryResult, Properties, Reservability, SendSyncMapping},
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

pub struct Builder<'a> {
    main: Option<Main<'a>>,
    map: FnvHashMap<u32, (u32, &'a dyn SendSyncMapping<'a>)>,
}

impl<'a> Builder<'a> {
    pub fn with_mapping(mut self, mapping: &'a dyn SendSyncMapping<'a>) -> Self {
        let props = mapping.properties();
        for i in 0..props.frame_count() {
            if self.map.contains_key(&(props.base_frame() + i as u32)) {
                panic!("Tried to build bus with overlapping mappings!");
            }

            self.map
                .insert(props.base_frame() + i as u32, (props.base_frame(), mapping));
        }

        self
    }

    pub fn with_main_memory(mut self, frame_count: u32) -> Self {
        if self.main.is_some() {
            panic!("Tried to build bus with main memory twice!");
        }

        self.main.replace(Main::new(0, frame_count));

        self
    }

    pub fn build(self) -> Bus<'a> {
        if self.main.is_none() {
            panic!("Tried to build bus without main memory!")
        }

        Bus {
            main: self.main.unwrap(),
            map: self.map,
        }
    }
}

pub struct Bus<'a> {
    /// The main memory segment starts at address 0 and has some size.
    main: Main<'a>,

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
    /// at different frame numbers may have different implementations.
    /// We also require that these mappings are safe to interact with across
    /// threads, hence the &'a dyn SendSyncMapping.
    map: FnvHashMap<u32, (u32, &'a dyn SendSyncMapping<'a>)>,
}

impl<'a> Bus<'a> {
    pub fn builder() -> Builder<'a> {
        Builder {
            main: None,
            map: HashMap::default(),
        }
    }

    pub fn main_memory_size(&self) -> u32 {
        self.main.properties().frame_count() * 4096
    }

    pub fn set_mm(&self, data: &[u8]) -> MemoryResult<usize> {
        self.main.block_write(0, data)
    }
}

impl<'a> Mapping<'a> for Bus<'a> {
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

    fn store_half_word(&self, _offset: u32, _jalf_word: u16) -> MemoryResult<()> {
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

    fn store_conditional(
        &self,
        _offset: u32,
        _src: u32,
        _reservation: &AtomicU32,
        _should_be: u32,
    ) -> MemoryResult<u32> {
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
        Properties::new(0, 0xfffff)
    }

    fn register_reservation_set(&'a self, set: &'a AtomicU32) {
        self.main.register_reservation_set(set);
        let mut seen = FnvHashSet::default();
        for (_frame_number, (base, mapping)) in self.map.iter() {
            if seen.contains(base) {
                // this is an alias and we should not add the callback multiple times
                continue;
            }

            seen.insert(*base);

            if mapping.attributes().reservability() == Reservability::None {
                continue;
            }

            mapping.register_reservation_set(set);
        }
    }
}
