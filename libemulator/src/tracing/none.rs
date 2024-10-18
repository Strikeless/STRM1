use crate::Emulator;

use super::TraceData;

#[derive(Debug, Clone, Copy, Default)]
pub struct NoTraceData;

impl TraceData for NoTraceData {
    type Trace = ();

    fn trace_from_state(_: &Emulator<Self>) -> Self::Trace {
        ()
    }

    fn add_trace(&mut self, _: Self::Trace) {}
}
