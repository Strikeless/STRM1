use anyhow::anyhow;
use builder::{VarDefinition, VarTableBuilder};
use indexmap::IndexMap;
use libisa::{Register, Word};
use serde::{Deserialize, Serialize};
use usagemap::RangedUsageMap;

use crate::lir::LIRVarId;

pub mod builder;
mod usagemap;

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VarTable {
    pub allocations: IndexMap<VarKey, VarAllocation>,

    reg_usage: RangedUsageMap,
    mem_usage: RangedUsageMap,
}

impl Default for VarTable {
    fn default() -> Self {
        Self {
            allocations: IndexMap::default(),
            reg_usage: RangedUsageMap::new(libisa::REGISTER_COUNT).preallocated(),

            // FIXME: The usable memory region for variables should probably start from the end of the program code.
            //        This is not the way to go for sure, larger programs will break.
            mem_usage: RangedUsageMap::new_with_usable_region(1024..libisa::Word::MAX as usize),
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
            // cargo fmt has ruined it.
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

            let instruction_range = definition.instruction_range();

            match allocation {
                VarAllocationKind::Register(reg_index) => {
                    self.reg_usage.reserve(reg_index, instruction_range, key)
                }
                VarAllocationKind::Memory(mem_addr) => {
                    self.mem_usage
                        .reserve(mem_addr as usize, instruction_range, key)
                }
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
        // Find a register allocation that doesn't need a register and is colder than this definition or needs a register itself.
        let cold_key = *self
            .allocations
            .iter()
            .filter(|(_, alloc)| alloc.kind.is_register())
            .filter(|(_, alloc)| !alloc.definition.needs_register)
            .filter(|(_, alloc)| {
                definition.needs_register || alloc.definition.heat < definition.heat
            })
            .min_by_key(|(_, alloc)| alloc.definition.heat)?
            .0;

        let cold = self.allocations.get(&cold_key).unwrap();

        // HACK: Quick hack to use find_free_memory after it was changed to require mutable access.
        //       This method could made to look a lot nicer by changing some of the method parameters to be more specific.
        let cold_definition = cold.definition.clone();

        // Find free memory for the cold variable to be moved to.
        let cold_new_addr = self.find_free_memory(&cold_definition)?;

        let cold = self.allocations.get(&cold_key).unwrap();

        // Mark the cold variable's register allocation as free, it'll be reused with the new instruction range later.
        let reg_index = cold.kind.as_register().unwrap();
        self.reg_usage.free(
            reg_index,
            &cold.definition.instruction_range(),
            &cold.definition.key,
        );

        //
        // Allocate the cold variable to the free memory.
        //
        let cold = self.allocations.get_mut(&cold_key).unwrap();
        cold.kind = VarAllocationKind::Memory(cold_new_addr);

        let cold = self.allocations.get(&cold_key).unwrap();
        self.mem_usage.reserve(
            cold_new_addr as usize,
            cold.definition.instruction_range(),
            cold.definition.key,
        );

        Some(reg_index)
    }

    fn find_free_register(&mut self, definition: &VarDefinition) -> Option<Register> {
        self.reg_usage.free_slot(&definition.instruction_range())
    }

    fn find_free_memory(&mut self, definition: &VarDefinition) -> Option<Word> {
        self.mem_usage
            .free_slot(&definition.instruction_range())
            .map(|index| index as Word)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum VarKey {
    Normal(LIRVarId),
    Special(LIRVarId),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct VarAllocation {
    pub kind: VarAllocationKind,
    definition: VarDefinition,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VarAllocationKind {
    Register(Register),
    Memory(Word),
}

impl VarAllocationKind {
    pub fn is_register(&self) -> bool {
        matches!(self, VarAllocationKind::Register(..))
    }

    #[allow(dead_code)] // Shut up
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
