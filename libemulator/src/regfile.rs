use std::array;

use libisa::Word;

use crate::tracing::{TraceData, Traced};

pub struct RegFile<T>([Traced<Word, T>; libisa::REGISTER_COUNT])
where
    T: TraceData;

impl<T> RegFile<T>
where
    T: TraceData,
{
    pub fn new() -> Self {
        Self(array::from_fn(|_| Traced::new(0)))
    }

    pub fn iter_untraced(&self) -> impl Iterator<Item = &Word> {
        self.0.iter().map(|traced| traced.value())
    }

    pub fn array_clone_untraced(&self) -> [Word; libisa::REGISTER_COUNT] {
        self.0.clone().map(|traced| *traced.value())
    }

    // TODO: Register accesses should probably return options, as they may be used externally without that much thought.

    pub fn register(&self, index: usize) -> Word {
        *self
            .0
            .get(index)
            .expect("Out of bounds register access")
            .value()
    }

    pub fn register_mut(&mut self, trace: T::Trace, index: usize) -> &mut Word {
        self.0
            .get_mut(index)
            .expect("Out of bounds register access")
            .value_mut(trace)
    }

    pub fn register_mut_untraced(&mut self, index: usize) -> &mut Word {
        self.0
            .get_mut(index)
            .expect("Out of bounds register access")
            .value_mut_untraced()
    }

    pub fn trace(&self, index: usize) -> &T {
        &self.0.get(index).expect("Out of bounds register trace access").trace_data
    }
}
