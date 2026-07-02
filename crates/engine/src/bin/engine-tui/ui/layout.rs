//! Main layout definition

use ratatui::layout::{Rect, Layout, Constraint, Direction};

pub struct AppLayout {
    pub game_state_area: Rect,
    pub trace_log_area: Rect,
    pub input_area: Rect,
    pub controls_area: Rect,
}

impl AppLayout {
    pub fn new(area: Rect) -> Self {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(65),
                Constraint::Percentage(25),
                Constraint::Length(1),
            ])
            .split(area);

        let top_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(40),
                Constraint::Percentage(60),
            ])
            .split(chunks[0]);

        let input_and_controls = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(70),
                Constraint::Percentage(30),
            ])
            .split(chunks[1]);

        Self {
            game_state_area: top_chunks[0],
            trace_log_area: top_chunks[1],
            input_area: input_and_controls[0],
            controls_area: input_and_controls[1],
        }
    }
}