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

fn default_input() -> Input {
    Input {
        player_id: "P1".into(),
        kind: InputKind::Choice { idx: 0 },
    }
}

#[test]
fn turn_end_turn_advances_current_player() {
    let ir = load_game("turn_end_turn.cgdsl");
    let gd = run_game(
        ir,
        GameData::new(),
        InputSource::Player(Box::new(|_it: InputType| default_input())),
        None,
        None,
    )
    .expect("game should complete");

    let current = gd.get_current_player().expect("should have current player");
    assert_eq!(current.name, "P2");
}

#[test]
fn turn_cycle_to_named_sets_player() {
    let ir = load_game("turn_cycle_to_named.cgdsl");
    let gd = run_game(
        ir,
        GameData::new(),
        InputSource::Player(Box::new(|_it: InputType| default_input())),
        None,
        None,
    )
    .expect("game should complete");

    let current = gd.get_current_player().expect("should have current player");
    assert_eq!(current.name, "P2");
}

#[test]
fn turn_end_stage_named_pops_stage_stack() {
    let ir = load_game("turn_end_stage_named.cgdsl");
    let gd = run_game(
        ir,
        GameData::new(),
        InputSource::Player(Box::new(|_it: InputType| default_input())),
        None,
        None,
    )
    .expect("game should complete");

    assert!(
        !gd.stage_stack.contains(&"Play".to_string()),
        "stage stack should not contain Play"
    );
}
