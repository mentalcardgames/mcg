//! Integration tests for `action.rs` arms, driven through `run_game` against
//! `.cgdsl` fixtures. See `crates/engine/docs/testing.md` §3.3.

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use cgdsl_engine::{
    run_game_with, GameData, Input, InputKind, InputSource, InputType, RunOptions, TraceEntry,
    TraceEvent,
};
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

fn always_choice_0() -> InputSource {
    InputSource::Player(Box::new(|_it: InputType| Input {
        player_id: "P1".into(),
        kind: InputKind::Choice { idx: 0 },
    }))
}

/// `move top(Stock) private to Hand` where Hand exists: card moves.
#[test]
fn move_top_card_to_hand_succeeds() {
    let ir = load_game("action_move_top_to_hand.cgdsl");
    let gd = run_game_with(
        ir,
        GameData::new(),
        always_choice_0(),
        RunOptions::default(),
    )
    .expect("game should complete");
    let stock = gd
        .locations
        .iter()
        .find(|l| l.name == "Stock")
        .expect("Stock exists");
    let hand = gd
        .locations
        .iter()
        .find(|l| l.name == "Hand")
        .expect("Hand exists");
    assert_eq!(stock.cards.len(), 0, "Stock drained by 1");
    assert_eq!(hand.cards.len(), 1, "Hand received 1 card");
}

/// I-5 regression: the `StageRoundCounter` payload must increment the
/// stage counter exactly once per traversal (not twice, as the pre-fix
/// double-application did). The current code applies it in
/// `Interpreter::step` and no-ops in `action::execute` via the `_ => {}`
/// catch-all (see `crates/engine/src/action.rs:53-57`).
#[test]
fn stage_round_counter_incremented_exactly_once_per_traversal() {
    let ir = load_game("turn_switch.cgdsl");
    let trace: Arc<Mutex<Vec<TraceEntry>>> = Arc::new(Mutex::new(Vec::new()));
    let trace_clone = trace.clone();
    let gd = run_game_with(
        ir,
        GameData::new(),
        always_choice_0(),
        RunOptions::new().with_trace_sender(Box::new(move |e: TraceEntry| {
            trace_clone.lock().unwrap().push(e);
        })),
    )
    .expect("game should complete");

    let play_rounds = trace
        .lock()
        .unwrap()
        .iter()
        .filter(|e| {
            matches!(
                e,
                TraceEntry::Step {
                    event: TraceEvent::StageRoundCounter { stage, .. },
                    ..
                } if stage.as_str() == "Play"
            )
        })
        .count();
    assert_eq!(play_rounds, 2, "turn_switch.cgdsl runs 2 Play rounds");

    // The stage counter for "Play" must equal the number of StageRoundCounter
    // traversals (2), NOT 4 (which would indicate double-application).
    assert_eq!(
        gd.stage_counters.get("Play"),
        Some(&2),
        "I-5: stage counter incremented once per traversal, not twice"
    );
}

/// I-13 relaxed (2026-08-10): `cycle to next` with no eligible player at all
/// is now a no-op, and the stage auto-ends when no players are in the game
/// — the game completes with an empty winner set instead of erroring.
#[test]
fn cycle_to_next_with_no_eligible_player_auto_ends() {
    let ir = load_game("errors_cycle_no_next.cgdsl");
    let gd = run_game_with(
        ir,
        GameData::new(),
        always_choice_0(),
        RunOptions::default(),
    )
    .expect("cycle to next with nobody eligible must no-op and auto-end");
    assert!(
        gd.players.iter().all(|p| !p.in_game),
        "all players out => empty winner set"
    );
}

/// 2026-08-10: an ineligible (or missing) current player's instructions are
/// skipped entirely — `Counter is 5` after the last player was eliminated
/// never runs, and the stage auto-ends. (`SetMemory` still errors without a
/// current player when reached at top level.)
#[test]
fn set_memory_without_current_player_is_skipped_and_auto_ends() {
    let ir = load_game("errors_set_memory_no_current.cgdsl");
    let gd = run_game_with(
        ir,
        GameData::new(),
        always_choice_0(),
        RunOptions::default(),
    )
    .expect("skipped SetMemory must not error");
    assert!(
        gd.players.iter().all(|p| !p.in_game),
        "empty winner set after auto-end"
    );
}

/// D-11 regression (2026-08): a move into an empty `where`-set resolves to
/// the base location ("Second"), not the location-0 sentinel ("First").
#[test]
fn empty_where_set_destination_uses_base_location() {
    let ir = load_game("fix_empty_where_dest.cgdsl");
    let gd = run_game_with(
        ir,
        GameData::new(),
        always_choice_0(),
        RunOptions::default(),
    )
    .expect("game should complete");
    let first = gd.locations.iter().find(|l| l.name == "First").unwrap();
    let second = gd.locations.iter().find(|l| l.name == "Second").unwrap();
    assert_eq!(first.cards.len(), 0, "location 0 must NOT receive the card");
    assert_eq!(second.cards.len(), 1, "base location receives the card");
}

/// D-5 regression (2026-08): `same Rank` combos match only the paired cards.
#[test]
fn combo_same_rank_matches_only_pairs() {
    let ir = load_game("fix_combo_same_rank.cgdsl");
    let gd = run_game_with(
        ir,
        GameData::new(),
        always_choice_0(),
        RunOptions::default(),
    )
    .expect("game should complete");
    assert_eq!(gd.players[0].score, 2, "pair = 2 cards, not 3");
}
