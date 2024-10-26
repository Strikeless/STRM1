use std::{fs, path::PathBuf};

use itertools::{ExactlyOneError, Itertools};
use lazy_static::lazy_static;
use libemulator::{tracing::pc::PCTraceData, Emulator};
use libisa::{
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
        test.dump_panic(
            "addition",
            format!("Not exactly one expected value in register: {}", e),
        );
    }
}

#[test]
fn two_additions_emulated() {
    let a = 1;
    let b = 2;
    let c = 3;

    let expected_first = a + b;
    let expected_second = expected_first + c;

    let test = EmulatorTest::new(vec![
        LIRInstruction::LoadIAConst { value: a },
        LIRInstruction::LoadIBConst { value: b },
        LIRInstruction::Add,
        LIRInstruction::DefineVar { id: 0 },
        LIRInstruction::StoreOVar { id: 0 },
        LIRInstruction::LoadIAVar { id: 0 },
        LIRInstruction::LoadIBConst { value: c },
        LIRInstruction::DefineVar { id: 1 },
        LIRInstruction::StoreOVar { id: 1 },
        LIR_HALT.clone(),
    ])
    .unwrap();

    let first_result = test.single_reg(|(_, value)| *value == expected_first);
    let second_result = test.single_reg(|(_, value)| *value == expected_second);

    if let Err(e) = first_result {
        test.dump_panic(
            "two_additions",
            format!("Not exactly one expected first value in register: {}", e),
        );
    }

    if let Err(e) = second_result {
        test.dump_panic(
            "two_additions",
            format!("Not exactly one expected second value in register: {}", e),
        );
    }
}

struct EmulatorTest {
    program: Vec<u8>,
    emulator: Emulator<PCTraceData>,
}

impl EmulatorTest {
    pub fn new(program: Vec<LIRInstruction>) -> anyhow::Result<Self> {
        let program = strm1::transformer().run(program).expect("Error compiling");

        let mut emulator = Emulator::new(Word::MAX, program.clone()).unwrap();
        emulator.execute_to_halt().expect("Error executing");

        Ok(Self { program, emulator })
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

    pub fn dump_panic(&self, name: &'static str, panic_msg: String) {
        let dir_path = PathBuf::from("target").join("emulatortest");
        let file_path = dir_path.join(PathBuf::from(name).with_extension("bin"));

        println!(
            "Dumping program binary in '{}'",
            file_path.to_string_lossy().to_string()
        );

        fs::create_dir_all(&dir_path).unwrap();
        fs::write(file_path, &self.program).unwrap();

        panic!("{}", panic_msg)
    }
}
