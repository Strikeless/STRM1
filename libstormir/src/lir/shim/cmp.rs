use anyhow::anyhow;
use itertools::Itertools;

use crate::{
    lir::{self, LIRInstruction},
    transformer::{extra::Extra, Transformer},
};

/// Shim to replace uses of LIR instructions.
pub struct CmpShimTransformer;

impl Transformer for CmpShimTransformer {
    type Input = Vec<LIRInstruction>;
    type Output = Vec<LIRInstruction>;

    fn transform(&mut self, input: Extra<Self::Input>) -> anyhow::Result<Extra<Self::Output>> {
        input.try_map_data(|data| {
            // This prepass could be removed even without cloning instructions with a more thought out implementation.
            let replacements = data
                .iter()
                .filter(|lir| matches!(lir, LIRInstruction::BranchEqual { .. }))
                .count();

            // Can't use the iterator from free_var_ids after the instructions have been moved by into_iter,
            // which is why we're collecting a fixed amount and reiterating here.
            let mut free_vars = lir::free_var_ids(&data)
                .take(replacements)
                .collect_vec()
                .into_iter();

            data.into_iter()
                .map(|lir| match lir {
                    LIRInstruction::BranchEqual { addr, a, b } => {
                        // Good luck getting LIR large enough to not have free variables this far.
                        let temp_var = free_vars.next().ok_or_else(|| {
                            anyhow!("No free variable IDs in LIR for BranchEqual shim")
                        })?;

                        Ok(vec![
                            LIRInstruction::Sub { id: temp_var, a, b },
                            LIRInstruction::BranchZero {
                                addr,
                                test: temp_var,
                            },
                        ])
                    }
                    x => Ok(vec![x]),
                })
                .flatten_ok()
                .try_collect()
        })
    }
}
