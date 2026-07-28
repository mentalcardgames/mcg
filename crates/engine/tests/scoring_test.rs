use std::path::PathBuf;

use cgdsl_engine::game_data::MemoryValue;
use cgdsl_engine::{run_game, GameData, Input, InputKind, InputSource, InputType};
use front_end::ir::{Ir, LoweredPayLoad};
use front_end::validation::parse_document;

fn load_game(name: &str) -> Ir<LoweredPayLoad> {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let path = manifest.join("test_games").join(name);
    let src =
        std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {}", path.display(), e));
    let game = parse_document(&src).unwrap_or_else(|e| panic!("parse {}: {}", path.display(), e));
    game.to_lowered_graph()
}

#[test]
fn scoring_score_literal_adds_to_player_score() {
    let ir = load_game("scoring_score_literal_to_player.cgdsl");
    let gd = run_game(
        ir,
        GameData::new(),
        InputSource::Player(Box::new(|_it: InputType| Input {
            player_id: "P1".into(),
            kind: InputKind::Choice { idx: 0 },
        })),
        None,
        None,
    )
    .expect("game should complete");

    assert_eq!(gd.players[0].score, 10, "P1 score should be 10");
    assert_eq!(gd.players[1].score, 0, "P2 score unchanged");
    assert_eq!(gd.players[2].score, 0, "P3 score unchanged");
}

#[test]
fn scoring_score_int_expr_resolves_correctly() {
    let ir = load_game("scoring_score_int_expr_to_player.cgdsl");
    let gd = run_game(
        ir,
        GameData::new(),
        InputSource::Player(Box::new(|_it: InputType| Input {
            player_id: "P1".into(),
            kind: InputKind::Choice { idx: 0 },
        })),
        None,
        None,
    )
    .expect("game should complete");

    assert_eq!(
        gd.players[0].score, 5,
        "P1 score should equal memory value 5"
    );
}

#[test]
fn scoring_score_binary_adds_to_all_players() {
    let ir = load_game("scoring_score_binary_to_all.cgdsl");
    let gd = run_game(
        ir,
        GameData::new(),
        InputSource::Player(Box::new(|_it: InputType| Input {
            player_id: "P1".into(),
            kind: InputKind::Choice { idx: 0 },
        })),
        None,
        None,
    )
    .expect("game should complete");

    assert_eq!(gd.players[0].score, 3, "P1 score should be 1+2=3");
    assert_eq!(gd.players[1].score, 3, "P2 score should be 1+2=3");
    assert_eq!(gd.players[2].score, 3, "P3 score should be 1+2=3");
}

#[test]
fn scoring_score_memory_writes_to_global_slot() {
    let ir = load_game("scoring_score_memory_write.cgdsl");
    let gd = run_game(
        ir,
        GameData::new(),
        InputSource::Player(Box::new(|_it: InputType| Input {
            player_id: "P1".into(),
            kind: InputKind::Choice { idx: 0 },
        })),
        None,
        None,
    )
    .expect("game should complete");

    match gd.get_memory("P1_M") {
        Some(MemoryValue::Int(n)) => assert_eq!(*n, 5),
        other => panic!("expected Int(5), got {:?}", other),
    }

    assert_eq!(
        gd.players[0].score, 0,
        "P1 score unchanged (ScoreMemory does not affect scores)"
    );
    assert_eq!(gd.players[1].score, 0, "P2 score unchanged");
    assert_eq!(gd.players[2].score, 0, "P3 score unchanged");
}

#[test]
fn scoring_winner_explicit_single_eliminates_others() {
    let ir = load_game("scoring_winner_explicit_single.cgdsl");
    let gd = run_game(
        ir,
        GameData::new(),
        InputSource::Player(Box::new(|_it: InputType| Input {
            player_id: "P1".into(),
            kind: InputKind::Choice { idx: 0 },
        })),
        None,
        None,
    )
    .expect("game should complete");

    assert!(gd.players[0].in_game, "P1 should be in game");
    assert!(!gd.players[1].in_game, "P2 should be eliminated");
    assert!(!gd.players[2].in_game, "P3 should be eliminated");
}

#[test]
fn scoring_winner_explicit_multiple_keeps_named_players() {
    let ir = load_game("scoring_winner_explicit_multiple.cgdsl");
    let gd = run_game(
        ir,
        GameData::new(),
        InputSource::Player(Box::new(|_it: InputType| Input {
            player_id: "P1".into(),
            kind: InputKind::Choice { idx: 0 },
        })),
        None,
        None,
    )
    .expect("game should complete");

    assert!(gd.players[0].in_game, "P1 should be in game");
    assert!(gd.players[1].in_game, "P2 should be in game");
    assert!(!gd.players[2].in_game, "P3 should be eliminated");
}

#[test]
fn scoring_winner_with_max_score_eliminates_lower() {
    let ir = load_game("scoring_winner_with_max_score.cgdsl");
    let gd = run_game(
        ir,
        GameData::new(),
        InputSource::Player(Box::new(|_it: InputType| Input {
            player_id: "P1".into(),
            kind: InputKind::Choice { idx: 0 },
        })),
        None,
        None,
    )
    .expect("game should complete");

    assert_eq!(gd.players[0].score, 10);
    assert_eq!(gd.players[1].score, 5);
    assert_eq!(gd.players[2].score, 3);

    assert!(
        gd.players[0].in_game,
        "P1 (highest score 10) should be in game"
    );
    assert!(!gd.players[1].in_game, "P2 should be eliminated");
    assert!(!gd.players[2].in_game, "P3 should be eliminated");
}

#[test]
fn scoring_winner_with_min_score_eliminates_higher() {
    let ir = load_game("scoring_winner_with_min_score.cgdsl");
    let gd = run_game(
        ir,
        GameData::new(),
        InputSource::Player(Box::new(|_it: InputType| Input {
            player_id: "P1".into(),
            kind: InputKind::Choice { idx: 0 },
        })),
        None,
        None,
    )
    .expect("game should complete");

    assert!(!gd.players[0].in_game, "P1 should be eliminated");
    assert!(!gd.players[1].in_game, "P2 should be eliminated");
    assert!(
        gd.players[2].in_game,
        "P3 (lowest score 3) should be in game"
    );
}

#[test]
fn scoring_winner_with_tie_keeps_all_matching() {
    let ir = load_game("scoring_winner_with_tie.cgdsl");
    let gd = run_game(
        ir,
        GameData::new(),
        InputSource::Player(Box::new(|_it: InputType| Input {
            player_id: "P1".into(),
            kind: InputKind::Choice { idx: 0 },
        })),
        None,
        None,
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
    let gd = run_game(
        ir,
        GameData::new(),
        InputSource::Player(Box::new(|_it: InputType| Input {
            player_id: "P1".into(),
            kind: InputKind::Choice { idx: 0 },
        })),
        None,
        None,
    )
    .expect("game should complete");

    assert!(gd.players[0].in_game, "P1 (M=10) should be in game");
    assert!(!gd.players[1].in_game, "P2 (M=5) should be eliminated");
    assert!(!gd.players[2].in_game, "P3 (M=3) should be eliminated");
}
