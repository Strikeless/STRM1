use std::ops::Range;

use anyhow::anyhow;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use super::{VarKey, VarTable};

pub struct VarTableBuilder {
    pub(super) definitions: IndexMap<VarKey, VarDefinition>,
    current_index: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub(super) struct VarDefinition {
    pub key: VarKey,
    pub begin: usize,
    pub end: Option<usize>,

    pub needs_register: bool,
    pub heat: usize,
}

impl VarDefinition {
    pub fn instruction_range(&self) -> Range<usize> {
        self.begin..self.end.unwrap_or(usize::MAX)
    }
}

impl VarTableBuilder {
    pub fn new() -> Self {
        Self {
            definitions: IndexMap::new(),
            current_index: 0,
        }
    }

    pub fn set_current_index(&mut self, index: usize) {
        self.current_index = index;
    }

    pub fn define(&mut self, key: VarKey, needs_register: bool) -> anyhow::Result<()> {
        if self.definitions.contains_key(&key) {
            return Err(anyhow!("Reused variable key"));
        }

        self.definitions.insert(
            key,
            VarDefinition {
                key,
                begin: self.current_index,
                end: None,
                needs_register,
                heat: 0,
            },
        );

        Ok(())
    }

    pub fn drop(&mut self, key: VarKey, offset: usize) -> anyhow::Result<()> {
        let index = self.current_index + offset;

        let def = self.def_mut(key)?;
        def.end = Some(index);

        Ok(())
    }

    pub fn heaten(&mut self, key: VarKey) -> anyhow::Result<()> {
        let def = self.def_mut(key)?;
        def.heat += 1;
        Ok(())
    }

    pub fn build(self) -> anyhow::Result<VarTable> {
        VarTable::from_builder(self)
    }

    fn def_mut(&mut self, key: VarKey) -> anyhow::Result<&mut VarDefinition> {
        self.definitions
            .get_mut(&key)
            .ok_or_else(|| anyhow!("Undefined variable"))
    }
}
