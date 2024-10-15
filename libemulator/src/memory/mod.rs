use libstrmisa::Word;
use wordmut::MemoryWordMutPatch;

pub mod wordmut;

#[cfg(test)]
mod tests;

pub struct Memory(Vec<u8>);

impl Memory {
    pub fn new(data: Vec<u8>) -> Self {
        Self(data)
    }

    pub fn byte(&self, addr: Word) -> Option<u8> {
        self.0.get(addr as usize).copied()
    }

    pub fn byte_mut(&mut self, addr: Word) -> Option<&mut u8> {
        self.0.get_mut(addr as usize)
    }

    pub fn word(&self, addr: Word) -> Option<Word> {
        let first_word = self.byte(addr)?;
        let second_word = self.byte(addr + 1)?;

        Some(libstrmisa::bytes_to_word([first_word, second_word]))
    }

    pub fn word_mut(&mut self, addr: Word) -> Option<MemoryWordMutPatch> {
        let data = self.word(addr)?;

        // Since the memory is byte-backed, it would be really messy and dangerous treating it as words directly,
        // so there's now this cool little middle-layer to handle all of that in a nice way.
        Some(MemoryWordMutPatch {
            memory: self,
            addr,
            data,
        })
    }
}
