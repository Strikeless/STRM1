use libstrmisa::{
    instruction::{Instruction, InstructionDeassemblyError},
    Register, Word,
};
use thiserror::Error;

use crate::{memory::wordmut::MemoryWordMutPatch, tracing::TraceData, Emulator};

mod parsed;

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecuteOk {
    Normal,
    Halted,
}

#[derive(Debug, Error, Clone, Copy)]
pub enum ExecuteErr {
    #[error("Memory access violation")]
    MemoryAccessViolation,

    #[error("Illegal instruction ({0})")]
    IllegalInstruction(InstructionDeassemblyError),
}

impl<T> Emulator<T>
where
    T: TraceData,
{
    pub fn execute_to_halt(&mut self) -> Result<(), ExecuteErr> {
        loop {
            let state = self.execute_instruction()?;

            if state == ExecuteOk::Halted {
                break Ok(());
            }
        }
    }

    pub fn execute_instruction(&mut self) -> Result<ExecuteOk, ExecuteErr> {
        self.current_trace = T::trace_from_state(&self);
        let instruction = self.parse_next_instruction()?;
        self.execute_parsed_instruction(instruction)
    }

    fn parse_next_instruction(&mut self) -> Result<Instruction, ExecuteErr> {
        let instruction_word = self.pc_next_word()?;

        let mut instruction = Instruction::deassemble_instruction_word(instruction_word)
            .map_err(|e| ExecuteErr::IllegalInstruction(e))?;

        if instruction.kind.has_immediate() {
            let immediate = self.pc_next_word()?;
            instruction.immediate = Some(immediate);
        }

        Ok(instruction)
    }

    fn pc_next_word(&mut self) -> Result<Word, ExecuteErr> {
        let data = self.mem_word(self.pc)?;
        self.pc += libstrmisa::BYTES_PER_WORD as Word;
        Ok(data)
    }

    fn mem_word(&self, addr: Word) -> Result<Word, ExecuteErr> {
        self.memory
            .word(addr)
            .ok_or(ExecuteErr::MemoryAccessViolation)
    }

    fn mem_word_mut(&mut self, addr: Word) -> Result<MemoryWordMutPatch<T>, ExecuteErr> {
        self.memory
            .word_mut(self.current_trace, addr)
            .ok_or(ExecuteErr::MemoryAccessViolation)
    }

    fn mem_byte(&self, addr: Word) -> Result<u8, ExecuteErr> {
        self.memory
            .byte(addr)
            .ok_or(ExecuteErr::MemoryAccessViolation)
    }

    fn mem_byte_mut(&mut self, addr: Word) -> Result<&mut u8, ExecuteErr> {
        self.memory
            .byte_mut(self.current_trace, addr)
            .ok_or(ExecuteErr::MemoryAccessViolation)
    }

    fn reg(&self, index: Register) -> Word {
        self.reg_file.register(index)
    }

    fn reg_mut(&mut self, index: Register) -> &mut Word {
        self.reg_file.register_mut(self.current_trace, index)
    }
}
