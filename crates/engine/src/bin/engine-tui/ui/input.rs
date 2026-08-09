//! Bottom panel: perspective selector and action input

use cgdsl_engine::InputType;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, Paragraph};

/// Scroll offset (in rows) that keeps `cursor` visible within `visible`
/// rows of a `total`-row list. The cursor is always at the top of the
/// window while it is in the first `visible` rows, then the window follows
/// it so the cursor stays on the bottom row until the list end is reached.
fn cursor_scroll_offset(cursor: usize, visible: usize, total: usize) -> u16 {
    if visible == 0 || total <= visible {
        return 0;
    }
    let offset = if cursor >= visible {
        cursor - visible + 1
    } else {
        0
    };
    offset.min(total - visible) as u16
}

/// Render a one-line hint pinned at the top of `area` and return the rect
/// for the scrollable list below it.
fn render_hint(f: &mut ratatui::Frame, area: Rect, hint: &str) -> Rect {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0)])
        .split(area);
    f.render_widget(Paragraph::new(hint), chunks[0]);
    chunks[1]
}

/// Build the scrollable, cursor-highlighted list text from `items` and
/// render it into `area`, scrolling so the cursor stays visible.
fn render_list(
    f: &mut ratatui::Frame,
    area: Rect,
    items: impl Iterator<Item = String>,
    cursor: usize,
    total: usize,
) {
    let visible = area.height as usize;
    let offset = cursor_scroll_offset(cursor, visible, total);
    let text = ratatui::text::Text::from(
        items
            .enumerate()
            .map(|(i, line)| {
                if i == cursor {
                    ratatui::text::Line::styled(line, Style::default().fg(Color::Yellow))
                } else {
                    ratatui::text::Line::from(line)
                }
            })
            .collect::<Vec<_>>(),
    );
    f.render_widget(Paragraph::new(text).scroll((offset, 0)), area);
}

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
                        let list_area = render_hint(
                            f,
                            inner,
                            "Choose an option (↑↓ to move, Enter to confirm, 1-9 shortcut)",
                        );
                        render_list(
                            f,
                            list_area,
                            options.iter().enumerate().map(|(i, opt)| {
                                let prefix = if i == choose_cursor { ">" } else { " " };
                                format!("{} {}. {}", prefix, i + 1, opt)
                            }),
                            choose_cursor,
                            options.len(),
                        );
                    }
                    InputType::Optional(prompt) => {
                        let text = format!("{}: [y] Accept / [n] Decline", prompt);
                        f.render_widget(Paragraph::new(text), inner);
                    }
                    InputType::ChoosePlayer { candidates, prompt } => {
                        let list_area = render_hint(
                            f,
                            inner,
                            &format!("{} (↑↓ to move, Enter to confirm)", prompt),
                        );
                        render_list(
                            f,
                            list_area,
                            candidates.iter().enumerate().map(|(i, name)| {
                                let prefix = if i == choose_cursor { ">" } else { " " };
                                format!("{} {}. {}", prefix, i + 1, name)
                            }),
                            choose_cursor,
                            candidates.len(),
                        );
                    }
                    InputType::ChooseCards {
                        display,
                        min,
                        max,
                        prompt,
                    } => {
                        let selected_count = choose_selected.iter().filter(|&&s| s).count();
                        let list_area = render_hint(
                            f,
                            inner,
                            &format!(
                                "{} (select {}-{}, currently {} selected, ↑↓/Space/Enter)",
                                prompt, min, max, selected_count
                            ),
                        );
                        render_list(
                            f,
                            list_area,
                            display.iter().enumerate().map(|(i, card)| {
                                let check = if i < choose_selected.len() && choose_selected[i] {
                                    "[X]"
                                } else {
                                    "[ ]"
                                };
                                let cursor = if i == choose_cursor { ">" } else { " " };
                                let desc: Vec<String> =
                                    card.iter().map(|(k, v)| format!("{}: {}", k, v)).collect();
                                format!("{} {} {}. {}", cursor, check, i + 1, desc.join(", "))
                            }),
                            choose_cursor,
                            display.len(),
                        );
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::cursor_scroll_offset;

    #[test]
    fn no_scroll_when_list_fits() {
        assert_eq!(cursor_scroll_offset(0, 10, 5), 0);
        assert_eq!(cursor_scroll_offset(4, 10, 5), 0);
    }

    #[test]
    fn cursor_in_top_window_stays_pinned() {
        assert_eq!(cursor_scroll_offset(0, 5, 13), 0);
        assert_eq!(cursor_scroll_offset(4, 5, 13), 0);
    }

    #[test]
    fn window_follows_cursor_past_the_bottom_edge() {
        assert_eq!(cursor_scroll_offset(5, 5, 13), 1);
        assert_eq!(cursor_scroll_offset(6, 5, 13), 2);
    }

    #[test]
    fn scroll_clamps_at_list_end() {
        assert_eq!(
            cursor_scroll_offset(12, 5, 13),
            8,
            "max offset = total - visible"
        );
        assert_eq!(
            cursor_scroll_offset(99, 5, 13),
            8,
            "cursor never beyond last row"
        );
    }

    #[test]
    fn zero_height_is_safe() {
        assert_eq!(cursor_scroll_offset(3, 0, 13), 0);
    }
}
