//! Controls/help panel

use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Paragraph};

use super::EngineStatus;

pub struct ControlsPanel;

impl ControlsPanel {
    pub fn new() -> Self {
        Self
    }

    pub fn render(&self, f: &mut ratatui::Frame, area: Rect, status: &EngineStatus) {
        let block = Block::default().title("CONTROLS").borders(Borders::ALL);

        f.render_widget(&block, area);

        let inner = block.inner(area);

        let status_line = match status {
            EngineStatus::Running => Line::from(Span::styled(
                "Engine: RUNNING",
                Style::default().fg(Color::Yellow),
            )),
            EngineStatus::Finished => Line::from(Span::styled(
                "Engine: FINISHED",
                Style::default().fg(Color::Green),
            )),
            EngineStatus::Errored(msg) => Line::from(vec![
                Span::styled("Engine: ERROR ", Style::default().fg(Color::Red)),
                Span::raw(msg.clone()),
            ]),
            EngineStatus::Panicked(msg) => Line::from(vec![
                Span::styled("Engine: PANIC ", Style::default().fg(Color::Red)),
                Span::raw(msg.clone()),
            ]),
        };

        let text = Text::from(vec![
            status_line,
            Line::from(""),
            Line::from("q/F10: Quit | Tab: Focus panel"),
            Line::from("Up/Dn/PgUp/PgDn/Home/End: Scroll"),
            Line::from("↑↓/Enter: Choose option (1-9/0 shortcuts) | Space: Select card"),
            Line::from("y/n: Optional | l/t/p: Cycle views | r: Raw/simplified trace"),
        ]);

        let paragraph = Paragraph::new(text).wrap(ratatui::widgets::Wrap { trim: false });

        f.render_widget(paragraph, inner);
    }
}
