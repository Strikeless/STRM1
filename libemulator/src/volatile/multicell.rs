// NOTE: Everywhere in this file where we refer to a word, it refers to the unit of data stored in a single address of
//       the underlying volatile store. This is different from the architecture's word, and will be of a different size
//       when dealing with byte-backed memory under a 16-bit architecture, for example.

use std::ops::Deref;

use number_bytes::NumberBytes;

/// Dumb wrapper for multi-word values from the word backed store.
/// Useful for doing 16-bit word values on top of an 8-bit memory, for example.
pub struct VolatileMultiCell<M> {
    // Since we want to return references (for API consistency), we need to get and store the inner multi-word
    // value here because we cannot return a reference to a locally combined multi-word variable in the deref function.
    inner_value: M,
}

impl<M> VolatileMultiCell<M> where M: NumberBytes {
    pub fn new<W>(words: &[W]) -> Self where W: NumberBytes + Copy {
        let word_bytes: Vec<_> = words.into_iter()
            .flat_map(|word| word.to_be_bytes())
            .collect();

        let inner_value = M::from_be_bytes(&word_bytes)
            .expect("Got wrong amount of bytes");

        Self {
            inner_value,
        }
    }
}

impl<M> Deref for VolatileMultiCell<M> {
    type Target = M;

    fn deref(&self) -> &Self::Target {
        &self.inner_value
    }
}
