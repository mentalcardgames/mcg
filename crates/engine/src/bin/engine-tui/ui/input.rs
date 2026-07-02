//! Bottom panel: perspective selector and action input

use ratatui::layout::Rect;
use ratatui::widgets::{Block, Borders, Paragraph, List, ListItem};
use cgdsl_engine::InputType;

pub struct InputPanel {
    pub perspective_idx: usize,
    pub player_names: Vec<String>,
}

impl InputPanel {
    pub fn new(perspective_idx: usize, player_names: Vec<String>) -> Self {
        Self { perspective_idx, player_names }
    }

    pub fn render(&self, f: &mut ratatui::Frame, waiting: bool, input_type: Option<&InputType>, area: Rect) {
        let block = Block::default()
            .title(format!(
                "Perspective: {} | {}",
                self.player_names.get(self.perspective_idx).cloned().unwrap_or_default(),
                if waiting { "WAITING FOR INPUT" } else { "IDLE" }
            ))
            .borders(Borders::ALL);

        f.render_widget(&block, area);

        let inner = block.inner(area);

        if waiting {
            if let Some(it) = input_type {
                match it {
                    InputType::Choice { options, .. } => {
                        let items: Vec<_> = options.iter().enumerate().map(|(i, opt)| {
                            ListItem::new(format!("[{}] {}", i + 1, opt))
                        }).collect();
                        let list = List::new(items);
                        f.render_widget(list, inner);
                    }
                    InputType::Optional(prompt) => {
                        let text = format!("{}: [y] Accept / [n] Decline", prompt);
                        f.render_widget(Paragraph::new(text), inner);
                    }
                }
            }
        }
    }
}