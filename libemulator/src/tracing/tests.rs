use libisa::instruction::{kind::InstructionKind, Instruction};

use crate::Emulator;

#[test]
fn iteration_trace_gets_created() -> anyhow::Result<()> {
    let program = [Instruction::new(InstructionKind::Nop).assemble()?]
        .into_iter()
        .flatten()
        .collect();

    let mut emulator = Emulator::new(program)?;
    emulator.execute_instruction()?;

    emulator
        .tracing
        .trace_by_pc(0)
        .expect("No trace was created")
        .iteration(0)
        .expect("No iteration trace was created");

    Ok(())
}
