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

fn default_input() -> InputSource {
    InputSource::Player(Box::new(|_it: InputType| Input {
        player_id: "P1".into(),
        kind: InputKind::Choice { idx: 0 },
    }))
}

#[test]
fn memory_set_int_stores_value() {
    let ir = load_game("memory_set_int.cgdsl");
    let gd =
        run_game(ir, GameData::new(), default_input(), None, None).expect("game should complete");

    match gd.get_memory("P1_M") {
        Some(MemoryValue::Int(n)) => assert_eq!(*n, 42, "P1_M should be Int(42)"),
        other => panic!("expected Int(42), got {:?}", other),
    }
}

#[test]
fn memory_set_string_stores_value() {
    let ir = load_game("memory_set_string.cgdsl");
    let gd =
        run_game(ir, GameData::new(), default_input(), None, None).expect("game should complete");

    match gd.get_memory("P1_M") {
        Some(MemoryValue::String(s)) => assert_eq!(s, "Hello", "P1_M should be String(\"Hello\")"),
        other => panic!("expected String(\"Hello\"), got {:?}", other),
    }
}

#[test]
fn memory_reset_zeros_int() {
    let ir = load_game("memory_reset.cgdsl");
    let gd =
        run_game(ir, GameData::new(), default_input(), None, None).expect("game should complete");

    match gd.get_memory("P1_M") {
        Some(MemoryValue::Int(n)) => assert_eq!(*n, 0, "P1_M should be Int(0) after reset"),
        other => panic!("expected Int(0), got {:?}", other),
    }
}
