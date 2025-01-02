use libisa::instruction::{assembler, Instruction as TargetInstruction};

use crate::transformer::{extra::Extras, Transformer};

pub const EXTRAS_TARGET_INSTRUCTION_BYTE_MAPPING_KEY: &'static str =
    "strm1_target_instruction_byte_mapping";

pub struct MachinecodeTransformer;

impl Transformer for MachinecodeTransformer {
    type Input = Vec<TargetInstruction>;
    type Output = Vec<u8>;

    fn transform(
        &mut self,
        mut input: Extras<Self::Input>,
    ) -> anyhow::Result<Extras<Self::Output>> {
        let assembly_output = assembler::assemble_extra(
            input
                .data
                .drain(..)
                .enumerate()
                .map(|(instruction_index, instruction)| (instruction, instruction_index)),
        )?;

        Ok(input.map_data(|_| assembly_output.machine_code).with_extra(
            &EXTRAS_TARGET_INSTRUCTION_BYTE_MAPPING_KEY,
            &assembly_output.extra_map,
        ))
    }
}
