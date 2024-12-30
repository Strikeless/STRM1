pub mod alloc;
mod prealloc;

use alloc::AllocTransformer;
use libisa::instruction::Instruction as TargetInstruction;
use prealloc::codegen::PreallocCodegenTransformer;

use crate::{
    lir::{shim::cmp::CmpShimTransformer, LIRInstruction},
    transformer::{
        chain::TransformerChainExt, extra::Extras, runner::TransformerRunnerExt, Transformer,
    },
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

    fn transform(&mut self, input: Extras<Self::Input>) -> anyhow::Result<Extras<Self::Output>> {
        (CmpShimTransformer) // Remember to remove if codegen learns all the cmp tricks.
            .chain(PreallocCodegenTransformer {})
            .chain(AllocTransformer::new())
            .runner()
            .run_with_extras(input)
    }
}
