use anyhow::anyhow;
use itertools::Itertools;
use libisa::instruction::Instruction;

use crate::transformer::{extra::Extras, Transformer};

pub struct MachinecodeTransformer;

impl Transformer for MachinecodeTransformer {
    type Input = Vec<Instruction>;
    type Output = Vec<u8>;

    fn transform(&mut self, input: Extras<Self::Input>) -> anyhow::Result<Extras<Self::Output>> {
        input.try_map_data(|instructions| {
            instructions
                .into_iter()
                .map(|instruction| {
                    instruction
                        .assemble()
                        .map_err(|e| anyhow!("STRM1 instruction assembly failed: {:?}", e))
                })
                .flatten_ok()
                .try_collect()
        })
    }
}
