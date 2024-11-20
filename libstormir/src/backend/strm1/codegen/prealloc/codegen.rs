use anyhow::anyhow;
use itertools::Itertools;
use libdeassembler::Deassembler;

use crate::{
    lir::LIRInstruction,
    transformer::{extra::Extra, Transformer},
};

use super::PreallocInstruction;

pub struct PreallocCodegenTransformer {}

impl Transformer for PreallocCodegenTransformer {
    type Input = Vec<LIRInstruction>;
    type Output = Vec<PreallocInstruction>;

    fn transform(&mut self, input: Extra<Self::Input>) -> anyhow::Result<Extra<Self::Output>> {
        input.try_map_data(|lir| {
            lir.into_iter()
                .enumerate()
                .map(|(instruction_index, instruction)| {
                    self.transform_instruction(instruction_index, instruction)
                })
                .flatten_ok()
                .try_collect()
        })
    }
}

impl PreallocCodegenTransformer {
    fn transform_instruction(
        &mut self,
        instruction_index: usize,
        instruction: LIRInstruction,
    ) -> anyhow::Result<Vec<PreallocInstruction>> {
        Ok(match instruction {
            LIRInstruction::NativeMachinecode { code } => {
                let deassembler = Deassembler::new(code.iter());

                let instructions = deassembler
                    .deassemble()
                    .map_err(|e| anyhow!("Invalid passthrough machinecode: {}", e))?;

                vec![PreallocInstruction::TargetPassthrough { instructions }]
            }
            x => todo!("transform_instruction({:?})", x),
        })
    }
}
