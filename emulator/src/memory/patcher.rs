use std::collections::HashMap;

use crate::Word;

// Slow and stupid thing made so I'll never ever have to worry about byte endian again while working on this emulator

#[derive(Clone, Default)]
pub struct MemoryPatcher {
    mutable_values: HashMap<Word, Word>,
    immutable_values: HashMap<Word, Word>,
}

impl MemoryPatcher {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn sync(&mut self, memory: &mut [u8; crate::MEMORY_SIZE]) {
        if self.mutable_values.len() > 1 || self.immutable_values.len() > 1 {
            println!("mem patcher {} {}", self.mutable_values.len(), self.immutable_values.len());
        }

        let mutable_values: Vec<_> = self.mutable_values.drain().collect();

        for (addr, value) in mutable_values.into_iter() {
            self.direct_set(memory, addr, value);
        }

        self.immutable_values.clear();
    }

    pub fn get<'a>(&'a mut self, memory: &[u8; crate::MEMORY_SIZE], addr: Word) -> &'a Word {
        let value = self.direct_get(memory, addr);

        self.immutable_values.insert(addr, value);
        self.immutable_values.get(&addr).unwrap()
    }

    pub fn get_mut<'a>(
        &'a mut self,
        memory: &[u8; crate::MEMORY_SIZE],
        addr: Word,
    ) -> &'a mut Word {
        let value = self.direct_get(memory, addr);

        self.mutable_values.insert(addr, value);
        self.mutable_values.get_mut(&addr).unwrap()
    }

    fn direct_get(&self, memory: &[u8; crate::MEMORY_SIZE], addr: Word) -> Word {
        if self.mutable_values.contains_key(&addr) {
            return self.mutable_values[&addr];
        }

        let [high_addr, low_addr] = Self::map_addr(addr);

        let high = memory[high_addr];
        let low = memory[low_addr];

        ((high as u16) << 8) | low as u16
    }

    fn direct_set(&self, memory: &mut [u8; crate::MEMORY_SIZE], addr: Word, value: Word) {
        let high = value >> 8;
        let low = value & 0xFF;

        let [high_addr, low_addr] = Self::map_addr(addr);

        memory[high_addr] = high as u8;
        memory[low_addr] = low as u8;
    }

    fn map_addr(addr: Word) -> [usize; 2] {
        [addr as usize, (addr as usize + 1) % crate::MEMORY_SIZE]
    }
}
