use std::iter;

use alu::ALU;
use anyhow::anyhow;
use libstrmisa::Word;
use memory::Memory;

pub mod alu;
pub mod execute;
pub mod memory;

pub struct Emulator {
    pub memory: Memory,
    pub reg_file: [Word; libstrmisa::REGISTER_COUNT],
    pub alu: ALU,
    pub pc: Word,
}

impl Emulator {
    pub fn new(memory_size: Word, program: Vec<u8>) -> anyhow::Result<Self> {
        if program.len() > memory_size as usize {
            return Err(anyhow!("Program doesn't fit into memory of specified size"));
        }

        let memory_data = program
            .into_iter()
            .chain(iter::repeat(0))
            .take(memory_size as usize)
            .collect();

        Ok(Self {
            memory: Memory::new(memory_data),
            reg_file: [0; libstrmisa::REGISTER_COUNT],
            alu: ALU::new(),
            pc: 0,
        })
    }

    pub fn register(&self, index: usize) -> Word {
        *self
            .reg_file
            .get(index)
            .expect("Out of bounds register access")
    }

    pub fn register_mut(&mut self, index: usize) -> &mut Word {
        self.reg_file
            .get_mut(index)
            .expect("Out of bounds register access")
    }
}
