use libisa::instruction::Instruction as TargetInstruction;

use crate::{
    lir::LIRInstruction,
    transformer::{extra::Extra, Transformer},
};

pub struct CodegenTransformer {}

impl CodegenTransformer {
    pub fn new() -> Self {
        Self {}
    }
}

impl Transformer for CodegenTransformer {
    type Input = Vec<LIRInstruction>;
    type Output = Vec<TargetInstruction>;

    fn transform(&mut self, input: Extra<Self::Input>) -> anyhow::Result<Extra<Self::Output>> {
        todo!()
    }
}
