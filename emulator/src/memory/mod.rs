pub mod patcher;

use super::{Emulator, Word};

impl Emulator {
    pub fn memory(&mut self, addr: Word) -> &Word {
        self.memory_patcher.sync(&mut self.memory);
        self.memory_patcher.get(&self.memory, addr)
    }

    pub fn memory_mut(&mut self, addr: Word) -> &mut Word {
        self.memory_patcher.sync(&mut self.memory);
        self.memory_patcher.get_mut(&self.memory, addr)
    }

    pub fn register(&self, index: Word) -> &Word {
        &self.gpr_file[index as usize]
    }

    pub fn register_mut(&mut self, index: Word) -> &mut Word {
        &mut self.gpr_file[index as usize]
    }
}
