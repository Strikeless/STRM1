use std::collections::HashMap;

use anyhow::anyhow;
use itertools::Itertools;
use libdeassembler::Deassembler;
use libisa::instruction::{kind::InstructionKind, Instruction as TargetInstruction};

use crate::transformer::{extra::Extra, Transformer};

use super::prealloc::{MemoryVar, PreallocInstruction, RegisterVar};

mod usagemap;

#[derive(Debug, Default)]
pub struct AllocatorTransformer {
    reg_allocs: HashMap<RegisterVar, ()>,
    mem_allocs: HashMap<MemoryVar, ()>,
}

impl AllocatorTransformer {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Transformer for AllocatorTransformer {
    type Input = Vec<PreallocInstruction>;
    type Output = Vec<TargetInstruction>;

    fn transform(&mut self, input: Extra<Self::Input>) -> anyhow::Result<Extra<Self::Output>> {
        input.try_map_data(|lir| {
            lir.into_iter()
                .map(|instruction| self.transform_instruction(instruction))
                .flatten_ok()
                .try_collect()
        })
    }
}

impl AllocatorTransformer {
    fn transform_instruction(
        &mut self,
        instruction: PreallocInstruction,
    ) -> anyhow::Result<Vec<TargetInstruction>> {
        Ok(match instruction {
            PreallocInstruction::Add { a_reg, b_reg } => {
                self.transform_dual_reg_operand(InstructionKind::Add, a_reg, b_reg)?
            }
            PreallocInstruction::Sub { a_reg, b_reg } => {
                self.transform_dual_reg_operand(InstructionKind::Sub, a_reg, b_reg)?
            }
            PreallocInstruction::AddC { a_reg, b_reg } => {
                self.transform_dual_reg_operand(InstructionKind::AddC, a_reg, b_reg)?
            }
            PreallocInstruction::SubC { a_reg, b_reg } => {
                self.transform_dual_reg_operand(InstructionKind::SubC, a_reg, b_reg)?
            }
            PreallocInstruction::And { a_reg, b_reg } => {
                self.transform_dual_reg_operand(InstructionKind::And, a_reg, b_reg)?
            }

            PreallocInstruction::NativeMachinecode { code } => {
                let deassembler = Deassembler::new(code.iter());

                deassembler
                    .deassemble()
                    .map_err(|e| anyhow!("Invalid machinecode passthrough: {}", e))?
            }

            x => todo!("transform_instruction({:?})", x),
        })
    }

    fn transform_dual_reg_operand(
        &self,
        kind: InstructionKind,
        a: RegisterVar,
        b: RegisterVar,
    ) -> anyhow::Result<Vec<TargetInstruction>> {
        todo!()
    }
}
