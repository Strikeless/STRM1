use libisa::instruction::Instruction;

use crate::transformer::{extra::Extra, Transformer};

pub struct STRM1MachinecodeTransformer;

impl Transformer for STRM1MachinecodeTransformer {
    type Input = Vec<Instruction>;
    type Output = Vec<u8>;

    fn transform(&mut self, input: Extra<Self::Input>) -> anyhow::Result<Extra<Self::Output>> {
        let mut output = Vec::with_capacity(input.data.len());

        for instruction in &input.data {
            let mut assembled = instruction.assemble()?;
            output.append(&mut assembled);
        }

        Ok(input.new_preserve_extras(output))
    }
}
