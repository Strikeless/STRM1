use std::fs;

use backend::Backend;
use lir::{LIRInstruction, LIRVarKey};

mod backend;
mod lir;

fn main() {
    let input = LIRVarKey(0);
    let output = LIRVarKey(1);

    let ir = Vec::from([
        LIRInstruction::InitVar(input),
        LIRInstruction::ConstantA(1337),
        LIRInstruction::StoreA(input),

        LIRInstruction::InitVar(output),
        LIRInstruction::LoadA(input),
        LIRInstruction::StoreA(output),
        
        LIRInstruction::LoadA(output),
        LIRInstruction::LoadB(input),
        LIRInstruction::Add,
        LIRInstruction::StoreA(output),
        LIRInstruction::DropVar(input),
    ]);

    let output = Backend::new(&ir).compile().unwrap();
    fs::write("out.bin", output).unwrap();
}
