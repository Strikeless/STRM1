use codegen::CodegenTransformer;
use machinecode::MachinecodeTransformer;

use crate::{
    lir::LIRInstruction,
    transformer::{
        chain::TransformerChainExt, extra::Extra, runner::TransformerRunnerExt, Transformer,
    },
};

mod codegen;
mod machinecode;

#[cfg(test)]
mod tests;

pub struct STRM1Transformer;

impl STRM1Transformer {
    pub fn new() -> Self {
        Self {}
    }
}

impl Transformer for STRM1Transformer {
    type Input = Vec<LIRInstruction>;
    type Output = Vec<u8>;

    fn transform(&mut self, input: Extra<Self::Input>) -> anyhow::Result<Extra<Self::Output>> {
        CodegenTransformer::new()
            .chain(MachinecodeTransformer)
            .runner()
            .run_with_extras(input)
    }
}
