use std::ops::{Deref, DerefMut};

use libisa::Word;

use super::MemoryPatch;

pub struct MutByteWrapper<'a> {
    inner: &'a mut u8,
    addr: Word,

    patch_buffer: &'a mut Vec<MemoryPatch>,
    original_value: u8,
}

impl<'a> MutByteWrapper<'a> {
    pub(super) fn new(inner: &'a mut u8, addr: Word, patch_buffer: &'a mut Vec<MemoryPatch>) -> Self {
        Self {
            original_value: *inner,
            inner,
            addr,
            patch_buffer,
        }
    }
}

impl Deref for MutByteWrapper<'_> {
    type Target = u8;

    fn deref(&self) -> &Self::Target {
        self.inner
    }
}

impl DerefMut for MutByteWrapper<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner
    }
}

impl Drop for MutByteWrapper<'_> {
    fn drop(&mut self) {
        if *self.inner != self.original_value {
            self.patch_buffer.push(MemoryPatch {
                addr: self.addr,
                new_value: *self.inner,
            });
        }
    }
}
