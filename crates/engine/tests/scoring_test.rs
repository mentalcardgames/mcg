//! Integration tests for scoring rules that span evaluator boundaries:
//! winner-extrema over memories (per-player slots) and turn-order position,
//! aggregate int expressions in score targets, and tie handling.
//! (Literal score/winner mechanics are unit-tested in `src/action_tests.rs`.)

mod common;

use cgdsl_engine::{run_game_with, GameData, Input, InputKind, InputSource, InputType, RunOptions};

use common::load_game;

#[test]
fn scoring_winner_with_tie_keeps_all_matching() {
    let ir = load_game("scoring_winner_with_tie.cgdsl");
    let gd = run_game_with(
        ir,
        GameData::new(),
        InputSource::Player(Box::new(|_it: InputType| Input {
            player_id: "P1".into(),
            kind: InputKind::Choice { idx: 0 },
        })),
        RunOptions::default(),
    )
    .expect("game should complete");

    assert_eq!(gd.players[0].score, 10);
    assert_eq!(gd.players[1].score, 10);
    assert_eq!(gd.players[2].score, 5);

    assert!(
        gd.players[0].in_game,
        "P1 (tied for max score 10) should be in game"
    );
    assert!(
        gd.players[1].in_game,
        "P2 (tied for max score 10) should be in game"
    );
    assert!(!gd.players[2].in_game, "P3 should be eliminated");
}

#[test]
fn scoring_winner_with_highest_memory_wins() {
    let ir = load_game("scoring_winner_with_memory.cgdsl");
    let gd = run_game_with(
        ir,
        GameData::new(),
        InputSource::Player(Box::new(|_it: InputType| Input {
            player_id: "P1".into(),
            kind: InputKind::Choice { idx: 0 },
        })),
        RunOptions::default(),
    )
    .expect("game should complete");

    assert!(gd.players[0].in_game, "P1 (M=10) should be in game");
    assert!(!gd.players[1].in_game, "P2 (M=5) should be eliminated");
    assert!(!gd.players[2].in_game, "P3 (M=3) should be eliminated");
}

#[test]
fn scoring_aggregate_int_to_current() {
    let ir = load_game("scoring_aggregate_int_to_current.cgdsl");
    let gd = run_game_with(
        ir,
        GameData::new(),
        InputSource::Player(Box::new(|_it: InputType| Input {
            player_id: "P1".into(),
            kind: InputKind::Choice { idx: 0 },
        })),
        RunOptions::default(),
    )
    .expect("game should complete");

    assert_eq!(gd.players[0].score, 21, "P1 score should be 11+10=21");
    assert_eq!(gd.players[1].score, 21, "P2 score should be 11+10=21");
}

#[test]
fn scoring_winner_with_highest_position_wins() {
    let ir = load_game("scoring_winner_with_position.cgdsl");
    let gd = run_game_with(
        ir,
        GameData::new(),
        InputSource::Player(Box::new(|_it: InputType| Input {
            player_id: "P1".into(),
            kind: InputKind::Choice { idx: 0 },
        })),
        RunOptions::default(),
    )
    .expect("game should complete");

    assert!(
        gd.players[2].in_game,
        "P3 (highest position) should be in game"
    );
    assert!(!gd.players[0].in_game, "P1 should be eliminated");
    assert!(!gd.players[1].in_game, "P2 should be eliminated");
}
