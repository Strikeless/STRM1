use codegen::CodegenTransformer;
use machinecode::MachinecodeTransformer;

use crate::{
    lir::{shim::cmp::CmpShimTransformer, LIRInstruction},
    transformer::{
        chain::TransformerChainExt, extra::Extra, runner::TransformerRunnerExt, Transformer,
    },
};

mod codegen;
mod machinecode;

pub struct STRM1Transformer;

impl Transformer for STRM1Transformer {
    type Input = Vec<LIRInstruction>;
    type Output = Vec<u8>;

    fn transform(&mut self, input: Extra<Self::Input>) -> anyhow::Result<Extra<Self::Output>> {
        (CmpShimTransformer) // Remember to remove if codegen learns all the cmp tricks.
            .chain(CodegenTransformer::new())
            .chain(MachinecodeTransformer)
            .runner()
            .run_with_extras(input)
    }
}
