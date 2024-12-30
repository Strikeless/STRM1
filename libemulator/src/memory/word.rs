use std::ops::Deref;

use libisa::Word;

// Dumb wrapper for words from the byte-backed store, only purpose is to let
// us return words as references to be consistent with the byte counterpart.
pub struct WordWrapper {
    // Since we want to return references (for API consistency), we need to get and store the inner value here because
    // we cannot return a reference to a local variable in the deref function. This would introduce a problem with the
    // wrapped value not updating with the byte-backed source, but Rust's borrow rules don't allow that scenario to
    // happen, since we would need to have a mutable reference to the source and an immutable reference to this struct.
    inner_value: Word,
}

impl WordWrapper {
    pub(super) fn new(inner: &[u8; libisa::BYTES_PER_WORD]) -> Self {
        Self {
            inner_value: libisa::bytes_to_word(*inner),
        }
    }
}

impl Deref for WordWrapper {
    type Target = libisa::Word;

    fn deref(&self) -> &Self::Target {
        &self.inner_value
    }
}
