//! Right panel: IR trace log viewer

use crate::trace::{DisplayTraceEntry, TraceDetail};
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Text};
use ratatui::widgets::{Block, Borders, Paragraph};

pub struct TraceLogPanel {
    pub detail: TraceDetail,
}

impl TraceLogPanel {
    pub fn new(detail: TraceDetail) -> Self {
        Self { detail }
    }

    pub fn render(
        &self,
        f: &mut ratatui::Frame,
        entries: &[DisplayTraceEntry],
        area: Rect,
        scroll: u16,
        auto_scroll: bool,
        focused: bool,
    ) -> u16 {
        let title = format!("IR TRACE LOG ({:?})", self.detail);

        let block = if focused {
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow))
        } else {
            Block::default().title(title).borders(Borders::ALL)
        };

        f.render_widget(&block, area);

        let inner = block.inner(area);
        let inner_height = inner.height;

        let filtered: Vec<_> = entries
            .iter()
            .filter(|e| self.detail.passes(&e.entry))
            .collect();

        let visible: Vec<&DisplayTraceEntry> = if matches!(self.detail, TraceDetail::Last5) {
            let start = filtered.len().saturating_sub(5);
            filtered[start..].to_vec()
        } else {
            filtered
        };

        let mut lines: Vec<Line> = Vec::new();
        for e in &visible {
            let line = format!("[Step {:3}] {}", e.step_number, self.format_entry(e));
            lines.push(Line::from(line));
        }

        let scroll_offset = if auto_scroll {
            compute_auto_scroll(&lines, inner)
        } else {
            if scroll > compute_safe_max(inner) {
                compute_safe_max(inner)
            } else {
                scroll
            }
        };

        let text = Text::from(lines);
        let paragraph = Paragraph::new(text)
            .wrap(ratatui::widgets::Wrap { trim: true })
            .scroll((scroll_offset, 0));

        f.render_widget(paragraph, inner);

        inner_height
    }

    fn format_entry(&self, entry: &DisplayTraceEntry) -> String {
        format!("{}", entry.entry)
    }
}

fn estimate_wrapped_lines(lines: &[Line], inner_width: u16) -> usize {
    let width = inner_width.max(1) as usize;
    lines
        .iter()
        .map(|line| {
            let char_count = line.width();
            if char_count == 0 {
                1
            } else {
                (char_count + width - 1) / width
            }
        })
        .sum()
}

fn compute_safe_max(inner: Rect) -> u16 {
    u16::MAX.saturating_sub(inner.height)
}

fn compute_auto_scroll(lines: &[Line], inner: Rect) -> u16 {
    let total = estimate_wrapped_lines(lines, inner.width);
    let safe_max = compute_safe_max(inner) as usize;
    total.saturating_sub(inner.height as usize).min(safe_max) as u16
}
