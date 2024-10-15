use std::ops::{Deref, DerefMut};

use libstrmisa::Word;

use super::Memory;

/// Structure to store word accessed memory mutations during use and automatically patch the changes into the byte-backed memory once dropped.
/// This is mostly transparent to users thanks to the Deref and DerefMut implementations, and acts like a mutable word upon dereference.
/// Prefer dereferencing this as you would with a mutable word reference over directly mutating the data field.
pub struct MemoryWordMutPatch<'a> {
    pub(super) memory: &'a mut Memory,
    pub(super) addr: Word,
    pub data: Word,
}

impl<'a> Deref for MemoryWordMutPatch<'a> {
    type Target = Word;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<'a> DerefMut for MemoryWordMutPatch<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

impl<'a> Drop for MemoryWordMutPatch<'a> {
    fn drop(&mut self) {
        // Simultaneous mutable borrows to the same bytes aren't a data hazard since Rust's borrow checker won't allow that.

        let bytes = libstrmisa::word_to_bytes(self.data);

        self.memory.0[self.addr as usize] = bytes[0];
        self.memory.0[(self.addr + 1) as usize] = bytes[1];
    }
}
