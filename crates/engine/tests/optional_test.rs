mod common;

use cgdsl_engine::{run_game_with, GameData, Input, InputKind, InputSource, InputType, RunOptions};

use common::load_game;

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
