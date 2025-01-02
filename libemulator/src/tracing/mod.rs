use std::collections::HashMap;

use libisa::Word;

use crate::memory::MemoryPatch;

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct EmulatorTracing {
    pub traces_by_pc: HashMap<Word, EmulatorTrace>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct EmulatorTrace {
    pub iterations: Vec<EmulatorIterationTrace>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmulatorIterationTrace {
    pub memory_patches: HashMap<Word, MemoryPatch>,
}

impl EmulatorTracing {
    pub fn trace_by_pc(&self, pc: Word) -> Option<&EmulatorTrace> {
        self.traces_by_pc.get(&pc)
    }

    pub fn memory_byte_by_pc(&self, pc: Word, addr: Word) -> Option<u8> {
        for possible_patch_pc in (0..pc).rev() {
            let Some(trace) = self.trace_by_pc(possible_patch_pc) else {
                continue;
            };

            for possible_patch_iteration in trace.iterations.iter().rev() {
                let Some(patch) = possible_patch_iteration.memory_patches.get(&addr) else {
                    continue;
                };

                return Some(patch.new_value);
            }
        }

        None
    }

    pub fn memory_word_by_pc(&self, pc: Word, addr: Word) -> Option<Word> {
        let lower_byte = self.memory_byte_by_pc(pc, addr)?;
        let upper_byte = self.memory_byte_by_pc(pc, addr + 1)?;

        Some(libisa::bytes_to_word([lower_byte, upper_byte]))
    }

    pub(super) fn add_iteration_trace(
        &mut self,
        pc: Word,
        iteration_trace: EmulatorIterationTrace,
    ) {
        let trace = self.traces_by_pc.entry(pc).or_default();
        trace.iterations.push(iteration_trace);
    }
}

impl EmulatorTrace {
    pub fn iteration(&self, iteration: usize) -> Option<&EmulatorIterationTrace> {
        self.iterations.get(iteration)
    }
}
