#![feature(array_chunks, array_windows, trait_alias)]

mod alu;
mod execute;
mod volatile;
mod tracing;
mod volatilehelper;

use alu::ALU;
use anyhow::Context;
use libisa::{
    instruction::{Instruction, InstructionDeassemblyError},
    Word,
};
use thiserror::Error;
use tracing::{EmulatorIterationTrace, EmulatorTracing};
use volatile::Volatile;

pub struct Emulator {
    pub memory: Volatile<u8, Word>,
    pub reg_file: Volatile<Word, usize>,
    pub tracing: EmulatorTracing,

    pub alu: ALU,
    pub pc: Word,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecuteOk {
    Normal,
    Halted,
}

#[derive(Debug, Error, Clone, Copy, PartialEq, Eq)]
pub enum ExecuteErr {
    #[error("Memory access violation to 0x{0:04x}")]
    MemoryAccessViolation(Word),

    #[error("Illegal instruction ({0})")]
    IllegalInstruction(InstructionDeassemblyError),
}

impl Emulator {
    pub fn new(program: Vec<u8>) -> anyhow::Result<Self> {
        Ok(Self {
            memory: Volatile::new_with_data(program, Word::MAX)
                .with_context(|| "Loading program to memory")?,

            reg_file: Volatile::new(libisa::REGISTER_COUNT),

            tracing: EmulatorTracing::default(),

            alu: ALU::new(),
            pc: 0,
        })
    }

    pub fn execute_to_halt(&mut self) -> Result<(), ExecuteErr> {
        loop {
            let exec_ok = self.execute_instruction()?;

            if exec_ok == ExecuteOk::Halted {
                return Ok(());
            }
        }
    }

    pub fn execute_instruction(&mut self) -> Result<ExecuteOk, ExecuteErr> {
        let instruction_pc = self.pc;
        let instruction = self.parse_next_instruction()?;
        let exec_result = self.execute_parsed_instruction(instruction);

        let memory_patches = self.memory.pop_patches().collect();
        let register_patches = self.reg_file.pop_patches().collect();

        self.tracing.add_iteration_trace(instruction_pc, EmulatorIterationTrace {
            memory_patches,
            register_patches,
        });

        exec_result
    }

    fn parse_next_instruction(&mut self) -> Result<Instruction, ExecuteErr> {
        let instruction_word = self.pc_next()?;

        let mut instruction = Instruction::deassemble_instruction_word(instruction_word)
            .map_err(|e| ExecuteErr::IllegalInstruction(e))?;

        if instruction.kind.has_immediate() {
            let immediate_word = self.pc_next()?;
            instruction.immediate = Some(immediate_word);
        }

        Ok(instruction)
    }

    fn pc_next(&mut self) -> Result<Word, ExecuteErr> {
        let pc_word = self.mem_word_or_err(self.pc)?;

        self.pc = self
            .pc
            .wrapping_add_signed(libisa::BYTES_PER_WORD as libisa::WordSigned);

        Ok(*pc_word)
    }
}
