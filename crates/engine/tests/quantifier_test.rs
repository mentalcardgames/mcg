//! Integration tests for the quantifier preprocessor (see `crates/engine/src/quantifier.rs`).
//!
//! Each test loads a `.cgdsl` fixture, lowers it, and drives it through
//! `run_game` with an `InputSource::Player` closure that answers the
//! `ChoosePlayer` / `ChooseCards` prompts the preprocessor issues. Assertions
//! cover the resulting `GameData`, the trace, and the cleanup invariants
//! (synthetic memory removed; `self.ir` unchanged).

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use cgdsl_engine::{
    run_game, GameData, Input, InputKind, InputSource, InputType, Interpreter, Location,
    StepResult, TraceEntry, TraceEvent,
};
use front_end::ir::{Ir, LoweredPayLoad};
use front_end::validation::parse_document;

/// Load and lower a `test_games/<name>` fixture.
fn load_game(name: &str) -> Ir<LoweredPayLoad> {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let path = manifest.join("test_games").join(name);
    let src =
        std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {}", path.display(), e));
    let game = parse_document(&src).unwrap_or_else(|e| panic!("parse {}: {}", path.display(), e));
    game.to_lowered_graph()
}

/// Borrow a location of the given name owned by `player_idx`, if present.
fn player_location<'a>(
    gd: &'a GameData,
    player_idx: usize,
    loc_name: &str,
) -> Option<&'a Location> {
    gd.players.get(player_idx).and_then(|p| {
        p.owner
            .locations
            .iter()
            .find_map(|&li| gd.locations.get(li).filter(|l| l.name == loc_name))
    })
}

/// Borrow a table-owned location of the given name, if present.
fn table_location<'a>(gd: &'a GameData, loc_name: &str) -> Option<&'a Location> {
    gd.table
        .locations
        .iter()
        .find_map(|&li| gd.locations.get(li).filter(|l| l.name == loc_name))
}

/// Count `Action:Move` trace entries (the per-player / per-card dispatches).
fn move_traces(trace: &[TraceEntry]) -> usize {
    trace
        .iter()
        .filter(|e| {
            matches!(
                e,
                TraceEntry::Step {
                    event: TraceEvent::Action { subtype, .. },
                    ..
                } if subtype == "Action:Move"
            )
        })
        .count()
}

const SYNTH_KEY: &str = cgdsl_engine::quantifier::SYNTH_MEMORY_KEY;

/// `deal 1 from top(Stock) private to Hand of all` fans out to one dispatch
/// per resolved player; each player's `Hand` grows by exactly 1.
#[test]
fn quantifier_deal_all_fans_out_one_card_per_player() {
    let ir = load_game("quantifier_deal_all.cgdsl");
    let trace: Arc<Mutex<Vec<TraceEntry>>> = Arc::new(Mutex::new(Vec::new()));
    let trace_clone = trace.clone();

    let gd = run_game(
        ir,
        GameData::new(),
        InputSource::Player(Box::new(|_it: InputType| Input {
            player_id: "P1".into(),
            kind: InputKind::Choice { idx: 0 },
        })),
        None,
        Some(Box::new(move |e: TraceEntry| {
            trace_clone.lock().unwrap().push(e);
        })),
    )
    .expect("game should complete");

    assert_eq!(
        player_location(&gd, 0, "Hand").unwrap().cards.len(),
        1,
        "P1's Hand grew by 1"
    );
    assert_eq!(player_location(&gd, 1, "Hand").unwrap().cards.len(), 1);
    assert_eq!(player_location(&gd, 2, "Hand").unwrap().cards.len(), 1);
    assert_eq!(
        table_location(&gd, "Stock").unwrap().cards.len(),
        3,
        "Stock shrank by 3"
    );
    assert_eq!(
        move_traces(&trace.lock().unwrap()),
        3,
        "one Move dispatch per player"
    );
    assert!(
        !gd.memories.contains_key(&format!("Table_{}", SYNTH_KEY)),
        "synthetic memory slot removed"
    );
}

/// `deal any from Stock private to Hand of current` pauses with
/// `ChooseCards`; the selected card is moved to the current player's Hand.
#[test]
fn quantifier_deal_any_moves_chosen_card() {
    let ir = load_game("quantifier_deal_any.cgdsl");
    let trace: Arc<Mutex<Vec<TraceEntry>>> = Arc::new(Mutex::new(Vec::new()));
    let trace_clone = trace.clone();

    let gd = run_game(
        ir,
        GameData::new(),
        InputSource::Player(Box::new(|it: InputType| match it {
            InputType::ChooseCards { .. } => Input {
                player_id: "P1".into(),
                kind: InputKind::ChooseCards { selected: vec![0] },
            },
            _ => Input {
                player_id: "P1".into(),
                kind: InputKind::Choice { idx: 0 },
            },
        })),
        None,
        Some(Box::new(move |e: TraceEntry| {
            trace_clone.lock().unwrap().push(e);
        })),
    )
    .expect("game should complete");

    // current player is P1 (turn_order[0]); the chosen card (id 0) moved there.
    assert!(
        player_location(&gd, 0, "Hand").unwrap().cards.contains(&0),
        "chosen card in P1's Hand"
    );
    assert!(
        !table_location(&gd, "Stock").unwrap().cards.contains(&0),
        "chosen card left Stock"
    );
    assert_eq!(move_traces(&trace.lock().unwrap()), 1);
    assert!(!gd.memories.contains_key(&format!("Table_{}", SYNTH_KEY)));
}

/// `move >= 1 and <= 3 from Stock face down to Discard`: choosing 0 cards
/// (out of range) re-prompts, then choosing 2 moves exactly 2.
#[test]
fn quantifier_range_rejects_zero_then_moves_two() {
    let ir = load_game("quantifier_range.cgdsl");
    let calls = Arc::new(Mutex::new(0usize));
    let calls_clone = calls.clone();
    let trace: Arc<Mutex<Vec<TraceEntry>>> = Arc::new(Mutex::new(Vec::new()));
    let trace_clone = trace.clone();

    let gd = run_game(
        ir,
        GameData::new(),
        InputSource::Player(Box::new(move |it: InputType| {
            let mut c = calls_clone.lock().unwrap();
            *c += 1;
            let n = *c;
            drop(c);
            match it {
                InputType::ChooseCards { .. } => {
                    if n == 1 {
                        Input {
                            player_id: "P1".into(),
                            kind: InputKind::ChooseCards { selected: vec![] },
                        } // 0 cards: out of range
                    } else {
                        Input {
                            player_id: "P1".into(),
                            kind: InputKind::ChooseCards {
                                selected: vec![0, 1],
                            },
                        } // 2 cards: valid
                    }
                }
                _ => Input {
                    player_id: "P1".into(),
                    kind: InputKind::Choice { idx: 0 },
                },
            }
        })),
        None,
        Some(Box::new(move |e: TraceEntry| {
            trace_clone.lock().unwrap().push(e);
        })),
    )
    .expect("game should complete");

    assert_eq!(
        *calls.lock().unwrap(),
        2,
        "ChooseCards prompt issued twice (re-prompt then valid)"
    );
    assert_eq!(
        table_location(&gd, "Discard").unwrap().cards.len(),
        2,
        "2 cards moved to Discard"
    );
    assert_eq!(
        table_location(&gd, "Stock").unwrap().cards.len(),
        2,
        "Stock shrank by 2"
    );
    assert_eq!(move_traces(&trace.lock().unwrap()), 1);
    assert!(!gd.memories.contains_key(&format!("Table_{}", SYNTH_KEY)));
}

/// `deal 1 from top(Stock) private to Hand of any`: only the chosen player's
/// `Hand` grows.
#[test]
fn quantifier_dest_any_deals_to_chosen_player() {
    let ir = load_game("quantifier_dest_any.cgdsl");
    let trace: Arc<Mutex<Vec<TraceEntry>>> = Arc::new(Mutex::new(Vec::new()));
    let trace_clone = trace.clone();

    let gd = run_game(
        ir,
        GameData::new(),
        InputSource::Player(Box::new(|it: InputType| match it {
            InputType::ChoosePlayer { .. } => Input {
                player_id: "P1".into(),
                kind: InputKind::ChoosePlayer { idx: 1 },
            }, // P2
            _ => Input {
                player_id: "P1".into(),
                kind: InputKind::Choice { idx: 0 },
            },
        })),
        None,
        Some(Box::new(move |e: TraceEntry| {
            trace_clone.lock().unwrap().push(e);
        })),
    )
    .expect("game should complete");

    assert_eq!(
        player_location(&gd, 1, "Hand").unwrap().cards.len(),
        1,
        "P2's Hand grew"
    );
    assert_eq!(
        player_location(&gd, 0, "Hand").unwrap().cards.len(),
        0,
        "P1 unchanged"
    );
    assert_eq!(
        player_location(&gd, 2, "Hand").unwrap().cards.len(),
        0,
        "P3 unchanged"
    );
    assert_eq!(
        table_location(&gd, "Stock").unwrap().cards.len(),
        2,
        "Stock shrank by 1"
    );
    assert_eq!(move_traces(&trace.lock().unwrap()), 1);
    assert!(!gd.memories.contains_key(&format!("Table_{}", SYNTH_KEY)));
}

/// `deal any from Stock private to Hand of all`: exactly one `ChooseCards`
/// prompt, then a 3-player fan-out. The single chosen card is moved through
/// each player's Hand in turn (cards are single-instance), ending in exactly
/// one Hand.
#[test]
fn quantifier_all_then_any_single_prompt_then_fanout() {
    let ir = load_game("quantifier_all_then_any.cgdsl");
    let calls = Arc::new(Mutex::new(0usize));
    let calls_clone = calls.clone();
    let trace: Arc<Mutex<Vec<TraceEntry>>> = Arc::new(Mutex::new(Vec::new()));
    let trace_clone = trace.clone();

    let gd = run_game(
        ir,
        GameData::new(),
        InputSource::Player(Box::new(move |it: InputType| {
            if matches!(it, InputType::ChooseCards { .. }) {
                let mut c = calls_clone.lock().unwrap();
                *c += 1;
                Input {
                    player_id: "P1".into(),
                    kind: InputKind::ChooseCards { selected: vec![0] },
                }
            } else {
                Input {
                    player_id: "P1".into(),
                    kind: InputKind::Choice { idx: 0 },
                }
            }
        })),
        None,
        Some(Box::new(move |e: TraceEntry| {
            trace_clone.lock().unwrap().push(e);
        })),
    )
    .expect("game should complete");

    assert_eq!(*calls.lock().unwrap(), 1, "exactly one ChooseCards prompt");
    assert_eq!(
        move_traces(&trace.lock().unwrap()),
        3,
        "3 per-player fan-out dispatches"
    );
    let hands_with_card0 = (0..3)
        .filter(|&i| player_location(&gd, i, "Hand").unwrap().cards.contains(&0))
        .count();
    assert_eq!(
        hands_with_card0, 1,
        "chosen card ends in exactly one Hand (single-instance cards)"
    );
    assert!(
        !table_location(&gd, "Stock").unwrap().cards.contains(&0),
        "chosen card left Stock"
    );
    assert!(!gd.memories.contains_key(&format!("Table_{}", SYNTH_KEY)));
}

/// `self.ir` must be bit-for-bit unchanged after a quantifier run, and the
/// synthetic memory slot must be removed. Drives the interpreter manually
/// (rather than via `run_game`, which takes ownership of `ir`).
#[test]
fn quantifier_ir_not_mutated_and_memory_cleaned() {
    let ir = load_game("quantifier_deal_any.cgdsl");
    let ir_before = format!("{:?}", ir);
    let mut interp = Interpreter::new(ir, GameData::new(), None);

    loop {
        match interp.step() {
            StepResult::Ok => continue,
            StepResult::NeedsInput(it) => {
                let inp = match it {
                    InputType::ChooseCards { .. } => Input {
                        player_id: "P1".into(),
                        kind: InputKind::ChooseCards { selected: vec![0] },
                    },
                    InputType::ChoosePlayer { .. } => Input {
                        player_id: "P1".into(),
                        kind: InputKind::ChoosePlayer { idx: 0 },
                    },
                    _ => Input {
                        player_id: "P1".into(),
                        kind: InputKind::Choice { idx: 0 },
                    },
                };
                interp.provide_input(inp);
            }
            StepResult::GameOver => break,
            StepResult::Error(e) => panic!("step error: {e}"),
        }
    }

    let ir_after = format!("{:?}", interp.ir);
    assert_eq!(ir_before, ir_after, "self.ir must be bit-for-bit unchanged");
    assert!(
        !interp
            .game_data
            .memories
            .contains_key(&format!("Table_{}", SYNTH_KEY)),
        "synthetic memory slot removed after completion"
    );
}

#[test]
fn setup_location_all_creates_per_player_hands() {
    let ir = load_game("setup_location_all.cgdsl");
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
        player_location(&gd, 0, "Hand").is_some(),
        "P1 has a Hand location"
    );
    assert!(
        player_location(&gd, 1, "Hand").is_some(),
        "P2 has a Hand location"
    );
    assert!(
        player_location(&gd, 2, "Hand").is_some(),
        "P3 has a Hand location"
    );
}

#[test]
fn setup_location_literal_creates_per_player_hands() {
    let ir = load_game("setup_location_literal.cgdsl");
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
        player_location(&gd, 0, "Hand").is_some(),
        "P1 has a Hand location"
    );
    assert!(
        player_location(&gd, 1, "Hand").is_some(),
        "P2 has a Hand location"
    );
    assert!(
        player_location(&gd, 2, "Hand").is_some(),
        "P3 has a Hand location"
    );
}

#[test]
fn setup_location_any_returns_error() {
    let ir = load_game("setup_location_any_errors.cgdsl");
    let result = run_game(
        ir,
        GameData::new(),
        InputSource::Player(Box::new(|_it: InputType| Input {
            player_id: "P1".into(),
            kind: InputKind::Choice { idx: 0 },
        })),
        None,
        None,
    );
    match result {
        Err(e) => {
            assert!(
                e.to_string().contains("not supported in setup rules"),
                "error message should mention 'not supported in setup rules', got: {e}"
            );
        }
        Ok(gd) => {
            panic!(
                "expected error but got Ok(GameData). GameData has {:?} players, {:?} locations",
                gd.players.len(),
                gd.locations.len()
            );
        }
    }
}

#[test]
fn setup_turnorder_all_resolves_to_in_game_players() {
    let ir = load_game("setup_turnorder_all.cgdsl");
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
        gd.turn_order,
        vec![0, 1, 2],
        "turn_order should be all in-game players in declaration order"
    );
}

#[test]
fn setup_teams_all_resolves_all_players() {
    let ir = load_game("setup_teams_all.cgdsl");
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
        gd.teams[0].players.len(),
        3,
        "team T1 should have all 3 players"
    );
}
