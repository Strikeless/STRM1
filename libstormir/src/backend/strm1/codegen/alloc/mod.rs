///
/// The names "alloc" and "prealloc" really aren't all that describing.
///
/// Prealloc (other module) turns stormir LIR into it's own near-machine-code IR, and is where the actual code
/// generation happens, besides for register and memory allocation, which is handed over to the alloc module once
/// we know approximately what code to generate.
///
/// Alloc (this module) turns the near-machine-code IR produced by prealloc into actual machine code and is where the
/// actual register and memory allocation happens. It takes the IR from the prealloc module and translates the generic
/// variable operations into operations working on registers and memory understood by the target machine.
///
/// Thanks to this split:
/// +   Register allocation becomes easier, since we can delay it to a point where most of the code generation has been
///     done already, thus we already have an approximate idea of variable lifetimes and which registers are free.
/// +   Code generation becomes easier, since we don't need to deal with raw registers or memory, but the more generic
///     "variables", and pass the allocation problem to the alloc pass.
/// -   We get a lot more boilerplate and complexity in the backend.
///
use std::collections::HashMap;

use anyhow::{anyhow, Context};
use itertools::Itertools;
use lazy_static::lazy_static;
use libisa::instruction::{kind::InstructionKind, Instruction as TargetInstruction};
use varalloc::{
    allocator::{AllocRequirement, VarAllocator, VarDefinition},
    AllocMap, MemVarAlloc, RegVarAlloc, VarAlloc,
};

use crate::{
    backend::strm1::codegen::prealloc::{VarId, VarKey},
    transformer::{extra::Extras, Transformer},
};

use super::prealloc::{
    varidspace::VarIdSpace, MemVarKey, PreallocInstruction, RegVarKey, VarTrait,
};

mod varalloc;

#[cfg(test)]
pub mod tests;

// No reason for alloc extras to be public as the data structures are private.
const EXTRAS_ALLOC_MAP_KEY: &'static str = "strm1_alloc_map";
const EXTRAS_ALLOC_METADATA_KEY: &'static str = "strm1_alloc_metadata";

lazy_static! {
    static ref INTERNAL_VAR_SPACE: VarIdSpace = VarIdSpace::new();
}

#[derive(Debug, Default)]
pub struct AllocTransformer {
    alloc_map: AllocMap,
    alloc_metadata: HashMap<VarId, VarDefinition>,
}

impl AllocTransformer {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Transformer for AllocTransformer {
    type Input = Vec<PreallocInstruction>;
    type Output = Vec<TargetInstruction>;

    const PREPASSES: &[(&'static str, crate::transformer::PrepassFn<Self>)] = &[
        ("allocation prepass", Self::alloc_prepass),
        (
            "Von Neumann offset computation prepass",
            Self::neumann_offset_computation_prepass,
        ),
    ];

    fn transform(&mut self, input: Extras<Self::Input>) -> anyhow::Result<Extras<Self::Output>> {
        Ok(input
            .try_map_data(|prealloc_ir| self.transform_prealloc_ir(prealloc_ir))?
            .with_extra(&EXTRAS_ALLOC_MAP_KEY, &self.alloc_map)
            .with_extra(&EXTRAS_ALLOC_METADATA_KEY, &self.alloc_metadata))
    }
}

impl AllocTransformer {
    fn alloc_prepass(
        &mut self,
        input: &Extras<<Self as Transformer>::Input>,
    ) -> anyhow::Result<()> {
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

                PreallocInstruction::StoreVar { dest, .. } => {
                    let dest_var = allocator.get_definition(dest.id()).ok_or_else(|| {
                        anyhow!("Undefined variable")
                            .context("dest")
                            .context("StoreVar")
                    })?;

                    if dest_var.alloc_requirement != AllocRequirement::Register {
                        // The destination variable is heading towards memory, so the main pass requires an internal
                        // register for storing it's memory address, allocate that here.
                        let id = VarId(*INTERNAL_VAR_SPACE, instruction_index as u64);
                        allocator.define(id, instruction_index, 0, AllocRequirement::Register)?;
                        allocator.extend_lifetime(&id, instruction_index + 1)?; // The register is only needed for this instruction.
                    }
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

        self.alloc_metadata = allocator.definition_map().clone();
        self.alloc_map = allocator.build()?;
        Ok(())
    }

    fn transform_prealloc_ir(
        &mut self,
        prealloc_ir: Vec<PreallocInstruction>,
    ) -> anyhow::Result<Vec<TargetInstruction>> {
        prealloc_ir
            .into_iter()
            .enumerate()
            .map(|(instruction_index, instruction)| {
                self.transform_instruction(instruction_index, instruction)
            })
            .flatten_ok()
            .try_collect()
    }

    fn transform_instruction(
        &mut self,
        instruction_index: usize,
        instruction: PreallocInstruction,
    ) -> anyhow::Result<Vec<TargetInstruction>> {
        // NOTE: Remember to also update the Neumann offset computation prepass!
        //       Going to be a very nasty bug to find when variables are overwriting code otherwise.

        Ok(match instruction {
            // Handled in prepasses
            PreallocInstruction::DefineVar(..)
            | PreallocInstruction::ExplicitRegister { .. }
            | PreallocInstruction::ExplicitMemory { .. } => vec![],

            PreallocInstruction::LoadImmediate { dest, value } => {
                let dest_reg = self.reg_var(&dest).context("LoadImmediate")?.0;

                vec![TargetInstruction::new(InstructionKind::LoadI)
                    .with_reg_a(dest_reg)
                    .with_immediate(value)]
            }

            PreallocInstruction::LoadVar { dest, src } => {
                let dest_reg = self.reg_var(&dest).context("dest").context("LoadVar")?.0;
                let src_alloc = self.var(&src).context("src").context("LoadVar")?;

                match src_alloc {
                    VarAlloc::Register(src_reg) => vec![
                        // Simple register copy from the source register to the destination register.
                        TargetInstruction::new(InstructionKind::Cpy)
                            .with_reg_a(dest_reg)
                            .with_reg_b(src_reg.0),
                    ],
                    VarAlloc::Memory(src_mem) => vec![
                        // Load the source address to dest_reg.
                        TargetInstruction::new(InstructionKind::LoadI)
                            .with_reg_a(dest_reg)
                            .with_immediate(src_mem.0),
                        // And now load the value of that address to dest_reg.
                        TargetInstruction::new(InstructionKind::Load)
                            .with_reg_a(dest_reg)
                            .with_reg_b(dest_reg),
                    ],
                }
            }

            PreallocInstruction::StoreVar { dest, src } => {
                let dest_alloc = self.var(&dest).context("dest").context("StoreVar")?;
                let src_reg = self.reg_var(&src).context("src").context("StoreVar")?.0;

                match dest_alloc {
                    VarAlloc::Register(dest_reg) => {
                        vec![TargetInstruction::new(InstructionKind::Cpy)
                            .with_reg_a(dest_reg.0)
                            .with_reg_b(src_reg)]
                    }
                    VarAlloc::Memory(dest_mem) => {
                        // Allocated in prepass.
                        let internal_addr_var_id =
                            &RegVarKey(VarId(*INTERNAL_VAR_SPACE, instruction_index as u64));
                        let internal_addr_reg = self.reg_var(internal_addr_var_id)?.0;

                        vec![
                            // Load the memory address of the destination variable to the internal register.
                            TargetInstruction::new(InstructionKind::LoadI)
                                .with_reg_a(internal_addr_reg)
                                .with_immediate(dest_mem.0),
                            // Store the source value to the loaded address (destination variable).
                            TargetInstruction::new(InstructionKind::Store)
                                .with_reg_a(internal_addr_reg)
                                .with_reg_b(src_reg),
                        ]
                    }
                }
            }

            PreallocInstruction::Jmp(addr) => self
                .transform_single_reg_operand(InstructionKind::Jmp, addr)
                .context("addr")
                .context("Jmp")?,

            PreallocInstruction::JmpC(addr) => self
                .transform_single_reg_operand(InstructionKind::JmpC, addr)
                .context("addr")
                .context("JmpC")?,

            PreallocInstruction::JmpZ(addr) => self
                .transform_single_reg_operand(InstructionKind::JmpZ, addr)
                .context("addr")
                .context("JmpZ")?,

            PreallocInstruction::Add(a_reg, b_reg) => self
                .transform_dual_reg_operand(InstructionKind::Add, a_reg, b_reg)
                .context("Add")?,
            PreallocInstruction::Sub(a_reg, b_reg) => self
                .transform_dual_reg_operand(InstructionKind::Sub, a_reg, b_reg)
                .context("Sub")?,
            PreallocInstruction::AddC(a_reg, b_reg) => self
                .transform_dual_reg_operand(InstructionKind::AddC, a_reg, b_reg)
                .context("AddC")?,
            PreallocInstruction::SubC(a_reg, b_reg) => self
                .transform_dual_reg_operand(InstructionKind::SubC, a_reg, b_reg)
                .context("SubC")?,
            PreallocInstruction::And(a_reg, b_reg) => self
                .transform_dual_reg_operand(InstructionKind::And, a_reg, b_reg)
                .context("And")?,

            PreallocInstruction::TargetPassthrough { instructions } => instructions,
        })
    }

    fn transform_single_reg_operand(
        &self,
        kind: InstructionKind,
        reg: RegVarKey,
    ) -> anyhow::Result<Vec<TargetInstruction>> {
        let reg_a = self.reg_var(&reg)?.0;

        Ok(vec![TargetInstruction::new(kind).with_reg_a(reg_a)])
    }

    fn transform_dual_reg_operand(
        &self,
        kind: InstructionKind,
        a: RegVarKey,
        b: RegVarKey,
    ) -> anyhow::Result<Vec<TargetInstruction>> {
        let reg_a = self.reg_var(&a).context("a")?.0;

        let reg_b = self.reg_var(&b).context("b")?.0;

        Ok(vec![TargetInstruction::new(kind)
            .with_reg_a(reg_a)
            .with_reg_b(reg_b)])
    }

    fn var(&self, key: &VarKey) -> anyhow::Result<&VarAlloc> {
        self.alloc_map
            .get(key)
            .ok_or_else(|| anyhow!("Undefined variable"))
    }

    fn reg_var(&self, key: &RegVarKey) -> anyhow::Result<&RegVarAlloc> {
        self.alloc_map
            .get_reg(key)
            .ok_or_else(|| anyhow!("Undefined variable"))
    }

    #[allow(unused)] // It's here for consistency with reg_var and potential future use.
    fn mem_var(&self, key: &MemVarKey) -> anyhow::Result<&MemVarAlloc> {
        self.alloc_map
            .get_mem(key)
            .ok_or_else(|| anyhow!("Undefined variable"))
    }
}
