use std::collections::HashMap;

use libisa::{Register, Word};

use crate::backend::strm1::codegen::prealloc::{MemVarKey, RegVarKey, VarId, VarKey, VarTrait};

pub mod allocator;
mod usagemap;

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RegVarAlloc(Register);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MemVarAlloc(Word);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VarAlloc {
    Register(RegVarAlloc),
    Memory(MemVarAlloc),
}

#[derive(Debug, Clone)]
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

    pub fn get_mem(&self, key: &MemVarKey) -> Option<&MemVarAlloc> {
        match self.0.get(key.id()) {
            Some(VarAlloc::Memory(mem_alloc)) => Some(mem_alloc),
            Some(_) => panic!("MemVarKey corresponds to a non-memory allocation"),
            None => None,
        }
    }
}
