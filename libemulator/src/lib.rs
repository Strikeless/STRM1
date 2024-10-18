use std::iter;

use alu::ALU;
use anyhow::anyhow;
use libstrmisa::Word;
use memory::Memory;
use regfile::RegFile;
use tracing::TraceData;

pub mod alu;
pub mod execute;
pub mod memory;
pub mod regfile;
pub mod tracing;

pub struct Emulator<T>
where
    T: TraceData,
{
    pub memory: Memory<T>,
    pub reg_file: RegFile<T>,
    pub alu: ALU,
    pub pc: Word,

    current_trace: T::Trace,
}

impl<T> Emulator<T>
where
    T: TraceData,
{
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
            reg_file: RegFile::new(),
            alu: ALU::new(),
            pc: 0,
            current_trace: T::Trace::default(),
        })
    }
}
