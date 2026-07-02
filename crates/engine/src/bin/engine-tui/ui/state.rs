//! TUI application state

use crossbeam_channel::Sender;

use cgdsl_engine::{Input, InputType, TraceEntry, DebugLevel};
use crate::trace::{DisplayTraceEntry, TraceDetail};

pub struct TuiState {
    pub trace_entries: Vec<DisplayTraceEntry>,
    pub step_count: usize,
    pub trace_detail: TraceDetail,
    pub state_detail: DebugLevel,
    pub perspective_idx: usize,
    pub pending_input: Option<InputType>,
    pub waiting_for_input: bool,
    pub input_tx: Option<Sender<Input>>,
}

impl TuiState {
    pub fn new() -> Self {
        Self {
            trace_entries: Vec::new(),
            step_count: 0,
            trace_detail: TraceDetail::Evaluations,
            state_detail: DebugLevel::Medium,
            perspective_idx: 0,
            pending_input: None,
            waiting_for_input: false,
            input_tx: None,
        }
    }

    pub fn push_trace(&mut self, entry: TraceEntry) {
        self.step_count += 1;
        self.trace_entries.push(DisplayTraceEntry::from_trace_entry(self.step_count, entry));
    }

    pub fn cycle_trace_detail(&mut self) {
        self.trace_detail.cycle();
    }

    pub fn cycle_state_detail(&mut self) {
        self.state_detail = match self.state_detail {
            DebugLevel::Low => DebugLevel::Medium,
            DebugLevel::Medium => DebugLevel::High,
            DebugLevel::High => DebugLevel::Low,
        };
    }
}