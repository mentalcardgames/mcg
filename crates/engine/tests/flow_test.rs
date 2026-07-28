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

fn default_input() -> InputSource {
    InputSource::Player(Box::new(|_it: InputType| Input {
        player_id: "P1".into(),
        kind: InputKind::Choice { idx: 0 },
    }))
}

#[test]
fn flow_if_true_executes_body() {
    let ir = load_game("flow_if_true.cgdsl");
    let gd =
        run_game(ir, GameData::new(), default_input(), None, None).expect("game should complete");

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
    let gd =
        run_game(ir, GameData::new(), default_input(), None, None).expect("game should complete");

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
    let gd = run_game(
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
        None,
        None,
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
fn test_cgdsl_parses_and_lowers() {
    let ir = load_game("test.cgdsl");
    assert!(!ir.states.is_empty(), "IR should have states");
    assert!(
        ir.states.contains_key(&ir.entry),
        "entry state should exist"
    );
}
