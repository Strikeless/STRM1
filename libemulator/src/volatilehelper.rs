use libisa::Word;

use crate::{
    volatile::{multicell::VolatileMultiCell, mutcell::VolatileMutCell, mutmulticell::VolatileMutMultiCell}, Emulator, ExecuteErr
};

impl Emulator {
    pub(super) fn reg_word(&self, index: usize) -> &Word {
        self.reg_file
            .get(index)
            .expect("Out of bounds register access")
    }

    pub(super) fn reg_word_mut(&mut self, index: usize) -> VolatileMutCell<'_, Word, usize> {
        self.reg_file
            .get_mut(index)
            .expect("Out of bounds register access")
    }

    pub(super) fn mem_byte_or_err(&self, addr: Word) -> Result<&u8, ExecuteErr> {
        self.memory
            .get(addr)
            .ok_or(ExecuteErr::MemoryAccessViolation(addr))
    }

    pub(super) fn mem_byte_mut_or_err(&mut self, addr: Word) -> Result<VolatileMutCell<'_, u8, Word>, ExecuteErr> {
        self.memory
            .get_mut(addr)
            .ok_or(ExecuteErr::MemoryAccessViolation(addr))
    }

    pub(super) fn mem_word_or_err(&self, addr: Word) -> Result<VolatileMultiCell<Word>, ExecuteErr> {
        self.memory
            .get_multi(addr)
            .ok_or(ExecuteErr::MemoryAccessViolation(addr))
    }

    pub(super) fn mem_word_mut_or_err(&mut self, addr: Word) -> Result<VolatileMutMultiCell<'_, Word, u8, Word>, ExecuteErr> {
        self.memory
            .get_mut_multi(addr)
            .ok_or(ExecuteErr::MemoryAccessViolation(addr))
    }
}
