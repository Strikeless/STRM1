use crate::backend::strm1::codegen::var::{VarAllocationKind, VarTable};

use super::{builder::VarTableBuilder, VarKey};

#[test]
fn register_alloc() {
    let mut builder = VarTableBuilder::new();

    let index = 10;
    let drop_offset = 5;
    let key = VarKey::Normal(1);

    builder.set_current_index(index);
    builder.define(key, false).unwrap(); // needs_register false to make sure that allocations primarily prefer registers.
    builder.drop(key, drop_offset).unwrap();

    let var_table = builder.build().unwrap();
    let alloc = var_table.allocations.get(&key).unwrap();

    assert_eq!(alloc.definition.begin, index, "definition index wrong");
    assert_eq!(
        alloc.definition.end,
        Some(index + drop_offset),
        "drop index wrong"
    );

    if let VarAllocationKind::Memory(_) = alloc.kind {
        panic!("first allocation didn't prefer a register");
    }
}

#[test]
fn forced_memory_alloc_by_heat() {
    let mut builder = VarTableBuilder::new();
    builder.set_current_index(0);

    // Use all the registers with heated variables
    for i in 0..libisa::REGISTER_COUNT {
        let key = VarKey::Normal(i);
        builder.define(key, false).unwrap();
        builder.heaten(key).unwrap();
    }

    // Add one more variable that is not heated
    let key = VarKey::Special(0);
    builder.define(key, false).unwrap();

    let var_table = builder.build().unwrap();
    let var = var_table.get(key).expect("Variable wasn't allocated");

    assert!(var.kind.is_memory(), "Cold variable did not go to memory");
}

#[test]
fn forced_memory_alloc_by_needing_registers() {
    let mut builder = VarTableBuilder::new();
    builder.set_current_index(0);

    // Use all the registers with variables that need registers
    for i in 0..libisa::REGISTER_COUNT {
        builder.define(VarKey::Normal(i), true).unwrap();
    }

    // Add one more variable that doesn't need a register
    let key = VarKey::Special(0);
    builder.define(key, false).unwrap();

    let var_table = builder.build().unwrap();
    let var = var_table.get(key).expect("Variable wasn't allocated");

    assert!(
        var.kind.is_memory(),
        "Variable that doesn't need register did not go to memory"
    );
}

#[test]
// Having the variable allocator be deterministic helps crack down other bugs, and certainly isn't a bad thing anyway.
fn determinism() {
    fn build_var_table() -> VarTable {
        let mut builder = VarTableBuilder::new();
        builder.set_current_index(0);

        for i in 0..2 {
            builder.define(VarKey::Normal(i), false).unwrap();
        }

        builder.build().unwrap()
    }

    let mut previous_var_table = build_var_table();

    for i in 2..=50 {
        let new_var_table = build_var_table();

        // assert_eq's output is hard to read even with a few allocations.
        if previous_var_table.allocations != new_var_table.allocations {
            panic!("Var table allocations differed on build {}", i);
        }

        previous_var_table = new_var_table;
    }
}
