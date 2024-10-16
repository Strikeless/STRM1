use libstrmisa::instruction::{kind::InstructionKind, Instruction};

use crate::Emulator;

#[test]
fn immediate_store() {
    let addr = 10;
    let value = 1337;

    let emulator = exec_parsed(vec![
        Instruction::new(InstructionKind::LoadI)
            .with_immediate(addr)
            .with_reg_a(0),

        Instruction::new(InstructionKind::LoadI)
            .with_immediate(value)
            .with_reg_a(1),

        Instruction::new(InstructionKind::Store)
            .with_reg_a(0)
            .with_reg_b(1)
    ]);

    assert_eq!(emulator.memory.word(addr).unwrap(), value)
}

#[test]
fn add() {
    let a = 1;
    let b = 2;
    let expected = a + b;

    let emulator = exec_parsed(vec![
        Instruction::new(InstructionKind::LoadI)
            .with_immediate(a)
            .with_reg_a(0),

        Instruction::new(InstructionKind::LoadI)
            .with_immediate(b)
            .with_reg_a(1),

        Instruction::new(InstructionKind::Add)
            .with_reg_a(0)
            .with_reg_b(1)
    ]);

    assert_eq!(emulator.register(0), expected)
}

fn exec_parsed(instructions: Vec<Instruction>) -> Emulator {
    // The memory size is arbitrary. These tests don't need much.
    let mut emulator = Emulator::new(256, vec![]).unwrap();

    for instruction in instructions {
        emulator.execute_parsed_instruction(instruction).unwrap();
    }

    emulator
}
