//! Controls/help panel

use ratatui::layout::Rect;
use ratatui::text::{Line, Text};
use ratatui::widgets::{Block, Borders, Paragraph};

pub struct ControlsPanel;

impl ControlsPanel {
    pub fn new() -> Self {
        Self
    }

    pub fn render(&self, f: &mut ratatui::Frame, area: Rect) {
        let block = Block::default().title("CONTROLS").borders(Borders::ALL);

        f.render_widget(&block, area);

        let inner = block.inner(area);

        let text = Text::from(vec![
            Line::from("q/F10: Quit | Tab: Focus panel"),
            Line::from("Up/Dn/PgUp/PgDn/Home/End: Scroll"),
            Line::from("1-9/y/n/Enter: Input | l/t/p: Cycle views"),
        ]);

        let paragraph = Paragraph::new(text).wrap(ratatui::widgets::Wrap { trim: false });

        f.render_widget(paragraph, inner);
    }
}
