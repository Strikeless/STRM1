use std::{collections::HashMap, ops::Range};

use anyhow::anyhow;
use builder::{VarDefinition, VarTableBuilder};
use libstrmisa::{Register, Word};

use crate::lir::LIRVarId;

pub mod builder;

#[cfg(test)]
mod tests;

#[derive(Debug, Clone)]
pub struct VarTable {
    allocations: HashMap<VarKey, VarAllocation>,

    reg_usage: RangedUsageMap,
    mem_usage: RangedUsageMap,
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

impl Default for VarTable {
    fn default() -> Self {
        Self {
            allocations: HashMap::default(),
            reg_usage: RangedUsageMap(vec![Vec::new(); libstrmisa::REGISTER_COUNT]),

            // FIXME: It's stupid that we're instantly allocating the usage map for the entire memory space,
            //        but find_free doesn't work as expected if the vector isn't already filled.
            //        Maybe add a maximum size to RangedUsageMap and let it handle this stuff?
            mem_usage: RangedUsageMap(vec![Vec::new(); Word::MAX as usize])
        }
    }
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
                /*.or_else(|| {
                    self.try_steal_register(&definition)
                        .map(|index| VarAllocationKind::Register(index))
                })*/
                .or_else(|| {
                    if definition.needs_register {
                        return None;
                    }

                    self.find_free_memory(&definition)
                        .map(|addr| VarAllocationKind::Memory(addr as Word))
                })
                .ok_or_else(|| anyhow!("Out of variable space"))?;

            match allocation {
                VarAllocationKind::Register(index) => self.reg_usage.mark_used(index, definition.instruction_range()),
                VarAllocationKind::Memory(addr) => self.mem_usage.mark_used(addr as usize, definition.instruction_range()),
            }

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
        // Don't forget to handle the usage maps when implementing this!
        None
    }

    fn find_free_register(&self, definition: &VarDefinition) -> Option<Register> {
        self.reg_usage
            .find_free(&definition.instruction_range())
    }

    fn find_free_memory(&self, definition: &VarDefinition) -> Option<Word> {
        self.mem_usage
            .find_free(&definition.instruction_range())
            .map(|index| index as Word)
    }
}

#[derive(Debug, Clone, Default)]
struct RangedUsageMap(Vec<Vec<Range<usize>>>);

impl RangedUsageMap {
    pub fn mark_used(&mut self, index: usize, instruction_range: Range<usize>) {
        self.0.get_mut(index).unwrap()
            .push(instruction_range);
    }

    pub fn find_free(&self, instruction_range: &Range<usize>) -> Option<usize> {
        // Find the index of the first element...
        self.0.iter().position(|usage| {
            // where no other usage overlaps at the same instruction range
            usage
                .iter()
                .all(|used_range| !Self::ranges_overlap(instruction_range, used_range))
        })
    }

    fn ranges_overlap(a: &Range<usize>, b: &Range<usize>) -> bool {
        // I hope this is correct, my brains aren't really cooperating at the moment.
        a.start <= b.end && b.start <= a.end
    }
}
