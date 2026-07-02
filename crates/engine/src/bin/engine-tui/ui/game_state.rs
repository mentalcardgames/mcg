//! Left panel: Game state viewer using existing format_game_data()

use ratatui::layout::Rect;
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::text::Text;

use cgdsl_engine::{format_game_data, DebugLevel, GameData};

pub struct GameStatePanel {
    pub detail: DebugLevel,
}

impl GameStatePanel {
    pub fn new(detail: DebugLevel) -> Self {
        Self { detail }
    }

    pub fn render(&self, f: &mut ratatui::Frame, game_data: &GameData, area: Rect) {
        let content = format_game_data(game_data, self.detail);

        let block = Block::default()
            .title("GAME STATE")
            .borders(Borders::ALL);

        let inner = block.inner(area);

        f.render_widget(block, area);

        let text: Text = content.into();
        let paragraph = Paragraph::new(text);

        f.render_widget(paragraph, inner);
    }
}