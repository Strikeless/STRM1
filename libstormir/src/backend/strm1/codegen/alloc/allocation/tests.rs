use std::assert_matches::assert_matches;

use crate::backend::strm1::codegen::{alloc::allocation::VarAlloc, prealloc::VarId};

use super::allocator::{AllocRequirement, VarAllocator};

#[test]
fn build_bigger_alloc_map() -> anyhow::Result<()> {
    let mut allocator = VarAllocator::new();

    for i in 0..10000 {
        let id = VarId::Internal(i);
        allocator.define(id, 0, 0, AllocRequirement::Generic)?;
        allocator.extend_lifetime(&id, 1)?;
    }

    allocator.build().map(|_| ())
}

#[test]
fn register_alloc_by_default() -> anyhow::Result<()> {
    let mut allocator = VarAllocator::new();

    let id = VarId::Internal(0);

    allocator.define(id, 0, 0, AllocRequirement::Generic)?;
    allocator.extend_lifetime(&id, 1)?;

    let alloc_map = allocator.build()?;

    assert_matches!(
        alloc_map.get_by_id(&id),
        Some(VarAlloc::Register(..)),
        "Allocation didn't go into a register by default"
    );

    Ok(())
}

#[test]
fn register_alloc_by_importance() -> anyhow::Result<()> {
    let mut allocator = VarAllocator::new();

    let id = VarId::Internal(0);

    // Try to fill up any free registers with low importance variables.
    for i in 0..libisa::REGISTER_COUNT {
        let id = VarId::Internal(1 + i);
        allocator.define(id, 0, 0, AllocRequirement::Generic)?;
        allocator.extend_lifetime(&id, 1)?;
    }

    // Note the higher importance of 1 than the previous definitions.
    allocator.define(id, 0, 1, AllocRequirement::Generic)?;
    allocator.extend_lifetime(&id, 1)?;

    let alloc_map = allocator.build()?;

    assert_matches!(
        alloc_map.get_by_id(&id),
        Some(VarAlloc::Register(..)),
        "Allocation didn't go into a register by importance"
    );

    Ok(())
}

#[test]
fn register_alloc_by_requirement() -> anyhow::Result<()> {
    let mut allocator = VarAllocator::new();

    let id = VarId::Internal(0);

    // Try to fill up any free registers with generic, yet high importance variables.
    for i in 0..libisa::REGISTER_COUNT {
        let id = VarId::Internal(1 + i);
        allocator.define(id, 0, usize::MAX, AllocRequirement::Generic)?;
        allocator.extend_lifetime(&id, 1)?;
    }

    allocator.define(id, 0, 0, AllocRequirement::Register)?;
    allocator.extend_lifetime(&id, 1)?;

    let alloc_map = allocator.build()?;

    assert_matches!(
        alloc_map.get_by_id(&id),
        Some(VarAlloc::Register(..)),
        "Allocation didn't go into a register by importance"
    );

    Ok(())
}

#[test]
fn memory_alloc_by_default_fallback() -> anyhow::Result<()> {
    let mut allocator = VarAllocator::new();

    let id = VarId::Internal(0);

    // Try to fill up any free registers with variables that require the registers.
    for i in 0..libisa::REGISTER_COUNT {
        let id = VarId::Internal(1 + i);
        allocator.define(id, 0, 0, AllocRequirement::Register)?;
        allocator.extend_lifetime(&id, 1)?;
    }

    allocator.define(id, 0, 0, AllocRequirement::Generic)?;
    allocator.extend_lifetime(&id, 1)?;

    let alloc_map = allocator.build()?;

    assert_matches!(
        alloc_map.get_by_id(&id),
        Some(VarAlloc::Memory(..)),
        "Allocation didn't go into memory by default fallback"
    );

    Ok(())
}
