use std::{collections::HashMap, ops::Range};

use anyhow::{anyhow, Context};
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

impl VarAllocationKind {
    pub fn is_register(&self) -> bool {
        matches!(self, VarAllocationKind::Register(..))
    }

    pub fn is_memory(&self) -> bool {
        matches!(self, VarAllocationKind::Memory(..))
    }

    pub fn as_register(&self) -> Option<Register> {
        match self {
            Self::Register(index) => Some(*index),
            _ => None,
        }
    }
}

impl Default for VarTable {
    fn default() -> Self {
        Self {
            allocations: HashMap::default(),
            reg_usage: RangedUsageMap(vec![Vec::new(); libstrmisa::REGISTER_COUNT]),

            // FIXME: It's stupid that we're instantly allocating the usage map for the entire memory space,
            //        but find_free doesn't work as expected if the vector isn't already filled.
            //        Maybe add a maximum size to RangedUsageMap and let it handle this stuff?
            mem_usage: RangedUsageMap(vec![Vec::new(); Word::MAX as usize]),
        }
    }
}

impl VarTable {
    pub fn get(&self, key: VarKey) -> Option<&VarAllocation> {
        self.allocations.get(&key)
    }

    fn from_builder(builder: VarTableBuilder) -> anyhow::Result<Self> {
        let mut this = Self::default();
        this.populate_from_builder(builder)?;
        Ok(this)
    }

    fn populate_from_builder(&mut self, builder: VarTableBuilder) -> anyhow::Result<()> {
        for (key, definition) in builder.definitions {
            let allocation = (self
                .find_free_register(&definition)
                .or_else(|| self.try_steal_register(&definition))
                .map(|index| VarAllocationKind::Register(index)))
            .or_else(|| {
                if definition.needs_register {
                    return None;
                }

                self.find_free_memory(&definition)
                    .map(|addr| VarAllocationKind::Memory(addr))
            })
            .ok_or_else(|| anyhow!("Out of variable space"))?;

            match allocation {
                VarAllocationKind::Register(index) => self
                    .reg_usage
                    .mark_used(index, key, definition.instruction_range()),
                VarAllocationKind::Memory(addr) => self
                    .mem_usage
                    .mark_used(addr as usize, key, definition.instruction_range()),
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
        // Find a register allocation that doesn't need a register and is colder than this definition.
        let cold_key = *self.allocations.iter()
            .filter(|(_, alloc)| alloc.kind.is_register())
            .filter(|(_, alloc)| !alloc.definition.needs_register)
            .filter(|(_, alloc)| definition.needs_register || alloc.definition.heat < definition.heat)
            .min_by_key(|(_, alloc)| alloc.definition.heat)?
            .0;

        let cold = self.allocations.get(&cold_key).unwrap();

        println!("try_steal_register: found cold {:?}", cold_key);

        // Find free memory for the cold variable to be moved to.
        let cold_new_addr = self.find_free_memory(&cold.definition)?;

        // Mark the cold variable's register allocation as free, it'll be reused with the new instruction range later.
        let reg_index = cold.kind.as_register().unwrap();
        self.reg_usage.mark_free(reg_index, cold.definition.key);

        // Allocate the cold variable to the free memory. 
        let cold = self.allocations.get_mut(&cold_key).unwrap();
        cold.kind = VarAllocationKind::Memory(cold_new_addr);
        let cold = self.allocations.get(&cold_key).unwrap();
        self.mem_usage.mark_used(cold_new_addr as usize, cold.definition.key, cold.definition.instruction_range());

        Some(reg_index)
    }

    fn find_free_register(&self, definition: &VarDefinition) -> Option<Register> {
        self.reg_usage.find_free(&definition.instruction_range())
    }

    fn find_free_memory(&self, definition: &VarDefinition) -> Option<Word> {
        self.mem_usage
            .find_free(&definition.instruction_range())
            .map(|index| index as Word)
    }
}

#[derive(Debug, Clone, Default)]
struct RangedUsageMap(Vec<Vec<(VarKey, Range<usize>)>>);

impl RangedUsageMap {
    pub fn mark_used(&mut self, index: usize, key: VarKey, instruction_range: Range<usize>) {
        self.0.get_mut(index).unwrap().push((key, instruction_range));
    }

    pub fn mark_free(&mut self, index: usize, key: VarKey) {
        let usage = self.0.get_mut(index).unwrap();
        
        let index = usage.iter().position(|(used_key, _)| *used_key == key)
            .expect("Free on index-key pair with no allocation");

        usage.remove(index);
    }

    pub fn find_free(&self, instruction_range: &Range<usize>) -> Option<usize> {
        // Find the index of the first element...
        self.0.iter().position(|usage| {
            // where no other usage overlaps at the same instruction range
            usage
                .iter()
                .all(|(_, used_range)| !Self::ranges_overlap(instruction_range, used_range))
        })
    }

    fn ranges_overlap(a: &Range<usize>, b: &Range<usize>) -> bool {
        // I hope this is correct, my brains aren't really cooperating at the moment.
        a.start <= b.end && b.start <= a.end
    }
}
