use std::{fs, path::PathBuf};

use indexmap::IndexMap;
use lazy_static::lazy_static;
use libemulator::{tracing::pc::PCTraceData, Emulator};
use libisa::{
    instruction::{kind::InstructionKind, Instruction},
    Word,
};

use crate::{
    backend::strm1,
    lir::LIRInstruction,
    transformer::{extra::Extra, runner::TransformerRunner},
};

use super::var::{VarAllocation, VarAllocationKind, VarKey};

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

    let actual = test.var_value(0);

    if actual != Some(expected) {
        test.dump_panic(
            "addition",
            format!("Expected: {:?}, got: {:?}", Some(expected), actual),
        );
    }
}

#[test]
fn two_additions_emulated() {
    let a = 1;
    let b = 2;
    let c = 3;

    let first_expected = a + b;
    let second_expected = first_expected + c;

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
        LIRInstruction::Add,
        LIRInstruction::DefineVar { id: 1 },
        LIRInstruction::StoreOVar { id: 1 },
        LIR_HALT.clone(),
    ])
    .unwrap();

    let first_actual = test.var_value(0);
    let second_actual = test.var_value(1);

    if first_actual != Some(first_expected) {
        test.dump_panic(
            "two_additions",
            format!(
                "First expected: {:?}, got: {:?}",
                Some(first_expected),
                first_actual
            ),
        );
    }

    if second_actual != Some(second_expected) {
        test.dump_panic(
            "two_additions",
            format!(
                "Second expected: {:?}, got: {:?}",
                Some(second_expected),
                second_actual
            ),
        );
    }
}

#[test]
// It would be a shame if the other tests failed or succeeded randomly. Determinism isn't exactly something we need
// in this codegen, but so far I've only seen non-determinism be caused by actual problems that should be found.
// This test mostly focuses on variable allocations, most instructions aren't even used.
fn binary_determinism() {
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

    let mut previous_binary = TransformerRunner::new(&mut strm1::transformer())
        .run(program_lir.clone())
        .expect("Compilation failed on first run")
        .data;

    for i in 2..=50 {
        let new_binary = TransformerRunner::new(&mut strm1::transformer())
            .run(program_lir.clone())
            .expect(&format!("Compilation failed on run {}", i))
            .data;

        assert_eq!(
            previous_binary, new_binary,
            "Compilations differed on run {}",
            i
        );

        previous_binary = new_binary;
    }
}

struct EmulatorTest {
    program: Extra<Vec<u8>>,
    var_allocs: IndexMap<VarKey, VarAllocation>,
    emulator: Emulator<PCTraceData>,
}

impl EmulatorTest {
    pub fn new(program: Vec<LIRInstruction>) -> anyhow::Result<Self> {
        let program = TransformerRunner::new(&mut strm1::transformer())
            .run(program)
            .expect("Error compiling");

        let var_allocs = {
            let rmp = program
                .extra
                .get(strm1::codegen::EXTRA_VAR_ALLOCATIONS_KEY_RMP)
                .expect("No var alloc rmp extra in transformer output");

            rmp_serde::from_slice(&rmp).expect("Couldn't deserialize var alloc rmp extra")
        };

        let mut emulator = Emulator::new(Word::MAX, program.data.clone()).unwrap();
        emulator.execute_to_halt().expect("Error executing");

        Ok(Self {
            program,
            var_allocs,
            emulator,
        })
    }

    pub fn var_value(&self, var_id: usize) -> Option<Word> {
        let alloc = self.var_allocs.get(&VarKey::Normal(var_id))?;

        match alloc.kind {
            VarAllocationKind::Register(reg_index) => {
                Some(self.emulator.reg_file.register(reg_index))
            }
            VarAllocationKind::Memory(mem_addr) => self.emulator.memory.word(mem_addr),
        }
    }

    pub fn dump_panic(&self, name: &'static str, panic_msg: String) {
        let dir_path = PathBuf::from("target").join("emulatortest").join(name);

        println!(
            "Dumping compilation output to '{}'",
            dir_path.to_string_lossy().to_string()
        );

        fs::create_dir_all(&dir_path).unwrap();

        let binary_path = dir_path.join("program.bin");
        fs::write(binary_path, &self.program.data).unwrap();

        for (key, value) in &self.program.extra {
            let path = dir_path.join(format!("extra-{}", key));
            fs::write(path, value).unwrap();
        }

        panic!("{}", panic_msg)
    }
}
