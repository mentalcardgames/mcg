//! Integration tests for elimination semantics: ineligible-player skip and
//! stage auto-end (I-24), cycle/`next` eligibility (I-13), and the winner
//! set at game end (I-25).

mod common;

use std::sync::{Arc, Mutex};

use cgdsl_engine::game_data::MemoryValue;
use cgdsl_engine::{
    run_game_with, GameData, Input, InputKind, InputSource, RunOptions, TraceEntry, TraceEvent,
};

use common::{accept_everything, load_game, total_cards};

#[test]
fn eliminated_players_are_never_prompted_and_game_auto_ends() {
    let ir = load_game("skip_ineligible.cgdsl");
    let prompts = Arc::new(Mutex::new(0usize));
    let (input, sender) = accept_everything(prompts.clone());
    let gd = run_game_with(
        ir,
        GameData::new(),
        input,
        RunOptions::new().with_event_sender(sender),
    )
    .expect("skip-mode game should complete");

    assert_eq!(
        total_cards(&gd),
        2,
        "both drawn cards are conserved in the players' hands"
    );
    assert!(
        gd.players.iter().all(|p| !p.in_game),
        "every player who accepted was eliminated => empty winner set"
    );
    assert_eq!(
        gd.players[0].score, 0,
        "the post-elimination score line was skipped"
    );
    assert_eq!(
        *prompts.lock().unwrap(),
        2,
        "the optional was offered exactly twice (P1, P2) — never to an out-of-game player, \
         and the 10-iteration cap was never reached"
    );
}

#[test]
fn eliminated_players_instructions_are_traced_as_skipped() {
    let ir = load_game("skip_ineligible.cgdsl");
    let trace: Arc<Mutex<Vec<TraceEntry>>> = Arc::new(Mutex::new(Vec::new()));
    let trace_clone = trace.clone();
    let (input, sender) = accept_everything(Arc::new(Mutex::new(0usize)));
    let gd = run_game_with(
        ir,
        GameData::new(),
        input,
        RunOptions::new()
            .with_event_sender(sender)
            .with_trace_sender(Box::new(move |e: TraceEntry| {
                trace_clone.lock().unwrap().push(e);
            })),
    )
    .expect("game should complete");

    let skipped = trace
        .lock()
        .unwrap()
        .iter()
        .filter(|e| {
            matches!(
                e,
                TraceEntry::Step {
                    event: TraceEvent::Skipped { .. },
                    ..
                }
            )
        })
        .count();
    assert!(
        skipped >= 2,
        "the post-elimination score line is skipped on both turns: got {skipped}"
    );
    assert!(gd.players.iter().all(|p| !p.in_game));
    assert_eq!(gd.players[0].score, 0, "skipped score never fired");
}

#[test]
fn out_of_game_players_are_skipped_by_cycles_and_next_expressions() {
    // Regression: a player out of the GAME but still in the current stage
    // must never become current via `cycle to next` / `cycle to previous`,
    // and must be skipped by the `next` expression. (The old `previous`
    // ignored eligibility entirely — D-12, fixed 2026-08-10; the forward
    // path always required `in_game && in_stage`.)
    let ir = load_game("cycle_skips_out_of_game.cgdsl");
    let current: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let current_clone = current.clone();
    let gd = run_game_with(
        ir,
        GameData::new(),
        InputSource::Player(Box::new(|_| Input {
            player_id: "P1".into(),
            kind: InputKind::Choice { idx: 0 },
        })),
        RunOptions::new().with_event_sender(Box::new(move |gd: &GameData| {
            if let Some(p) = gd.get_current_player() {
                current_clone.lock().unwrap().push(p.name.clone());
            }
        })),
    )
    .expect("game should complete");

    assert_eq!(
        gd.get_memory("Table_WhoNext"),
        Some(&MemoryValue::String("P3".to_string())),
        "`next` from P1 must skip the out-of-game P2 and resolve to P3"
    );
    let seen = current.lock().unwrap();
    assert!(
        !seen.iter().any(|n| n == "P2"),
        "P2 (out of game, still in stage) must never become current; saw {:?}",
        *seen
    );
    // Collapse consecutive snapshots (the event sender fires every step):
    // Stage A: P1. Stage B (cycle to next): P1 -> P3 -> P1.
    // Stage C (cycle to previous): P1 -> P3 -> P1.
    let mut runs: Vec<String> = Vec::new();
    for name in seen.iter() {
        if runs.last().map(|l| l != name).unwrap_or(true) {
            runs.push(name.clone());
        }
    }
    assert_eq!(runs, vec!["P1", "P3", "P1", "P3", "P1"]);
}

#[test]
fn game_over_trace_names_the_winner_set() {
    // No explicit winner statement: winners = players left in game.
    let ir = load_game("winner_set_remaining.cgdsl");
    let trace: Arc<Mutex<Vec<TraceEntry>>> = Arc::new(Mutex::new(Vec::new()));
    let trace_clone = trace.clone();
    let gd = run_game_with(
        ir,
        GameData::new(),
        InputSource::Player(Box::new(|_| Input {
            player_id: "P1".into(),
            kind: InputKind::Choice { idx: 0 },
        })),
        RunOptions::new().with_trace_sender(Box::new(move |e: TraceEntry| {
            trace_clone.lock().unwrap().push(e);
        })),
    )
    .expect("game should complete");

    let winners = trace
        .lock()
        .unwrap()
        .iter()
        .find_map(|e| match e {
            TraceEntry::Step {
                event: TraceEvent::GameOver { winners },
                ..
            } => Some(winners.clone()),
            _ => None,
        })
        .expect("a GameOver trace event must be emitted");
    assert_eq!(winners, vec!["P1".to_string(), "P3".to_string()]);
    assert_eq!(gd.winner_names(), winners, "GameData agrees with the trace");
}

#[test]
fn game_over_trace_names_declared_winner() {
    // `end game with winner P:P1` eliminates everyone else (2026-08-10) —
    // the survivor is the winner set.
    let ir = load_game("winner_set_declared.cgdsl");
    let trace: Arc<Mutex<Vec<TraceEntry>>> = Arc::new(Mutex::new(Vec::new()));
    let trace_clone = trace.clone();
    let gd = run_game_with(
        ir,
        GameData::new(),
        InputSource::Player(Box::new(|_| Input {
            player_id: "P1".into(),
            kind: InputKind::Choice { idx: 0 },
        })),
        RunOptions::new().with_trace_sender(Box::new(move |e: TraceEntry| {
            trace_clone.lock().unwrap().push(e);
        })),
    )
    .expect("game should complete");

    let winners = trace
        .lock()
        .unwrap()
        .iter()
        .find_map(|e| match e {
            TraceEntry::Step {
                event: TraceEvent::GameOver { winners },
                ..
            } => Some(winners.clone()),
            _ => None,
        })
        .expect("a GameOver trace event must be emitted");
    assert_eq!(winners, vec!["P1".to_string()]);
    assert!(gd.players.iter().all(|p| p.in_game == (p.name == "P1")));
}
