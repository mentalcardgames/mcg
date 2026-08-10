use std::path::PathBuf;

use cgdsl_engine::{run_game_with, GameData, Input, InputKind, InputSource, InputType, RunOptions};
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
fn setup_create_combo_stores_entry() {
    let ir = load_game("setup_create_combo.cgdsl");
    let gd = run_game_with(
        ir,
        GameData::new(),
        InputSource::Player(Box::new(|_it: InputType| default_input())),
        RunOptions::default(),
    )
    .expect("game should complete");

    assert!(!gd.combos.is_empty(), "combo should be stored");
    assert_eq!(gd.combos[0].name, "TwoOfAKind");
}

#[test]
fn setup_create_precedence_stores_entry() {
    let ir = load_game("setup_create_precedence.cgdsl");
    let gd = run_game_with(
        ir,
        GameData::new(),
        InputSource::Player(Box::new(|_it: InputType| default_input())),
        RunOptions::default(),
    )
    .expect("game should complete");

    assert!(!gd.precedences.is_empty());
    assert_eq!(gd.precedences[0].key, "Rank");
}

#[test]
fn setup_create_pointmap_stores_entry() {
    let ir = load_game("setup_create_pointmap.cgdsl");
    let gd = run_game_with(
        ir,
        GameData::new(),
        InputSource::Player(Box::new(|_it: InputType| default_input())),
        RunOptions::default(),
    )
    .expect("game should complete");

    assert!(!gd.point_maps.is_empty());
    assert_eq!(gd.point_maps[0].name, "Values");
    // Verify actual point values from the DSL (Ace: 1, Two: 2, Three: 3)
    assert_eq!(gd.point_maps[0].map.get("Rank:Ace"), Some(&1));
    assert_eq!(gd.point_maps[0].map.get("Rank:Two"), Some(&2));
    assert_eq!(gd.point_maps[0].map.get("Rank:Three"), Some(&3));
}

#[test]
fn setup_create_memory_initializes_slot() {
    let ir = load_game("setup_create_memory.cgdsl");
    let gd = run_game_with(
        ir,
        GameData::new(),
        InputSource::Player(Box::new(|_it: InputType| default_input())),
        RunOptions::default(),
    )
    .expect("game should complete");

    use cgdsl_engine::game_data::MemoryValue;
    match gd.get_memory("Table_M") {
        Some(MemoryValue::Int(n)) => assert_eq!(*n, 0),
        other => panic!("expected Int(0), got {:?}", other),
    }
}
