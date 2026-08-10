use super::*;
use front_end::ast::{
    ActionRule, BoolExpr, CompareBool, EndCondition as AstEndCondition, GameRule, Group, Groupable,
    IntCompare, IntExpr, MoveCardSet, MoveType, Status,
};

/// `deal 1 from Deck private to Hand of P:P1`-shaped move rule.
fn deal_rule() -> GameRule {
    let loc = |name: &str| front_end::ast::CardSet::Group {
        group: Group::Groupable {
            groupable: Groupable::Location {
                name: name.to_string(),
            },
        },
    };
    GameRule::Action {
        action: ActionRule::Move {
            move_type: MoveType::Deal {
                deal: front_end::ast::DealMove::MoveCardSet {
                    deal_cs: MoveCardSet::Move {
                        from: loc("Deck"),
                        status: Status::Private,
                        to: loc("Hand"),
                    },
                },
            },
        },
    }
}

/// `(2 == 2)`-shaped boolean expression.
fn eq_expr() -> BoolExpr {
    BoolExpr::Aggregate {
        aggregate: front_end::ast::AggregateBool::Compare {
            cmp_bool: CompareBool::Int {
                int: IntExpr::Literal { int: 2 },
                cmp: IntCompare::Eq,
                int1: IntExpr::Literal { int: 2 },
            },
        },
    }
}

/// `until (2 == 2)`-shaped end condition.
fn until_expr() -> AstEndCondition {
    AstEndCondition::UntilBool {
        bool_expr: eq_expr(),
    }
}

#[test]
fn trace_entry_step_displays_bracketed_transition() {
    let entry = TraceEntry::Step {
        from: 5,
        to: 7,
        event: TraceEvent::Trigger,
    };
    let s = format!("{}", entry);
    assert!(s.contains("[5->7]"), "got: {s}");
    assert!(s.contains("Trigger"));
}

#[test]
fn trace_event_action_displays_subtype_and_detail() {
    let event = TraceEvent::Action { rule: deal_rule() };
    let s = format!("{}", event);
    assert!(s.contains("Action:Move"));
    assert!(s.contains("Deck"));
    assert!(s.contains("Hand"), "pretty shows the DSL text: {s}");
    let raw = event.raw();
    assert!(
        raw.contains("Move { move_type"),
        "raw mode shows Debug: {raw}"
    );
    assert!(
        !raw.contains("deal 1 from Deck"),
        "raw mode hides pretty text: {raw}"
    );
}

#[test]
fn trace_event_action_carries_the_typed_rule() {
    let event = TraceEvent::Action { rule: deal_rule() };
    match &event {
        TraceEvent::Action { rule } => match rule {
            GameRule::Action {
                action: ActionRule::Move { move_type },
            } => assert!(matches!(move_type, MoveType::Deal { .. })),
            other => panic!("expected a move rule, got {:?}", other),
        },
        other => panic!("expected Action event, got {:?}", other),
    }
    // The structured summary is derived from the typed payload.
    let summary = event.summary();
    assert!(
        summary.contains("Deck"),
        "summary names the source: {summary}"
    );
    assert!(
        summary.contains("Hand"),
        "summary names the target: {summary}"
    );
}

#[test]
fn trace_event_condition_displays_result_and_neg() {
    let event = TraceEvent::Condition {
        expr: eq_expr(),
        result: true,
        negated: false,
        took_else: true,
    };
    let s = format!("{}", event);
    assert!(s.contains("Condition:"));
    assert!(s.contains("2"), "pretty shows the DSL text: {s}");
    assert!(s.contains("true"));
    assert!(s.contains("neg=false"));
    assert!(s.contains("body=true"), "the body edge was taken: {s}");
    let raw = event.raw();
    assert!(raw.contains("Aggregate {"), "raw mode shows Debug: {raw}");
    assert!(!raw.contains("== 2"), "raw mode hides pretty text: {raw}");
}

#[test]
fn trace_event_condition_carries_the_typed_expr() {
    let event = TraceEvent::Condition {
        expr: eq_expr(),
        result: true,
        negated: false,
        took_else: true,
    };
    match &event {
        TraceEvent::Condition { expr, result, .. } => {
            assert!(*result);
            assert!(
                matches!(
                    expr,
                    BoolExpr::Aggregate {
                        aggregate: front_end::ast::AggregateBool::Compare {
                            cmp_bool: CompareBool::Int { .. }
                        }
                    }
                ),
                "hosts can inspect the typed expression"
            );
        }
        other => panic!("expected Condition event, got {:?}", other),
    }
}

#[test]
fn trace_event_end_condition_displays_stage_and_exited() {
    let event = TraceEvent::EndCondition {
        expr: until_expr(),
        result: false,
        stage: "Play".to_string(),
        exited: true,
    };
    let s = format!("{}", event);
    assert!(s.contains("EndCondition(Play)"));
    assert!(s.contains("exited=true"));
    assert!(event.raw().contains("UntilBool"));
}

#[test]
fn trace_event_choice_displays_one_based_index() {
    let event = TraceEvent::Choice {
        chosen_idx: 2,
        options: vec!["a".to_string(), "b".to_string(), "c".to_string()],
    };
    let s = format!("{}", event);
    assert!(
        s.contains("chose 3"),
        "Choice display must be 1-based; got: {s}"
    );
    assert!(s.contains("a"));
}

#[test]
fn trace_event_optional_accept_displays_accepted() {
    assert!(format!("{}", TraceEvent::OptionalAccept).contains("ACCEPTED"));
}

#[test]
fn trace_event_optional_decline_displays_declined() {
    assert!(format!("{}", TraceEvent::OptionalDecline).contains("DECLINED"));
}

#[test]
fn trace_event_stage_round_counter_displays_count() {
    let event = TraceEvent::StageRoundCounter {
        stage: "Play".to_string(),
        new_count: 3,
    };
    let s = format!("{}", event);
    assert!(s.contains("Play"));
    assert!(s.contains("3"));
}

#[test]
fn trace_event_end_stage_displays_stage() {
    let event = TraceEvent::EndStage {
        stage: "Play".to_string(),
    };
    let s = format!("{}", event);
    assert!(s.contains("EndStage"));
    assert!(s.contains("Play"));
}

#[test]
fn trace_event_trigger_displays_trigger() {
    assert!(format!("{}", TraceEvent::Trigger).contains("Trigger"));
}

#[test]
fn trace_event_quantifier_displays_kind_and_detail() {
    let event = TraceEvent::Quantifier {
        kind: "ChooseCards".to_string(),
        detail: "pick 2".to_string(),
    };
    let s = format!("{}", event);
    assert!(s.contains("Quantifier:ChooseCards"));
    assert!(s.contains("pick 2"));
}

#[test]
fn trace_event_summary_covers_memory_and_cycle_actions() {
    let set = TraceEvent::Action {
        rule: GameRule::Action {
            action: ActionRule::SetMemory {
                memory: "score".to_string(),
                memory_type: front_end::ast::MemoryType::Int {
                    int: IntExpr::Literal { int: 10 },
                },
            },
        },
    };
    assert_eq!(set.summary(), "set score := 10");
    let cycle = TraceEvent::Action {
        rule: GameRule::Action {
            action: ActionRule::CycleAction {
                player: front_end::ast::PlayerExpr::Runtime {
                    runtime: front_end::ast::RuntimePlayer::Next,
                },
            },
        },
    };
    assert!(cycle.summary().starts_with("cycle to "));
    // Events without a structured form fall back to the pretty rendering.
    let trigger = TraceEvent::Trigger;
    assert_eq!(trigger.summary(), "Trigger");
}

#[test]
fn summary_is_derived_not_stored() {
    // The summary must reflect the payload, not a cached string.
    let event = TraceEvent::Action { rule: deal_rule() };
    assert!(event.summary().contains("Private"));
}
