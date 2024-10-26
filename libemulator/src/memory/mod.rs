use libisa::Word;
use wordmut::MemoryWordMutPatch;

use crate::tracing::{TraceData, Traced};

pub mod wordmut;

#[cfg(test)]
mod tests;

pub struct Memory<T>(Vec<Traced<u8, T>>)
where
    T: TraceData;

impl<T> Memory<T>
where
    T: TraceData,
{
    pub fn new(data: Vec<u8>) -> Self {
        Self(data.into_iter().map(|value| Traced::new(value)).collect())
    }

    pub fn iter_untraced(&self) -> impl Iterator<Item = &u8> {
        self.0.iter().map(|traced| traced.value())
    }

    pub fn byte(&self, addr: Word) -> Option<u8> {
        self.0
            .get(addr as usize)
            .map(|traced| traced.value())
            .copied()
    }

    pub fn byte_mut(&mut self, trace: T::Trace, addr: Word) -> Option<&mut u8> {
        self.0
            .get_mut(addr as usize)
            .map(|traced| traced.value_mut(trace))
    }

    pub fn byte_mut_untraced(&mut self, addr: Word) -> Option<&mut u8> {
        self.0
            .get_mut(addr as usize)
            .map(|traced| traced.value_mut_untraced())
    }

    pub fn word(&self, addr: Word) -> Option<Word> {
        let first_word = self.byte(addr)?;
        let second_word = self.byte(addr + 1)?;

        Some(libisa::bytes_to_word([first_word, second_word]))
    }

    pub fn word_mut(&mut self, trace: T::Trace, addr: Word) -> Option<MemoryWordMutPatch<T>> {
        let data = self.word(addr)?;

        // Since the memory is byte-backed, it would be really messy and dangerous treating it as words directly,
        // so there's now this cool little middle-layer to handle all of that in a nice way.
        Some(MemoryWordMutPatch {
            memory: self,
            addr,
            data,
            trace: Some(trace),
        })
    }

    pub fn word_mut_untraced(&mut self, addr: Word) -> Option<MemoryWordMutPatch<T>> {
        let data = self.word(addr)?;

        // Since the memory is byte-backed, it would be really messy and dangerous treating it as words directly,
        // so there's now this cool little middle-layer to handle all of that in a nice way.
        Some(MemoryWordMutPatch {
            memory: self,
            addr,
            data,
            trace: None,
        })
    }

    pub fn trace(&self, addr: Word) -> Option<&T> {
        self.0.get(addr as usize).map(|traced| &traced.trace_data)
    }
}
