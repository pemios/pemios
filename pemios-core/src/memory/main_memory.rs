// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// This Source Code Form is "Incompatible With Secondary Licenses", as
// defined by the Mozilla Public License, v. 2.0.
//
// Copyright © 2022 mumblingdrunkard

use std::sync::Mutex;

use super::memory_region::{Mapping, MemoryError, MemoryResult, Pma, Properties};

type Frame = [u32; 1024];

pub struct MainMemory {
    frames: Vec<Mutex<Frame>>,
}

impl Mapping for MainMemory {
    fn block_write(&self, offset: u32, src: &[u8]) -> MemoryResult<usize> {
        (offset & 3 == 0)
            .then_some(())
            .ok_or(MemoryError::StoreMisaligned {
                offset,
                alignment: 4,
            })?;

        let start = offset as usize >> 12;
        let end = (offset as usize + src.len() - 1) >> 12;

        if end >= self.frames.len() {
            return Err(MemoryError::OutOfBoundsAccess { offset });
        }

        let mut frame_offs = offset as usize & 0xfff; // frame offset
        let mut src_offs = 0; // data offset

        for pfn in start..=end {
            self.frames.get(pfn).and_then(|m| {
                m.lock()
                    .and_then(|mut g| {
                        let (_, dst, _) = unsafe { g.align_to_mut::<u8>() };
                        let n = std::cmp::min(dst.len() - frame_offs, src.len() - src_offs);
                        dst[frame_offs..frame_offs + n]
                            .clone_from_slice(&src[src_offs..src_offs + n]);
                        src_offs += n;
                        frame_offs = 0;

                        Ok(())
                    })
                    .expect(
                        "Tried to acquire frame, but .lock() returned an error.\
Did a thread exit unexpectedly while holding this Mutex?",
                    );
                Some(())
            });
        }

        assert_eq!(src_offs, src.len(), "Failed to store all elements from src");

        Ok(src_offs)
    }

    fn block_write_masked(&self, offset: u32, src: &[u8], _mask: &[u8]) -> MemoryResult<usize> {
        if _mask.len() * 8 < src.len() {
            panic!("Mask must contain enough bits to mask src!");
        }

        (offset & 3 == 0)
            .then_some(())
            .ok_or(MemoryError::StoreMisaligned {
                offset,
                alignment: 4,
            })?;

        let start = offset as usize >> 12;
        let end = (offset as usize + src.len() - 1) >> 12;

        if end >= self.frames.len() {
            return Err(MemoryError::OutOfBoundsAccess { offset });
        }

        let mut frame_offs = offset as usize & 0xfff; // frame offset
        let mut src_offs = 0; // data offset
        let mut written = 0;

        for pfn in start..=end {
            self.frames.get(pfn).and_then(|m| {
                m.lock()
                    .and_then(|mut g| {
                        let (_, dst, _) = unsafe { g.align_to_mut::<u8>() };
                        let n = std::cmp::min(dst.len() - frame_offs, src.len() - src_offs);
                        for i in 0..n {
                            let mask_index = src_offs + i;
                            let mask_byte = mask_index >> 3;
                            let mask_bit = mask_index & 7;
                            // Only copies the byte if the mask contains a 1 at the corresponding position
                            if (unsafe { _mask.get_unchecked(mask_byte) } >> mask_bit) & 1 == 1 {
                                written += 1;
                                dst[frame_offs + i] = src[src_offs + i];
                            }
                        }
                        src_offs += n;
                        frame_offs = 0;

                        Ok(())
                    })
                    .expect(
                        "Tried to acquire frame, but .lock() returned an error.\
Did a thread exit unexpectedly while holding this Mutex?",
                    );
                Some(())
            });
        }

        // assert_eq!(src_offs, src.len(), "Failed to store all elements from src");

        Ok(written)
    }

    fn block_read(&self, offset: u32, dst: &mut [u8]) -> Result<usize, MemoryError> {
        (offset & 3 == 0)
            .then_some(())
            .ok_or(MemoryError::LoadMisaligned {
                offset,
                alignment: 2,
            })?;

        let start = offset as usize >> 12;
        let end = (offset as usize + dst.len() - 1) >> 12;

        if end >= self.frames.len() {
            return Err(MemoryError::OutOfBoundsAccess { offset });
        }

        let mut frame_offs = offset as usize & 0xfff; // frame offset
        let mut dst_offs = 0; // data offset

        for pfn in start..=end {
            self.frames.get(pfn).and_then(|m| {
                m.lock()
                    .and_then(|g| {
                        // calculate number of elements
                        let (_, src, _) = unsafe { g.align_to::<u8>() };
                        let n = std::cmp::min(src.len() - frame_offs, dst.len() - dst_offs);

                        // clone into dst
                        dst[dst_offs..dst_offs + n]
                            .clone_from_slice(&src[frame_offs..frame_offs + n]);

                        // next loop
                        dst_offs += n;
                        frame_offs = 0;

                        Ok(())
                    })
                    .expect(
                        "Tried to acquire frame, but .lock() returned an error.\
Did a thread exit unexpectedly while holding this Mutex?",
                    );
                Some(())
            });
        }

        assert_eq!(
            dst_offs,
            dst.len(),
            "Failed to read enough elements to fill dst"
        );

        Ok(dst_offs)
    }

    fn block_read_masked(
        &self,
        _offset: u32,
        _dst: &mut [u8],
        _mask: &[u8],
    ) -> MemoryResult<usize> {
        if _mask.len() * 8 < _dst.len() {
            panic!("Mask must contain enough bits to mask src!");
        }
        todo!()
    }

    fn stream_write(&self, _offset: u32, _writes: &[(u16, u8, u32)]) -> MemoryResult<usize> {
        todo!()
    }

    fn stream_read(&self, _offset: u32, _reads: &[(u16, u8)], _dst: &mut [u32]) {
        todo!()
    }

    fn store_byte(&self, offset: u32, byte: u8) -> Result<(), MemoryError> {
        let pfn = offset as usize >> 12;
        let b = offset as usize & 0xfff;

        if pfn >= self.frames.len() {
            return Err(MemoryError::OutOfBoundsAccess { offset });
        }

        self.frames
            .get(pfn)
            .and_then(|m| {
                m.lock()
                    .and_then(|mut g| {
                        let (_, bytes, _) = unsafe { g.align_to_mut::<u8>() };
                        unsafe { *bytes.get_unchecked_mut(b) = byte };
                        Ok(())
                    })
                    .expect(
                        "Tried to acquire frame, but .lock() returned an error.\
Did a thread exit unexpectedly while holding this Mutex?",
                    );
                Some(())
            })
            .ok_or(MemoryError::OutOfBoundsAccess { offset })
    }

    fn store_half_word(&self, offset: u32, half_word: u16) -> Result<(), MemoryError> {
        (offset & 1 == 0)
            .then_some(())
            .ok_or(MemoryError::StoreMisaligned {
                offset,
                alignment: 2,
            })?;

        let pfn = offset as usize >> 12;
        let hw = (offset as usize & 0xfff) >> 1;

        if pfn >= self.frames.len() {
            return Err(MemoryError::OutOfBoundsAccess { offset });
        }

        self.frames
            .get(pfn)
            .and_then(|m| {
                m.lock()
                    .and_then(|mut g| {
                        let (_, half_words, _) = unsafe { g.align_to_mut::<u16>() };
                        unsafe { *half_words.get_unchecked_mut(hw) = half_word.to_le() };
                        Ok(())
                    })
                    .expect(
                        "Tried to acquire frame, but .lock() returned an error.\
Did a thread exit unexpectedly while holding this Mutex?",
                    );
                Some(())
            })
            .ok_or(MemoryError::OutOfBoundsAccess { offset })
    }

    fn store_word(&self, offset: u32, word: u32) -> Result<(), MemoryError> {
        (offset & 3 == 0)
            .then_some(())
            .ok_or(MemoryError::StoreMisaligned {
                offset,
                alignment: 4,
            })?;

        let pfn = offset as usize >> 12;
        let w = (offset as usize & 0xfff) >> 2;

        if pfn >= self.frames.len() {
            return Err(MemoryError::OutOfBoundsAccess { offset });
        }

        self.frames
            .get(pfn)
            .and_then(|m| {
                m.lock()
                    .and_then(|mut g| {
                        unsafe { *g.get_unchecked_mut(w) = word.to_le() };
                        Ok(())
                    })
                    .expect(
                        "Tried to acquire frame, but .lock() returned an error.\
Did a thread exit unexpectedly while holding this Mutex?",
                    );
                Some(())
            })
            .ok_or(MemoryError::OutOfBoundsAccess { offset })
    }

    fn load_byte(&self, offset: u32) -> Result<u8, MemoryError> {
        let pfn = offset as usize >> 12;
        let b = offset as usize & 0xfff;

        if pfn >= self.frames.len() {
            return Err(MemoryError::OutOfBoundsAccess { offset });
        }

        self.frames
            .get(pfn)
            .and_then(|m| {
                let b = m
                    .lock()
                    .and_then(|mut g| {
                        let (_, bytes, _) = unsafe { g.align_to_mut::<u8>() };
                        Ok(unsafe { *bytes.get_unchecked(b) })
                    })
                    .expect(
                        "Tried to acquire frame, but .lock() returned an error.\
Did a thread exit unexpectedly while holding this Mutex?",
                    );
                Some(b)
            })
            .ok_or(MemoryError::OutOfBoundsAccess { offset })
    }

    fn load_half_word(&self, offset: u32) -> Result<u16, MemoryError> {
        (offset & 1 == 0)
            .then_some(())
            .ok_or(MemoryError::LoadMisaligned {
                offset,
                alignment: 2,
            })?;

        let pfn = offset as usize >> 12;
        let hw = (offset as usize & 0xfff) >> 1;

        if pfn >= self.frames.len() {
            return Err(MemoryError::OutOfBoundsAccess { offset });
        }

        self.frames
            .get(pfn)
            .and_then(|m| {
                let hw = m
                    .lock()
                    .and_then(|mut g| {
                        let (_, bytes, _) = unsafe { g.align_to_mut::<u16>() };
                        Ok(unsafe { *bytes.get_unchecked(hw) })
                    })
                    .expect(
                        "Tried to acquire frame, but .lock() returned an error.\
Did a thread exit unexpectedly while holding this Mutex?",
                    );
                Some(hw)
            })
            .ok_or(MemoryError::OutOfBoundsAccess { offset })
    }

    fn load_word(&self, offset: u32) -> Result<u32, MemoryError> {
        (offset & 3 == 0)
            .then_some(())
            .ok_or(MemoryError::LoadMisaligned {
                offset,
                alignment: 4,
            })?;

        let pfn = offset as usize >> 12;
        let w = (offset as usize & 0xfff) >> 2;

        if pfn >= self.frames.len() {
            return Err(MemoryError::OutOfBoundsAccess { offset });
        }

        self.frames
            .get(pfn)
            .and_then(|m| {
                let w = m
                    .lock()
                    .and_then(|g| Ok(unsafe { *g.get_unchecked(w) }))
                    .expect(
                        "Tried to acquire frame, but .lock() returned an error.\
Did a thread exit unexpectedly while holding this Mutex?",
                    );
                Some(w)
            })
            .ok_or(MemoryError::OutOfBoundsAccess { offset })
    }

    fn load_reserved(&self, _offset: u32, _src: u32) -> Result<u32, MemoryError> {
        todo!()
    }

    fn store_conditional(&self, _offset: u32, _src: u32) -> Result<u32, MemoryError> {
        todo!()
    }

    fn amoswap_w(&self, _offset: u32, _src: u32) -> Result<u32, MemoryError> {
        todo!()
    }

    fn amoadd_w(&self, _offset: u32, _src: u32) -> Result<u32, MemoryError> {
        todo!()
    }

    fn amoand_w(&self, _offset: u32, _src: u32) -> Result<u32, MemoryError> {
        todo!()
    }

    fn amoor_w(&self, _offset: u32, _src: u32) -> Result<u32, MemoryError> {
        todo!()
    }

    fn amoxor_w(&self, _offset: u32, _src: u32) -> Result<u32, MemoryError> {
        todo!()
    }

    fn amomax_w(&self, _offset: u32, _src: u32) -> Result<u32, MemoryError> {
        todo!()
    }

    fn amomaxu_w(&self, _offset: u32, _src: u32) -> Result<u32, MemoryError> {
        todo!()
    }

    fn amomin_w(&self, _offset: u32, _src: u32) -> Result<u32, MemoryError> {
        todo!()
    }

    fn amominu_w(&self, _offset: u32, _src: u32) -> Result<u32, MemoryError> {
        todo!()
    }

    fn attributes(&self) -> Pma {
        todo!()
    }

    fn properties(&self) -> Properties {
        Properties::new(self.frames.len())
    }
}

impl MainMemory {
    pub fn new(pages: usize) -> MainMemory {
        let frames = (0..pages).map(|_| Mutex::new([0; 1024])).collect();
        Self { frames }
    }
}

#[cfg(test)]
mod tests {
    use crate::memory::{
        main_memory::MainMemory,
        memory_region::{Mapping, MemoryResult},
    };

    #[test]
    fn load_store() -> MemoryResult<()> {
        let m = MainMemory::new(1);
        m.store_word(0x60, 69)?;
        if let Ok(w) = m.load_word(0x60) {
            assert_eq!(w, 69, "Store or load failed");
        }
        Ok(())
    }

    #[test]
    fn block_read_write() -> MemoryResult<()> {
        let m = MainMemory::new(1);
        let b = [69; 0x1000];
        let mut c = [0; 0x1000];
        m.block_write(0, &b[..])?;
        m.block_read(0, &mut c[..])?;
        assert_eq!(c, b, "Write or read failed");
        Ok(())
    }
}
