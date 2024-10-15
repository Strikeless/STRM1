use libstrmisa::{
    instruction::{Instruction, InstructionDeassemblyError},
    Word,
};
use thiserror::Error;

use crate::{memory::wordmut::MemoryWordMutPatch, Emulator};

mod parsed;

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

impl Emulator {
    pub fn execute_instruction(&mut self) -> Result<ExecuteOk, ExecuteErr> {
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

    fn mem_word_mut(&mut self, addr: Word) -> Result<MemoryWordMutPatch, ExecuteErr> {
        self.memory
            .word_mut(addr)
            .ok_or(ExecuteErr::MemoryAccessViolation)
    }

    fn mem_byte(&self, addr: Word) -> Result<u8, ExecuteErr> {
        self.memory
            .byte(addr)
            .ok_or(ExecuteErr::MemoryAccessViolation)
    }

    fn mem_byte_mut(&mut self, addr: Word) -> Result<&mut u8, ExecuteErr> {
        self.memory
            .byte_mut(addr)
            .ok_or(ExecuteErr::MemoryAccessViolation)
    }
}
