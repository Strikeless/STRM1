use std::ops::{Deref, DerefMut};

use libstrmisa::Word;

use crate::tracing::TraceData;

use super::Memory;

/// Structure to store word accessed memory mutations during use and automatically patch the changes into the byte-backed memory once dropped.
/// This is mostly transparent to users thanks to the Deref and DerefMut implementations, and acts like a mutable word upon dereference.
/// Prefer dereferencing this as you would with a mutable word reference over directly mutating the data field.
pub struct MemoryWordMutPatch<'a, T>
where
    T: TraceData,
{
    pub(super) memory: &'a mut Memory<T>,
    pub(super) addr: Word,
    pub data: Word,
    pub trace: Option<T::Trace>,
}

impl<'a, T> Deref for MemoryWordMutPatch<'a, T>
where
    T: TraceData,
{
    type Target = Word;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<'a, T> DerefMut for MemoryWordMutPatch<'a, T>
where
    T: TraceData,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

impl<'a, T> Drop for MemoryWordMutPatch<'a, T>
where
    T: TraceData,
{
    fn drop(&mut self) {
        // Simultaneous mutable borrows to the same bytes aren't a data hazard since Rust's borrow checker won't allow that.

        let bytes = libstrmisa::word_to_bytes(self.data);

        match self.trace {
            Some(trace) => {
                *self.memory.0[self.addr as usize].value_mut(trace) = bytes[0];
                *self.memory.0[(self.addr + 1) as usize].value_mut(trace) = bytes[1];
            }
            None => {
                *self.memory.0[self.addr as usize].value_mut_untraced() = bytes[0];
                *self.memory.0[(self.addr + 1) as usize].value_mut_untraced() = bytes[1];
            }
        }
    }
}
