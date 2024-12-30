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
    pub memory_patches: Vec<MemoryPatch>,
}

impl EmulatorTracing {
    pub fn trace_by_pc(&self, pc: Word) -> Option<&EmulatorTrace> {
        self.traces_by_pc.get(&pc)
    }

    pub(super) fn add_iteration_trace(&mut self, pc: Word, iteration_trace: EmulatorIterationTrace) {
        let trace = self.traces_by_pc.entry(pc).or_default();
        trace.iterations.push(iteration_trace);
    }
}

impl EmulatorTrace {
    pub fn iteration(&self, iteration: usize) -> Option<&EmulatorIterationTrace> {
        self.iterations.get(iteration)
    }
}
