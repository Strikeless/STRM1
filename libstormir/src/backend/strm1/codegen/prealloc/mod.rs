use libisa::{Register, Word};

use crate::lir::LIRVarId;

pub mod codegen;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VarSpace {
    /// Variable emitted by the compiler for internal use.
    Internal(usize),

    /// Variable defined in the LIR.
    LIR(LIRVarId),
}

/// A variable that must reside in a register.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RegisterVar(VarSpace);

/// A variable that must reside in memory.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MemoryVar(VarSpace);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Var {
    /// A variable that has no demands of where to reside.
    Generic(VarSpace),

    Register(RegisterVar),
    Memory(MemoryVar),
}

/// A code representation based on target machine code that abstracts away some of the register/memory use
/// and register allocation. This is essentially a middle ground between LIR and target machine code, with
/// most of the actual code generation emitting to this and register and memory allocations happening after that.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PreallocInstruction {
    /// Demand a register variable reside in the given register index.
    ExplicitRegister {
        index: Register,
    },

    /// Demand a memory variable reside in the given memory address.
    ExplicitMemory {
        addr: Word,
    },

    /// Load an immediate value to the destination register variable
    LoadImmediate {
        dest_reg: RegisterVar,
        value: Word,
    },

    /// Load a variable to the destination register variable
    LoadVar {
        dest_reg: RegisterVar,
        src_var: Var,
    },

    /// Store the contents of the source register variable in the destination variable
    StoreVar {
        dest_var: Var,
        src_reg: RegisterVar,
    },

    Add {
        a_reg: RegisterVar,
        b_reg: RegisterVar,
    },
    Sub {
        a_reg: RegisterVar,
        b_reg: RegisterVar,
    },
    AddC {
        a_reg: RegisterVar,
        b_reg: RegisterVar,
    },
    SubC {
        a_reg: RegisterVar,
        b_reg: RegisterVar,
    },

    And {
        a_reg: RegisterVar,
        b_reg: RegisterVar,
    },

    // Halt should not be emitted by the compiler, thus not implemented here.

    // TODO: Should high/low byte loads/stores be implemented here (types handled in LIR->prealloc transformation),
    //       or should types be brought into this representation (types handled in prealloc->target transformation)?

    NativeMachinecode {
        code: Vec<u8>,
    },
}
