use crate::Emulator;

use super::TraceData;

#[derive(Debug, Clone, Default)]
pub struct PCTraceData {
    pub traces: Vec<usize>,
}

impl TraceData for PCTraceData {
    type Trace = usize;

    fn trace_from_state(emulator: &Emulator<Self>) -> Self::Trace {
        emulator.pc as usize
    }

    fn add_trace(&mut self, trace: Self::Trace) {
        self.traces.push(trace);
    }
}
