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
    assert_eq!(alloc.definition.end, Some(index + drop_offset), "drop index wrong");
    
    if let VarAllocationKind::Memory(_) = alloc.kind {
        panic!("first allocation didn't prefer a register");
    }
}

#[test]
fn memory_alloc() {
    let mut builder = VarTableBuilder::new();

    builder.set_current_index(0);

    let mem_key = VarKey::Normal(0);
    builder.define(mem_key, false).unwrap();
    builder.drop(mem_key, 10).unwrap();

    for i in 0..libstrmisa::REGISTER_COUNT {
        let key = VarKey::Normal(1 + i);
        builder.define(key, true).unwrap();
        builder.drop(key, 1).unwrap();
    }    
    
    let var_table = builder.build().unwrap();
    let mem_alloc = var_table.allocations.get(&mem_key).unwrap();
    

    if let VarAllocationKind::Register(_) = mem_alloc.kind {
        panic!("allocation must be overlapping or have stolen a register from a definition that needs one");
    }
}
