use super::*;
use crate::game_data::GameData;
use front_end::ast::*;
use front_end::ir::{Edge, Ir, LoweredPayLoad, Payload, StateID};
use std::collections::HashMap;

fn state_id(n: u32) -> StateID {
    unsafe { std::mem::transmute(n) }
}

fn make_move_action(
    from_card_set: CardSet,
    status: Status,
    to_card_set: CardSet,
) -> LoweredPayLoad {
    Payload::Action(GameRule::Action {
        action: ActionRule::Move {
            move_type: MoveType::Classic {
                classic: ClassicMove::MoveCardSet {
                    move_cs: MoveCardSet::Move {
                        from: from_card_set,
                        status,
                        to: to_card_set,
                    },
                },
            },
        },
    })
}

fn make_card_set_top(location: &str) -> CardSet {
    CardSet::Group {
        group: Group::CardPosition {
            card_position: CardPosition::Query {
                query: QueryCardPosition::Top {
                    location: location.to_string(),
                },
            },
        },
    }
}

fn make_card_set_location(name: &str) -> CardSet {
    CardSet::Group {
        group: Group::Groupable {
            groupable: Groupable::Location {
                name: name.to_string(),
            },
        },
    }
}

#[test]
fn payload_label_action_renders_display() {
    let from = make_card_set_top("Hand");
    let to = make_card_set_location("Table");
    let payload = make_move_action(from, Status::FaceDown, to);
    let label = payload_label(&payload);
    assert!(
        label.contains("move"),
        "label should contain 'move': {}",
        label
    );
    assert!(
        label.contains("face down"),
        "label should contain 'face down': {}",
        label
    );
}

#[test]
fn payload_label_condition_renders_if() {
    let payload = Payload::Condition {
        expr: BoolExpr::Aggregate {
            aggregate: AggregateBool::CardSetEmpty {
                card_set: make_card_set_location("Hand"),
            },
        },
        negated: false,
    };
    let label = payload_label(&payload);
    assert!(
        label.contains("if "),
        "label should contain 'if ': {}",
        label
    );
}

#[test]
fn payload_label_condition_renders_unless() {
    let payload = Payload::Condition {
        expr: BoolExpr::Aggregate {
            aggregate: AggregateBool::CardSetEmpty {
                card_set: make_card_set_location("Hand"),
            },
        },
        negated: true,
    };
    let label = payload_label(&payload);
    assert!(
        label.contains("unless"),
        "label should contain 'unless': {}",
        label
    );
}

#[test]
fn payload_label_choice_is_choose() {
    let payload = Payload::Choice;
    assert_eq!(payload_label(&payload), "choose");
}

#[test]
fn payload_label_optional_is_optional() {
    let payload = Payload::Optional;
    assert_eq!(payload_label(&payload), "optional");
}

#[test]
fn payload_label_trigger_is_trigger() {
    let payload = Payload::Trigger;
    assert_eq!(payload_label(&payload), "trigger");
}

#[test]
fn edge_labels_uses_payload_label() {
    let mut ir = Ir::<LoweredPayLoad>::default();
    let s0 = ir.entry;
    let s1 = state_id(1);
    let s2 = state_id(2);
    let s3 = state_id(3);

    let move_down = make_move_action(
        make_card_set_top("Hand"),
        Status::FaceDown,
        make_card_set_location("Table"),
    );
    let move_up = make_move_action(
        make_card_set_top("Hand"),
        Status::FaceUp,
        make_card_set_location("Table"),
    );

    ir.states.insert(
        s0,
        vec![
            Edge {
                to: s1,
                payload: Payload::Choice,
                meta: None,
            },
            Edge {
                to: s2,
                payload: Payload::Choice,
                meta: None,
            },
        ],
    );
    ir.states.insert(
        s1,
        vec![Edge {
            to: s3,
            payload: move_down,
            meta: None,
        }],
    );
    ir.states.insert(
        s2,
        vec![Edge {
            to: s3,
            payload: move_up,
            meta: None,
        }],
    );
    ir.states.insert(s3, vec![]);

    let labels = ir.edge_labels(s0);
    assert_eq!(labels.len(), 2);
    assert!(
        labels[0].contains("move"),
        "label[0] should contain 'move': {}",
        labels[0]
    );
    assert!(
        labels[0].contains("face down"),
        "label[0] should contain 'face down': {}",
        labels[0]
    );
    assert!(
        labels[1].contains("move"),
        "label[1] should contain 'move': {}",
        labels[1]
    );
    assert!(
        labels[1].contains("face up"),
        "label[1] should contain 'face up': {}",
        labels[1]
    );
}

#[test]
fn edge_labels_falls_back_when_target_empty() {
    let mut ir = Ir::<LoweredPayLoad>::default();
    let s0 = ir.entry;
    let s1 = state_id(1);

    ir.states.insert(
        s0,
        vec![Edge {
            to: s1,
            payload: Payload::Choice,
            meta: None,
        }],
    );
    ir.states.insert(s1, vec![]);

    let labels = ir.edge_labels(s0);
    assert_eq!(labels.len(), 1);
    assert_eq!(labels[0], "Option 1");
}

#[test]
fn step_choice_emits_rich_options_in_needs_input() {
    let mut ir = Ir::<LoweredPayLoad>::default();
    let s0 = ir.entry;
    let s1 = state_id(1);
    let s2 = state_id(2);
    let s3 = state_id(3);

    let move_down = make_move_action(
        make_card_set_top("Hand"),
        Status::FaceDown,
        make_card_set_location("Table"),
    );
    let move_up = make_move_action(
        make_card_set_top("Hand"),
        Status::FaceUp,
        make_card_set_location("Table"),
    );

    ir.states.insert(
        s0,
        vec![
            Edge {
                to: s1,
                payload: Payload::Choice,
                meta: None,
            },
            Edge {
                to: s2,
                payload: Payload::Choice,
                meta: None,
            },
        ],
    );
    ir.states.insert(
        s1,
        vec![Edge {
            to: s3,
            payload: move_down,
            meta: None,
        }],
    );
    ir.states.insert(
        s2,
        vec![Edge {
            to: s3,
            payload: move_up,
            meta: None,
        }],
    );
    ir.states.insert(s3, vec![]);

    let mut interpreter = Interpreter {
        ir,
        game_data: GameData::new(),
        input_buffer: Vec::new(),
        current_state: s0,
        trace_sender: None,
        pending_overlay: HashMap::new(),
        next_synth: u32::MAX - 1,
        pending_quant: None,
    };

    let result = interpreter.step();
    match result {
        StepResult::NeedsInput(InputType::Choice { options, max_index }) => {
            assert_eq!(max_index, 1);
            assert_eq!(options.len(), 2);
            assert!(
                options[0].contains("move"),
                "options[0] should contain 'move': {}",
                options[0]
            );
            assert!(
                options[0].contains("face down"),
                "options[0] should contain 'face down': {}",
                options[0]
            );
            assert!(
                options[1].contains("move"),
                "options[1] should contain 'move': {}",
                options[1]
            );
            assert!(
                options[1].contains("face up"),
                "options[1] should contain 'face up': {}",
                options[1]
            );
        }
        _ => panic!("expected NeedsInput(Choice), got {:?}", result),
    }
}

#[test]
fn step_optional_prompt_contains_accept_action() {
    let mut ir = Ir::<LoweredPayLoad>::default();
    let s0 = ir.entry;
    let s1 = state_id(1);
    let s9 = state_id(9);
    let s3 = state_id(3);

    let deal_action = make_move_action(
        make_card_set_top("Stock"),
        Status::Private,
        make_card_set_location("Hand"),
    );

    ir.states.insert(
        s0,
        vec![
            Edge {
                to: s1,
                payload: Payload::Optional,
                meta: None,
            },
            Edge {
                to: s9,
                payload: Payload::Optional,
                meta: None,
            },
        ],
    );
    ir.states.insert(
        s1,
        vec![Edge {
            to: s3,
            payload: deal_action,
            meta: None,
        }],
    );
    ir.states.insert(s9, vec![]);
    ir.states.insert(s3, vec![]);

    let mut interpreter = Interpreter {
        ir,
        game_data: GameData::new(),
        input_buffer: Vec::new(),
        current_state: s0,
        trace_sender: None,
        pending_overlay: HashMap::new(),
        next_synth: u32::MAX - 1,
        pending_quant: None,
    };

    let result = interpreter.step();
    match result {
        StepResult::NeedsInput(InputType::Optional(prompt)) => {
            assert!(
                prompt.contains("Do you want to:"),
                "prompt should contain 'Do you want to:': {}",
                prompt
            );
            assert!(
                prompt.contains("move"),
                "prompt should contain 'move': {}",
                prompt
            );
        }
        _ => panic!("expected NeedsInput(Optional), got {:?}", result),
    }
}

#[test]
fn step_optional_prompt_fallback_when_no_accept_edge() {
    let mut ir = Ir::<LoweredPayLoad>::default();
    let s0 = ir.entry;
    let s1 = state_id(1);
    let s9 = state_id(9);

    ir.states.insert(
        s0,
        vec![
            Edge {
                to: s1,
                payload: Payload::Optional,
                meta: None,
            },
            Edge {
                to: s9,
                payload: Payload::Optional,
                meta: None,
            },
        ],
    );
    ir.states.insert(s1, vec![]);
    ir.states.insert(s9, vec![]);

    let mut interpreter = Interpreter {
        ir,
        game_data: GameData::new(),
        input_buffer: Vec::new(),
        current_state: s0,
        trace_sender: None,
        pending_overlay: HashMap::new(),
        next_synth: u32::MAX - 1,
        pending_quant: None,
    };

    let result = interpreter.step();
    match result {
        StepResult::NeedsInput(InputType::Optional(prompt)) => {
            assert_eq!(prompt, "Do you want to take this optional action? (y/n)");
        }
        _ => panic!("expected NeedsInput(Optional), got {:?}", result),
    }
}
