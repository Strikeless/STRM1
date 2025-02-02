// NOTE: Everywhere in this file where we refer to a word, it refers to the unit of data stored in a single address of
//       the underlying volatile store. This is different from the architecture's word, and will be of a different size
//       when dealing with byte-backed memory under a 16-bit architecture, for example.

use std::{collections::HashMap, fmt::Debug, hash::Hash, ops::{Deref, DerefMut}};

use num::Num;
use number_bytes::NumberBytes;

use super::patch::VolatilePatch;

pub trait Multi = Copy + Eq + NumberBytes;
pub trait Word = Copy + NumberBytes;
pub trait Addr = Copy + Eq + Hash + Num + TryFrom<usize, Error: Debug>;

/// Dumb wrapper for multi-word values from the word backed store.
/// Useful for doing 16-bit word values on top of an 8-bit memory, for example.
pub struct VolatileMutMultiCell<'a, M, W, A> where M: Multi, W: Word, A: Addr {
    // Since we want to return references (for API consistency), we need to get and store the inner multi-word
    // value here because we cannot return a reference to a locally combined multi-word variable in the deref function.
    value: M,
    original_value: M,
    
    addr: A,
    storage_words: &'a mut [W],
    patch_buffer: &'a mut HashMap<A, VolatilePatch<W>>,
}

impl<'a, M, W, A> VolatileMutMultiCell<'a, M, W, A> where M: Multi, W: Word, A: Addr {
    pub fn new(words: &'a mut [W], addr: A, patch_buffer: &'a mut HashMap<A, VolatilePatch<W>>) -> Self {
        let word_bytes: Vec<_> = words.into_iter()
            .flat_map(|word| word.to_be_bytes())
            .collect();

        let value = M::from_be_bytes(&word_bytes)
            .expect("Got wrong amount of bytes");

        Self {
            value,
            original_value: value,
            addr,
            storage_words: words,
            patch_buffer,
        }
    }
}

impl<M, W, A> Deref for VolatileMutMultiCell<'_, M, W, A> where M: Multi, W: Word, A: Addr {
    type Target = M;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<M, W, A> DerefMut for VolatileMutMultiCell<'_, M, W, A> where M: Multi, W: Word, A: Addr {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

impl<M, W, A> Drop for VolatileMutMultiCell<'_, M, W, A> where M: Multi, W: Word, A: Addr {
    fn drop(&mut self) {
        // If the value was changed, update and register a patch for every word in the multi-cell.
        // Note that even though we're only updating the actual stored value upon dropping, this doesn't cause any
        // issues with outdated values elsewhere, since Rust's borrow checker doesn't allow reads from anywhere else
        // until this cell gets dropped.
        if self.value != self.original_value {
            let value_bytes = self.value.to_be_bytes();

            let value_words = value_bytes.chunks_exact(W::BYTES)
                .map(|word_bytes| W::from_be_bytes(word_bytes).unwrap());

            for (word_index, word_value) in value_words.enumerate() {
                let word_patch = VolatilePatch {
                    new_value: word_value,
                };

                // SAFETY: Word index will never be out of bounds, assuming the word and multi widths are calculated correctly.
                self.storage_words[word_index] = word_value;

                // SAFETY: The conversion from usize to address should never fail,
                //         as we can't have word indices that go outside of the address space.
                let word_addr = self.addr + A::try_from(word_index).unwrap();

                self.patch_buffer.insert(
                    word_addr,
                    word_patch,
                );
            }
        }
    }
}
