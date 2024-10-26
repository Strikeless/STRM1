pub type LIRVarId = usize;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LIRInstruction {
    /// Define variable with the given ID.
    DefineVar { id: LIRVarId },

    /// Mark variable as unused and to be deallocated.
    DropVar { id: LIRVarId },

    /// Load constant value to input accumulator A.
    LoadIAConst { value: u16 },

    /// Load constant value to input accumulator B.
    LoadIBConst { value: u16 },

    /// Load value of variable to input accumulator A.
    LoadIAVar { id: LIRVarId },

    /// Load value of variable to input accumulator B.
    LoadIBVar { id: LIRVarId },

    /// Load current instruction address to input accumulator A.
    LoadIALabel,

    /// Load current instruction address to input accumulator B.
    LoadIBLabel,

    /// Store value of output accumulator to variable.
    /// NOTE: This instruction is allowed to forget loaded accumulators. This may simplify backend codegen.
    ///       Always load input accumulators again after a store if needed. You're on your own otherwise.
    StoreOVar { id: LIRVarId },

    /// Copy value of input accumulator A to output accumulator.
    Cpy,

    /// Add values of input accumulators and store the result in the output accumulator.
    Add,

    /// Subtract values of input accumulators and store the result in the output accumulator.
    Sub,

    /// Unconditionally move code execution to address in input accumulator A.
    Goto,

    /// If value in input accumulator A is zero, Move code execution to address in input accumulator B.
    GotoIfZero,

    /// Native machine code pass-through. May or may not be validated.
    NativeMachinecode { code: Vec<u8> },
}
