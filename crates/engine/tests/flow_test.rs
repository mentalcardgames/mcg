mod common;

use cgdsl_engine::{run_game_with, GameData, Input, InputKind, InputSource, InputType, RunOptions};

use common::{default_input, load_game};

#[test]
fn flow_if_true_executes_body() {
    let ir = load_game("flow_if_true.cgdsl");
    let gd = run_game_with(ir, GameData::new(), default_input(), RunOptions::default())
        .expect("game should complete");

    assert_eq!(
        gd.players[0].score, 10,
        "P1 score should be 10 (if-body executed)"
    );
    assert_eq!(gd.players[1].score, 0, "P2 score unchanged");
    assert_eq!(gd.players[2].score, 0, "P3 score unchanged");
}

#[test]
fn flow_unless_true_skips_body() {
    let ir = load_game("flow_unless_true.cgdsl");
    let gd = run_game_with(ir, GameData::new(), default_input(), RunOptions::default())
        .expect("game should complete");

    assert_eq!(
        gd.players[0].score, 0,
        "P1 score should remain 0 (body skipped)"
    );
    assert_eq!(gd.players[1].score, 0, "P2 score unchanged");
    assert_eq!(gd.players[2].score, 0, "P3 score unchanged");
}

#[test]
fn flow_optional_accept_executes_body() {
    let ir = load_game("flow_optional_accept.cgdsl");
    let gd = run_game_with(
        ir,
        GameData::new(),
        InputSource::Player(Box::new(|it: InputType| match it {
            InputType::Optional(_) => Input {
                player_id: "P1".into(),
                kind: InputKind::OptionalAccept,
            },
            _ => Input {
                player_id: "P1".into(),
                kind: InputKind::Choice { idx: 0 },
            },
        })),
        RunOptions::default(),
    )
    .expect("game should complete");

    assert_eq!(
        gd.players[0].score, 10,
        "P1 score should be 10 (optional accepted)"
    );
    assert_eq!(gd.players[1].score, 0, "P2 score unchanged");
    assert_eq!(gd.players[2].score, 0, "P3 score unchanged");
}

#[test]
fn flow_optional_bust_eliminates_on_over_limit() {
    let ir = load_game("flow_optional_bust.cgdsl");
    let gd = run_game_with(
        ir,
        GameData::new(),
        InputSource::Player(Box::new(|it: InputType| match it {
            InputType::Optional(_) => Input {
                player_id: "P1".into(),
                kind: InputKind::OptionalAccept,
            },
            _ => Input {
                player_id: "P1".into(),
                kind: InputKind::Choice { idx: 0 },
            },
        })),
        RunOptions::default(),
    )
    .expect("game should complete");

    assert!(!gd.players[0].in_game, "P1 should be eliminated (bust)");
    assert!(gd.players[1].in_game, "P2 should still be in game");
}

#[test]
fn flow_compare_aggregate_true_branch_executes() {
    let ir = load_game("flow_compare_aggregate.cgdsl");
    let gd = run_game_with(ir, GameData::new(), default_input(), RunOptions::default())
        .expect("game should complete");

    assert_eq!(
        gd.players[0].score, 1,
        "P1 score should be 1 (if-true executed)"
    );
    assert_eq!(gd.players[1].score, 0, "P2 score unchanged");
}

#[test]
fn optional_multi_action_accept_fires_both_rules() {
    let ir = load_game("optional_multi_action.cgdsl");
    let gd = run_game_with(
        ir,
        GameData::new(),
        InputSource::Player(Box::new(|it: InputType| match it {
            InputType::Optional { .. } => Input {
                player_id: "P1".into(),
                kind: InputKind::OptionalAccept,
            },
            _ => Input {
                player_id: "P1".into(),
                kind: InputKind::Choice { idx: 0 },
            },
        })),
        RunOptions::default(),
    )
    .expect("game should complete");

    assert_eq!(gd.players[0].score, 5, "P1 score should be 5");
    assert_eq!(gd.players[1].score, 3, "P2 score should be 3");
}

#[test]
fn optional_multi_action_decline_fires_neither() {
    let ir = load_game("optional_multi_action.cgdsl");
    let gd = run_game_with(
        ir,
        GameData::new(),
        InputSource::Player(Box::new(|it: InputType| match it {
            InputType::Optional { .. } => Input {
                player_id: "P1".into(),
                kind: InputKind::OptionalDecline,
            },
            _ => Input {
                player_id: "P1".into(),
                kind: InputKind::Choice { idx: 0 },
            },
        })),
        RunOptions::default(),
    )
    .expect("game should complete");

    assert_eq!(gd.players[0].score, 0, "P1 score should be 0");
    assert_eq!(gd.players[1].score, 0, "P2 score should be 0");
}
