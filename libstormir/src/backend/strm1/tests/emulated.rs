/* Tests that verify behavior of compiled LIR by emulating the output and examining the emulator's state. */

use crate::backend::strm1::codegen::alloc::tests::TestEmulateExt;

use super::{Test, LIR_HALT};

#[test]
fn simple_halt() {
    let program = [LIR_HALT.clone()];

    Test::new("simple_halt", program).emulate_dump_panicking(|test| test.run_till_halt())
}
