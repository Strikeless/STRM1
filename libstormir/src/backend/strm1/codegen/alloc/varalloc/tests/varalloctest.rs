use anyhow::Context;
use libisa::Word;

use crate::{backend::strm1::{codegen::{alloc::{varalloc::{AllocMap, MemVarAlloc, RegVarAlloc, VarAlloc}, ALLOC_MAP_RMP_EXTRA_KEY}, prealloc::{self, VarId, VarKey}}, tests::Test}, lir::LIRVarId};

// TODO: Emulator tests before this ofc
pub struct VarAllocTest {
    pub inner: Test,
    alloc_map: AllocMap,
}

impl VarAllocTest {
    pub fn new(inner: Test) -> anyhow::Result<Self> {
        let alloc_map_rmp = inner.compilation_output.extra
            .get(ALLOC_MAP_RMP_EXTRA_KEY)
            .context("No alloc map RMP extra in compilation output")?;

        let alloc_map = rmp_serde::from_slice(&alloc_map_rmp)
            .context("Error parsing alloc map RMP extra from compilation output")?;

        Ok(Self {
            inner,
            alloc_map,
        })
    }

    pub fn get_var(&self, lir_id: LIRVarId) -> Option<Word> {
        let var_key = VarKey::Generic(VarId(*prealloc::codegen::LIR_VAR_SPACE, lir_id));
        let var_alloc = self.alloc_map.get(&var_key)?;

        match var_alloc {
            VarAlloc::Register(RegVarAlloc(reg_index)) => todo!(),
            VarAlloc::Memory(MemVarAlloc(mem_addr)) => todo!(),
        }
    }
}
