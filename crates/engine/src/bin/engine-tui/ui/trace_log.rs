//! Right panel: IR trace log viewer

use crate::trace::{DisplayTraceEntry, TraceDetail};
use cgdsl_engine::{TraceEntry, TraceEvent};
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Paragraph};

pub struct TraceLogPanel {
    pub detail: TraceDetail,
    /// `false` = simplified (DSL text); `true` = raw (`Debug` output).
    pub raw: bool,
}

impl TraceLogPanel {
    pub fn new(detail: TraceDetail, raw: bool) -> Self {
        Self { detail, raw }
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
        let title = format!(
            "IR TRACE LOG ({:?}, {})",
            self.detail,
            if self.raw { "raw" } else { "simplified" }
        );

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
            .filter(|e| match e {
                DisplayTraceEntry::TurnSeparator { .. } => true,
                DisplayTraceEntry::Entry { entry, .. } => self.detail.passes(entry),
            })
            .collect();

        let visible: Vec<&DisplayTraceEntry> = if matches!(self.detail, TraceDetail::Last5) {
            let mut result = Vec::new();
            let mut count = 0;
            for e in filtered.iter().rev() {
                if matches!(e, DisplayTraceEntry::Entry { .. }) {
                    count += 1;
                }
                result.push(*e);
                if count >= 5 {
                    break;
                }
            }
            result.reverse();
            result
        } else {
            filtered
        };

        let mut lines: Vec<Line> = Vec::new();
        for e in &visible {
            lines.push(self.format_entry(e));
        }

        let scroll_offset = if auto_scroll {
            compute_auto_scroll(&lines, inner)
        } else if scroll > compute_safe_max(inner) {
            compute_safe_max(inner)
        } else {
            scroll
        };

        let text = Text::from(lines);
        let paragraph = Paragraph::new(text)
            .wrap(ratatui::widgets::Wrap { trim: true })
            .scroll((scroll_offset, 0));

        f.render_widget(paragraph, inner);

        inner_height
    }

    fn format_entry(&self, entry: &DisplayTraceEntry) -> Line<'static> {
        match entry {
            DisplayTraceEntry::Entry {
                step_number,
                entry: TraceEntry::Step { from, to, event },
            } => {
                let step = Span::styled(
                    format!("[Step {:3}]", step_number),
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
            DisplayTraceEntry::TurnSeparator { player_name } => {
                let width = 60;
                let label = format!(" Turn: {} ", player_name);
                let line = format!("{:─^width$}", label);
                Line::from(Span::styled(line, Style::default().fg(Color::Magenta)))
            }
        }
    }

    fn format_event(&self, event: &TraceEvent) -> Vec<Span<'static>> {
        match event {
            TraceEvent::Action {
                subtype,
                detail,
                raw_detail,
            } => {
                if self.raw {
                    vec![
                        Span::styled(
                            format!("Action:{}", subtype),
                            Style::default().fg(Color::LightGreen),
                        ),
                        Span::raw(format!(" {}", raw_detail)),
                    ]
                } else {
                    // Simplified mode: the DSL line alone — the action text
                    // is self-describing ("deal …", "shuffle …", "score …").
                    vec![Span::styled(
                        detail.clone(),
                        Style::default().fg(Color::LightGreen),
                    )]
                }
            }
            TraceEvent::Choice {
                chosen_idx,
                options,
            } => {
                let opt_labels: Vec<String> = options
                    .iter()
                    .enumerate()
                    .map(|(i, o)| format!("{}. {}", i + 1, o))
                    .collect();
                vec![
                    Span::styled("Choice:", Style::default().fg(Color::Yellow)),
                    Span::raw(format!(
                        " chose {} (from [{}])",
                        chosen_idx + 1,
                        opt_labels.join(", ")
                    )),
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
            TraceEvent::Condition {
                expr,
                raw_expr,
                result,
                negated,
                took_else,
            } => {
                let r_color = if *result { Color::Green } else { Color::Red };
                let shown = if self.raw { raw_expr } else { expr };
                vec![
                    Span::styled("Condition: ", Style::default().fg(Color::LightMagenta)),
                    Span::raw(shown.clone()),
                    Span::raw(" = "),
                    Span::styled(result.to_string(), Style::default().fg(r_color)),
                    Span::styled(
                        format!(" (neg={}, body={})", negated, took_else),
                        Style::default().fg(Color::LightBlue),
                    ),
                ]
            }
            TraceEvent::EndCondition {
                expr,
                raw_expr,
                result,
                stage,
                exited,
            } => {
                let r_color = if *result { Color::Green } else { Color::Red };
                let x_color = if *exited { Color::Green } else { Color::Red };
                let shown = if self.raw { raw_expr } else { expr };
                vec![
                    Span::styled(
                        format!("EndCondition({}): ", stage),
                        Style::default().fg(Color::LightMagenta),
                    ),
                    Span::raw(shown.clone()),
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
            TraceEvent::Trigger => vec![Span::styled(
                "Trigger",
                Style::default().fg(Color::LightBlue),
            )],
            TraceEvent::Quantifier { kind, detail } => vec![
                Span::styled(
                    format!("Quantifier:{}", kind),
                    Style::default().fg(Color::Cyan),
                ),
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
                char_count.div_ceil(width)
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
