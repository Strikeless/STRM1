use itertools::{ExactlyOneError, Itertools};
use lazy_static::lazy_static;
use libemulator::{tracing::pc::PCTraceData, Emulator};
use libstrmisa::{
    instruction::{kind::InstructionKind, Instruction},
    Word,
};

use crate::{backend::strm1, lir::LIRInstruction, transformer::Transformer};

lazy_static! {
    static ref LIR_HALT: LIRInstruction = LIRInstruction::NativeMachinecode {
        code: Instruction::new(InstructionKind::Halt).assemble().unwrap()
    };
}

#[test]
fn addition_emulated() {
    let a = 1;
    let b = 2;
    let expected = a + b;

    let test = EmulatorTest::new(vec![
        LIRInstruction::LoadIAConst { value: a },
        LIRInstruction::LoadIBConst { value: b },
        LIRInstruction::Add,
        LIRInstruction::DefineVar { id: 0 },
        LIRInstruction::StoreOVar { id: 0 },
        LIR_HALT.clone(),
    ])
    .unwrap();

    // With tracing info from the backend these emulator tests could be made way more rigid.
    let result = test.single_reg(|(_, value)| *value == expected);

    // The error type doesn't implement Debug so we can't use expect, seems to implement Display though...
    if let Err(e) = result {
        panic!("Not exactly one expected register: {}", e);
    }
}

struct EmulatorTest {
    emulator: Emulator<PCTraceData>,
}

impl EmulatorTest {
    pub fn new(program: Vec<LIRInstruction>) -> anyhow::Result<Self> {
        let program = strm1::transformer().run(program).expect("Error compiling");

        let mut emulator = Emulator::new(Word::MAX, program).unwrap();
        emulator.execute_to_halt().expect("Error executing");

        Ok(Self { emulator })
    }

    // Holy shit
    pub fn single_reg<'a, P>(
        &'a self,
        predicate: P,
    ) -> Result<(usize, Word), ExactlyOneError<impl Iterator<Item = (usize, Word)> + 'a>>
    where
        P: FnMut(&(usize, Word)) -> bool + 'a,
    {
        self.emulator
            .reg_file
            .iter_untraced()
            .enumerate()
            .map(|(key, value_ref)| (key, *value_ref))
            .filter(predicate)
            .exactly_one()
    }
}
