//! Bottom panel: perspective selector and action input

use cgdsl_engine::InputType;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
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

    // The render call takes the full prompt context; a struct would churn
    // every call site for no behavioural gain.
    #[allow(clippy::too_many_arguments)]
    pub fn render(
        &self,
        f: &mut ratatui::Frame,
        waiting: bool,
        input_type: Option<&InputType>,
        area: Rect,
        choose_cursor: usize,
        choose_selected: &[bool],
        current_player_name: &str,
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
            let perspective_name = self
                .player_names
                .get(self.perspective_idx)
                .map(|s| s.as_str())
                .unwrap_or("");
            if !current_player_name.is_empty() && perspective_name != current_player_name {
                let msg = format!(
                    "Waiting for {}'s turn... (you are viewing as {})",
                    current_player_name, perspective_name
                );
                f.render_widget(Paragraph::new(msg), inner);
                return;
            }
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
                    InputType::ChoosePlayer { candidates, prompt } => {
                        let line_count = candidates.len();
                        let mut lines = Vec::with_capacity(line_count + 2);
                        lines.push(format!("{} (↑↓ to move, Enter to confirm)", prompt));
                        lines.push(String::new());
                        for (i, name) in candidates.iter().enumerate() {
                            let prefix = if i == choose_cursor { ">" } else { " " };
                            lines.push(format!("{} {}. {}", prefix, i + 1, name));
                        }
                        let text = ratatui::text::Text::from(
                            lines
                                .iter()
                                .enumerate()
                                .map(|(i, line)| {
                                    if i >= 2 && (i - 2) == choose_cursor {
                                        ratatui::text::Line::styled(
                                            line.clone(),
                                            Style::default().fg(Color::Yellow),
                                        )
                                    } else {
                                        ratatui::text::Line::from(line.as_str())
                                    }
                                })
                                .collect::<Vec<_>>(),
                        );
                        f.render_widget(Paragraph::new(text), inner);
                    }
                    InputType::ChooseCards {
                        display,
                        min,
                        max,
                        prompt,
                    } => {
                        let selected_count = choose_selected.iter().filter(|&&s| s).count();
                        let mut lines = Vec::with_capacity(display.len() + 3);
                        lines.push(format!(
                            "{} (select {}-{}, currently {} selected, ↑↓/Space/Enter)",
                            prompt, min, max, selected_count
                        ));
                        lines.push(String::new());
                        for (i, card) in display.iter().enumerate() {
                            let check = if i < choose_selected.len() && choose_selected[i] {
                                "[X]"
                            } else {
                                "[ ]"
                            };
                            let cursor = if i == choose_cursor { ">" } else { " " };
                            let desc: Vec<String> =
                                card.iter().map(|(k, v)| format!("{}: {}", k, v)).collect();
                            lines.push(format!(
                                "{} {} {}. {}",
                                cursor,
                                check,
                                i + 1,
                                desc.join(", ")
                            ));
                        }
                        let text = ratatui::text::Text::from(
                            lines
                                .iter()
                                .enumerate()
                                .map(|(i, line)| {
                                    if i >= 2 && (i - 2) == choose_cursor {
                                        ratatui::text::Line::styled(
                                            line.clone(),
                                            Style::default().fg(Color::Yellow),
                                        )
                                    } else {
                                        ratatui::text::Line::from(line.as_str())
                                    }
                                })
                                .collect::<Vec<_>>(),
                        );
                        f.render_widget(Paragraph::new(text), inner);
                    }
                }
            }
        }
    }
}
