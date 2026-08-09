//! Left panel: Game state viewer using existing format_game_data()

use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::Text;
use ratatui::widgets::{Block, Borders, Paragraph};

use cgdsl_engine::{format_game_data, DebugLevel, GameData};

pub struct GameStatePanel {
    pub detail: DebugLevel,
}

impl GameStatePanel {
    pub fn new(detail: DebugLevel) -> Self {
        Self { detail }
    }

    pub fn render(
        &self,
        f: &mut ratatui::Frame,
        game_data: &GameData,
        area: Rect,
        scroll: u16,
        auto_scroll: bool,
        focused: bool,
    ) -> u16 {
        let content = format_game_data(game_data, self.detail);

        let block = if focused {
            Block::default()
                .title("GAME STATE")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow))
        } else {
            Block::default().title("GAME STATE").borders(Borders::ALL)
        };

        f.render_widget(&block, area);

        let inner = block.inner(area);
        let inner_height = inner.height;

        let line_strings: Vec<&str> = content.lines().collect();

        let scroll_offset = if auto_scroll {
            compute_auto_scroll(&line_strings, inner)
        } else if scroll > compute_safe_max(inner) {
            compute_safe_max(inner)
        } else {
            scroll
        };

        let text: Text = content.into();
        let paragraph = Paragraph::new(text)
            .wrap(ratatui::widgets::Wrap { trim: false })
            .scroll((scroll_offset, 0));

        f.render_widget(paragraph, inner);

        inner_height
    }
}

fn estimate_wrapped_lines(lines: &[&str], inner_width: u16) -> usize {
    let width = inner_width.max(1) as usize;
    lines
        .iter()
        .map(|line| {
            let char_count = line.chars().count();
            if char_count == 0 {
                1
            } else {
                char_count.div_ceil(width)
            }
        })
        .sum()
}

fn compute_safe_max(inner: Rect) -> u16 {
    u16::MAX.saturating_sub(inner.height)
}

fn compute_auto_scroll(lines: &[&str], inner: Rect) -> u16 {
    let total = estimate_wrapped_lines(lines, inner.width);
    let safe_max = compute_safe_max(inner) as usize;
    total.saturating_sub(inner.height as usize).min(safe_max) as u16
}
