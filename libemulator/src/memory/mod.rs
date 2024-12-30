use std::marker::PhantomData;

use anyhow::anyhow;
use libisa::Word;
use mutbyte::MutByteWrapper;
use mutword::MutWordWrapper;
use word::WordWrapper;

pub mod mutbyte;
pub mod mutword;
pub mod word;

#[cfg(test)]
mod tests;

pub struct Memory<const SIZE_BYTES: usize> {
    store: [u8; SIZE_BYTES],
    patch_buffer: Vec<MemoryPatch>,
    _address_type_phantom: PhantomData<Word>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MemoryPatch {
    pub addr: Word,
    pub new_value: u8,
}

impl<const SIZE_BYTES: usize> Memory<SIZE_BYTES> {
    pub fn new() -> Self {
        Self {
            store: [0; SIZE_BYTES],
            patch_buffer: Vec::new(),
            _address_type_phantom: PhantomData,
        }
    }

    pub fn new_with_data<I>(data: I) -> anyhow::Result<Self>
    where
        I: IntoIterator<Item = u8>,
    {
        let mut data: Vec<_> = data.into_iter().collect();

        if data.len() > SIZE_BYTES {
            return Err(anyhow!("Provided volatile data is too long"));
        }

        data.resize(SIZE_BYTES, 0);
        let store = data.try_into().unwrap();

        Ok(Self {
            store,
            ..Self::new()
        })
    }

    pub fn byte(&self, addr: Word) -> Option<&u8> {
        self.store.get(addr as usize)
    }

    pub fn byte_mut(&mut self, addr: Word) -> Option<MutByteWrapper> {
        self.store
            .get_mut(addr as usize)
            .map(|byte| MutByteWrapper::new(byte, addr, &mut self.patch_buffer))
    }

    pub fn word(&self, addr: Word) -> Option<WordWrapper> {
        let byte_chunk = self
            .store
            .array_windows::<{ libisa::BYTES_PER_WORD }>()
            .nth(addr.into())?;

        Some(WordWrapper::new(byte_chunk))
    }

    pub fn word_mut(&mut self, addr: Word) -> Option<MutWordWrapper> {
        let addr_usize = addr as usize;
        
        // windows_mut isn't a thing since that doesn't make sense, do it's work manually.
        let byte_chunk = self
            .store
            .get_mut(addr_usize..addr_usize + libisa::BYTES_PER_WORD)?
            .try_into()
            .unwrap();

        Some(MutWordWrapper::new(byte_chunk, addr, &mut self.patch_buffer))
    }

    pub fn iter_bytes(&self) -> impl Iterator<Item = &u8> {
        self.store.iter()
    }

    pub fn iter_words(
        &self,
    ) -> impl Iterator<Item = WordWrapper> + use<'_, SIZE_BYTES> {
        self.store
            .array_windows::<{ libisa::BYTES_PER_WORD }>()
            .map(|word_bytes| WordWrapper::new(word_bytes))
    }

    pub fn iter_words_non_overlapping(
        &self,
    ) -> impl Iterator<Item = WordWrapper> + use<'_, SIZE_BYTES> {
        self.store
            .array_chunks::<{ libisa::BYTES_PER_WORD }>()
            .map(|word_bytes| WordWrapper::new(word_bytes))
    }
    
    pub fn pop_patches(&mut self) -> impl Iterator<Item = MemoryPatch> + use<'_, SIZE_BYTES> {
        self.patch_buffer.drain(..)
    }
}
