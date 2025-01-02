use std::{collections::HashMap, fmt::Debug, fs, panic, path::PathBuf};

use anyhow::Context;
use libemulator::{Emulator, ExecuteOk};
use libisa::Word;

use crate::{
    backend::strm1::{
        codegen::prealloc::{self, VarId, VarKey, VarTrait},
        tests::Test,
    },
    lir::LIRVarId,
};

use super::{
    varalloc::{allocator::VarDefinition, AllocMap, MemVarAlloc, RegVarAlloc, VarAlloc},
    EXTRAS_ALLOC_MAP_KEY, EXTRAS_ALLOC_METADATA_KEY,
};

/* Tests implemented in backend root, this is just for the emulator test API, as it uses some private features internally. */

pub struct EmulatorTest {
    pub inner: Test,
    pub emulator: Emulator,

    alloc_map: AllocMap,
    alloc_metadata: HashMap<VarId, VarDefinition>,
}

impl EmulatorTest {
    pub fn new(inner: Test) -> anyhow::Result<Self> {
        let program = inner.compilation_output.data.clone();

        let emulator = Emulator::new(program).context("Error creating emulator")?;

        let alloc_map = inner
            .compilation_output
            .extra(EXTRAS_ALLOC_MAP_KEY)
            .context("No alloc map extra in compilation output")??;

        let alloc_metadata = inner
            .compilation_output
            .extra(EXTRAS_ALLOC_METADATA_KEY)
            .context("No alloc metadata extra in compilation output")??;

        Ok(Self {
            inner,
            emulator,
            alloc_map,
            alloc_metadata,
        })
    }

    /// Get the last value assigned directly to the LIR variable.
    /// NOTE: Assignments from different LIR variables that point to the exact same physical data address may not be reflected here.
    ///       e.g. variable A is assigned to register 0 at the same scope as variable B which is assigned to register 0 explicitly,
    ///            operations that change variable B may not be reflected here with variable A.
    pub fn get_var(&self, lir_id: LIRVarId) -> Option<Word> {
        let var_key = VarKey::Generic(VarId(*prealloc::codegen::LIR_VAR_SPACE, lir_id));

        let var_alloc = self.alloc_map.get(&var_key)?;
        let var_metadata = self.alloc_metadata.get(var_key.id())?;

        // TODO: Utilize emulator tracing to get the variable value. Need metadata for PC to instruction index
        //       and instruction index to LIR index?

        todo!()
    }

    /// Get the current value of the data cell the given LIR variable refers to.
    /// NOTE: This function does not care if the data has been overwritten by another variable!
    ///       Use get_var everywhere where this behaviour is not desirable!
    pub fn get_var_ignorant(&self, lir_id: LIRVarId) -> Option<Word> {
        let var_key = VarKey::Generic(VarId(*prealloc::codegen::LIR_VAR_SPACE, lir_id));
        let var_alloc = self.alloc_map.get(&var_key)?;

        match var_alloc {
            VarAlloc::Register(RegVarAlloc(reg_index)) => {
                self.emulator.reg_file.get(*reg_index).copied()
            }
            VarAlloc::Memory(MemVarAlloc(mem_addr)) => {
                self.emulator.memory.word(*mem_addr).as_deref().copied()
            }
        }
    }

    pub fn run_till<F>(&mut self, mut condition_fn: F) -> anyhow::Result<()>
    where
        F: FnMut(&mut Self, Option<ExecuteOk>) -> bool,
    {
        let mut last_execute_result = None;

        while condition_fn(self, last_execute_result) {
            last_execute_result = Some(self.emulator.execute_instruction()?);
        }

        Ok(())
    }

    pub fn run_till_halt(&mut self) -> anyhow::Result<()> {
        self.run_till(|_, exec_result| exec_result != Some(ExecuteOk::Halted))
    }

    pub fn dump_panic<D>(&self, cause: D) -> !
    where
        D: Debug,
    {
        let dir_path = PathBuf::from("target")
            .join("strm1_emutest")
            .join(self.inner.name);

        println!(
            "Dumping test output to '{}'.",
            dir_path.to_string_lossy().to_string()
        );

        let compilation_output = &self.inner.compilation_output;

        let binary_path = dir_path.join("program.bin");
        fs::create_dir_all(&dir_path).unwrap();
        fs::write(binary_path, &compilation_output.data).unwrap();

        for (key, value) in &compilation_output.extras {
            let path = dir_path.join(format!("extra-{}", key));
            fs::write(path, value).unwrap();
        }

        let cause_path = dir_path.join("cause.txt");
        fs::write(cause_path, format!("{:?}", cause)).unwrap();

        panic!("{:?}", cause)
    }

    pub fn dump_panic_on_err<F, O>(&mut self, func: F) -> O
    where
        F: FnOnce(&mut Self) -> anyhow::Result<O>,
    {
        match func(self) {
            Ok(output) => output,
            Err(e) => self.dump_panic(e),
        }
    }
}

pub trait TestEmulateExt {
    fn emulate(self) -> anyhow::Result<EmulatorTest>;

    fn emulate_dump_panicking<F, O>(self, func: F) -> O
    where
        F: FnOnce(&mut EmulatorTest) -> anyhow::Result<O>;
}

impl TestEmulateExt for Test {
    fn emulate(self) -> anyhow::Result<EmulatorTest> {
        EmulatorTest::new(self)
    }

    fn emulate_dump_panicking<F, O>(self, func: F) -> O
    where
        F: FnOnce(&mut EmulatorTest) -> anyhow::Result<O>,
    {
        // Can't dump panic on normal tests, so just panic. Bit of a design failure, but not all
        // that problematic as there isn't much to dump anyway, and the problem is probably elsewhere.
        let mut emulated = self.emulate().expect("Error emulating test");

        emulated.dump_panic_on_err(|emulated| func(emulated))
    }
}
