use anyhow::anyhow;
use itertools::Itertools;
use lazy_static::lazy_static;
use libdeassembler::Deassembler;

use crate::{
    backend::strm1::codegen::prealloc::{RegVarKey, VarId, VarKey},
    lir::{LIRInstruction, LIRValue, LIRVarId},
    transformer::{extra::Extra, Transformer},
};

use super::{varidspace::VarIdSpace, PreallocInstruction};

lazy_static! {
    // Public for VarAllocTest
    pub(crate) static ref LIR_VAR_SPACE: VarIdSpace = VarIdSpace::new();

    static ref INTERNAL_VAR_SPACE: VarIdSpace = VarIdSpace::new();

    // A second internal space may be needed to have two simultaneously allocated internal registers.
    static ref SECOND_INTERNAL_VAR_SPACE: VarIdSpace = VarIdSpace::new();
}

pub struct PreallocCodegenTransformer {}

impl Transformer for PreallocCodegenTransformer {
    type Input = Vec<LIRInstruction>;
    type Output = Vec<PreallocInstruction>;

    fn transform(&mut self, input: Extra<Self::Input>) -> anyhow::Result<Extra<Self::Output>> {
        input.try_map_data(|lir| {
            lir.into_iter()
                .enumerate()
                .map(|(instruction_index, instruction)| {
                    self.transform_instruction(instruction_index, instruction)
                })
                .flatten_ok()
                .try_collect()
        })
    }
}

impl PreallocCodegenTransformer {
    fn transform_instruction(
        &mut self,
        instruction_index: usize,
        instruction: LIRInstruction,
    ) -> anyhow::Result<Vec<PreallocInstruction>> {
        Ok(match instruction {
            LIRInstruction::Const { id, value } => {
                let tmp_reg_key = Self::internal_reg_key(instruction_index);
                let out_var_key = Self::lir_var_key(id);

                match value {
                    LIRValue::Uint16(value) => vec![
                        // First load the value to a temporary register...
                        PreallocInstruction::DefineVar(VarKey::Register(tmp_reg_key)),
                        PreallocInstruction::LoadImmediate {
                            dest: tmp_reg_key,
                            value,
                        },
                        // ...and then store to the new output variable.
                        PreallocInstruction::DefineVar(out_var_key),
                        PreallocInstruction::StoreVar {
                            dest: out_var_key,
                            src: tmp_reg_key,
                        },
                    ],

                    _ => todo!(),
                }
            }

            LIRInstruction::Copy { id, src } => {
                let tmp_reg_key = Self::internal_reg_key(instruction_index);
                let out_var_key = Self::lir_var_key(id);
                let src_var_key = Self::lir_var_key(src);

                vec![
                    // First load the source value to a temporary register...
                    PreallocInstruction::DefineVar(VarKey::Register(tmp_reg_key)),
                    PreallocInstruction::LoadVar {
                        dest: tmp_reg_key,
                        src: src_var_key,
                    },
                    // ...and then store to the new output variable.
                    PreallocInstruction::DefineVar(out_var_key),
                    PreallocInstruction::StoreVar {
                        dest: out_var_key,
                        src: tmp_reg_key,
                    },
                ]
            }

            LIRInstruction::Add { id, a, b } => {
                self.transform_dual_operand(instruction_index, id, a, b, |a, b| {
                    PreallocInstruction::Add(a, b)
                })
            }

            LIRInstruction::Sub { id, a, b } => {
                self.transform_dual_operand(instruction_index, id, a, b, |a, b| {
                    PreallocInstruction::Sub(a, b)
                })
            }

            LIRInstruction::Mul { id, a, b } => {
                todo!()
            }

            LIRInstruction::Branch { addr } => {
                let addr_var_key = Self::lir_var_key(addr);
                let tmp_addr_reg_key = Self::internal_reg_key(instruction_index);

                vec![
                    // Load the address variable to a temporary register and jump to it.
                    PreallocInstruction::DefineVar(VarKey::Register(tmp_addr_reg_key)),
                    PreallocInstruction::LoadVar {
                        dest: tmp_addr_reg_key,
                        src: addr_var_key,
                    },
                    PreallocInstruction::Jmp(tmp_addr_reg_key),
                ]
            }

            LIRInstruction::BranchZero { addr, test } => {
                let addr_var_key = Self::lir_var_key(addr);
                let test_var_key = Self::lir_var_key(test);

                let tmp_addr_reg_key = Self::internal_reg_key(instruction_index);
                let tmp_test_reg_key =
                    Self::custom_internal_reg_key(&SECOND_INTERNAL_VAR_SPACE, instruction_index);

                vec![
                    // Load the address variable to a temporary register so it can be jumped to if the branch is taken.
                    PreallocInstruction::DefineVar(VarKey::Register(tmp_addr_reg_key)),
                    PreallocInstruction::LoadVar {
                        dest: tmp_addr_reg_key,
                        src: addr_var_key,
                    },
                    // Load the test variable to a temporary register so its flags can be loaded.
                    PreallocInstruction::DefineVar(VarKey::Register(tmp_test_reg_key)),
                    PreallocInstruction::LoadVar {
                        dest: tmp_test_reg_key,
                        src: test_var_key,
                    },
                    // And the test register with itself to load its flags.
                    PreallocInstruction::And(tmp_test_reg_key, tmp_test_reg_key),
                    // Finally jump by the address register if the zero flag was set.
                    PreallocInstruction::JmpZ(tmp_addr_reg_key),
                ]
            }

            LIRInstruction::BranchEqual { .. } => unimplemented!("Relying on cmp shim"),

            LIRInstruction::NativeMachinecode { code } => {
                let deassembler = Deassembler::new(code.iter());

                let instructions = deassembler
                    .deassemble()
                    .map_err(|e| anyhow!("Invalid passthrough machinecode: {}", e))?;

                vec![PreallocInstruction::TargetPassthrough { instructions }]
            }
        })
    }

    fn transform_dual_operand<F>(
        &self,
        instruction_index: usize,
        out_id: u64,
        a_id: u64,
        b_id: u64,
        op_callback: F,
    ) -> Vec<PreallocInstruction>
    where
        F: FnOnce(RegVarKey, RegVarKey) -> PreallocInstruction,
    {
        let out_var_key = Self::lir_var_key(out_id);
        let a_var_key = Self::lir_var_key(a_id);
        let b_var_key = Self::lir_var_key(b_id);

        let tmp_a_reg_key = Self::internal_reg_key(instruction_index);
        let tmp_b_reg_key =
            Self::custom_internal_reg_key(&SECOND_INTERNAL_VAR_SPACE, instruction_index);

        vec![
            // Load the A variable to the A tmp register.
            PreallocInstruction::DefineVar(VarKey::Register(tmp_a_reg_key)),
            PreallocInstruction::LoadVar {
                dest: tmp_a_reg_key,
                src: a_var_key,
            },
            // Load the B variable to the B tmp register.
            PreallocInstruction::DefineVar(VarKey::Register(tmp_b_reg_key)),
            PreallocInstruction::LoadVar {
                dest: tmp_b_reg_key,
                src: b_var_key,
            },
            // Run the operation on the A and B tmp registers and then store the output from the A tmp register.
            op_callback(tmp_a_reg_key, tmp_b_reg_key),
            PreallocInstruction::StoreVar {
                dest: out_var_key,
                src: tmp_a_reg_key,
            },
        ]
    }

    fn lir_var_key(lir_id: LIRVarId) -> VarKey {
        VarKey::Generic(VarId(*LIR_VAR_SPACE, lir_id))
    }

    fn internal_reg_key(instruction_index: usize) -> RegVarKey {
        Self::custom_internal_reg_key(&INTERNAL_VAR_SPACE, instruction_index)
    }

    fn custom_internal_reg_key(space: &VarIdSpace, instruction_index: usize) -> RegVarKey {
        RegVarKey(VarId(*space, instruction_index as u64))
    }
}
