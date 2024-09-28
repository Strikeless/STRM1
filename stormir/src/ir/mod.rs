#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LirOp {
    /// Define a new variable with the given id.
    /// NOTE: Variable id's shall not be reused even after being dropped.
    InitVar(usize),

    /// Forget the variable with the given id and free it's resources for use.
    DropVar(usize),

    /// Loads the variable with the given id to the virtual accumulator.
    /// NOTE: Mutations to the accumulator's value shall not directly affect the variable it was loaded from.
    LoadAccumulator(usize),

    /// Stores the virtual accumulator's value to the variable with the given id.
    StoreAccumulator(usize),

    /// Loads the variable with the given id to the virtual secondary register,
    /// used as the second operand for operations that need one.
    /// NOTE: The value of the virtual secondary register shall never be mutated.
    /// NOTE: For now, the value can be considered undefined after a store operation to avoid some complexity, although this may not often be the case.
    LoadSecondary(usize),

    /// Loads the constant value to the virtual accumulator.
    LoadConstant(u16),

    /// Adds the values of the accumulator and secondary register and writes the result back into the accumulator.
    Add,

    /// Subtracts the values of the accumulator and secondary register and writes the result back into the accumulator.
    Sub,

    InitLabel(usize),

    Goto(usize),

    /// Finishes execution of the program.
    /// NOTE: Not sure whether or not this should exist at all in the IR, but for now it's here so that emulator tests know when to halt.
    Finish,
}
