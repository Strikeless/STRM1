pub mod shim;

pub type LIRVarId = usize;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LIRInstruction {
    /// Declare variable with the given ID and constant value.
    Const {
        id: LIRVarId,
        value: LIRValue,
    },

    /// Declare variable with the given ID and value copied from the source variable.
    Copy {
        id: LIRVarId,
        src: LIRVarId,
    },

    Add {
        id: LIRVarId,
        a: LIRVarId,
        b: LIRVarId,
    },
    Sub {
        id: LIRVarId,
        a: LIRVarId,
        b: LIRVarId,
    },
    Mul {
        id: LIRVarId,
        a: LIRVarId,
        b: LIRVarId,
    },

    Branch {
        addr: LIRVarId,
    },
    BranchZero {
        addr: LIRVarId,
        test: LIRVarId,
    },
    // TODO: How should a carry branch be implemented?
    BranchEqual {
        addr: LIRVarId,
        a: LIRVarId,
        b: LIRVarId,
    },

    /// Native machine code passthrough. May or may not be validated.
    NativeMachinecode {
        code: Vec<u8>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LIRValue {
    Uint8(u8),
    Uint16(u16),
}

// That use<'_> bound is some black magic that tells Rust that the iterator's lifetime depends on the lir reference.
pub fn free_var_ids(lir: &Vec<LIRInstruction>) -> impl Iterator<Item = LIRVarId> + use<'_> {
    // Horribly inefficient implementation if iterated on a lot.
    (0..LIRVarId::MAX).filter(|id| {
        !lir.iter()
            .any(|instruction| instruction.introduced_var_ids().contains(&id))
    })
}

impl LIRInstruction {
    pub fn introduced_var_ids(&self) -> Vec<&LIRVarId> {
        match self {
            Self::Const { id, .. }
            | Self::Copy { id, .. }
            | Self::Add { id, .. }
            | Self::Sub { id, .. }
            | Self::Mul { id, .. } => vec![id],
            _ => vec![],
        }
    }
}
