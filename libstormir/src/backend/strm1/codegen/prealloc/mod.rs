use libisa::{instruction::Instruction as TargetInstruction, Register, Word};

use crate::lir::LIRVarId;

pub mod codegen;

/// The actual identifier part of variables that must be unique for every new variable.
/// Everything on top of this is just type enforcement, and does not affect which allocation is referred to.
///
/// e.g. a [`RegVarKey(VarId::Internal(1))`] may refer to the same allocation as
/// [`VarKey(MemVarKey(VarId::Internal(1)))`], and these two should never coexist.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VarId {
    /// Variable emitted by the compiler for internal use.
    Internal(usize),

    /// Variable defined in the LIR.
    LIR(LIRVarId),
}

/// A variable that must reside in a register.
#[derive(Debug, Clone, Copy)]
pub struct RegVarKey(pub VarId);

/// A variable that must reside in memory.
#[derive(Debug, Clone, Copy)]
pub struct MemVarKey(pub VarId);

#[derive(Debug, Clone, Copy)]
pub enum VarKey {
    /// A variable that has no demands of where to reside.
    Generic(VarId),

    Register(RegVarKey),
    Memory(MemVarKey),
}

/// A code representation based on target machine code that abstracts away some of the register/memory use
/// and register allocation. This is essentially a middle ground between LIR and target machine code, with
/// most of the actual code generation emitting to this and register and memory allocations happening after that.
#[derive(Debug, Clone)]
pub enum PreallocInstruction {
    DefineVar(VarKey),

    /// Demand a register variable reside in the given register index.
    ExplicitRegister {
        var: RegVarKey,
        reg_index: Register,
    },

    /// Demand a memory variable reside in the given memory address.
    ExplicitMemory {
        var: MemVarKey,
        mem_addr: Word,
    },

    /// Load an immediate value to the destination register variable
    LoadImmediate {
        dest: RegVarKey,
        value: Word,
    },

    /// Load a variable to the destination register variable
    LoadVar {
        dest: RegVarKey,
        src: VarKey,
    },

    /// Store the contents of the source register variable in the destination variable
    StoreVar {
        dest: VarKey,
        src: RegVarKey,
    },

    Add(RegVarKey, RegVarKey),
    Sub(RegVarKey, RegVarKey),
    AddC(RegVarKey, RegVarKey),
    SubC(RegVarKey, RegVarKey),

    And(RegVarKey, RegVarKey),

    // TODO: Should high/low byte loads/stores be implemented here (types handled in LIR->prealloc transformation),
    //       or should types be brought into this representation (types handled in prealloc->target transformation)?
    TargetPassthrough {
        instructions: Vec<TargetInstruction>,
    },
}

impl PreallocInstruction {
    /// Returns a list of variable IDs whose data this instruction reads or writes.
    pub fn used_vars(&self) -> Vec<&VarId> {
        match self {
            Self::LoadImmediate { dest: dest_reg, .. } => vec![dest_reg.id()],
            Self::LoadVar {
                dest: dest_reg,
                src: src_var,
            } => vec![dest_reg.id(), src_var.id()],
            Self::StoreVar {
                dest: dest_var,
                src: src_reg,
            } => vec![dest_var.id(), src_reg.id()],

            Self::Add(a, b)
            | Self::Sub(a, b)
            | Self::AddC(a, b)
            | Self::SubC(a, b)
            | Self::And(a, b) => vec![a.id(), b.id()],

            // Not matching the all the rest with _ because I would forget to update this without the compile time error.
            Self::DefineVar(..)
            | Self::ExplicitMemory { .. }
            | Self::ExplicitRegister { .. }
            | Self::TargetPassthrough { .. } => vec![],
        }
    }
}

pub trait VarTrait {
    fn id(&self) -> &VarId;
}

impl VarTrait for VarKey {
    fn id(&self) -> &VarId {
        match self {
            Self::Generic(id) => id,
            Self::Register(reg_var) => reg_var.id(),
            Self::Memory(mem_var) => mem_var.id(),
        }
    }
}

impl VarTrait for RegVarKey {
    fn id(&self) -> &VarId {
        &self.0
    }
}

impl VarTrait for MemVarKey {
    fn id(&self) -> &VarId {
        &self.0
    }
}
