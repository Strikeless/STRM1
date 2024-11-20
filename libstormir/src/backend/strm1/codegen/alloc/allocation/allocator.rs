use std::{collections::HashMap, ops::Range};

use anyhow::anyhow;
use itertools::Itertools;
use libisa::Word;

use crate::backend::strm1::codegen::prealloc::VarId;

use super::{usagemap::RangedUsageMap, AllocMap, MemVarAlloc, RegVarAlloc, VarAlloc};

#[derive(Debug, Default)]
pub struct VarAllocator {
    definitions: HashMap<VarId, VarDefinition>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AllocRequirement {
    #[default]
    Generic,

    Register,
    Memory,
}

impl VarAllocator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn define(
        &mut self,
        id: VarId,
        lifetime_start: usize,
        importance: usize,
        alloc_requirement: AllocRequirement,
    ) -> anyhow::Result<()> {
        if self.definitions.contains_key(&id) {
            return Err(anyhow!("Duplicate variable allocation"));
        }

        self.definitions.insert(
            id,
            VarDefinition {
                lifetime: lifetime_start..lifetime_start,
                importance,
                alloc_requirement,
            },
        );

        Ok(())
    }

    pub fn extend_lifetime(&mut self, id: &VarId, lifetime_end: usize) -> anyhow::Result<()> {
        let var = self
            .definitions
            .get_mut(id)
            .ok_or_else(|| anyhow!("Lifetime extension on undefined variable"))?;

        var.lifetime.end = var.lifetime.end.max(lifetime_end);
        Ok(())
    }

    pub fn add_importance(&mut self, id: &VarId, importance: usize) -> anyhow::Result<()> {
        let var = self
            .definitions
            .get_mut(id)
            .ok_or_else(|| anyhow!("Importance addition on undefined variable"))?;

        // Don't overflow or panic if the max importance is crossed.
        var.importance = var.importance.saturating_add(importance);
        Ok(())
    }

    pub fn build(self) -> anyhow::Result<AllocMap> {
        InnerBuilder::new().build(self.definitions)
    }

    pub fn contains_id(&self, id: &VarId) -> bool {
        self.definitions.contains_key(id)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
struct VarDefinition {
    lifetime: Range<usize>,
    importance: usize,
    alloc_requirement: AllocRequirement,
}

struct InnerBuilder {
    reg_usage_map: RangedUsageMap,
    mem_usage_map: RangedUsageMap,
}

impl InnerBuilder {
    pub fn new() -> Self {
        Self {
            reg_usage_map: RangedUsageMap::new(libisa::REGISTER_COUNT).preallocated(),
            mem_usage_map: RangedUsageMap::new(Word::MAX as usize), // TODO: Offset memory accesses in a post-pass, not done here.
        }
    }

    pub fn build(mut self, definitions: HashMap<VarId, VarDefinition>) -> anyhow::Result<AllocMap> {
        let mut id_vars: Vec<_> = definitions.into_iter().collect();

        // First sort the variables by importance, prioritizing high importance.
        id_vars.sort_by_key(|(_, definition)| usize::MAX - definition.importance);

        // ...then by allocation requirements, prioritizing register and de-prioritizing memory requirements.
        // NOTE: This must be the last sort to guarantee allocation requirements are met whenever possible.
        id_vars.sort_by_key(|(_, definition)| match definition.alloc_requirement {
            AllocRequirement::Register => 0,
            AllocRequirement::Generic => 1,
            AllocRequirement::Memory => 2,
        });

        let inner_alloc_map = id_vars
            .into_iter()
            .map(|(id, definition)| self.allocate_var(definition).map(|alloc| (id, alloc)))
            .try_collect()?;

        Ok(AllocMap::new(inner_alloc_map))
    }

    fn allocate_var(&mut self, definition: VarDefinition) -> anyhow::Result<VarAlloc> {
        match definition.alloc_requirement {
            AllocRequirement::Register => self
                .allocate_reg(definition.lifetime)
                .ok_or_else(|| anyhow!("Out of free registers on forced register allocation")),

            AllocRequirement::Memory => self
                .allocate_mem(definition.lifetime)
                .ok_or_else(|| anyhow!("Out of free memory on forced memory allocation")),

            AllocRequirement::Generic => self
                .allocate_reg(definition.lifetime.clone())
                .or_else(|| self.allocate_mem(definition.lifetime))
                .ok_or_else(|| anyhow!("Out of space for variable allocation")),
        }
    }

    fn allocate_reg(&mut self, lifetime: Range<usize>) -> Option<VarAlloc> {
        self.reg_usage_map
            .reserve_free(lifetime)
            .map(|reg_index| VarAlloc::Register(RegVarAlloc(reg_index)))
    }

    fn allocate_mem(&mut self, lifetime: Range<usize>) -> Option<VarAlloc> {
        self.mem_usage_map
            .reserve_free(lifetime)
            .map(|mem_addr| VarAlloc::Memory(MemVarAlloc(mem_addr as Word)))
    }
}
