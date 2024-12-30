/* Tests that verify behavior of compiled LIR by emulating the output and examining the emulator's state. */

use anyhow::{anyhow, Context};

use crate::{
    backend::strm1::codegen::alloc::tests::TestEmulateExt,
    lir::{LIRInstruction, LIRValue},
};

use super::{Test, LIR_HALT};

#[test]
fn simple_halt() {
    let program = [LIR_HALT.clone()];

    Test::new("simple_halt", program).emulate_dump_panicking(|test| test.run_till_halt())
}

#[test]
fn variable_assignment_ignorant() {
    let var_id = 1;
    let expected_value = 0xABCD;

    let program = [
        LIRInstruction::Const {
            id: var_id,
            value: LIRValue::Uint16(expected_value),
        },
        LIR_HALT.clone(),
    ];

    Test::new("variable_assignment_ignorant", program).emulate_dump_panicking(|test| {
        test.run_till_halt()?;

        let var_value = test
            .get_var_ignorant(var_id)
            .context("Variable wasn't found")?;

        if var_value != expected_value {
            return Err(anyhow!(
                "Variable value {} differs from expected {}",
                var_value,
                expected_value
            ));
        }

        if false {
            return Ok(());
        }

        Err(anyhow!("success"))
    });
}
