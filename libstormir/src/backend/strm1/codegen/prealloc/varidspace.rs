use std::sync::atomic::{AtomicU64, Ordering};

use serde::{Deserialize, Serialize};

type BackingId = u64;
type AtomicBackingId = AtomicU64;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct VarIdSpace(u64);

impl VarIdSpace {
    /// Create a new and unique space.
    pub fn new() -> Self {
        Self(take_next_id())
    }
}

static ID_COUNTER: AtomicBackingId = AtomicBackingId::new(0);

fn take_next_id() -> BackingId {
    ID_COUNTER.fetch_add(1, Ordering::AcqRel)
}
