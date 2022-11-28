// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// This Source Code Form is "Incompatible With Secondary Licenses", as
// defined by the Mozilla Public License, v. 2.0.
//
// Copyright © 2022 mumblingdrunkard

#[allow(unused)]
#[derive(Debug, Clone, Copy)]
pub enum MemoryKind {
    Main = 0,
    Io,
}

#[allow(unused)]
#[derive(Debug, Clone, Copy)]
pub enum AmoClass {
    /// No atomics; all atomic operations will fail
    None = 0,

    /// Swap atomics; amoswap is supported
    Swap,

    /// Logical atomics; amoand, amoor, amoxor + Swap atomics are supported
    Logical,

    /// Arithmetic atomics; amoadd, amomin[u], amomax[u] + Logical + Swap atomics are supported
    Arithmetic,
}

#[allow(unused)]
#[derive(Debug, Clone, Copy)]
// TODO For a mapping to be reservable, it must be able to register callbacks when stores occur
pub enum Reservability {
    /// No reservability; lr and sc instructions are unsupported.
    None = 0,

    /// Reservability, but no eventuality guarantees; sc is allowed to never succeed.
    NonEventual,

    /// Reservability and eventuality; sc must eventually succeed if other conditions are upheld
    Eventual,
}

#[allow(unused)]
#[derive(Debug, Clone, Copy)]
pub enum Idempotency {
    /// This region is not idempotent and spurious reads or writes must not occur.
    /// This implies no caching.
    /// Usually the case for stream I/O devices such as UART.
    NonIdempotent = 0,

    /// This region is idempotent and spurious reads or writes can occur.
    Idempotent,
}

#[allow(unused)]
#[derive(Debug, Clone, Copy)]
pub enum Cacheability {
    /// This region cannot be cached in any way. All loads and stores must be coherent.
    NonCacheable = 0,

    /// This region accepts streamed writes or loads.
    Stream,

    /// This region accepts streamed writes. Loads can be cached.
    WriteStreamLoadCache,

    /// This region is fully cacheable
    Cacheable,
}

#[allow(unused)]
#[derive(Debug, Clone, Copy)]
pub struct Pma {
    kind: MemoryKind,
    amo: AmoClass,
    reservability: Reservability,
    idempotency: Idempotency,
    cacheability: Cacheability,
}

impl Default for Pma {
    fn default() -> Self {
        Self {
            kind: MemoryKind::Main,
            amo: AmoClass::Arithmetic,
            reservability: Reservability::Eventual,
            idempotency: Idempotency::Idempotent,
            cacheability: Cacheability::Cacheable,
        }
    }
}

impl Pma {
    pub fn packed(&self) -> PmaPacked {
        let (kind, amo, reservability, idempotency, cacheability) = (
            self.kind as u8,
            self.amo as u8,
            self.reservability as u8,
            self.idempotency as u8,
            self.cacheability as u8,
        );

        PmaPacked {
            internal: kind
                | (amo << 1)
                | (reservability << 3)
                | (idempotency << 5)
                | (cacheability << 6),
        }
    }
}

#[allow(unused)]
#[derive(Debug, Clone, Copy)]
pub struct PmaPacked {
    // cacheability | idempotency | reservability | amoclass | kind
    //            2             1               2          2      1
    internal: u8,
}

impl Default for PmaPacked {
    fn default() -> Self {
        Pma::default().packed()
    }
}

impl PmaPacked {
    pub fn kind(&self) -> MemoryKind {
        match self.internal & 1 {
            0 => MemoryKind::Main,
            1 => MemoryKind::Io,
            _ => unreachable!(),
        }
    }

    pub fn amo(&self) -> AmoClass {
        match (self.internal >> 1) & 3 {
            0 => AmoClass::None,
            1 => AmoClass::Swap,
            2 => AmoClass::Logical,
            3 => AmoClass::Arithmetic,
            _ => unreachable!(),
        }
    }

    pub fn reservability(&self) -> Reservability {
        match (self.internal >> 3) & 3 {
            0 | 3 => Reservability::None,
            1 => Reservability::NonEventual,
            2 => Reservability::Eventual,
            _ => unreachable!(),
        }
    }

    pub fn idempotency(&self) -> Idempotency {
        match (self.internal >> 5) & 1 {
            0 => Idempotency::NonIdempotent,
            1 => Idempotency::Idempotent,
            _ => unreachable!(),
        }
    }

    pub fn cacheability(&self) -> Cacheability {
        match (self.internal >> 6) & 3 {
            0 => Cacheability::NonCacheable,
            1 => Cacheability::Stream,
            2 => Cacheability::WriteStreamLoadCache,
            3 => Cacheability::Cacheable,
            _ => unreachable!(),
        }
    }

    pub fn unpacked(&self) -> Pma {
        Pma {
            kind: self.kind(),
            amo: self.amo(),
            reservability: self.reservability(),
            idempotency: self.idempotency(),
            cacheability: self.cacheability(),
        }
    }
}

#[allow(unused)]
#[derive(Debug)]
pub enum MemoryError {
    OutOfBoundsAccess { offset: u32 },
    AmoUnsupported { amo: AmoClass },
    AmoMisaligned { offset: u32, amo: AmoClass },
    LoadMisaligned { offset: u32, alignment: u32 },
    StoreMisaligned { offset: u32, alignment: u32 },
    SizeUnsupported { offset: u32, size: u32 },
    BlockOperationUnsupported,
}

pub type MemoryResult<T> = std::result::Result<T, MemoryError>;

#[allow(unused)]
pub struct Properties {
    pages: usize,
}

impl Properties {
    pub fn new(pages: usize) -> Self {
        Self { pages }
    }
}

#[allow(unused)]
pub trait Mapping {
    /// Intended for writing chunks of bytes from a region of memory.
    ///
    /// `block_write` should work across sequential pages in the same memory mapping.
    /// I.e. if a device exposes 4 sequential pages of mapped memory, a `block_write` with
    /// `src.len()` == 16384 and offset = 0 should succeed.
    ///
    /// Not all mappings may support `block_write`.
    fn block_write(&self, offset: u32, src: &[u8]) -> MemoryResult<usize>;

    /// Like `block_write`, but a bit-mask can be provided to only write the desired bytes.
    ///
    /// `mask` is a slice of u8 to give the most generic possible interface and the function will
    /// panic if `mask.len() * 8 < dst.len()`.
    /// A byte `src[i]` will only be written if `(mask[i >> 3] >> (i & 7)) & 1 == 1`
    ///
    /// Like `block_write`, `block_write_masked` should work across sequential pages in the same
    /// memory mapping.
    /// I.e. if a device exposes 4 sequential pages of mapped memory, a `block_write_masked` with
    /// `src.len()` == 16384 and offset = 0 should succeed.
    ///
    /// Like `block_write`, not all mappings may support `block_write_masked`, *but* if
    /// `block_write` is supported, `block_write_masked` must also be supported.
    fn block_write_masked(&self, offset: u32, src: &[u8], mask: &[u8]) -> MemoryResult<usize>;

    /// Intended for reading chunks of bytes from a region of memory.
    ///
    /// `block_read` should work across sequential pages in the same memory mapping.
    /// I.e. if a device exposes 4 sequential pages of mapped memory, a `block_read` with
    /// `dst.len()` == 16384 and offset = 0 should succeed.
    fn block_read(&self, offset: u32, dst: &mut [u8]) -> MemoryResult<usize>;

    /// Like `block_read`, but a bit-mask can be provided to only read the desired bytes.
    ///
    /// `mask` is a slice of u8 to give the most generic possible interface and the function will
    /// panic if `mask.len() * 8 < dst.len()`.
    /// A byte `dst[i]` will only be written to if `(mask[i >> 3] >> (i & 7)) & 1 == 1`
    ///
    /// Like `block_read`, `block_read_masked` should work across sequential pages in the same
    /// memory mapping.
    /// I.e. if a device exposes 4 sequential pages of mapped memory, a `block_read_masked` with
    /// `dst.len()` == 16384 and offset = 0 should succeed.
    ///
    /// Like `block_read`, not all mappings may support `block_read_masked`, *but* if
    /// `block_read` is supported, `block_read_masked` must also be supported.
    fn block_read_masked(&self, offset: u32, dst: &mut [u8], mask: &[u8]) -> MemoryResult<usize>;

    /// Intended to perform a stream of writes in a single call.
    /// Useful for write-combine operations on I/O that may otherwise be very slow; e.g. when
    /// writing a stream of bytes to UART.
    ///
    /// Unlike block operations, stream operations are only supported on one frame at a time.
    /// This is done for technical reasons to simplify the implementation of the write-combinging
    /// buffers.
    /// It also enables encoding writes as an offset into the frame, saving a few bytes.
    ///
    /// Writes are encoded as `(offset: u16, width: u8, value: u32)`
    /// `offset` should be a 12-bit offset into the frame,
    /// `width` should be 1, 2, or 4; the number of bytes to be written, and
    /// `value` should be a `width` byte wide, right-aligned value to be written.
    ///
    /// If the mapping does not support misaligned writes and a misaligned write is encountered,
    /// this function **must panic**
    /// This is because later instructions -- issued after writes in the buffer -- may have
    /// completed, breaking precise exceptions.
    /// It is therefore important that the mapping is queried for support of misaligned stores
    /// before the write is added to a write buffer.
    fn stream_write(&self, frame: u32, writes: &[(u16, u8, u32)]) -> MemoryResult<usize>;

    /// Intended to perform a stream of reads in a single call.
    /// Useful for read-combine operations on I/O that may otherwise be very slow; e.g. when
    /// reading a stream of bytes from UART.
    ///
    /// Unlike block operations, stream operations are only supported on one frame at a time.
    /// This is done for technical reasons to simplify the implementation of the read-combinging
    /// buffers.
    /// It also enables encoding reads as an offset into the frame, saving a few bytes.
    ///
    /// This function will panic when `reads.len() != dst.len()`.
    ///
    /// Reads are encoded as `(offset: u16, width: u8)`
    /// `offset` should be a 12-bit offset into the frame, and
    /// `width` should be 1, 2, or 4; the number of bytes to be read.
    ///
    /// If the mapping does not support misaligned reads and a misaligned read is encountered,
    /// this function **must panic**
    /// This is because later instructions -- issued after reads in the buffer -- may have
    /// completed, breaking precise exceptions.
    /// It is therefore important that the mapping is queried for support of misaligned reads
    /// before the read is added to a read buffer.
    fn stream_read(&self, frame: u32, reads: &[(u16, u8)], dst: &mut [u32]);

    fn store_byte(&self, offset: u32, byte: u8) -> MemoryResult<()>;
    fn store_half_word(&self, offset: u32, half_word: u16) -> MemoryResult<()>;
    fn store_word(&self, offset: u32, word: u32) -> MemoryResult<()>;

    fn load_byte(&self, offset: u32) -> MemoryResult<u8>;
    fn load_half_word(&self, offset: u32) -> MemoryResult<u16>;
    fn load_word(&self, offset: u32) -> MemoryResult<u32>;

    fn load_reserved(&self, offset: u32, src: u32) -> MemoryResult<u32>;
    fn store_conditional(&self, offset: u32, src: u32) -> MemoryResult<u32>;

    fn amoswap_w(&self, offset: u32, src: u32) -> MemoryResult<u32>;
    fn amoadd_w(&self, offset: u32, src: u32) -> MemoryResult<u32>;
    fn amoand_w(&self, offset: u32, src: u32) -> MemoryResult<u32>;
    fn amoor_w(&self, offset: u32, src: u32) -> MemoryResult<u32>;
    fn amoxor_w(&self, offset: u32, src: u32) -> MemoryResult<u32>;
    fn amomax_w(&self, offset: u32, src: u32) -> MemoryResult<u32>;
    fn amomaxu_w(&self, offset: u32, src: u32) -> MemoryResult<u32>;
    fn amomin_w(&self, offset: u32, src: u32) -> MemoryResult<u32>;
    fn amominu_w(&self, offset: u32, src: u32) -> MemoryResult<u32>;

    fn attributes(&self) -> Pma;
    fn properties(&self) -> Properties;

    /// Register a callback that should be called every time a change is made to the underlying
    /// memory.
    /// The callback should accept the offset that the store occured at.
    ///
    /// This is useful for informing reservation sets when devices make changes to memory or for
    /// raising interrupts when operations complete or new data is available.
    fn register_store_callback(&self, f: Box<dyn Fn(u32)>);
}
