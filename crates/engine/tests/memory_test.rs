mod common;

use cgdsl_engine::game_data::MemoryValue;
use cgdsl_engine::{run_game_with, GameData, RunOptions};

use common::{default_input, load_game};

#[test]
fn memory_set_int_stores_value() {
    let ir = load_game("memory_set_int.cgdsl");
    let gd = run_game_with(ir, GameData::new(), default_input(), RunOptions::default())
        .expect("game should complete");

    // D-14 fixed (2026-08-10): `memory M on table` + `M is 42` targets the
    // DECLARED slot (Table_M), not the current player's P1_M.
    match gd.get_memory("Table_M") {
        Some(MemoryValue::Int(n)) => assert_eq!(*n, 42, "Table_M should be Int(42)"),
        other => panic!("expected Int(42), got {:?}", other),
    }
    assert!(gd.get_memory("P1_M").is_none(), "no stray P1_M slot");
}

#[test]
fn memory_set_string_stores_value() {
    let ir = load_game("memory_set_string.cgdsl");
    let gd = run_game_with(ir, GameData::new(), default_input(), RunOptions::default())
        .expect("game should complete");

    match gd.get_memory("Table_M") {
        Some(MemoryValue::String(s)) => {
            assert_eq!(s, "Hello", "Table_M should be String(\"Hello\")")
        }
        other => panic!("expected String(\"Hello\"), got {:?}", other),
    }
}

#[test]
fn memory_reset_zeros_int() {
    let ir = load_game("memory_reset.cgdsl");
    let gd = run_game_with(ir, GameData::new(), default_input(), RunOptions::default())
        .expect("game should complete");

    match gd.get_memory("Table_M") {
        Some(MemoryValue::Int(n)) => assert_eq!(*n, 0, "Table_M should be Int(0) after reset"),
        other => panic!("expected Int(0), got {:?}", other),
    }
}

#[test]
fn memory_set_then_read_back_via_evaluator() {
    let ir = load_game("memory_read_back.cgdsl");
    let gd = run_game_with(ir, GameData::new(), default_input(), RunOptions::default())
        .expect("game should complete");

    assert_eq!(gd.players[0].score, 5, "P1 score should be 5");
}
