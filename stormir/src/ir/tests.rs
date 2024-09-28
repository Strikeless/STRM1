use emulator::Emulator;

use crate::transformer::Transformer;

use super::LirOp;

// These tests are really poorly made, and will certainly not catch everything.

#[test]
fn set_variable() {
    let value = 1337;

    let emulation = emulate_lir(vec![
        LirOp::InitVar(0),
        LirOp::LoadConstant(value),
        LirOp::StoreAccumulator(0),
        LirOp::LoadConstant(0), // Unused constant to try to overwrite the previous constant from registers
        LirOp::Finish,
    ]);

    assert!(emulation.gpr_file.iter().any(|gpr| *gpr == value));
}

#[test]
fn add_variables() {
    let value_a = 1337;
    let value_b = 2024;
    let sum = value_a + value_b;

    let emulation = emulate_lir(vec![
        LirOp::InitVar(0),
        LirOp::LoadConstant(value_a),
        LirOp::StoreAccumulator(0),
        
        LirOp::InitVar(1),
        LirOp::LoadConstant(value_b),
        LirOp::StoreAccumulator(1),

        LirOp::LoadAccumulator(0),
        LirOp::LoadSecondary(1),
        LirOp::Add,

        LirOp::InitVar(2),
        LirOp::StoreAccumulator(2),
        
        LirOp::LoadConstant(0), // Unused constant to try to overwrite the previous constant from registers
        LirOp::Finish,
    ]);

    assert!(emulation.gpr_file.iter().any(|gpr| *gpr == sum));
    assert!(emulation.gpr_file.iter().any(|gpr| *gpr == value_a));
    assert!(emulation.gpr_file.iter().any(|gpr| *gpr == value_b));
}

fn emulate_lir(lir: Vec<LirOp>) -> Emulator {
    let output = crate::lir_pipeline().transform(lir).unwrap();

    let mut emulator = Emulator::new(output.as_slice());
    emulator.execute_to_halt();

    emulator
}
