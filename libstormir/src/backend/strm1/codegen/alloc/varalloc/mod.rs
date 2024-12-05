use std::collections::HashMap;

use libisa::{Register, Word};
use serde::{Deserialize, Serialize};

use crate::backend::strm1::codegen::prealloc::{MemVarKey, RegVarKey, VarId, VarKey, VarTrait};

pub mod allocator;
mod neumannpass;
mod usagemap;

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegVarAlloc(pub Register);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemVarAlloc(pub Word);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VarAlloc {
    Register(RegVarAlloc),
    Memory(MemVarAlloc),
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AllocMap(HashMap<VarId, VarAlloc>);

impl AllocMap {
    fn new(inner: HashMap<VarId, VarAlloc>) -> Self {
        Self(inner)
    }

    pub fn get(&self, key: &VarKey) -> Option<&VarAlloc> {
        self.get_by_id(key.id())
    }

    pub fn get_by_id(&self, id: &VarId) -> Option<&VarAlloc> {
        self.0.get(id)
    }

    pub fn get_reg(&self, key: &RegVarKey) -> Option<&RegVarAlloc> {
        match self.0.get(key.id()) {
            Some(VarAlloc::Register(reg_alloc)) => Some(reg_alloc),
            Some(_) => panic!("RegVarKey corresponds to a non-register allocation"),
            None => None,
        }
    }

    #[allow(unused)] // It's here for consistency with get_reg and potential future use.
    pub fn get_mem(&self, key: &MemVarKey) -> Option<&MemVarAlloc> {
        match self.0.get(key.id()) {
            Some(VarAlloc::Memory(mem_alloc)) => Some(mem_alloc),
            Some(_) => panic!("MemVarKey corresponds to a non-memory allocation"),
            None => None,
        }
    }
}
