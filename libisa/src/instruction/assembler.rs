use std::hash::Hash;

use bimap::BiMap;

use super::{AssemblyError, Instruction};

#[derive(Default)]
pub struct AssemblyOutput<T>
where
    T: Hash + Eq,
{
    pub machine_code: Vec<u8>,

    /// Machine code byte index (left) to extra (right) or vice versa mapping.
    pub extra_map: BiMap<usize, T>,
}

pub fn assemble<I>(instructions: I) -> Result<AssemblyOutput<()>, AssemblyError>
where
    I: IntoIterator<Item = Instruction>,
{
    assemble_extra(
        instructions
            .into_iter()
            .map(|instruction| (instruction, ())),
    )
}

pub fn assemble_extra<I, T>(instructions: I) -> Result<AssemblyOutput<T>, AssemblyError>
where
    I: IntoIterator<Item = (Instruction, T)>,
    T: Hash + Eq + Default + Clone,
{
    instructions.into_iter().try_fold(
        AssemblyOutput::default(),
        |mut output, (instruction, extra)| {
            let instruction_machine_code = instruction.assemble()?;

            let instruction_byte_start = output.machine_code.len();
            let instruction_len_bytes = instruction_machine_code.len();
            let instruction_byte_range =
                instruction_byte_start..instruction_byte_start + instruction_len_bytes;

            output
                .extra_map
                .extend(instruction_byte_range.map(|byte_index| (byte_index, extra.clone())));
            output.machine_code.extend(instruction_machine_code);

            Ok(output)
        },
    )
}
