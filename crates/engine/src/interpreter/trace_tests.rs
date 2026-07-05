use super::*;

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
    let event = TraceEvent::Action {
        subtype: "Action:Move".to_string(),
        detail: "some detail".to_string(),
    };
    let s = format!("{}", event);
    assert!(s.contains("Action:Move"));
    assert!(s.contains("some detail"));
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
fn trace_event_condition_displays_result_and_neg() {
    let event = TraceEvent::Condition {
        expr: "some expr".to_string(),
        result: true,
        negated: false,
        took_else: true,
    };
    let s = format!("{}", event);
    assert!(s.contains("Condition:"));
    assert!(s.contains("some expr"));
    assert!(s.contains("true"));
    assert!(s.contains("neg=false"));
    assert!(s.contains("else=true"));
}

#[test]
fn trace_event_end_condition_displays_stage_and_exited() {
    let event = TraceEvent::EndCondition {
        expr: "e".to_string(),
        result: false,
        stage: "Play".to_string(),
        exited: true,
    };
    let s = format!("{}", event);
    assert!(s.contains("EndCondition(Play)"));
    assert!(s.contains("exited=true"));
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
