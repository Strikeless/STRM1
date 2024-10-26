use libisa::{
    instruction::{kind::InstructionKind, Instruction},
    Word,
};

use crate::{tracing::pc::PCTraceData, Emulator};

#[test]
fn nop_and_halt() {
    exec(vec![
        Instruction::new(InstructionKind::Nop),
        Instruction::new(InstructionKind::Halt),
    ]);
}

#[test]
fn immediate_store() {
    let addr = 1024;
    let value = 1337;

    let emulator = exec(vec![
        Instruction::new(InstructionKind::LoadI)
            .with_immediate(addr)
            .with_reg_a(0),
        Instruction::new(InstructionKind::LoadI)
            .with_immediate(value)
            .with_reg_a(1),
        Instruction::new(InstructionKind::Store)
            .with_reg_a(0)
            .with_reg_b(1),
        Instruction::new(InstructionKind::Halt),
    ]);

    assert_eq!(emulator.memory.word(addr).unwrap(), value)
}

#[test]
fn addition() {
    let a = 1;
    let b = 2;
    let expected = a + b;

    let emulator = exec(vec![
        Instruction::new(InstructionKind::LoadI)
            .with_immediate(a)
            .with_reg_a(0),
        Instruction::new(InstructionKind::LoadI)
            .with_immediate(b)
            .with_reg_a(1),
        Instruction::new(InstructionKind::Add)
            .with_reg_a(0)
            .with_reg_b(1),
        Instruction::new(InstructionKind::Halt),
    ]);

    assert_eq!(emulator.reg(0), expected)
}

#[test]
fn register_traces() {
    let reg_a = 0;
    let reg_b = 1;

    let emulator = exec(vec![
        // No-op to start actual traces from PC 1 instead of 0 which could be a default value.
        Instruction::new(InstructionKind::Nop),
        // Words 1 + 2 (immediate): Load an immediate to reg A, adding a trace to reg A.
        Instruction::new(InstructionKind::LoadI)
            .with_immediate(1337)
            .with_reg_a(reg_a),
        // Words 3 + 4 (immediate): Load an immediate to reg B, adding a trace to reg B.
        Instruction::new(InstructionKind::LoadI)
            .with_immediate(1000)
            .with_reg_a(reg_b),
        // Word 5: Copy reg A to reg B, adding a trace to reg B.
        Instruction::new(InstructionKind::Cpy)
            .with_reg_a(reg_b)
            .with_reg_b(reg_a),
        Instruction::new(InstructionKind::Halt),
    ]);

    let trace_a = emulator.reg_file.trace(reg_a).unwrap();
    let trace_b = emulator.reg_file.trace(reg_b).unwrap();

    assert_eq!(trace_a.traces, [1 * libisa::BYTES_PER_WORD], "register A");
    assert_eq!(
        trace_b.traces,
        [3 * libisa::BYTES_PER_WORD, 5 * libisa::BYTES_PER_WORD],
        "register B"
    );
}

fn exec(instructions: Vec<Instruction>) -> Emulator<PCTraceData> {
    let program = instructions
        .iter()
        .flat_map(|instruction| {
            instruction
                .assemble()
                .expect("Failed to assemble instruction")
        })
        .collect();

    let mut emulator = Emulator::new(Word::MAX, program).unwrap();
    emulator
        .execute_to_halt()
        .expect("Error executing instruction");

    emulator
}
