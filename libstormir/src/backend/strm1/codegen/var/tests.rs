use crate::backend::strm1::codegen::var::VarAllocationKind;

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
    for i in 0..libstrmisa::REGISTER_COUNT {
        builder.define_normal(i).unwrap();
        builder.heaten_normal(i).unwrap();
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
    for i in 0..libstrmisa::REGISTER_COUNT {
        builder.define(VarKey::Normal(i), true).unwrap();
    }

    // Add one more variable that doesn't need a register
    let key = VarKey::Special(0);
    builder.define(key, false).unwrap();
    
    let var_table = builder.build().unwrap();
    let var = var_table.get(key).expect("Variable wasn't allocated");

    assert!(var.kind.is_memory(), "Variable that doesn't need register did not go to memory");
}
