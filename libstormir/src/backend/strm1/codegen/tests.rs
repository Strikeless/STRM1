use std::{fmt::Debug, fs, path::PathBuf};

use anyhow::{anyhow, Context};
use indexmap::IndexMap;
use lazy_static::lazy_static;
use libemulator::{execute::ExecuteOk, tracing::pc::PCTraceData, Emulator};
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

#[test]
fn addition_emulated() {
    let a = 1;
    let b = 2;
    let expected = a + b;

    let var_id = 0;

    let test = Test::new(
        "addition_emulated",
        [
            LIRInstruction::LoadIAConst { value: a },
            LIRInstruction::LoadIBConst { value: b },
            LIRInstruction::Add,
            LIRInstruction::DefineVar { id: var_id },
            LIRInstruction::StoreOVar { id: var_id },
            LIR_HALT.clone(),
        ],
    );

    test.dump_panic_on_err(|| {
        let emulated = test.emulate()?.executed_to_halt()?;

        let actual = emulated.var(var_id);

        if actual != Some(expected) {
            Err(anyhow!("Expected: {:?}, got: {:?}", Some(expected), actual))
        } else {
            Ok(())
        }
    });
}

#[test]
fn two_additions_emulated() {
    let a = 1;
    let b = 2;
    let c = 3;

    let first_expected = a + b;
    let second_expected = first_expected + c;

    let first_var_id = 0;
    let second_var_id = 1;

    let test = Test::new(
        "two_additions_emulated",
        vec![
            // First addition: var 0 = a + b
            LIRInstruction::LoadIAConst { value: a },
            LIRInstruction::LoadIBConst { value: b },
            LIRInstruction::Add,
            LIRInstruction::DefineVar { id: first_var_id },
            LIRInstruction::StoreOVar { id: first_var_id },
            // Second addition: second_var = first_var + c
            LIRInstruction::LoadIAVar { id: first_var_id },
            LIRInstruction::LoadIBConst { value: c },
            LIRInstruction::Add,
            LIRInstruction::DefineVar { id: second_var_id },
            LIRInstruction::StoreOVar { id: second_var_id },
            LIR_HALT.clone(),
        ],
    );

    test.dump_panic_on_err(|| {
        let emulation = test.emulate()?.executed_to_halt()?;

        let first_actual = emulation.var(first_var_id);
        let second_actual = emulation.var(second_var_id);

        if first_actual != Some(first_expected) {
            test.dump_panic(format!(
                "First expected: {:?}, got: {:?}",
                Some(first_expected),
                first_actual
            ));
        }

        if second_actual != Some(second_expected) {
            test.dump_panic(format!(
                "Second expected: {:?}, got: {:?}",
                Some(second_expected),
                second_actual
            ));
        }

        Ok(())
    })
}

#[test]
fn many_chained_additions_emulated() {
    let addition_count = 50;
    let addition_step = 5;
    let initial_value = 2;

    let expected = initial_value + addition_step * addition_count;

    let addition_step_var_id = 0;
    let mut output_var_id = 1;

    let mut program_lir = vec![
        // Set var 0 to contain the addition step value
        LIRInstruction::DefineVar {
            id: addition_step_var_id,
        },
        LIRInstruction::LoadIAConst {
            value: addition_step,
        },
        LIRInstruction::Cpy,
        LIRInstruction::StoreOVar { id: 0 },
        // Set the output var to the init value
        LIRInstruction::DefineVar { id: output_var_id },
        LIRInstruction::LoadIAConst {
            value: initial_value,
        },
        LIRInstruction::Cpy,
        LIRInstruction::StoreOVar { id: output_var_id },
    ];

    for _ in 0..addition_count {
        let next_output_var_id = output_var_id + 1;

        // This is primarily made to test memory variables, which is why we're not dropping any variables.
        program_lir.extend([
            LIRInstruction::LoadIAVar { id: output_var_id }, // IA = previous addition result
            LIRInstruction::LoadIBVar {
                id: addition_step_var_id,
            }, // IB = addition step
            LIRInstruction::Add,
            LIRInstruction::DefineVar {
                id: next_output_var_id,
            },
            LIRInstruction::StoreOVar {
                id: next_output_var_id,
            },
        ]);

        output_var_id = next_output_var_id;
    }

    program_lir.push(LIR_HALT.clone());

    let test = Test::new("many_chained_additions_emulated", program_lir);

    test.dump_panic_on_err(|| {
        let emulation = test.emulate()?.executed_to_halt()?;

        let actual = emulation.var(output_var_id);

        if actual != Some(expected) {
            Err(anyhow!("Expected: {:?}, got: {:?}", Some(expected), actual))
        } else {
            Ok(())
        }
    });
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

#[test]
fn emulated_infinite_loop_does_not_halt() {
    use anyhow::anyhow;

    let test = Test::new(
        "infinite_loop",
        [
            LIRInstruction::LoadIALabel,
            LIRInstruction::Cpy,
            LIRInstruction::Goto,
            LIR_HALT.clone(),
        ],
    );

    test.dump_panic_on_err(|| {
        let emulation = test.emulate()?.executed(1000)?; // Arbitrary instruction count to be executed

        match emulation.halted {
            true => Err(anyhow!("Infinite loop halted")),
            false => Ok(()),
        }
    });
}

struct Test {
    name: &'static str,
    program: Extra<Vec<u8>>,
    var_allocs: IndexMap<VarKey, VarAllocation>,
}

impl Test {
    pub fn new<I>(name: &'static str, lir: I) -> Self
    where
        I: IntoIterator<Item = LIRInstruction>,
    {
        let lir: Vec<_> = lir.into_iter().collect();

        let program = TransformerRunner::new(&mut strm1::transformer())
            .run(lir)
            .expect("Error compiling LIR");

        let var_allocs = Self::parse_var_allocs(&program).expect("Error parsing var allocs");

        Self {
            name,
            program,
            var_allocs,
        }
    }

    pub fn emulate(&self) -> anyhow::Result<Emulation> {
        Emulation::new(self)
    }

    pub fn dump_panic_on_err<F, O>(&self, func: F) -> O
    where
        F: FnOnce() -> anyhow::Result<O>,
    {
        match func() {
            Ok(output) => output,
            Err(e) => self.dump_panic(e),
        }
    }

    pub fn dump_panic<D>(&self, cause: D) -> !
    where
        D: Debug,
    {
        let dir_path = PathBuf::from("target").join("strm1test").join(self.name);

        println!(
            "Dumping test output to '{}'",
            dir_path.to_string_lossy().to_string()
        );

        fs::create_dir_all(&dir_path).unwrap();

        let binary_path = dir_path.join("program.bin");
        fs::write(binary_path, &self.program.data).unwrap();

        for (key, value) in &self.program.extra {
            let path = dir_path.join(format!("extra-{}", key));
            fs::write(path, value).unwrap();
        }

        let cause_path = dir_path.join("cause.txt");
        fs::write(cause_path, format!("{:?}", cause)).unwrap();

        panic!("{:?}", cause)
    }

    fn parse_var_allocs(
        program: &Extra<Vec<u8>>,
    ) -> anyhow::Result<IndexMap<VarKey, VarAllocation>> {
        let rmp = program
            .extra
            .get(strm1::codegen::EXTRA_VAR_ALLOCATIONS_KEY_RMP)
            .context("No var alloc rmp in extras")?;

        rmp_serde::from_slice(&rmp).context("Couldn't deserialize rmp")
    }
}

struct Emulation<'a> {
    test: &'a Test,
    emulator: Emulator<PCTraceData>,
    pub halted: bool,
}

impl<'a> Emulation<'a> {
    fn new(test: &'a Test) -> anyhow::Result<Self> {
        let emulator = Emulator::new(Word::MAX, test.program.data.clone())?;
        Ok(Self {
            test,
            emulator,
            halted: false,
        })
    }

    pub fn executed_to_halt(mut self) -> anyhow::Result<Self> {
        self.execute_to_halt().map(|_| self)
    }

    pub fn execute_to_halt(&mut self) -> anyhow::Result<()> {
        self.emulator
            .execute_to_halt()
            .context("Executing emulation to halt")?;

        self.halted = true;
        Ok(())
    }

    pub fn executed(mut self, instruction_count: usize) -> anyhow::Result<Self> {
        self.execute(instruction_count).map(|_| self)
    }

    pub fn execute(&mut self, instruction_count: usize) -> anyhow::Result<()> {
        for i in 0..instruction_count {
            let state = self
                .emulator
                .execute_instruction()
                .with_context(|| format!("Executing emulation on instruction {}", i))?;

            if state == ExecuteOk::Halted {
                self.halted = true;
            }
        }

        Ok(())
    }

    pub fn var(&self, id: usize) -> Option<Word> {
        let var_alloc = self.test.var_allocs.get(&VarKey::Normal(id))?;

        match var_alloc.kind {
            VarAllocationKind::Register(reg_index) => {
                Some(self.emulator.reg_file.register(reg_index))
            }
            VarAllocationKind::Memory(mem_addr) => self.emulator.memory.word(mem_addr),
        }
    }
}
