//! Bottom panel: perspective selector and action input

use cgdsl_engine::InputType;
use ratatui::layout::Rect;
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};

pub struct InputPanel {
    pub perspective_idx: usize,
    pub player_names: Vec<String>,
}

impl InputPanel {
    pub fn new(perspective_idx: usize, player_names: Vec<String>) -> Self {
        Self {
            perspective_idx,
            player_names,
        }
    }

    pub fn render(
        &self,
        f: &mut ratatui::Frame,
        waiting: bool,
        input_type: Option<&InputType>,
        area: Rect,
    ) {
        let block = Block::default()
            .title(format!(
                "Perspective: {} | {}",
                self.player_names
                    .get(self.perspective_idx)
                    .cloned()
                    .unwrap_or_default(),
                if waiting { "WAITING FOR INPUT" } else { "IDLE" }
            ))
            .borders(Borders::ALL);

        f.render_widget(&block, area);

        let inner = block.inner(area);

        if waiting {
            if let Some(it) = input_type {
                match it {
                    InputType::Choice { options, .. } => {
                        let items: Vec<_> = options
                            .iter()
                            .enumerate()
                            .map(|(i, opt)| ListItem::new(format!("[{}] {}", i + 1, opt)))
                            .collect();
                        let list = List::new(items);
                        f.render_widget(list, inner);
                    }
                    InputType::Optional(prompt) => {
                        let text = format!("{}: [y] Accept / [n] Decline", prompt);
                        f.render_widget(Paragraph::new(text), inner);
                    }
                    // Minimal render for the quantifier prompts. Full keyboard
                    // selection in the TUI is a follow-up; the engine-level
                    // round trip is exercised via `InputSource::Player`
                    // closures in tests and via `cgdsl-play` interactively.
                    InputType::ChoosePlayer { candidates, prompt } => {
                        let body = candidates
                            .iter()
                            .enumerate()
                            .map(|(i, n)| format!("[{}] {}", i + 1, n))
                            .collect::<Vec<_>>()
                            .join("\n");
                        let text = format!("{}\n{}", prompt, body);
                        f.render_widget(Paragraph::new(text), inner);
                    }
                    InputType::ChooseCards {
                        display,
                        min,
                        max,
                        prompt,
                    } => {
                        let body = display
                            .iter()
                            .enumerate()
                            .map(|(i, c)| {
                                let desc = c
                                    .get("Rank")
                                    .or_else(|| c.values().next())
                                    .cloned()
                                    .unwrap_or_else(|| format!("card {}", i + 1));
                                format!("[{}] {}", i + 1, desc)
                            })
                            .collect::<Vec<_>>()
                            .join("\n");
                        let text = format!("{} (choose {}-{})\n{}", prompt, min, max, body);
                        f.render_widget(Paragraph::new(text), inner);
                    }
                }
            }
        }
    }
}
