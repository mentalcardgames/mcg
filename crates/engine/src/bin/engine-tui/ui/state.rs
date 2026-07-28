//! TUI application state

use crossbeam_channel::Sender;

use crate::trace::{DisplayTraceEntry, TraceDetail};
use cgdsl_engine::{DebugLevel, GameData, Input, InputType, TraceEntry};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PanelFocus {
    GameState,
    TraceLog,
}

pub struct TuiState {
    pub trace_entries: Vec<DisplayTraceEntry>,
    pub step_count: usize,
    pub trace_detail: TraceDetail,
    pub state_detail: DebugLevel,
    pub perspective_idx: usize,
    pub pending_input: Option<InputType>,
    pub waiting_for_input: bool,
    pub input_tx: Option<Sender<Input>>,
    pub current_state: Option<GameData>,
    pub current_player_name: String,
    pub prev_player_name: String,
    pub focus: PanelFocus,
    pub trace_scroll: u16,
    pub trace_auto_scroll: bool,
    pub trace_inner_height: u16,
    pub game_state_scroll: u16,
    pub game_state_auto_scroll: bool,
    pub game_state_inner_height: u16,
    pub choose_cursor: usize,
    pub choose_selected: Vec<bool>,
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
            current_state: None,
            focus: PanelFocus::TraceLog,
            trace_scroll: 0,
            trace_auto_scroll: true,
            trace_inner_height: 0,
            game_state_scroll: 0,
            game_state_auto_scroll: true,
            game_state_inner_height: 0,
            choose_cursor: 0,
            choose_selected: Vec::new(),
            current_player_name: String::new(),
            prev_player_name: String::new(),
        }
    }

    pub fn push_trace(&mut self, entry: TraceEntry) {
        self.step_count += 1;
        self.trace_entries.push(DisplayTraceEntry::Entry {
            step_number: self.step_count,
            entry,
        });
        self.trace_auto_scroll = true;
    }

    pub fn detect_turn_change(&mut self) {
        if !self.current_player_name.is_empty()
            && !self.prev_player_name.is_empty()
            && self.current_player_name != self.prev_player_name
        {
            self.trace_entries.push(DisplayTraceEntry::TurnSeparator {
                player_name: self.current_player_name.clone(),
            });
        }
        self.prev_player_name = self.current_player_name.clone();
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
