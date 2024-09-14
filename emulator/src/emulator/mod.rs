use std::iter;

use alu::ALU;
use memory::patcher::MemoryPatcher;

pub type Word = u16;
pub type AddressableUnit = u8;

pub const MEMORY_SIZE: usize = 2usize.pow(16);
pub const GPR_COUNT: usize = 2usize.pow(4);
pub const AUS_PER_WORD: u32 = Word::BITS / AddressableUnit::BITS;

mod alu;
mod execute;
mod flags;
mod memory;

pub struct Emulator {
    pub memory: [AddressableUnit; MEMORY_SIZE],
    pub memory_patcher: MemoryPatcher,

    pub gpr_file: [Word; GPR_COUNT],
    pub program_counter: Word,
    pub alu: ALU,
}

impl Emulator {
    pub fn new(bios: &[AddressableUnit]) -> Self {
        if bios.len() > MEMORY_SIZE {
            // TODO: Proper error handling
            panic!(
                "BIOS ({} words) too large for memory space ({} words)!",
                bios.len(),
                MEMORY_SIZE
            );
        }

        // This is stupid.
        let memory = bios
            .into_iter()
            .map(|byte_ref| *byte_ref)
            .chain(iter::repeat(0))
            .take(MEMORY_SIZE)
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();

        Self {
            memory,
            memory_patcher: MemoryPatcher::new(),
            gpr_file: [0; GPR_COUNT],
            program_counter: 0,
            alu: ALU::new(),
        }
    }
}
