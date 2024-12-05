use lazy_static::lazy_static;
use libisa::instruction::{kind::InstructionKind, Instruction};

use crate::{
    lir::LIRInstruction,
    transformer::{extra::Extra, runner::TransformerRunnerExt},
};

use super::STRM1Transformer;

mod emulated;

lazy_static! {
    static ref LIR_HALT: LIRInstruction = LIRInstruction::NativeMachinecode {
        code: Instruction::new(InstructionKind::Halt).assemble().unwrap()
    };
}

pub struct Test {
    pub name: &'static str,
    pub compilation_output: Extra<Vec<u8>>,
}

impl Test {
    pub fn new<I>(name: &'static str, lir: I) -> Self
    where
        I: IntoIterator<Item = LIRInstruction>,
    {
        let lir = lir.into_iter().collect();

        let compilation_output = STRM1Transformer::new()
            .runner()
            .run(lir)
            .expect("Error compiling LIR");

        Self {
            name,
            compilation_output,
        }
    }
}
