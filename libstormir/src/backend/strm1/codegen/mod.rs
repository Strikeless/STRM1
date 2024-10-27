use std::iter;

use anyhow::{anyhow, Context};
use indexmap::IndexMap;
use libisa::{
    instruction::{kind::InstructionKind, Instruction},
    Register, Word,
};
use var::{builder::VarTableBuilder, VarAllocation, VarAllocationKind, VarKey, VarTable};

use crate::{
    lir::{LIRInstruction, LIRVarId},
    transformer::{extra::Extra, Transformer},
};

mod var;

#[cfg(test)]
mod tests;

// Warning: big mess ahead

pub const EXTRA_INSTRUCTION_MAPPINGS_KEY: &str = "strm1-instruction-mappings";
pub const EXTRA_VAR_ALLOCATIONS_KEY_JSON: &str = "strm1-var-allocations-json";
pub const EXTRA_VAR_ALLOCATIONS_KEY_RMP: &str = "strm1-var-allocations-rmp";

pub struct STRM1CodegenTransformer {
    output: Vec<Instruction>,
    var_table: VarTable,

    extra_instruction_mappings: Vec<usize>,
}

impl STRM1CodegenTransformer {
    pub fn new() -> Self {
        Self {
            output: Vec::new(),
            var_table: VarTable::default(),

            extra_instruction_mappings: Vec::new(),
        }
    }
}

impl Transformer for STRM1CodegenTransformer {
    type Input = Vec<LIRInstruction>;
    type Output = Vec<Instruction>;

    fn prepass(&mut self, input: &Extra<Self::Input>) -> anyhow::Result<()> {
        let mut var_table_builder = VarTableBuilder::new();

        let mut ia_definition_key = None;
        let mut ib_definition_key = None;
        let mut o_definition_key = None;

        fn alloc_accumulator(
            index: usize,
            var_table_builder: &mut VarTableBuilder,
            definition_key: &mut Option<VarKey>,
        ) -> anyhow::Result<()> {
            // Previous input accumulator allocation can be dropped as we overwrite it here.
            dealloc_accumulator(var_table_builder, definition_key)?;

            let key = VarKey::Special(index);
            var_table_builder.define(key, true)?;
            *definition_key = Some(key);

            Ok(())
        }

        fn dealloc_accumulator(
            var_table_builder: &mut VarTableBuilder,
            definition_key: &mut Option<VarKey>,
        ) -> anyhow::Result<()> {
            if let Some(definition_key) = definition_key.take() {
                var_table_builder.drop(definition_key, 0)?;
            }

            Ok(())
        }

        for (index, lir_instruction) in input.data.iter().enumerate() {
            var_table_builder.set_current_index(index);

            match lir_instruction {
                LIRInstruction::DefineVar { id } => {
                    var_table_builder.define(VarKey::Normal(*id), false)?;
                }
                LIRInstruction::DropVar { id } => {
                    var_table_builder.drop(VarKey::Normal(*id), 0)?;
                }

                //
                // Loads to input accumulators need a special register allocation to hold the value in
                //
                LIRInstruction::LoadIAConst { .. } | LIRInstruction::LoadIALabel => {
                    alloc_accumulator(index, &mut var_table_builder, &mut ia_definition_key)?;
                }
                LIRInstruction::LoadIBConst { .. } | LIRInstruction::LoadIBLabel => {
                    alloc_accumulator(index, &mut var_table_builder, &mut ib_definition_key)?;
                }

                LIRInstruction::LoadIAVar { id } => {
                    alloc_accumulator(index, &mut var_table_builder, &mut ia_definition_key)?;
                    var_table_builder.heaten(VarKey::Normal(*id))?
                }
                LIRInstruction::LoadIBVar { id } => {
                    alloc_accumulator(index, &mut var_table_builder, &mut ib_definition_key)?;
                    var_table_builder.heaten(VarKey::Normal(*id))?
                }

                LIRInstruction::StoreOVar { .. } => {
                    // The LIR documentation allows dropping all accumulators (just specified it there hehe) upon
                    // an output accumulator store, so do that here for simplicity.
                    dealloc_accumulator(&mut var_table_builder, &mut ia_definition_key)?;
                    dealloc_accumulator(&mut var_table_builder, &mut ib_definition_key)?;
                    dealloc_accumulator(&mut var_table_builder, &mut o_definition_key)?;

                    // Store needs a scratchpad register to store the address of an in-memory target variable.
                    let key = VarKey::Special(index);

                    var_table_builder.define(key, true).unwrap();

                    // The register is only needed during execution of this instruction, so drop it by the next instruction.
                    var_table_builder.drop(key, 1).unwrap();
                }

                LIRInstruction::Cpy | LIRInstruction::Add | LIRInstruction::Sub => {
                    alloc_accumulator(index, &mut var_table_builder, &mut o_definition_key)?;
                }

                _ => {}
            }
        }

        self.var_table = var_table_builder.build()?;
        Ok(())
    }

    fn transform(&mut self, input: Extra<Self::Input>) -> anyhow::Result<Extra<Self::Output>> {
        let mut ia_register = None;
        let mut ib_register = None;
        let mut o_register = None;

        for (index, lir_instruction) in input.data.iter().enumerate() {
            match lir_instruction {
                // Handled during prepass
                LIRInstruction::DefineVar { .. } | LIRInstruction::DropVar { .. } => {}

                LIRInstruction::LoadIAConst { value } => {
                    self.transform_load_const(index, &mut ia_register, *value)?
                }
                LIRInstruction::LoadIBConst { value } => {
                    self.transform_load_const(index, &mut ib_register, *value)?
                }

                LIRInstruction::LoadIAVar { id } => {
                    self.transform_load_var(index, &mut ia_register, *id)?
                }
                LIRInstruction::LoadIBVar { id } => {
                    self.transform_load_var(index, &mut ib_register, *id)?
                }

                LIRInstruction::LoadIALabel => {
                    self.transform_load_label(index, &mut ia_register)?
                }
                LIRInstruction::LoadIBLabel => {
                    self.transform_load_label(index, &mut ib_register)?
                }

                LIRInstruction::StoreOVar { id } => {
                    let o_register = o_register.context("StoreOVar without set O accumulator")?;

                    let var = self.var(*id)?;

                    match var.kind {
                        VarAllocationKind::Register(var_reg) => {
                            // Simple register copy from O register to the variable's register.
                            self.extend_output(
                                index,
                                [Instruction::new(InstructionKind::Cpy)
                                    .with_reg_a(var_reg)
                                    .with_reg_b(o_register)],
                            );
                        }

                        VarAllocationKind::Memory(var_addr) => {
                            // Scratchpad register to hold the variable's memory address in.
                            let addr_var = self.special_var(index)?;
                            let addr_reg = addr_var.kind.as_register().unwrap();

                            self.extend_output(
                                index,
                                [
                                    // Load the variable address to the address scratchpad register.
                                    Instruction::new(InstructionKind::LoadI)
                                        .with_reg_a(addr_reg)
                                        .with_immediate(var_addr),
                                    // Store the data in O register to the address of the variable.
                                    Instruction::new(InstructionKind::Store)
                                        .with_reg_a(addr_reg)
                                        .with_reg_b(o_register),
                                ],
                            );
                        }
                    }
                }

                LIRInstruction::Cpy => {
                    self.transform_accumulator_instruction(
                        index,
                        InstructionKind::Cpy,
                        &mut o_register,
                        &ia_register,
                        &ib_register,
                    )?;
                }

                LIRInstruction::Add => {
                    self.transform_accumulator_instruction(
                        index,
                        InstructionKind::Add,
                        &mut o_register,
                        &ia_register,
                        &ib_register,
                    )?;
                }

                LIRInstruction::Sub => {
                    self.transform_accumulator_instruction(
                        index,
                        InstructionKind::Sub,
                        &mut o_register,
                        &ia_register,
                        &ib_register,
                    )?;
                }

                LIRInstruction::NativeMachinecode { code } => {
                    let mut code_iter = code.iter().peekable();

                    while code_iter.peek().is_some() {
                        let mut next = || {
                            code_iter
                                .next()
                                .map(|byte_ref| *byte_ref)
                                .ok_or_else(|| anyhow!("Incomplete instruction"))
                        };

                        let instruction_word = libisa::bytes_to_word([next()?, next()?]);
                        let mut instruction_deassembled =
                            Instruction::deassemble_instruction_word(instruction_word)
                                .map_err(|e| anyhow!("Invalid instruction: {}", e))?;

                        if instruction_deassembled.kind.has_immediate() {
                            let immediate_word = libisa::bytes_to_word([next()?, next()?]);
                            instruction_deassembled.immediate = Some(immediate_word);
                        }

                        self.extend_output(index, [instruction_deassembled]);
                    }
                }

                x => {
                    todo!("{:?}", x)
                }
            }
        }

        let output: Vec<_> = self.output.drain(..).collect();

        let instruction_mappings_extra = {
            let mappings: Vec<_> = self.extra_instruction_mappings.drain(..).collect();

            if mappings.len() != output.len() {
                panic!("Instruction mapping extras size doesn't match output size");
            }

            serde_json::to_string(&mappings).expect("Couldn't serialize instruction mapping extras")
        };

        let var_alloc_extra_json = {
            // Must convert keys to string for serialization to work.
            let allocs: IndexMap<_, _> = self
                .var_table
                .allocations
                .iter()
                .map(|(key, value)| (format!("{:?}", key), value))
                .collect();

            serde_json::to_string(&allocs)
                .expect("Couldn't serialize var table allocations with serde_json")
        };

        let var_alloc_extra_rmp = rmp_serde::to_vec(&self.var_table.allocations)
            .expect("Couldn't serialize var table allocations with rmp_serde");

        Ok(input
            .new_preserve_extras(output)
            .with_extra(
                EXTRA_INSTRUCTION_MAPPINGS_KEY,
                instruction_mappings_extra.bytes(),
            )
            .with_extra(EXTRA_VAR_ALLOCATIONS_KEY_JSON, var_alloc_extra_json.bytes())
            .with_extra(EXTRA_VAR_ALLOCATIONS_KEY_RMP, var_alloc_extra_rmp))
    }
}

impl STRM1CodegenTransformer {
    fn transform_load_const(
        &mut self,
        index: usize,
        acc_reg: &mut Option<Register>,
        value: Word,
    ) -> anyhow::Result<()> {
        let acc_var = self.special_var(index)?;
        *acc_reg = Some(acc_var.kind.as_register().unwrap());

        self.extend_output(
            index,
            [Instruction::new(InstructionKind::LoadI)
                .with_reg_a(acc_reg.unwrap())
                .with_immediate(value)],
        );

        Ok(())
    }

    fn transform_load_var(
        &mut self,
        index: usize,
        acc_reg: &mut Option<Register>,
        var_id: LIRVarId,
    ) -> anyhow::Result<()> {
        let acc_var = self.special_var(index)?;
        *acc_reg = Some(acc_var.kind.as_register().unwrap());

        let var = self.var(var_id)?;

        match var.kind {
            // Simple register copy. This could be made to be a no-op, but that would make variable
            // allocation more complex, and I think it's simpler to optimize this away later on.
            VarAllocationKind::Register(var_reg) => self.extend_output(
                index,
                [Instruction::new(InstructionKind::Cpy)
                    .with_reg_a(acc_reg.unwrap())
                    .with_reg_b(var_reg)],
            ),

            VarAllocationKind::Memory(var_addr) => self.extend_output(
                index,
                [
                    Instruction::new(InstructionKind::LoadI)
                        .with_reg_a(acc_reg.unwrap())
                        .with_immediate(var_addr),
                    Instruction::new(InstructionKind::Load)
                        .with_reg_a(acc_reg.unwrap())
                        .with_reg_b(acc_reg.unwrap()),
                ],
            ),
        }

        Ok(())
    }

    fn transform_load_label(
        &mut self,
        index: usize,
        acc_reg: &mut Option<Register>,
    ) -> anyhow::Result<()> {
        let label_addr = self
            .output
            .len()
            .try_into()
            .context("Label out of maximum address space")?;
        self.transform_load_const(index, acc_reg, label_addr)
    }

    fn transform_accumulator_instruction(
        &mut self,
        index: usize,
        instruction_kind: InstructionKind,
        o: &mut Option<Register>,
        ia: &Option<Register>,
        ib: &Option<Register>,
    ) -> anyhow::Result<()> {
        let o_var = self.special_var(index).unwrap().kind;
        let o_reg = o_var.as_register().unwrap();
        *o = Some(o_reg);

        let ia = ia.context("Input accumulator A not set")?;
        let ib = ib.context("Input accumulator B not set")?;

        self.extend_output(
            index,
            [
                // Copy IA to O to not mutate IA
                Instruction::new(InstructionKind::Cpy)
                    .with_reg_a(o_reg)
                    .with_reg_b(ia),
                Instruction::new(instruction_kind)
                    .with_reg_a(o_reg)
                    .with_reg_b(ib),
            ],
        );

        Ok(())
    }

    fn extend_output<I>(&mut self, index: usize, instruction_iter: I)
    where
        I: IntoIterator<Item = Instruction>,
    {
        let instructions: Vec<_> = instruction_iter.into_iter().collect();
        self.extra_instruction_mappings
            .extend(iter::repeat(index).take(instructions.len()));
        self.output.extend(instructions);
    }

    fn special_var(&self, index: usize) -> anyhow::Result<&VarAllocation> {
        self.var_table
            .get(VarKey::Special(index))
            .ok_or_else(|| anyhow!("Special variable not allocated"))
    }

    fn var(&self, id: LIRVarId) -> anyhow::Result<&VarAllocation> {
        self.var_table
            .get(VarKey::Normal(id))
            .ok_or_else(|| anyhow!("Undefined variable"))
    }
}
