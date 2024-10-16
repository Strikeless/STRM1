use std::{collections::HashMap, ops::Range};

use anyhow::anyhow;
use builder::{VarDefinition, VarTableBuilder};
use libstrmisa::{Register, Word};

use crate::lir::LIRVarId;

pub mod builder;

#[derive(Debug, Clone, Default)]
pub struct VarTable {
    allocations: HashMap<VarKey, VarAllocation>,

    reg_usage: TimedUsageMap,
    mem_usage: TimedUsageMap,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VarKey {
    Normal(LIRVarId),
    Special(LIRVarId),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VarAllocation {
    definition: VarDefinition,
    kind: VarAllocationKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VarAllocationKind {
    Register(Register),
    Memory(Word),
}

impl VarTable {
    fn from_builder(builder: VarTableBuilder) -> anyhow::Result<Self> {
        let mut this = Self::default();
        this.populate_from_builder(builder)?;
        Ok(this)
    }

    fn populate_from_builder(&mut self, builder: VarTableBuilder) -> anyhow::Result<()> {
        for (key, definition) in builder.definitions {
            let allocation = self
                .find_free_register(&definition)
                .map(|index| VarAllocationKind::Register(index))
                .or_else(|| {
                    self.try_steal_register(&definition)
                        .map(|index| VarAllocationKind::Register(index))
                })
                .or_else(|| {
                    if definition.needs_register {
                        return None;
                    }

                    self.find_free_memory(&definition)
                        .map(|addr| VarAllocationKind::Memory(addr as Word))
                })
                .ok_or_else(|| anyhow!("Out of variable space"))?;

            self.allocations.insert(
                key,
                VarAllocation {
                    definition,
                    kind: allocation,
                },
            );
        }

        Ok(())
    }

    fn try_steal_register(&mut self, definition: &VarDefinition) -> Option<Register> {
        // TODO
        None
    }

    fn find_free_register(&self, definition: &VarDefinition) -> Option<Register> {
        let time_range = definition.begin..definition.end.unwrap_or(usize::MAX);
        self.reg_usage.find_free(&time_range)
    }

    fn find_free_memory(&self, definition: &VarDefinition) -> Option<Word> {
        let time_range = definition.begin..definition.end.unwrap_or(usize::MAX);

        self.mem_usage
            .find_free(&time_range)
            .map(|index| index as Word)
    }
}

#[derive(Debug, Clone, Default)]
struct TimedUsageMap(Vec<Vec<Range<usize>>>);

impl TimedUsageMap {
    pub fn find_free(&self, time_range: &Range<usize>) -> Option<usize> {
        // Find the index of the first element...
        self.0.iter().position(|usage| {
            // where no other usage overlaps at the same time range
            usage
                .iter()
                .all(|used_time_range| !Self::ranges_overlap(time_range, used_time_range))
        })
    }

    fn ranges_overlap(a: &Range<usize>, b: &Range<usize>) -> bool {
        // I hope this is correct, my brains aren't really cooperating at the moment.
        a.start <= b.end && b.start <= a.end
    }
}
