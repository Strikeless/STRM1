pub use codegen::CodegenTransformer as STRM1CodegenTransformer;
use target::Instruction;

use crate::{ir::LirOp, transformer::{chain::TransformerChainExt, Transformer}};

mod target;
mod var;
mod codegen;

pub fn lir_transformer() -> impl Transformer<Input = LirOp, Output = u8> {
    STRM1CodegenTransformer::new().chain(InstructionTransformer {})
}

struct InstructionTransformer;

impl Transformer for InstructionTransformer {
    type Input = Instruction;
    type Output = u8;

    fn transform(&mut self, input: Vec<Self::Input>) -> anyhow::Result<Vec<Self::Output>> {
        Ok(
            input.into_iter()
                .map(|instruction| instruction.build())
                .flatten()
                .collect()
        )
    }
}
