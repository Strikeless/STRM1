use std::collections::HashMap;

use libisa::{Register, Word};

use crate::volatile::patch::VolatilePatch;

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
    pub memory_patches: HashMap<Word, VolatilePatch<u8>>,
    pub register_patches: HashMap<Register, VolatilePatch<Word>>,
}

impl EmulatorTracing {
    pub fn trace_by_pc(&self, pc: Word) -> Option<&EmulatorTrace> {
        self.traces_by_pc.get(&pc)
    }

    pub fn iterations_by_pc(&self, pc: Word) -> impl Iterator<Item = &EmulatorIterationTrace> + use<'_> {
        (0..pc) // Iterate over every PC value up to the specified value.
            .filter_map(|pc| self.trace_by_pc(pc)) // Get traces for the PC values.
            .flat_map(|trace| &trace.iterations) // Get all the trace iterations.
    }

    pub fn register_by_pc(&self, pc: Word, index: Register) -> Option<&Word> {
        self.iterations_by_pc(pc) // Get all iteration traces up to the PC.
            .filter_map(|iter_trace| iter_trace.register_patches.get(&index)) // Filter and map the traces to register patches on the specified register.
            .map(|memory_patch| &memory_patch.new_value) // Map the patch to the new value it applies.
            .last() // Get the latest value.
    }

    pub fn memory_byte_by_pc(&self, pc: Word, addr: Word) -> Option<&u8> {
        self.iterations_by_pc(pc) // Get all iteration traces up to the PC.
            .filter_map(|iter_trace| iter_trace.memory_patches.get(&addr)) // Filter and map the traces to memory patches on the specified register.
            .map(|memory_patch| &memory_patch.new_value) // Map the patch to the new value it applies.
            .last() // Get the latest value.
    }

    pub fn memory_word_by_pc(&self, pc: Word, addr: Word) -> Option<Word> {
        let lower_byte = *self.memory_byte_by_pc(pc, addr)?;
        let upper_byte = *self.memory_byte_by_pc(pc, addr + 1)?;

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
