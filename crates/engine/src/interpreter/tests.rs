use super::InputKind;
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

// -----------------------------------------------------------------------------
// Invariant, dispatch-arm, and trace-emission tests (test-plan-3).
// See crates/engine/docs/invariants.md: I-3, I-4, I-7, I-8.
// -----------------------------------------------------------------------------

use front_end::ast::EndCondition as AstEndCondition;
use std::sync::{Arc, Mutex};

/// Add a real empty location named `name` to `gd`. Required so that
/// `eval_cardset` on a `Groupable::Location { name }` succeeds (otherwise
/// it returns `Err("Location {name} not found")`).
fn add_empty_location(gd: &mut GameData, name: &str) {
    gd.add_location(
        "Table".to_string(),
        crate::game_data::Location {
            name: name.to_string(),
            cards: vec![],
        },
    );
}

/// `BoolExpr` that evaluates to `true` against `gd_for_bool()` (an empty
/// location named `EmptyLoc`). There is no `BoolExpr::Literal` variant, so we
/// route through `CardSetEmpty`.
fn const_true_expr() -> BoolExpr {
    BoolExpr::Aggregate {
        aggregate: AggregateBool::CardSetEmpty {
            card_set: make_card_set_location("EmptyLoc"),
        },
    }
}

/// `BoolExpr` that evaluates to `false` against `gd_for_bool()`.
fn const_false_expr() -> BoolExpr {
    BoolExpr::Aggregate {
        aggregate: AggregateBool::CardSetNotEmpty {
            card_set: make_card_set_location("EmptyLoc"),
        },
    }
}

/// `GameData` pre-populated with the empty `EmptyLoc` location required by
/// `const_true_expr` / `const_false_expr`.
fn gd_for_bool() -> GameData {
    let mut gd = GameData::new();
    add_empty_location(&mut gd, "EmptyLoc");
    gd
}

macro_rules! make_interp {
    ($ir:expr, $gd:expr, $state:expr) => {{
        Interpreter {
            ir: $ir,
            game_data: $gd,
            input_buffer: Vec::new(),
            current_state: $state,
            trace_sender: None,
            pending_overlay: HashMap::new(),
            next_synth: u32::MAX - 1,
            pending_quant: None,
        }
    }};
}

// ===== Task 1: I-4 — GameOver vs dead-end-non-goal Error =====

#[test]
fn step_returns_game_over_at_goal_with_no_edges() {
    // I-4: GameOver requires BOTH no outgoing edges AND current_state == ir.goal.
    let mut ir = Ir::<LoweredPayLoad>::default();
    let s0 = ir.entry; // entry == goal in Ir::default
    ir.states.insert(s0, vec![]);

    let mut interp = make_interp!(ir, GameData::new(), s0);
    assert!(matches!(interp.step(), StepResult::GameOver));
}

#[test]
fn step_returns_error_at_dead_end_non_goal_state() {
    // I-4: a dead-end state that is NOT the goal yields Error, not GameOver.
    let mut ir = Ir::<LoweredPayLoad>::default();
    let s0 = ir.entry;
    let s1 = state_id(1); // not the goal
    ir.goal = state_id(999);
    ir.states.insert(
        s0,
        vec![Edge {
            to: s1,
            payload: Payload::Trigger,
            meta: None,
        }],
    );
    ir.states.insert(s1, vec![]); // dead end, s1 != goal

    let mut interp = make_interp!(ir, GameData::new(), s0);
    assert!(matches!(interp.step(), StepResult::Ok)); // advance s0 -> s1 via Trigger
    let r = interp.step();
    match r {
        StepResult::Error(msg) => assert!(
            msg.contains("not at goal state"),
            "expected 'not at goal state' error, got: {msg}"
        ),
        other => panic!("expected Error, got {:?}", other),
    }
}

#[test]
fn step_returns_error_when_current_state_not_in_ir() {
    let ir = Ir::<LoweredPayLoad>::default();
    let mut interp = make_interp!(ir, GameData::new(), state_id(424242));
    match interp.step() {
        StepResult::Error(msg) => assert!(
            msg.contains("not found in IR"),
            "expected 'not found in IR' error, got: {msg}"
        ),
        other => panic!("expected Error, got {:?}", other),
    }
}

// ===== Task 2: I-3 — Condition vs EndCondition inverted edge indexing =====
//
// For `Payload::Condition`: `should_take_else = result != negated`;
//   true  => edges[1] (the "else"/true branch)
//   false => edges[0] (the false branch)
//
// For `Payload::EndCondition`: the engine code at interpreter/mod.rs:287
// computes `should_exit = result == *negated`; therefore
//   should_exit true  => edges[0] (exit)
//   should_exit false => edges[1] (continue)
//
// Notes:
//   - `BoolExpr` has no `Literal` variant; we use `CardSetEmpty` /
//     `CardSetNotEmpty` against a real (empty) location via `gd_for_bool()`.
//   - Both edges in each test carry the *same* `expr` and `negated`; the
//     dispatcher only consults the first edge's payload fields, but we mirror
//     the value on the second edge to keep the IR internally consistent.

fn condition_state_edges(
    expr: BoolExpr,
    true_to: StateID,
    false_to: StateID,
) -> Vec<Edge<LoweredPayLoad>> {
    vec![
        Edge {
            to: true_to,
            payload: Payload::Condition {
                expr: expr.clone(),
                negated: false,
            },
            meta: None,
        },
        Edge {
            to: false_to,
            payload: Payload::Condition {
                expr,
                negated: true,
            },
            meta: None,
        },
    ]
}

#[test]
fn condition_true_takes_true_branch() {
    let mut ir = Ir::<LoweredPayLoad>::default();
    let s0 = ir.entry;
    let s_true = state_id(10);
    let s_false = state_id(20);
    let s_end = state_id(30);
    ir.states.insert(
        s0,
        condition_state_edges(const_true_expr(), s_true, s_false),
    );
    ir.states.insert(
        s_true,
        vec![Edge {
            to: s_end,
            payload: Payload::Trigger,
            meta: None,
        }],
    );
    ir.states.insert(
        s_false,
        vec![Edge {
            to: s_end,
            payload: Payload::Trigger,
            meta: None,
        }],
    );
    ir.states.insert(s_end, vec![]);

    let mut interp = make_interp!(ir, gd_for_bool(), s0);
    interp.step();
    assert_eq!(interp.current_state, s_true, "true => if-body branch");
}

#[test]
fn condition_false_takes_false_branch() {
    let mut ir = Ir::<LoweredPayLoad>::default();
    let s0 = ir.entry;
    let s_true = state_id(10);
    let s_false = state_id(20);
    let s_end = state_id(30);
    ir.states.insert(
        s0,
        condition_state_edges(const_false_expr(), s_true, s_false),
    );
    ir.states.insert(
        s_true,
        vec![Edge {
            to: s_end,
            payload: Payload::Trigger,
            meta: None,
        }],
    );
    ir.states.insert(
        s_false,
        vec![Edge {
            to: s_end,
            payload: Payload::Trigger,
            meta: None,
        }],
    );
    ir.states.insert(s_end, vec![]);

    let mut interp = make_interp!(ir, gd_for_bool(), s0);
    interp.step();
    assert_eq!(interp.current_state, s_false, "false => else/skip branch");
}

// EndCondition: per actual code at interpreter/mod.rs:287,
// `should_exit = result == *negated`. So:
//   result=true,  negated=false -> should_exit=false -> edges[1] (continue)
//   result=false, negated=false -> should_exit=true  -> edges[0] (exit)
//   result=true,  negated=true  -> should_exit=true  -> edges[0] (exit)
//   result=false, negated=true  -> should_exit=false -> edges[1] (continue)

fn end_cond_payload(expr: AstEndCondition, negated: bool, stage: &str) -> LoweredPayLoad {
    Payload::EndCondition {
        expr,
        negated,
        stage: stage.to_string(),
    }
}

fn until_bool(expr: BoolExpr) -> AstEndCondition {
    AstEndCondition::UntilBool { bool_expr: expr }
}

#[test]
fn end_condition_continues_when_expr_true_and_not_negated() {
    // result=true, negated=false -> should_exit = true == false = false -> edges[1] (continue)
    let mut ir = Ir::<LoweredPayLoad>::default();
    let s0 = ir.entry;
    let s_exit = state_id(10);
    let s_continue = state_id(20);
    let s_end = state_id(30);
    let ec = until_bool(const_true_expr());
    ir.states.insert(
        s0,
        vec![
            Edge {
                to: s_exit,
                payload: end_cond_payload(ec.clone(), false, "Play"),
                meta: None,
            },
            Edge {
                to: s_continue,
                payload: end_cond_payload(ec, false, "Play"),
                meta: None,
            },
        ],
    );
    ir.states.insert(
        s_exit,
        vec![Edge {
            to: s_end,
            payload: Payload::Trigger,
            meta: None,
        }],
    );
    ir.states.insert(
        s_continue,
        vec![Edge {
            to: s_end,
            payload: Payload::Trigger,
            meta: None,
        }],
    );
    ir.states.insert(s_end, vec![]);

    let mut interp = make_interp!(ir, gd_for_bool(), s0);
    interp.step();
    assert_eq!(
        interp.current_state, s_continue,
        "EndCondition true & not-negated => edges[1] (continue, since result==negated==false)"
    );
}

#[test]
fn end_condition_exits_when_expr_false_and_not_negated() {
    // result=false, negated=false -> should_exit = false == false = true -> edges[0] (exit)
    let mut ir = Ir::<LoweredPayLoad>::default();
    let s0 = ir.entry;
    let s_exit = state_id(10);
    let s_continue = state_id(20);
    let s_end = state_id(30);
    let ec = until_bool(const_false_expr());
    ir.states.insert(
        s0,
        vec![
            Edge {
                to: s_exit,
                payload: end_cond_payload(ec.clone(), false, "Play"),
                meta: None,
            },
            Edge {
                to: s_continue,
                payload: end_cond_payload(ec, false, "Play"),
                meta: None,
            },
        ],
    );
    ir.states.insert(
        s_exit,
        vec![Edge {
            to: s_end,
            payload: Payload::Trigger,
            meta: None,
        }],
    );
    ir.states.insert(
        s_continue,
        vec![Edge {
            to: s_end,
            payload: Payload::Trigger,
            meta: None,
        }],
    );
    ir.states.insert(s_end, vec![]);

    let mut interp = make_interp!(ir, gd_for_bool(), s0);
    interp.step();
    assert_eq!(
        interp.current_state, s_exit,
        "EndCondition false & not-negated => edges[0] (exit, since result==negated==true)"
    );
}

#[test]
fn end_condition_exits_when_expr_true_and_negated() {
    // result=true, negated=true -> should_exit = true == true = true -> edges[0] (exit)
    let mut ir = Ir::<LoweredPayLoad>::default();
    let s0 = ir.entry;
    let s_exit = state_id(10);
    let s_continue = state_id(20);
    let s_end = state_id(30);
    let ec = until_bool(const_true_expr());
    ir.states.insert(
        s0,
        vec![
            Edge {
                to: s_exit,
                payload: end_cond_payload(ec.clone(), true, "Play"),
                meta: None,
            },
            Edge {
                to: s_continue,
                payload: end_cond_payload(ec, true, "Play"),
                meta: None,
            },
        ],
    );
    ir.states.insert(
        s_exit,
        vec![Edge {
            to: s_end,
            payload: Payload::Trigger,
            meta: None,
        }],
    );
    ir.states.insert(
        s_continue,
        vec![Edge {
            to: s_end,
            payload: Payload::Trigger,
            meta: None,
        }],
    );
    ir.states.insert(s_end, vec![]);

    let mut interp = make_interp!(ir, gd_for_bool(), s0);
    interp.step();
    assert_eq!(
        interp.current_state, s_exit,
        "EndCondition true & negated => edges[0] (exit, since result==negated==true)"
    );
}

#[test]
fn end_condition_continues_when_expr_false_and_negated() {
    // result=false, negated=true -> should_exit = false == true = false -> edges[1] (continue)
    let mut ir = Ir::<LoweredPayLoad>::default();
    let s0 = ir.entry;
    let s_exit = state_id(10);
    let s_continue = state_id(20);
    let s_end = state_id(30);
    let ec = until_bool(const_false_expr());
    ir.states.insert(
        s0,
        vec![
            Edge {
                to: s_exit,
                payload: end_cond_payload(ec.clone(), true, "Play"),
                meta: None,
            },
            Edge {
                to: s_continue,
                payload: end_cond_payload(ec, true, "Play"),
                meta: None,
            },
        ],
    );
    ir.states.insert(
        s_exit,
        vec![Edge {
            to: s_end,
            payload: Payload::Trigger,
            meta: None,
        }],
    );
    ir.states.insert(
        s_continue,
        vec![Edge {
            to: s_end,
            payload: Payload::Trigger,
            meta: None,
        }],
    );
    ir.states.insert(s_end, vec![]);

    let mut interp = make_interp!(ir, gd_for_bool(), s0);
    interp.step();
    assert_eq!(
        interp.current_state, s_continue,
        "EndCondition false & negated => edges[1] (continue)"
    );
}

#[test]
fn condition_with_wrong_edge_count_returns_error() {
    let mut ir = Ir::<LoweredPayLoad>::default();
    let s0 = ir.entry;
    ir.states.insert(
        s0,
        vec![Edge {
            to: state_id(1),
            payload: Payload::Condition {
                expr: const_true_expr(),
                negated: false,
            },
            meta: None,
        }], // only 1 edge, need 2
    );

    let mut interp = make_interp!(ir, gd_for_bool(), s0);
    match interp.step() {
        StepResult::Error(msg) => assert!(msg.contains("exactly 2 edges"), "got: {msg}"),
        other => panic!("expected Error, got {:?}", other),
    }
}

#[test]
fn end_condition_with_wrong_edge_count_returns_error() {
    let mut ir = Ir::<LoweredPayLoad>::default();
    let s0 = ir.entry;
    let ec = until_bool(const_true_expr());
    ir.states.insert(
        s0,
        vec![
            Edge {
                to: state_id(1),
                payload: end_cond_payload(ec.clone(), false, "Play"),
                meta: None,
            },
            Edge {
                to: state_id(2),
                payload: end_cond_payload(ec.clone(), false, "Play"),
                meta: None,
            },
            Edge {
                to: state_id(3),
                payload: end_cond_payload(ec, false, "Play"),
                meta: None,
            },
        ],
    ); // 3 edges, need 2

    let mut interp = make_interp!(ir, gd_for_bool(), s0);
    match interp.step() {
        StepResult::Error(msg) => assert!(msg.contains("exactly 2 edges"), "got: {msg}"),
        other => panic!("expected Error, got {:?}", other),
    }
}

// ===== Task 3: I-7 — Input buffer is LIFO (last pushed is popped first) =====

#[test]
fn input_buffer_is_lifo_not_fifo() {
    // I-7: provide_input pushes, step pops; multiple inputs are consumed
    // in reverse order. Build a Choice state with 2 outgoing edges; push
    // Choice{idx:1} then Choice{idx:0}; the first step() should consume
    // Choice{idx:0} (LIFO) and advance to edges[0].
    let mut ir = Ir::<LoweredPayLoad>::default();
    let s0 = ir.entry;
    let s1 = state_id(1);
    let s2 = state_id(2);
    let s_end = state_id(99);
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
            to: s_end,
            payload: Payload::Trigger,
            meta: None,
        }],
    );
    ir.states.insert(
        s2,
        vec![Edge {
            to: s_end,
            payload: Payload::Trigger,
            meta: None,
        }],
    );
    ir.states.insert(s_end, vec![]);

    let mut interp = make_interp!(ir, GameData::new(), s0);

    // First step: NeedsInput (Choice)
    assert!(matches!(interp.step(), StepResult::NeedsInput(_)));
    // Push idx:1 first, then idx:0
    interp.provide_input(Input {
        player_id: "P1".into(),
        kind: InputKind::Choice { idx: 1 },
    });
    interp.provide_input(Input {
        player_id: "P1".into(),
        kind: InputKind::Choice { idx: 0 },
    });
    // Second step: pops idx:0 (LIFO) -> edges[0] -> s1
    assert!(matches!(interp.step(), StepResult::Ok));
    assert_eq!(
        interp.current_state, s1,
        "LIFO: idx:0 was pushed last, popped first"
    );
    // Third step: at s1 the edge is a Trigger so the remaining Choice{idx:1}
    // stays buffered; advance to s_end.
    assert!(matches!(interp.step(), StepResult::Ok));
    assert_eq!(interp.current_state, s_end);
    // The buffered Choice{idx:1} is still in the buffer (never consumed
    // because s1/s_end aren't Choice states).
    assert_eq!(
        interp.input_buffer.len(),
        1,
        "remaining input still buffered"
    );
}

// ===== Task 4: I-8 — Out-of-range Choice silently stalls =====

#[test]
fn out_of_range_choice_silently_stalls() {
    // I-8: if input.idx() is out of range, execute_edge is skipped,
    // current_state is NOT advanced, and the next step() re-enters the
    // same state. With an empty buffer, the next step yields NeedsInput.
    let mut ir = Ir::<LoweredPayLoad>::default();
    let s0 = ir.entry;
    let s1 = state_id(1);
    let s_end = state_id(99);
    ir.states.insert(
        s0,
        vec![Edge {
            to: s1,
            payload: Payload::Choice,
            meta: None,
        }], // only 1 edge; valid idx is 0
    );
    ir.states.insert(
        s1,
        vec![Edge {
            to: s_end,
            payload: Payload::Trigger,
            meta: None,
        }],
    );
    ir.states.insert(s_end, vec![]);

    let mut interp = make_interp!(ir, GameData::new(), s0);

    // First step: NeedsInput
    assert!(matches!(interp.step(), StepResult::NeedsInput(_)));
    // Push an out-of-range idx
    interp.provide_input(Input {
        player_id: "P1".into(),
        kind: InputKind::Choice { idx: 5 },
    }); // only edges[0] exists
        // Second step: execute_edge is skipped, returns Ok, current_state unchanged
    assert!(matches!(interp.step(), StepResult::Ok));
    assert_eq!(
        interp.current_state, s0,
        "I-8: out-of-range Choice does not advance"
    );
    // Third step: buffer empty, NeedsInput again
    assert!(matches!(interp.step(), StepResult::NeedsInput(_)));
}

// ===== Task 5: StageRoundCounter / EndStage / Trigger arms in step() =====

#[test]
fn stage_round_counter_arm_increments_and_advances() {
    let mut ir = Ir::<LoweredPayLoad>::default();
    let s0 = ir.entry;
    let s1 = state_id(1);
    let s_end = state_id(99);
    ir.states.insert(
        s0,
        vec![Edge {
            to: s1,
            payload: Payload::StageRoundCounter("Play".to_string()),
            meta: None,
        }],
    );
    ir.states.insert(
        s1,
        vec![Edge {
            to: s_end,
            payload: Payload::Trigger,
            meta: None,
        }],
    );
    ir.states.insert(s_end, vec![]);

    let mut interp = make_interp!(ir, GameData::new(), s0);

    assert!(matches!(interp.step(), StepResult::Ok));
    assert_eq!(interp.current_state, s1);
    assert_eq!(
        interp.game_data.get_stage_counter("Play".to_string()),
        1,
        "StageRoundCounter arm increments the counter"
    );
    assert!(
        interp.game_data.stage_stack.contains(&"Play".to_string()),
        "ensure_stage_entered was called"
    );
}

#[test]
fn end_stage_arm_leaves_stage_and_advances() {
    let mut ir = Ir::<LoweredPayLoad>::default();
    let s0 = ir.entry;
    let s1 = state_id(1);
    let s_end = state_id(99);
    ir.states.insert(
        s0,
        vec![Edge {
            to: s1,
            payload: Payload::EndStage("Play".to_string()),
            meta: None,
        }],
    );
    ir.states.insert(
        s1,
        vec![Edge {
            to: s_end,
            payload: Payload::Trigger,
            meta: None,
        }],
    );
    ir.states.insert(s_end, vec![]);

    let mut gd = GameData::new();
    gd.add_player("Alice".to_string());
    gd.enter_stage("Play".to_string(), vec!["Alice".to_string()]);
    assert_eq!(gd.stage_stack.len(), 1);

    let mut interp = make_interp!(ir, gd, s0);

    assert!(matches!(interp.step(), StepResult::Ok));
    assert_eq!(interp.current_state, s1);
    assert!(
        interp.game_data.stage_stack.is_empty(),
        "EndStage arm pops the stage"
    );
}

#[test]
fn trigger_arm_advances_without_mutation() {
    let mut ir = Ir::<LoweredPayLoad>::default();
    let s0 = ir.entry;
    let s_end = state_id(99);
    ir.states.insert(
        s0,
        vec![Edge {
            to: s_end,
            payload: Payload::Trigger,
            meta: None,
        }],
    );
    ir.states.insert(s_end, vec![]);

    let mut interp = make_interp!(ir, GameData::new(), s0);

    assert!(matches!(interp.step(), StepResult::Ok));
    assert_eq!(interp.current_state, s_end);
    // No stage counter, no stage stack changes
    assert!(interp.game_data.stage_counters.is_empty());
    assert!(interp.game_data.stage_stack.is_empty());
}

// ===== Task 6: Trace emission for each payload arm =====

fn capture_trace(interp: &mut Interpreter) -> Vec<TraceEntry> {
    let trace: Arc<Mutex<Vec<TraceEntry>>> = Arc::new(Mutex::new(Vec::new()));
    let trace_clone = trace.clone();
    interp.trace_sender = Some(Box::new(move |e| {
        trace_clone.lock().unwrap().push(e);
    }));
    interp.step();
    // drop the sender so we can unwrap the Arc
    let sender = std::mem::take(&mut interp.trace_sender);
    drop(sender);
    Arc::try_unwrap(trace).unwrap().into_inner().unwrap()
}

#[test]
fn trace_emits_condition_event() {
    let mut ir = Ir::<LoweredPayLoad>::default();
    let s0 = ir.entry;
    let s1 = state_id(1);
    let s2 = state_id(2);
    ir.states
        .insert(s0, condition_state_edges(const_true_expr(), s2, s1));
    ir.states.insert(s1, vec![]);
    ir.states.insert(s2, vec![]);

    let mut interp = make_interp!(ir, gd_for_bool(), s0);
    let trace = capture_trace(&mut interp);
    assert_eq!(trace.len(), 1);
    assert!(matches!(
        trace[0],
        TraceEntry::Step {
            event: TraceEvent::Condition { .. },
            ..
        }
    ));
}

#[test]
fn trace_emits_stage_round_counter_event() {
    let mut ir = Ir::<LoweredPayLoad>::default();
    let s0 = ir.entry;
    let s1 = state_id(1);
    ir.states.insert(
        s0,
        vec![Edge {
            to: s1,
            payload: Payload::StageRoundCounter("Play".to_string()),
            meta: None,
        }],
    );
    ir.states.insert(s1, vec![]);

    let mut interp = make_interp!(ir, GameData::new(), s0);
    let trace = capture_trace(&mut interp);
    assert!(matches!(
        trace[0],
        TraceEntry::Step {
            event: TraceEvent::StageRoundCounter { ref stage, .. },
            ..
        } if stage == "Play"
    ));
}

#[test]
fn trace_emits_end_stage_event() {
    let mut ir = Ir::<LoweredPayLoad>::default();
    let s0 = ir.entry;
    let s1 = state_id(1);
    ir.states.insert(
        s0,
        vec![Edge {
            to: s1,
            payload: Payload::EndStage("Play".to_string()),
            meta: None,
        }],
    );
    ir.states.insert(s1, vec![]);
    let mut gd = GameData::new();
    gd.add_player("Alice".to_string());
    gd.enter_stage("Play".to_string(), vec!["Alice".to_string()]);

    let mut interp = make_interp!(ir, gd, s0);
    let trace = capture_trace(&mut interp);
    assert!(matches!(
        trace[0],
        TraceEntry::Step {
            event: TraceEvent::EndStage { ref stage },
            ..
        } if stage == "Play"
    ));
}

#[test]
fn trace_emits_trigger_event() {
    let mut ir = Ir::<LoweredPayLoad>::default();
    let s0 = ir.entry;
    let s1 = state_id(1);
    ir.states.insert(
        s0,
        vec![Edge {
            to: s1,
            payload: Payload::Trigger,
            meta: None,
        }],
    );
    ir.states.insert(s1, vec![]);

    let mut interp = make_interp!(ir, GameData::new(), s0);
    let trace = capture_trace(&mut interp);
    assert!(matches!(
        trace[0],
        TraceEntry::Step {
            event: TraceEvent::Trigger,
            ..
        }
    ));
}
