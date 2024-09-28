use std::collections::HashMap;

use anyhow::{anyhow, Context};

use super::target;

pub struct VarAllocator {
    reg_bitmap: [bool; target::REG_COUNT],
    mem_bitmap: [bool; target::MEM_LENGTH], // This is not optimal, certainly a memory hog if that ever becomes a problem.
    allocations: HashMap<VarKey, VarAllocation>,
}

impl VarAllocator {
    pub fn new() -> Self {
        Self {
            reg_bitmap: [false; target::REG_COUNT],
            mem_bitmap: [false; target::MEM_LENGTH],
            allocations: HashMap::new(),
        }
    }

    pub fn allocate(&mut self, id: VarKey, heat: usize) -> anyhow::Result<()> {
        if self.allocations.contains_key(&id) {
            return Err(anyhow!(
                "Variable with the same id has already been allocated"
            ));
        }

        let reg_var = self
            .free_reg()
            .or_else(|| self.steal_reg(heat).ok())
            .map(|reg| Var::Register { reg });

        let var = reg_var
            .or_else(|| self.free_mem().map(|addr| Var::Memory { addr }))
            .context("No free registers or memory for variable")?;

        self.allocations.insert(id, VarAllocation {
            var,
            heat,
            freed: false,
        });
        Ok(())
    }

    pub fn free(&mut self, id: VarKey) -> anyhow::Result<()> {
        let allocation = self.allocations.get_mut(&id).context("Free called with unrecognized variable id")?;

        match allocation.var {
            Var::Register { reg } => self.reg_bitmap[reg] = false,
            Var::Memory { addr } => self.mem_bitmap[addr] = false,
        }

        allocation.freed = true;
        Ok(())
    }

    pub fn vars(&mut self) -> HashMap<VarKey, Var> {
        self.allocations.iter()
            .map(|(key, var_alloc)| (*key, var_alloc.var))
            .collect()
    }

    fn free_reg(&self) -> Option<usize> {
        self.reg_bitmap.iter().position(|used| !used)
    }

    fn free_mem(&self) -> Option<usize> {
        self.mem_bitmap.iter().position(|used| !used)
    }

    fn steal_reg(&mut self, heat: usize) -> anyhow::Result<usize> {
        let cold_new_addr = self.free_mem()
            .context("No free memory to move colder variable to")?;

        let cold_alloc = self.allocations.values_mut()
            .filter(|var_alloc| var_alloc.heat < heat)
            .filter(|var_alloc| !var_alloc.freed)
            .filter(|var_alloc| var_alloc.var.in_register())
            .min_by_key(|var_alloc| var_alloc.heat)
            .context("No colder variables")?;

        let Var::Register { reg: stolen_reg } = cold_alloc.var else {
            panic!()
        };

        cold_alloc.var = Var::Memory { addr: cold_new_addr };
        Ok(stolen_reg)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VarKey {
    Code(usize),
    Compiler(usize),
}

struct VarAllocation {
    var: Var,
    heat: usize,
    freed: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Var {
    Register { reg: usize },
    Memory { addr: usize },
}

impl Var {
    pub fn in_register(&self) -> bool {
        match self {
            Self::Register { .. } => true,
            Self::Memory { .. } => false,
        }
    }

    pub fn in_memory(&self) -> bool {
        match self {
            Self::Memory { .. } => true,
            Self::Register { .. } => false,
        }
    }
}
