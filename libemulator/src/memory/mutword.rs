use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
};

use libisa::Word;

use super::MemoryPatch;

pub struct MutWordWrapper<'a> {
    source: &'a mut [u8; libisa::BYTES_PER_WORD],
    addr: Word,

    patch_buffer: &'a mut HashMap<Word, MemoryPatch>,

    // We must have a copy of the word value here, the borrow checker guarantees that this won't cause data desync
    // issues here as well. Read the explanation in the non-mut variant WordWrapper.
    inner_value: Word,

    original_value: Word,
}

impl<'a> MutWordWrapper<'a> {
    pub fn new(
        inner: &'a mut [u8; libisa::BYTES_PER_WORD],
        addr: Word,
        patch_buffer: &'a mut HashMap<Word, MemoryPatch>,
    ) -> Self {
        let original_value = libisa::bytes_to_word(*inner);

        Self {
            source: inner,
            addr,
            patch_buffer,
            inner_value: original_value,
            original_value,
        }
    }
}

impl Deref for MutWordWrapper<'_> {
    type Target = Word;

    fn deref(&self) -> &Self::Target {
        &self.inner_value
    }
}

impl DerefMut for MutWordWrapper<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner_value
    }
}

impl Drop for MutWordWrapper<'_> {
    fn drop(&mut self) {
        *self.source = libisa::word_to_bytes(self.inner_value);

        if self.inner_value != self.original_value {
            for (byte_index, byte) in self.source.iter().enumerate() {
                let byte_addr = self.addr + byte_index as Word;

                let patch = MemoryPatch { new_value: *byte };

                self.patch_buffer.insert(byte_addr, patch);
            }
        }
    }
}
