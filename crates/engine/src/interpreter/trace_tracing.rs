//! `tracing` integration bridge (feature: `tracing`).
//!
//! With the `tracing` cargo feature enabled, [`tracing_trace_sender`] returns
//! a closure that forwards every [`TraceEntry`] to the `tracing` crate as a
//! structured event on the `cgdsl_engine::trace` target at `TRACE` level.
//! Wire it into a run with `RunOptions::new().with_trace_sender(tracing_trace_sender())`
//! and filter with `RUST_LOG=cgdsl_engine::trace=trace` (or your subscriber's
//! own filter).

use crate::interpreter::TraceEntry;

/// Build a `trace_sender` bridge from the engine to the `tracing` crate.
///
/// Each emitted event carries structured fields:
/// - `from` / `to` — the raw `StateID`s of the transition;
/// - `pretty` — the one-line DSL rendering (same text as the trace file);
/// - `summary` — the compact structured rendering ([`TraceEvent::summary`]);
/// - `raw` — the `Debug` rendering ([`TraceEvent::raw`]).
pub fn tracing_trace_sender() -> Box<dyn Fn(TraceEntry) + Send> {
    Box::new(|entry: TraceEntry| {
        let TraceEntry::Step { from, to, event } = &entry;
        tracing::trace!(
            target: "cgdsl_engine::trace",
            from = *from,
            to = *to,
            pretty = %entry,
            summary = %event.summary(),
            raw = %event.raw(),
            "fsm step",
        );
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interpreter::{TraceEntry as EngineTraceEntry, TraceEvent};
    use front_end::ast::{ActionRule, GameRule, IntExpr, MemoryType};
    use std::fmt::Write as _;
    use std::sync::{Arc, Mutex};

    /// Minimal `tracing::Subscriber` that captures the recorded fields of
    /// every event, so the bridge can be tested without `tracing-subscriber`.
    struct CollectingSubscriber {
        events: Arc<Mutex<Vec<String>>>,
    }

    impl tracing::Subscriber for CollectingSubscriber {
        fn enabled(&self, _metadata: &tracing::Metadata<'_>) -> bool {
            true
        }
        fn new_span(&self, _span: &tracing::span::Attributes<'_>) -> tracing::span::Id {
            tracing::span::Id::from_u64(1)
        }
        fn record(&self, _span: &tracing::span::Id, _values: &tracing::span::Record<'_>) {}
        fn record_follows_from(&self, _span: &tracing::span::Id, _follows: &tracing::span::Id) {}
        fn event(&self, event: &tracing::Event<'_>) {
            let mut buf = String::new();
            event.record(&mut FieldCollector(&mut buf));
            self.events.lock().unwrap().push(buf);
        }
        fn enter(&self, _span: &tracing::span::Id) {}
        fn exit(&self, _span: &tracing::span::Id) {}
    }

    struct FieldCollector<'a>(&'a mut String);

    impl tracing::field::Visit for FieldCollector<'_> {
        fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
            let _ = write!(self.0, "{}={:?} ", field.name(), value);
        }
        fn record_u64(&mut self, field: &tracing::field::Field, value: u64) {
            let _ = write!(self.0, "{}={} ", field.name(), value);
        }
    }

    #[test]
    fn bridge_forwards_entries_as_structured_events() {
        let events = Arc::new(Mutex::new(Vec::new()));
        let subscriber = CollectingSubscriber {
            events: events.clone(),
        };
        let bridge = tracing_trace_sender();

        tracing::subscriber::with_default(subscriber, || {
            bridge(EngineTraceEntry::Step {
                from: 1,
                to: 2,
                event: TraceEvent::Trigger,
            });
            let rule = GameRule::Action {
                action: ActionRule::SetMemory {
                    memory: "m".to_string(),
                    memory_type: MemoryType::Int {
                        int: IntExpr::Literal { int: 3 },
                    },
                },
            };
            bridge(EngineTraceEntry::Step {
                from: 2,
                to: 3,
                event: TraceEvent::Action { rule },
            });
        });

        let captured = events.lock().unwrap();
        assert_eq!(captured.len(), 2, "one tracing event per trace entry");
        assert!(
            captured[0].contains("from=1") && captured[0].contains("to=2"),
            "transition ids must be structured fields: {}",
            captured[0]
        );
        assert!(
            captured[1].contains("summary=set m := 3"),
            "summary field must carry the structured rendering: {}",
            captured[1]
        );
        assert!(
            captured[1].contains("pretty=") && captured[1].contains("raw="),
            "pretty and raw views must be present: {}",
            captured[1]
        );
    }
}
