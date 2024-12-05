use anyhow::Context;
use itertools::Itertools;
use libisa::Word;

use crate::{
    backend::strm1::codegen::{alloc::AllocTransformer, prealloc::PreallocInstruction},
    transformer::{extra::Extra, Transformer},
};

use super::VarAlloc;

impl AllocTransformer {
    pub fn neumann_offset_computation_prepass(
        &mut self,
        input: &Extra<<Self as Transformer>::Input>,
    ) -> anyhow::Result<()> {
        let code_len = input
            .data
            .iter()
            .map(|instruction| self.compute_instruction_code_len(instruction))
            .fold_ok(0, |acc, instruction_code_len| acc + instruction_code_len)?;

        // Scary access to the alloc map, let's not fuck anything up as I have a tendency to O_O
        let mem_allocs = self
            .alloc_map
            .0
            .iter_mut()
            .filter_map(|(_, alloc)| match alloc {
                VarAlloc::Memory(mem_alloc) => Some(mem_alloc),
                VarAlloc::Register(..) => None,
            });

        // TODO: Explicit addresses shouldn't be changed here when they get implemented.
        // Quite safe to assume for this project that the backend doesn't do any polymorphism that this would break.
        for mem_alloc in mem_allocs {
            mem_alloc.0 += code_len;
        }

        Ok(())
    }

    fn compute_instruction_code_len(
        &self,
        instruction: &PreallocInstruction,
    ) -> anyhow::Result<Word> {
        Ok(match instruction {
            PreallocInstruction::DefineVar(..)
            | PreallocInstruction::ExplicitRegister { .. }
            | PreallocInstruction::ExplicitMemory { .. } => 0,

            PreallocInstruction::LoadVar { src, .. } => {
                let src_alloc = self.var(&src).context("src").context("LoadVar")?;

                match src_alloc {
                    VarAlloc::Register(..) => 1,
                    VarAlloc::Memory(..) => 2,
                }
            }

            PreallocInstruction::StoreVar { dest, .. } => {
                let dest_alloc = self.var(&dest).context("dest").context("StoreVar")?;

                match dest_alloc {
                    VarAlloc::Register(..) => 1,
                    VarAlloc::Memory(..) => 2,
                }
            }

            PreallocInstruction::TargetPassthrough { instructions } => instructions.len() as Word,

            PreallocInstruction::LoadImmediate { .. }
            | PreallocInstruction::Jmp(..)
            | PreallocInstruction::JmpC(..)
            | PreallocInstruction::JmpZ(..)
            | PreallocInstruction::Add(..)
            | PreallocInstruction::Sub(..)
            | PreallocInstruction::AddC(..)
            | PreallocInstruction::SubC(..)
            | PreallocInstruction::And(..) => 1,
        })
    }
}
