//! Right panel: IR trace log viewer

use ratatui::layout::Rect;
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::text::{Text, Line};
use crate::trace::{DisplayTraceEntry, TraceDetail};

pub struct TraceLogPanel {
    pub detail: TraceDetail,
}

impl TraceLogPanel {
    pub fn new(detail: TraceDetail) -> Self {
        Self { detail }
    }

    pub fn render(&self, f: &mut ratatui::Frame, entries: &[DisplayTraceEntry], area: Rect) {
        let block = Block::default()
            .title(format!("IR TRACE LOG ({:?})", self.detail))
            .borders(Borders::ALL);

        f.render_widget(&block, area);

        let inner = block.inner(area);

        let filtered: Vec<_> = entries.iter()
            .filter(|e| self.detail.passes(&e.entry))
            .collect();

        let mut lines: Vec<Line> = Vec::new();
        for e in filtered {
            let line = format!("[Step {:3}] {}", e.step_number, self.format_entry(e));
            lines.push(Line::from(line));
        }

        let text = Text::from(lines);
        let paragraph = Paragraph::new(text)
            .wrap(ratatui::widgets::Wrap { trim: true })
            .scroll((0, 0));

        f.render_widget(paragraph, inner);
    }

    fn format_entry(&self, entry: &DisplayTraceEntry) -> String {
        use cgdsl_engine::TraceEntry::*;
        match &entry.entry {
            Step { state_id, payload_type } =>
                format!("Step @{:?}: {:?}", state_id, payload_type),
            ChoiceMade { chosen_idx } =>
                format!("-> Chose Option {}", chosen_idx + 1),
            OptionalAccepted =>
                format!("-> Optional ACCEPTED"),
            OptionalDeclined =>
                format!("-> Optional DECLINED"),
            ConditionEvaluated { expr, result, negated, took_else } =>
                format!("Condition {} = {} (neg={}, took_else={})", expr, result, negated, took_else),
            EndConditionEvaluated { expr, result, stage, exited } =>
                format!("EndCondition({}) {} = {} (exited={})", stage, expr, result, exited),
            ActionExecuted { action_name } =>
                format!("-> Action: {}", action_name),
        }
    }
}