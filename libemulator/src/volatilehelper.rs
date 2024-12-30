use libisa::Word;

use crate::{memory::{mutbyte::MutByteWrapper, mutword::MutWordWrapper, word::WordWrapper}, Emulator, ExecuteErr};

impl Emulator {
    pub(super) fn reg_word(&self, index: usize) -> &Word {
        self.reg_file.get(index)
            .expect("Out of bounds register access")
    }

    pub(super) fn reg_word_mut(&mut self, index: usize) -> &mut Word {
        self.reg_file.get_mut(index)
            .expect("Out of bounds register access")
    }

    pub(super) fn mem_byte_or_err(&self, addr: Word) -> Result<&u8, ExecuteErr> {
        self.memory.byte(addr)
            .ok_or(ExecuteErr::MemoryAccessViolation(addr))
    }

    pub(super) fn mem_byte_mut_or_err(&mut self, addr: Word) -> Result<MutByteWrapper, ExecuteErr> {
        self.memory.byte_mut(addr)
            .ok_or(ExecuteErr::MemoryAccessViolation(addr))
    }
    
    pub(super) fn mem_word_or_err(&self, addr: Word) -> Result<WordWrapper, ExecuteErr> {
        self.memory.word(addr)
            .ok_or(ExecuteErr::MemoryAccessViolation(addr))
    }

    pub(super) fn mem_word_mut_or_err(&mut self, addr: Word) -> Result<MutWordWrapper, ExecuteErr> {
        self.memory.word_mut(addr)
            .ok_or(ExecuteErr::MemoryAccessViolation(addr))
    }
}
