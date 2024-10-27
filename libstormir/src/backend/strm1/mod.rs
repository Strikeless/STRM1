use codegen::STRM1CodegenTransformer;
use machinecode::STRM1MachinecodeTransformer;

use crate::{
    lir::LIRInstruction,
    transformer::{chain::TransformerChainExt, Transformer},
};

pub mod codegen;
pub mod machinecode;

pub fn transformer() -> impl Transformer<Input = Vec<LIRInstruction>, Output = Vec<u8>> {
    STRM1CodegenTransformer::new().chain(STRM1MachinecodeTransformer)
}
