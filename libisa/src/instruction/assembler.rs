use std::{collections::HashMap, hash::Hash, ops::Range};

use crate::Word;

use super::{AssemblyError, Instruction};

pub struct AssemblyOutput<T> {
    pub machine_code: Vec<u8>,

    /// Machine code byte index to extra mapping.
    pub byte_to_extra_map: HashMap<Word, T>,

    /// Extra to machine code byte indices mapping.
    pub extra_to_bytes_map: HashMap<T, Range<Word>>,
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
    T: Clone + Hash + Eq,
{
    let mut output = AssemblyOutput {
        machine_code: Vec::new(),
        byte_to_extra_map: HashMap::new(),
        extra_to_bytes_map: HashMap::new(),
    };

    for (instruction, instruction_extra) in instructions {
        assemble_instruction(&mut output, instruction, instruction_extra)?;
    }

    Ok(output)
}

fn assemble_instruction<T>(
    output: &mut AssemblyOutput<T>,
    instruction: Instruction,
    extra: T,
) -> Result<(), AssemblyError>
where
    T: Clone + Hash + Eq,
{
    let instruction_machine_code = instruction.assemble()?;

    let instruction_start_byte = output.machine_code.len() as Word;
    let instruction_len_bytes = instruction_machine_code.len() as Word;

    let instruction_byte_range =
        instruction_start_byte..instruction_start_byte + instruction_len_bytes;

    let extra_by_byte_indices = instruction_byte_range
        .clone()
        .map(|byte_index| (byte_index, extra.clone()));

    output.byte_to_extra_map.extend(extra_by_byte_indices);
    output
        .extra_to_bytes_map
        .insert(extra, instruction_byte_range);

    output.machine_code.extend(instruction_machine_code);

    Ok(())
}
