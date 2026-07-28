//! Right panel: IR trace log viewer

use crate::trace::{DisplayTraceEntry, TraceDetail};
use cgdsl_engine::{TraceEntry, TraceEvent};
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span, Text};
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
            lines.push(self.format_entry(e));
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

    fn format_entry(&self, entry: &DisplayTraceEntry) -> Line<'static> {
        let TraceEntry::Step { from, to, event } = &entry.entry;

        let step = Span::styled(
            format!("[Step {:3}]", entry.step_number),
            Style::default().fg(Color::LightBlue),
        );
        let arrow = Span::styled(
            format!("[{:->4}]", format!("{}->{}", from, to)),
            Style::default().fg(Color::Cyan),
        );

        let mut spans = vec![step, Span::raw(" "), arrow, Span::raw(" ")];
        spans.extend(self.format_event(event));
        Line::from(spans)
    }

    fn format_event(&self, event: &TraceEvent) -> Vec<Span<'static>> {
        match event {
            TraceEvent::Action { subtype, detail } => vec![
                Span::styled(format!("Action:{}", subtype), Style::default().fg(Color::LightGreen)),
                Span::raw(format!(" {}", detail)),
            ],
            TraceEvent::Choice { chosen_idx, options } => {
                let opt_labels: Vec<String> = options.iter().enumerate()
                    .map(|(i, o)| format!("{}. {}", i + 1, o))
                    .collect();
                vec![
                    Span::styled("Choice:", Style::default().fg(Color::Yellow)),
                    Span::raw(format!(" chose {} (from [{}])", chosen_idx + 1, opt_labels.join(", "))),
                ]
            }
            TraceEvent::OptionalAccept => vec![
                Span::styled("Optional:", Style::default().fg(Color::Yellow)),
                Span::styled(" ACCEPTED", Style::default().fg(Color::Green)),
            ],
            TraceEvent::OptionalDecline => vec![
                Span::styled("Optional:", Style::default().fg(Color::Yellow)),
                Span::styled(" DECLINED", Style::default().fg(Color::Red)),
            ],
            TraceEvent::Condition { expr, result, negated, took_else } => {
                let r_color = if *result { Color::Green } else { Color::Red };
                vec![
                    Span::styled("Condition: ", Style::default().fg(Color::LightMagenta)),
                    Span::raw(expr.clone()),
                    Span::raw(" = "),
                    Span::styled(result.to_string(), Style::default().fg(r_color)),
                    Span::styled(
                        format!(" (neg={}, else={})", negated, took_else),
                        Style::default().fg(Color::LightBlue),
                    ),
                ]
            }
            TraceEvent::EndCondition { expr, result, stage, exited } => {
                let r_color = if *result { Color::Green } else { Color::Red };
                let x_color = if *exited { Color::Green } else { Color::Red };
                vec![
                    Span::styled(
                        format!("EndCondition({}): ", stage),
                        Style::default().fg(Color::LightMagenta),
                    ),
                    Span::raw(expr.clone()),
                    Span::raw(" = "),
                    Span::styled(result.to_string(), Style::default().fg(r_color)),
                    Span::raw(" (exited="),
                    Span::styled(exited.to_string(), Style::default().fg(x_color)),
                    Span::raw(")"),
                ]
            }
            TraceEvent::StageRoundCounter { stage, new_count } => vec![
                Span::styled("StageRoundCounter: ", Style::default().fg(Color::LightCyan)),
                Span::raw(stage.clone()),
                Span::raw(" -> "),
                Span::styled(new_count.to_string(), Style::default().fg(Color::Yellow)),
            ],
            TraceEvent::EndStage { stage } => vec![
                Span::styled("EndStage: ", Style::default().fg(Color::LightCyan)),
                Span::raw(stage.clone()),
            ],
            TraceEvent::Trigger => vec![
                Span::styled("Trigger", Style::default().fg(Color::LightBlue)),
            ],
            TraceEvent::Quantifier { kind, detail } => vec![
                Span::styled(format!("Quantifier:{}", kind), Style::default().fg(Color::Cyan)),
                Span::raw(format!(" {}", detail)),
            ],
        }
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
