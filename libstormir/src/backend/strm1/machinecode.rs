use libisa::instruction::{assembler, Instruction as TargetInstruction};

use crate::transformer::{extra::Extras, Transformer};

pub const EXTRAS_BYTE_TO_INSTRUCTION_INDEX_MAP_KEY: &'static str =
    "strm1_byte_to_instruction_index_map";

pub const EXTRAS_INSTRUCTION_TO_BYTE_INDEX_MAP_KEY: &'static str =
    "strm1_instruction_to_byte_index_map";

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

        Ok(input
            .map_data(|_| assembly_output.machine_code)
            .with_extra(
                &EXTRAS_BYTE_TO_INSTRUCTION_INDEX_MAP_KEY,
                &assembly_output.byte_to_extra_map,
            )
            .with_extra(
                &EXTRAS_INSTRUCTION_TO_BYTE_INDEX_MAP_KEY,
                &assembly_output.extra_to_bytes_map,
            ))
    }
}
