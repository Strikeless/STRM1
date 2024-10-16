use libstrmisa::instruction::Instruction;

use crate::transformer::Transformer;

pub struct STRM1MachinecodeTransformer;

impl Transformer for STRM1MachinecodeTransformer {
    type Input = Instruction;
    type Output = u8;

    fn transform(&mut self, input: Vec<Self::Input>) -> anyhow::Result<Vec<Self::Output>> {
        let mut output = Vec::with_capacity(input.len());

        for instruction in input {
            let mut assembled = instruction.assemble()?;
            output.append(&mut assembled);
        }

        Ok(output)
    }
}
