// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// This Source Code Form is "Incompatible With Secondary Licenses", as
// defined by the Mozilla Public License, v. 2.0.
//
// Copyright © 2022 mumblingdrunkard

use std::sync::Mutex;

use super::mapping::{Mapping, MemoryError, MemoryResult, Pma, Properties};

type Frame = [u32; 1024];

/// A main memory region that supports all memory operations
pub struct Main {
    frames: Vec<Mutex<Frame>>,
}

impl Main {
    fn store<const W: usize>(&self, offset: u32, val: u32) -> MemoryResult<()> {
        assert!(matches!(W, 1 | 2 | 4), "Store width must be 1, 2, or 4");

        if offset & (W as u32 - 1) != 0 {
            return Err(MemoryError::StoreMisaligned {
                offset,
                alignment: W as u32,
            });
        }

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
                        match W {
                            1 => unsafe {
                                let (_, bytes, _) = g.align_to_mut::<u8>();
                                *bytes.get_unchecked_mut(b) = val as u8
                            },
                            2 => unsafe {
                                let (_, half_words, _) = g.align_to_mut::<u16>();
                                *half_words.get_unchecked_mut(b >> 1) = val as u16
                            },
                            4 => unsafe { *g.get_unchecked_mut(b >> 2) = val },
                            _ => unsafe { std::hint::unreachable_unchecked() },
                        }
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

    fn load<const W: usize>(&self, offset: u32) -> Result<u32, MemoryError> {
        assert!(matches!(W, 1 | 2 | 4), "Load width must be 1, 2, or 4");

        if offset & (W as u32 - 1) != 0 {
            return Err(MemoryError::StoreMisaligned {
                offset,
                alignment: W as u32,
            });
        }

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
                    .and_then(|mut g| match W {
                        1 => unsafe {
                            let (_, bytes, _) = g.align_to_mut::<u8>();
                            Ok(*bytes.get_unchecked(b) as u32)
                        },
                        2 => unsafe {
                            let (_, half_words, _) = g.align_to_mut::<u16>();
                            Ok(*half_words.get_unchecked(b >> 1) as u32)
                        },
                        4 => unsafe { Ok(*g.get_unchecked(b >> 2)) },
                        _ => unsafe { std::hint::unreachable_unchecked() },
                    })
                    .expect(
                        "Tried to acquire frame, but .lock() returned an error.\
Did a thread exit unexpectedly while holding this Mutex?",
                    );
                Some(b)
            })
            .ok_or(MemoryError::OutOfBoundsAccess { offset })
    }
}

impl Mapping for Main {
    fn block_write(&self, offset: u32, src: &[u8]) -> MemoryResult<usize> {
        let start = offset as usize >> 12;
        let end = (offset as usize + src.len() - 1) >> 12;

        if end >= self.frames.len() {
            return Err(MemoryError::OutOfBoundsAccess { offset });
        }

        let mut frame_offs = offset as usize & 0xfff; // frame offset
        let mut src_offs = 0; // data offset

        self.frames[start..=end].iter().for_each(|frame| {
            frame
                .lock()
                .and_then(|mut g| {
                    let (_, dst, _) = unsafe { g.align_to_mut::<u8>() };
                    let n = std::cmp::min(dst.len() - frame_offs, src.len() - src_offs);
                    dst[frame_offs..frame_offs + n].clone_from_slice(&src[src_offs..src_offs + n]);
                    src_offs += n;
                    frame_offs = 0;

                    Ok(())
                })
                .expect(
                    "Tried to acquire frame, but .lock() returned an error.\
Did a thread exit unexpectedly while holding this Mutex?",
                )
        });

        assert_eq!(src_offs, src.len(), "Failed to store all elements from src");

        Ok(src_offs)
    }

    fn block_write_masked(&self, offset: u32, src: &[u8], _mask: &[u8]) -> MemoryResult<usize> {
        if _mask.len() * 8 < src.len() {
            panic!("Mask must contain enough bits to mask src!");
        }

        let start = offset as usize >> 12;
        let end = (offset as usize + src.len() - 1) >> 12;

        if end >= self.frames.len() {
            return Err(MemoryError::OutOfBoundsAccess { offset });
        }

        let mut frame_offs = offset as usize & 0xfff; // frame offset
        let mut src_offs = 0; // data offset
        let mut written = 0;

        self.frames[start..=end].iter().for_each(|frame| {
            frame
                .lock()
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
                )
        });

        // assert_eq!(src_offs, src.len(), "Failed to store all elements from src");

        Ok(written)
    }

    fn block_read(&self, offset: u32, dst: &mut [u8]) -> Result<usize, MemoryError> {
        let start = offset as usize >> 12;
        let end = (offset as usize + dst.len() - 1) >> 12;

        if end >= self.frames.len() {
            return Err(MemoryError::OutOfBoundsAccess { offset });
        }

        let mut frame_offs = offset as usize & 0xfff; // frame offset
        let mut dst_offs = 0; // data offset

        self.frames[start..=end].iter().for_each(|frame| {
            frame
                .lock()
                .and_then(|g| {
                    // calculate number of elements
                    let (_, src, _) = unsafe { g.align_to::<u8>() };
                    let n = std::cmp::min(src.len() - frame_offs, dst.len() - dst_offs);

                    // clone into dst
                    dst[dst_offs..dst_offs + n].clone_from_slice(&src[frame_offs..frame_offs + n]);

                    // next loop
                    dst_offs += n;
                    frame_offs = 0;

                    Ok(())
                })
                .expect(
                    "Tried to acquire frame, but .lock() returned an error.\
Did a thread exit unexpectedly while holding this Mutex?",
                )
        });

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

    fn stream_read(
        &self,
        _offset: u32,
        _reads: &[(u16, u8)],
        _dst: &mut [u32],
    ) -> MemoryResult<usize> {
        todo!()
    }

    fn store_byte(&self, offset: u32, byte: u8) -> MemoryResult<()> {
        self.store::<1>(offset, byte as u32)
    }

    fn store_half_word(&self, offset: u32, half_word: u16) -> MemoryResult<()> {
        self.store::<2>(offset, half_word as u32)
    }

    fn store_word(&self, offset: u32, word: u32) -> MemoryResult<()> {
        self.store::<4>(offset, word)
    }

    fn load_byte(&self, offset: u32) -> MemoryResult<u8> {
        self.load::<1>(offset).map(|x| x as u8)
    }

    fn load_half_word(&self, offset: u32) -> MemoryResult<u16> {
        self.load::<2>(offset).map(|x| x as u16)
    }

    fn load_word(&self, offset: u32) -> Result<u32, MemoryError> {
        self.load::<4>(offset)
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

    fn register_store_callback(&self, _f: Box<dyn Fn(u32)>) {
        todo!()
    }
}

impl Main {
    /// Create a new main memory with `pages` pages of 4096 bytes each.
    pub fn new(pages: usize) -> Main {
        let frames = (0..pages).map(|_| Mutex::new([0; 1024])).collect();
        Self { frames }
    }
}

#[cfg(test)]
mod tests {
    use crate::memory::{
        main::Main,
        mapping::{Mapping, MemoryResult},
    };

    #[test]
    fn load_store() -> MemoryResult<()> {
        let m = Main::new(1);
        m.store_word(0x60, 69)?;
        if let Ok(w) = m.load_word(0x60) {
            assert_eq!(w, 69, "Store or load failed");
        }
        Ok(())
    }

    #[test]
    fn block_read_write() -> MemoryResult<()> {
        let m = Main::new(1);
        let b = [69; 0x1000];
        let mut c = [0; 0x1000];
        m.block_write(0, &b[..])?;
        m.block_read(0, &mut c[..])?;
        assert_eq!(c, b, "Write or read failed");
        Ok(())
    }
}
