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
fn optional_multi_action_accept_fires_both_rules() {
    let ir = load_game("optional_multi_action.cgdsl");
    let gd = run_game(
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
        None,
        None,
    )
    .expect("game should complete");

    assert_eq!(gd.players[0].score, 5, "P1 score should be 5");
    assert_eq!(gd.players[1].score, 3, "P2 score should be 3");
}

#[test]
fn optional_multi_action_decline_fires_neither() {
    let ir = load_game("optional_multi_action.cgdsl");
    let gd = run_game(
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
        None,
        None,
    )
    .expect("game should complete");

    assert_eq!(gd.players[0].score, 0, "P1 score should be 0");
    assert_eq!(gd.players[1].score, 0, "P2 score should be 0");
}
