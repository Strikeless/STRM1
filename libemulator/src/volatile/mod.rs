use std::{collections::HashMap, fmt::Debug, hash::Hash};

use anyhow::bail;
use multicell::VolatileMultiCell;
use mutcell::VolatileMutCell;
use mutmulticell::VolatileMutMultiCell;
use num::Num;
use number_bytes::NumberBytes;
use patch::VolatilePatch;

pub mod patch;
pub mod mutcell;
pub mod multicell;
pub mod mutmulticell;

#[cfg(test)]
mod tests;

pub trait Word = Copy + Num + NumberBytes;
pub trait Addr = Copy + Eq + Hash + Num + Into<usize> + TryFrom<usize, Error: Debug>;

pub struct Volatile<W, A> {
    data: Vec<W>,
    patches: HashMap<A, VolatilePatch<W>>,
}

impl<W, A> Volatile<W, A> where W: Word, A: Addr {
    pub fn new(size: A) -> Self {
        // SAFETY: The empty vector will never be longer than the size, thus there will never be an error.
        Self::new_with_data(vec![], size).unwrap()
    }

    pub fn new_with_data<I>(data: I, size: A) -> anyhow::Result<Self> where I: IntoIterator<Item = W> {
        let mut data: Vec<_> = data.into_iter().collect();
        let size = size.into();

        if data.len() > size {
            bail!("Given data exceeds size");
        }

        data.resize(size, W::zero());

        Ok(Self {
            data,
            patches: HashMap::new(),
        })
    }

    pub fn get(&self, addr: A) -> Option<&W> {
        self.data.get(Self::addr_to_usize(addr))
    }

    pub fn get_mut(&mut self, addr: A) -> Option<VolatileMutCell<W, A>> {
        let inner = self.data.get_mut(Self::addr_to_usize(addr))?;
        Some(VolatileMutCell::new(inner, addr, &mut self.patches))
    }

    pub fn get_multi<M>(&self, addr: A) -> Option<VolatileMultiCell<M>> where M: NumberBytes + Copy {
        let words_per_multi = M::BYTES / W::BYTES;

        let addr = Self::addr_to_usize(addr);

        let inner = self.data.get(addr .. addr + words_per_multi)?;
        Some(VolatileMultiCell::new(inner))
    }

    pub fn get_mut_multi<M>(&mut self, addr: A) -> Option<VolatileMutMultiCell<M, W, A>> where M: mutmulticell::Multi {
        let words_per_multi = M::BYTES / W::BYTES;

        let addr_usize = Self::addr_to_usize(addr);
        let inner = self.data.get_mut(addr_usize .. addr_usize + words_per_multi)?;

        Some(VolatileMutMultiCell::new(inner, addr, &mut self.patches))
    }

    pub fn iter_words(&self) -> impl Iterator<Item = &W> {
        self.data.iter()
    }

    pub fn iter_multis<M>(&self) -> impl Iterator<Item = VolatileMultiCell<M>> + use<'_, M, W, A> where M: NumberBytes {
        let words_per_multi = M::BYTES / W::BYTES;

        self.data.windows(words_per_multi)
            .map(|multi_bytes| VolatileMultiCell::new(multi_bytes))
    }

    pub fn iter_multis_non_overlapping<M>(&self) -> impl Iterator<Item = VolatileMultiCell<M>> + use<'_, M, W, A> where M: NumberBytes {
        let words_per_multi = M::BYTES / W::BYTES;

        self.data.chunks(words_per_multi)
            .map(|multi_bytes| VolatileMultiCell::new(multi_bytes))
    }

    pub fn pop_patches(&mut self) -> impl Iterator<Item = (A, VolatilePatch<W>)> + use<'_, W, A> {
        self.patches.drain()
    }

    fn addr_to_usize(addr: A) -> usize {
        addr.into()
    }
}
