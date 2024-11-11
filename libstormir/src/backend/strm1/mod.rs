use codegen::STRM1CodegenTransformer;
use machinecode::STRM1MachinecodeTransformer;

use crate::{
    lir::LIRInstruction,
    transformer::{
        chain::TransformerChainExt, extra::Extra, runner::TransformerRunnerExt, Transformer,
    },
};

mod codegen;
mod machinecode;

pub fn transformer() -> impl Transformer<Input = Vec<LIRInstruction>, Output = Vec<u8>> {
    STRM1Transformer {}
}

pub struct STRM1Transformer;

impl Transformer for STRM1Transformer {
    type Input = Vec<LIRInstruction>;
    type Output = Vec<u8>;

    fn transform(&mut self, input: Extra<Self::Input>) -> anyhow::Result<Extra<Self::Output>> {
        (STRM1CodegenTransformer::new())
            .chain(STRM1MachinecodeTransformer)
            .runner()
            .run_with_extras(input)
    }
}
