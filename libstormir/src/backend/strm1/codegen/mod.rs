use anyhow::anyhow;
use libstrmisa::{
    instruction::{kind::InstructionKind, Instruction},
    Register, Word,
};
use var::{builder::VarTableBuilder, VarAllocation, VarAllocationKind, VarKey, VarTable};

use crate::{
    lir::{LIRInstruction, LIRVarId},
    transformer::Transformer,
};

mod var;

pub struct STRM1CodegenTransformer {
    var_table: VarTable,
    output: Vec<Instruction>,
}

impl STRM1CodegenTransformer {
    pub fn new() -> Self {
        Self {
            var_table: VarTable::default(),
            output: Vec::new(),
        }
    }

    fn transform_load_const(
        &mut self,
        index: usize,
        acc_reg: &mut Option<Register>,
        value: Word,
    ) -> anyhow::Result<()> {
        let acc_var = self.special_var(index)?;
        *acc_reg = Some(acc_var.kind.as_register().unwrap());

        self.output.extend([Instruction::new(InstructionKind::LoadI)
            .with_reg_a(acc_reg.unwrap())
            .with_immediate(value)]);

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
            VarAllocationKind::Register(var_reg) => {
                self.output.extend([Instruction::new(InstructionKind::Cpy)
                    .with_reg_a(acc_reg.unwrap())
                    .with_reg_b(var_reg)])
            }

            VarAllocationKind::Memory(var_addr) => self.output.extend([
                Instruction::new(InstructionKind::LoadI)
                    .with_reg_a(acc_reg.unwrap())
                    .with_immediate(var_addr),
                Instruction::new(InstructionKind::Load)
                    .with_reg_a(acc_reg.unwrap())
                    .with_reg_b(acc_reg.unwrap()),
            ]),
        }

        Ok(())
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

impl Transformer for STRM1CodegenTransformer {
    type Input = LIRInstruction;
    type Output = Instruction;

    fn prepass(&mut self, input: &Vec<Self::Input>) -> anyhow::Result<()> {
        let mut var_table_builder = VarTableBuilder::new();

        let mut ia_definition_key = None;
        let mut ib_definition_key = None;

        fn define_load_label(
            index: usize,
            var_table_builder: &mut VarTableBuilder,
            definition_key: &mut Option<VarKey>,
        ) {
            let key = VarKey::Special(index);

            // Previous input accumulator allocation can be dropped as we overwrite it here.
            if let Some(previous_key) = definition_key.take() {
                var_table_builder.drop(previous_key, 0).unwrap();
            }

            var_table_builder.define(key, true).unwrap();

            *definition_key = Some(key);
        }

        for (index, lir_instruction) in input.iter().enumerate() {
            var_table_builder.set_current_index(index);

            match lir_instruction {
                LIRInstruction::DefineVar { id } => {
                    var_table_builder.define(VarKey::Normal(*id), false)?
                }
                LIRInstruction::DropVar { id } => var_table_builder.drop(VarKey::Normal(*id), 0)?,

                //
                // Loads to input accumulators need a special register allocation to hold the value in
                //
                LIRInstruction::LoadIAConst { .. }
                | LIRInstruction::LoadIAVar { .. }
                | LIRInstruction::LoadIALabel => {
                    define_load_label(index, &mut var_table_builder, &mut ia_definition_key)
                }

                LIRInstruction::LoadIBConst { .. }
                | LIRInstruction::LoadIBVar { .. }
                | LIRInstruction::LoadIBLabel => {
                    define_load_label(index, &mut var_table_builder, &mut ib_definition_key)
                }

                LIRInstruction::StoreOVar { .. } => {
                    // The LIR documentation allows dropping input accumulators (just specified it there hehe) upon
                    // an output accumulator store, so do that here for simplicity.
                    if let Some(ia_definition_key) = ia_definition_key.take() {
                        var_table_builder.drop(ia_definition_key, 0).unwrap();
                    }
                    if let Some(ib_definition_key) = ib_definition_key.take() {
                        var_table_builder.drop(ib_definition_key, 0).unwrap();
                    }

                    // NOTE: This is reserving a register for a memory address even if the target variable is in-register,
                    //       so small optimization opportunity here. With the current VarTable builder approach, we would
                    //       need atleast a second prepass in order to only reserve a register for in-memory variables.
                    // Store needs a scratchpad register to store the address of an in-memory target variable.
                    let key = VarKey::Special(index);

                    var_table_builder.define(key, true).unwrap();

                    // The register is only needed during execution of this instruction, so drop it by the next instruction.
                    var_table_builder.drop(key, 1).unwrap();
                }

                _ => {}
            }
        }

        self.var_table = var_table_builder.build()?;
        Ok(())
    }

    fn transform(&mut self, input: Vec<Self::Input>) -> anyhow::Result<Vec<Self::Output>> {
        let mut ia_register = None;
        let mut ib_register = None;

        for (index, lir_instruction) in input.iter().enumerate() {
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

                _ => todo!(),
            }
        }

        Ok(self.output.drain(..).collect())
    }
}
