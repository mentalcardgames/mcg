use std::path::PathBuf;

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
fn out_of_game_eliminates_player() {
    let ir = load_game("out_of_game.cgdsl");
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
    assert!(gd.players[1].in_game, "P2 should still be in game");
    assert!(gd.players[2].in_game, "P3 should still be in game");
}

#[test]
fn out_of_stage_current_removes_from_stage() {
    let ir = load_game("out_of_stage_current.cgdsl");
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

    assert!(
        !gd.players[0].in_stage.get("Play").copied().unwrap_or(true),
        "P1 should be out of Play stage"
    );
    assert!(
        gd.players[1].in_stage.get("Play").copied().unwrap_or(false),
        "P2 should still be in Play stage"
    );
    assert!(
        gd.players[2].in_stage.get("Play").copied().unwrap_or(false),
        "P3 should still be in Play stage"
    );
}

#[test]
fn out_of_stage_named_removes_from_named_stage() {
    let ir = load_game("out_of_stage_named.cgdsl");
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

    assert!(
        !gd.players[0].in_stage.get("Play").copied().unwrap_or(true),
        "P1 should be out of Play stage"
    );
    assert!(
        gd.players[1].in_stage.get("Play").copied().unwrap_or(false),
        "P2 should still be in Play stage"
    );
    assert!(
        gd.players[2].in_stage.get("Play").copied().unwrap_or(false),
        "P3 should still be in Play stage"
    );
}

#[test]
fn out_runtime_current_eliminates_current_player() {
    let ir = load_game("out_runtime_current.cgdsl");
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

    assert!(!gd.players[0].in_game, "P1 (current) should be eliminated");
    assert!(gd.players[1].in_game, "P2 should still be in game");
    assert!(gd.players[2].in_game, "P3 should still be in game");
}
