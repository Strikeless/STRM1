#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct LIRVarKey(pub usize);

pub enum LIRInstruction {
    InitVar(LIRVarKey),
    DropVar(LIRVarKey),

    ConstantA(u16),

    LoadA(LIRVarKey),
    LoadB(LIRVarKey),
    StoreA(LIRVarKey),

    Add,
    Sub,
}
