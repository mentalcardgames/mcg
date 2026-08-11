//! Integration tests for memory semantics: declarations honour their typed
//! initial values, and the numeric `bid` prompt writes the owner's slot.
//! (The write primitives themselves — `M is X` / `reset M` — are unit-tested
//! in `src/action_tests.rs`; the write→read composition lives in
//! `tests/actions_test.rs`.)

mod common;

use cgdsl_engine::game_data::MemoryValue;
use cgdsl_engine::{run_game_with, GameData, Input, InputKind, InputSource, RunOptions};

use common::{load_game, test_file};

#[test]
fn memory_declarations_honor_initial_values() {
    let ir = load_game("memory_initial_value.cgdsl");
    let gd = run_game_with(
        ir,
        GameData::new(),
        InputSource::Player(Box::new(|_| Input {
            player_id: "P1".into(),
            kind: InputKind::Choice { idx: 0 },
        })),
        RunOptions::default(),
    )
    .expect("game should complete");

    assert_eq!(gd.get_memory("Table_Pot"), Some(&MemoryValue::Int(100)));
    assert_eq!(
        gd.get_memory("Table_Name"),
        Some(&MemoryValue::String("Alice".to_string()))
    );
    assert_eq!(
        gd.get_memory("Table_Winner"),
        Some(&MemoryValue::String("P1".to_string())),
        "Player-typed memory on table initialises to the evaluated player"
    );
    assert_eq!(gd.players[0].score, 100, "the initial value was readable");
}

#[test]
fn bid_any_prompts_for_a_number_and_range_rejects_out_of_bounds() {
    let ir = load_game("bid_prompt.cgdsl");
    let gd = run_game_with(
        ir,
        GameData::new(),
        test_file("bid_prompt.txt"),
        RunOptions::default(),
    )
    .expect("bid prompts should complete");

    assert_eq!(gd.get_memory("Table_Pot"), Some(&MemoryValue::Int(7)));
    assert_eq!(
        gd.get_memory("Table_Bet"),
        Some(&MemoryValue::Int(3)),
        "the out-of-range 99 was rejected and re-asked"
    );
}
