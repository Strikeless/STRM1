use libstrmisa::instruction::Instruction;
use var::{builder::VarTableBuilder, VarKey, VarTable};

use crate::{lir::LIRInstruction, transformer::Transformer};

mod var;

pub struct STRM1CodegenTransformer {
    var_table: VarTable,
}

impl STRM1CodegenTransformer {
    pub fn new() -> Self {
        Self {
            var_table: VarTable::default(),
        }
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
                LIRInstruction::DefineVar { id } => var_table_builder.define_normal(*id)?,
                LIRInstruction::DropVar { id } => var_table_builder.drop_normal(*id)?,

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
        let mut output = Vec::new();

        //let mut ia_register = None;
        //let mut ib_register = None;

        for lir_instruction in input {
            match lir_instruction {
                // Handled during prepass
                LIRInstruction::DefineVar { .. } | LIRInstruction::DropVar { .. } => {}

                _ => todo!(),
            }
        }

        Ok(output)
    }
}
