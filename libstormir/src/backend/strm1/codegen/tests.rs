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

// TODO: Make the backend emit tracing data for variables so emulated tests can access the correct
//       registers / memory addresses without any hackery like the currently used single_reg method.

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
        // First addition: var 0 = a + b
        LIRInstruction::LoadIAConst { value: a },
        LIRInstruction::LoadIBConst { value: b },
        LIRInstruction::Add,
        LIRInstruction::DefineVar { id: 0 },
        LIRInstruction::StoreOVar { id: 0 },
        // Second addition: var 1 = var 0 + c
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

#[test]
fn chained_additions_emulated() {
    let a = 1;
    let b = 2;
    let c = 3;

    let expected = a + b + c;

    let test = EmulatorTest::new(vec![
        // First addition: a + b
        LIRInstruction::LoadIAConst { value: a },
        LIRInstruction::LoadIBConst { value: b },
        LIRInstruction::Add,
        // Second addition: previous + c
        LIRInstruction::LoadIBConst { value: c },
        // Store to var 0
        LIRInstruction::DefineVar { id: 0 },
        LIRInstruction::StoreOVar { id: 0 },
        LIR_HALT.clone(),
    ])
    .unwrap();

    let result = test.single_reg(|(_, value)| *value == expected);

    // The error type doesn't implement Debug so we can't use expect, seems to implement Display though...
    if let Err(e) = result {
        test.dump_panic(
            "chained_additions",
            format!("Not exactly one expected value in register: {}", e),
        );
    }
}

#[test]
// It would be a shame if the other tests failed or succeeded randomly. Determinism isn't exactly something we need
// in this codegen, but so far I've only seen non-determinism be caused by actual problems that should be found.
// This test mostly focuses on variable allocations, most instructions aren't even used.
fn determinism() {
    // Complete nonsense LIR that still compiles.
    let program_lir = vec![
        LIRInstruction::DefineVar { id: 0 },
        LIRInstruction::DefineVar { id: 2 },
        LIRInstruction::LoadIAConst { value: 123 },
        LIRInstruction::DropVar { id: 0 },
        LIRInstruction::LoadIBConst { value: 14723 },
        LIRInstruction::DefineVar { id: 1 },
        LIRInstruction::Add,
        LIRInstruction::Sub,
        LIRInstruction::DefineVar { id: 3 },
        LIR_HALT.clone(),
        LIRInstruction::DropVar { id: 2 },
        LIRInstruction::Add,
        LIRInstruction::Cpy,
        LIR_HALT.clone(),
        LIRInstruction::StoreOVar { id: 2 },
    ];

    let mut previous_compilation = strm1::transformer()
        .run(program_lir.clone())
        .expect("Compilation failed on first run");

    for i in 2..=50 {
        let new_compilation = strm1::transformer()
            .run(program_lir.clone())
            .expect(&format!("Compilation failed on run {}", i));

        assert_eq!(
            previous_compilation, new_compilation,
            "Compilations differed on run {}",
            i
        );

        previous_compilation = new_compilation;
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
            "Dumping generated program binary to '{}'",
            file_path.to_string_lossy().to_string()
        );

        fs::create_dir_all(&dir_path).unwrap();
        fs::write(file_path, &self.program).unwrap();

        panic!("{}", panic_msg)
    }
}
