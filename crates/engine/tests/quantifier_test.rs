//! Integration tests for the quantifier preprocessor (see `crates/engine/src/quantifier.rs`).
//!
//! Each test loads a `.cgdsl` fixture, lowers it, and drives it through
//! `run_game` with an `InputSource::Player` closure that answers the
//! `ChoosePlayer` / `ChooseCards` prompts the preprocessor issues. Assertions
//! cover the resulting `GameData`, the trace, and the cleanup invariants
//! (synthetic memory removed; `self.ir` unchanged).

mod common;

use std::sync::{Arc, Mutex};

use cgdsl_engine::{
    run_game_with, GameData, Input, InputKind, InputSource, InputType, Interpreter, RunOptions,
    StepResult, TraceEntry,
};

use common::{load_game, move_traces, player_location, table_location};

const SYNTH_KEY: &str = cgdsl_engine::quantifier::SYNTH_MEMORY_KEY;

/// `deal 1 from top(Stock) private to Hand of all` fans out to one dispatch
/// per resolved player; each player's `Hand` grows by exactly 1.
#[test]
fn quantifier_deal_all_fans_out_one_card_per_player() {
    let ir = load_game("quantifier_deal_all.cgdsl");
    let trace: Arc<Mutex<Vec<TraceEntry>>> = Arc::new(Mutex::new(Vec::new()));
    let trace_clone = trace.clone();

    let gd = run_game_with(
        ir,
        GameData::new(),
        InputSource::Player(Box::new(|_it: InputType| Input {
            player_id: "P1".into(),
            kind: InputKind::Choice { idx: 0 },
        })),
        RunOptions::new().with_trace_sender(Box::new(move |e: TraceEntry| {
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

    let gd = run_game_with(
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
        RunOptions::new().with_trace_sender(Box::new(move |e: TraceEntry| {
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

    let gd = run_game_with(
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
        RunOptions::new().with_trace_sender(Box::new(move |e: TraceEntry| {
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

    let gd = run_game_with(
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
        RunOptions::new().with_trace_sender(Box::new(move |e: TraceEntry| {
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

/// `deal Hand where Rank is "Ace" of any …`: the *source* owner is `any` —
/// one `ChoosePlayer` prompt per ask, and the chosen player's Ace moves.
/// Asking a player without an Ace is a no-op (empty filtered set).
#[test]
fn quantifier_source_any_takes_from_chosen_player() {
    let ir = load_game("quantifier_source_any.cgdsl");
    let trace: Arc<Mutex<Vec<TraceEntry>>> = Arc::new(Mutex::new(Vec::new()));
    let trace_clone = trace.clone();
    // Ask 1: P1 (holds the Ace) -> Ace moves back to Stock. Ask 2: P2 (no
    // Ace) -> the filtered set is empty and the move is a no-op.
    let picks = Arc::new(Mutex::new(vec![0usize, 1]));
    let picks_clone = picks.clone();

    let gd = run_game_with(
        ir,
        GameData::new(),
        InputSource::Player(Box::new(move |it: InputType| match it {
            InputType::ChoosePlayer { .. } => {
                let mut p = picks_clone.lock().unwrap();
                let idx = p.remove(0);
                Input {
                    player_id: "P1".into(),
                    kind: InputKind::ChoosePlayer { idx },
                }
            }
            _ => Input {
                player_id: "P1".into(),
                kind: InputKind::Choice { idx: 0 },
            },
        })),
        RunOptions::new().with_trace_sender(Box::new(move |e: TraceEntry| {
            trace_clone.lock().unwrap().push(e);
        })),
    )
    .expect("game should complete");

    assert_eq!(
        player_location(&gd, 0, "Hand").unwrap().cards.len(),
        0,
        "P1's Ace moved out"
    );
    assert_eq!(
        player_location(&gd, 1, "Hand").unwrap().cards.len(),
        1,
        "P2 unchanged (miss)"
    );
    assert_eq!(
        player_location(&gd, 2, "Hand").unwrap().cards.len(),
        1,
        "P3 unchanged"
    );
    assert_eq!(
        table_location(&gd, "Stock").unwrap().cards.len(),
        1,
        "only the Ace returned to Stock"
    );
    assert_eq!(
        move_traces(&trace.lock().unwrap()),
        5,
        "3 setup deals + both asks dispatch as move edges (the miss is an empty-set no-op)"
    );
    assert!(!gd.memories.contains_key(&format!("Table_{}", SYNTH_KEY)));
}

/// `deal any from Hand of any …`: TWO sequential prompts — first the source
/// player (`ChoosePlayer`), then the cards (`ChooseCards`) — on a single
/// edge. The player choice must be resolved before the card choice, since a
/// multi-player owner cannot be evaluated.
#[test]
fn quantifier_chains_source_player_then_cards() {
    let ir = load_game("quantifier_source_any_cards.cgdsl");
    let prompts = Arc::new(Mutex::new(Vec::new()));
    let prompts_clone = prompts.clone();

    let gd = run_game_with(
        ir,
        GameData::new(),
        InputSource::Player(Box::new(move |it: InputType| {
            prompts_clone.lock().unwrap().push(it.clone());
            match it {
                InputType::ChoosePlayer { .. } => Input {
                    player_id: "P1".into(),
                    kind: InputKind::ChoosePlayer { idx: 0 }, // P1 (holds the Ace)
                },
                InputType::ChooseCards { .. } => Input {
                    player_id: "P1".into(),
                    kind: InputKind::ChooseCards { selected: vec![0] },
                },
                _ => Input {
                    player_id: "P1".into(),
                    kind: InputKind::Choice { idx: 0 },
                },
            }
        })),
        RunOptions::default(),
    )
    .expect("game should complete");

    let kinds: Vec<&str> = prompts
        .lock()
        .unwrap()
        .iter()
        .map(|it| match it {
            InputType::ChoosePlayer { .. } => "player",
            InputType::ChooseCards { .. } => "cards",
            _ => "other",
        })
        .collect();
    assert_eq!(
        kinds,
        vec!["player", "cards"],
        "source player must be prompted before the cards"
    );
    assert_eq!(
        player_location(&gd, 0, "Hand").unwrap().cards.len(),
        0,
        "P1's Ace moved out"
    );
    assert_eq!(
        player_location(&gd, 1, "Hand").unwrap().cards.len(),
        1,
        "P2 untouched"
    );
    assert_eq!(
        table_location(&gd, "Discard").unwrap().cards.len(),
        1,
        "exactly one card moved"
    );
}

/// `deal any from Deck private to Hand of any`: TWO sequential prompts —
/// destination player first, then the card choice (previously the card
/// choice was silently dropped after the player substitution).
#[test]
fn quantifier_chains_dest_player_then_cards() {
    let ir = load_game("quantifier_dest_any_cards.cgdsl");
    let prompts = Arc::new(Mutex::new(Vec::new()));
    let prompts_clone = prompts.clone();

    let gd = run_game_with(
        ir,
        GameData::new(),
        InputSource::Player(Box::new(move |it: InputType| {
            prompts_clone.lock().unwrap().push(it.clone());
            match it {
                InputType::ChoosePlayer { .. } => Input {
                    player_id: "P1".into(),
                    kind: InputKind::ChoosePlayer { idx: 1 }, // P2
                },
                InputType::ChooseCards { .. } => Input {
                    player_id: "P1".into(),
                    kind: InputKind::ChooseCards { selected: vec![0] },
                },
                _ => Input {
                    player_id: "P1".into(),
                    kind: InputKind::Choice { idx: 0 },
                },
            }
        })),
        RunOptions::default(),
    )
    .expect("game should complete");

    let kinds: Vec<&str> = prompts
        .lock()
        .unwrap()
        .iter()
        .map(|it| match it {
            InputType::ChoosePlayer { .. } => "player",
            InputType::ChooseCards { .. } => "cards",
            _ => "other",
        })
        .collect();
    assert_eq!(
        kinds,
        vec!["player", "cards"],
        "destination player must be prompted before the cards"
    );
    assert_eq!(
        player_location(&gd, 1, "Hand").unwrap().cards.len(),
        1,
        "P2 received the chosen card"
    );
    assert_eq!(
        player_location(&gd, 0, "Hand").unwrap().cards.len(),
        1,
        "P1 keeps the setup card"
    );
    assert_eq!(
        table_location(&gd, "Deck").unwrap().cards.len(),
        0,
        "the deck is drained"
    );
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

    let gd = run_game_with(
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
        RunOptions::new().with_trace_sender(Box::new(move |e: TraceEntry| {
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
