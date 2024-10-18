use std::fmt::Debug;

use crate::Emulator;

pub mod none;
pub mod pc;

#[derive(Debug, Clone, Copy)]
pub struct Traced<T, D>
where
    D: TraceData,
{
    pub trace_data: D,
    value: T,
}

impl<T, D> Traced<T, D>
where
    D: TraceData,
{
    pub fn new(value: T) -> Self {
        Self {
            value,
            trace_data: D::default(),
        }
    }

    pub fn value(&self) -> &T {
        &self.value
    }

    pub fn value_mut(&mut self, trace: D::Trace) -> &mut T {
        self.trace_data.add_trace(trace);
        self.value_mut_untraced()
    }

    pub fn value_mut_untraced(&mut self) -> &mut T {
        &mut self.value
    }
}

pub trait TraceData: Default + Debug + Clone {
    type Trace: Default + Debug + Copy;

    fn trace_from_state(emulator: &Emulator<Self>) -> Self::Trace;

    fn add_trace(&mut self, trace: Self::Trace);
}
