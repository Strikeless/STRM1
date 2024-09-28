use ir::LirOp;
use transformer::Transformer;

pub mod ir;
pub mod transformer;
mod backend;

pub fn lir_pipeline() -> impl Transformer<Input = LirOp, Output = u8> {
    backend::strm1::lir_transformer()
}
