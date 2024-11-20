use allocation::{
    allocator::{AllocRequirement, VarAllocator},
    AllocMap,
};
use anyhow::anyhow;
use itertools::Itertools;
use libisa::{
    instruction::{kind::InstructionKind, Instruction as TargetInstruction},
    Word,
};

use crate::{
    backend::strm1::codegen::prealloc::VarKey,
    transformer::{extra::Extra, Transformer},
};

use super::prealloc::{PreallocInstruction, RegVarKey, VarTrait};

mod allocation;

#[derive(Debug, Default)]
pub struct AllocatorTransformer {
    alloc_map: Option<AllocMap>,
    mem_addr_offset: Word,
}

impl AllocatorTransformer {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Transformer for AllocatorTransformer {
    type Input = Vec<PreallocInstruction>;
    type Output = Vec<TargetInstruction>;

    const PREPASSES: &[(&'static str, crate::transformer::PrepassFn<Self>)] = &[
        ("allocation prepass", Self::alloc_prepass),
        (
            "Von Neumann offset computation prepass",
            Self::von_neumann_offset_computation_prepass,
        ),
    ];

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
    fn alloc_prepass(&mut self, input: &Extra<<Self as Transformer>::Input>) -> anyhow::Result<()> {
        let mut allocator = VarAllocator::new();

        for (instruction_index, instruction) in input.data.iter().enumerate() {
            match instruction {
                PreallocInstruction::DefineVar(key) => {
                    let alloc_requirement = match key {
                        VarKey::Generic(..) => AllocRequirement::Generic,
                        VarKey::Register(..) => AllocRequirement::Register,
                        VarKey::Memory(..) => AllocRequirement::Memory,
                    };

                    allocator.define(*key.id(), instruction_index, 0, alloc_requirement)?;
                }

                PreallocInstruction::ExplicitRegister { .. }
                | PreallocInstruction::ExplicitMemory { .. } => todo!(), // TODO: Explicit register and memory addresses

                _ => {}
            }

            for used_var_id in instruction.used_vars() {
                if !allocator.contains_id(used_var_id) {
                    return Err(anyhow!("Undefined variable"));
                }

                allocator.extend_lifetime(used_var_id, instruction_index)?;
                allocator.add_importance(used_var_id, 1)?;
            }
        }

        self.alloc_map = Some(allocator.build()?);
        Ok(())
    }

    fn von_neumann_offset_computation_prepass(
        &mut self,
        input: &Extra<<Self as Transformer>::Input>,
    ) -> anyhow::Result<()> {
        // I am pretty sure that this is THE place to do this address thing in, because if this would be done...
        //
        // ...before the allocation prepass, we wouldn't at all know what the output would look like, thus wouldn't
        //    be able to know where code ends.
        //
        // ...after the main pass (codegen), we wouldn't know which memory writes/reads were just trying to do
        //    polymorphism or some bullshit like that. (or we would need a very ugly magic address patching thing)

        self.mem_addr_offset = 4096; // TODO: Make this actually be.
        Ok(())
    }

    fn transform_instruction(
        &mut self,
        instruction: PreallocInstruction,
    ) -> anyhow::Result<Vec<TargetInstruction>> {
        // Should never panic if the allocation prepass works as intended. If it doesn't, we very much should panic.
        let alloc_map = self.alloc_map.as_ref().unwrap();

        Ok(match instruction {
            PreallocInstruction::Add(a_reg, b_reg) => {
                self.transform_dual_reg_operand(InstructionKind::Add, a_reg, b_reg)?
            }
            PreallocInstruction::Sub(a_reg, b_reg) => {
                self.transform_dual_reg_operand(InstructionKind::Sub, a_reg, b_reg)?
            }
            PreallocInstruction::AddC(a_reg, b_reg) => {
                self.transform_dual_reg_operand(InstructionKind::AddC, a_reg, b_reg)?
            }
            PreallocInstruction::SubC(a_reg, b_reg) => {
                self.transform_dual_reg_operand(InstructionKind::SubC, a_reg, b_reg)?
            }
            PreallocInstruction::And(a_reg, b_reg) => {
                self.transform_dual_reg_operand(InstructionKind::And, a_reg, b_reg)?
            }

            PreallocInstruction::TargetPassthrough { instructions } => instructions,

            x => todo!("transform_instruction({:?})", x),
        })
    }

    fn transform_dual_reg_operand(
        &self,
        kind: InstructionKind,
        a: RegVarKey,
        b: RegVarKey,
    ) -> anyhow::Result<Vec<TargetInstruction>> {
        todo!()
    }
}
