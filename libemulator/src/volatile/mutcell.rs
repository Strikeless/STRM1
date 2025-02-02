use std::{
    collections::HashMap, hash::Hash, ops::{Deref, DerefMut}
};

use super::patch::VolatilePatch;

pub trait Word = PartialEq + Clone;
pub trait Addr = Copy + Eq + Hash;

pub struct VolatileMutCell<'a, W, A> where W: Word, A: Addr {
    value: &'a mut W,
    original_value: W,

    addr: A,
    patch_buffer: &'a mut HashMap<A, VolatilePatch<W>>,
}

impl<'a, W, A> VolatileMutCell<'a, W, A> where W: Word, A: Addr {
    pub(super) fn new(
        inner: &'a mut W,
        addr: A,
        patch_buffer: &'a mut HashMap<A, VolatilePatch<W>>,
    ) -> Self {
        Self {
            original_value: inner.clone(),
            value: inner,
            addr,
            patch_buffer,
        }
    }
}

impl<W, A> Deref for VolatileMutCell<'_, W, A> where W: Word, A: Addr {
    type Target = W;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<W, A> DerefMut for VolatileMutCell<'_, W, A> where W: Word, A: Addr {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

impl<W, A> Drop for VolatileMutCell<'_, W, A> where W: Word, A: Addr {
    fn drop(&mut self) {
        // If the value was changed, register a patch.
        if *self.value != self.original_value {
            let patch = VolatilePatch {
                new_value: self.value.clone(),
            };

            self.patch_buffer.insert(
                self.addr.clone(),
                patch
            );
        }
    }
}
