//! Integration tests for the 2026-08-10 ergonomics pass: ineligible-player
//! skip + auto-end, honored memory initial values, the numeric `bid` prompt,
//! and team-owned locations/memories.

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use cgdsl_engine::game_data::MemoryValue;
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

fn total_cards(gd: &GameData) -> usize {
    gd.locations.iter().map(|l| l.cards.len()).sum()
}

/// A player that always accepts optionals, with a prompt counter. Answers
/// carry the current player's name (tracked via `event_sender`, I-23).
#[allow(clippy::type_complexity)]
fn accept_everything(prompts: Arc<Mutex<usize>>) -> (InputSource, Box<dyn Fn(&GameData) + Send>) {
    let current: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
    let sender_current = current.clone();
    let sender: Box<dyn Fn(&GameData) + Send> = Box::new(move |gd: &GameData| {
        *sender_current.lock().unwrap() = gd.get_current_player().map(|p| p.name.clone());
    });
    let source = InputSource::Player(Box::new(move |it: InputType| {
        let mut n = prompts.lock().unwrap();
        *n += 1;
        drop(n);
        let who = current
            .lock()
            .unwrap()
            .clone()
            .unwrap_or_else(|| "P1".to_string());
        match it {
            InputType::Optional { .. } => Input {
                player_id: who,
                kind: InputKind::OptionalAccept,
            },
            _ => Input {
                player_id: who,
                kind: InputKind::Choice { idx: 0 },
            },
        }
    }));
    (source, sender)
}

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
        InputSource::TestFile(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("test_games")
                .join("bid_prompt.txt"),
        ),
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
fn team_owned_locations_and_memories_are_per_member() {
    let ir = load_game("team_locations.cgdsl");
    let gd = run_game_with(
        ir,
        GameData::new(),
        InputSource::Player(Box::new(|_| Input {
            player_id: "P1".into(),
            kind: InputKind::Choice { idx: 0 },
        })),
        RunOptions::default(),
    )
    .expect("team-location game should complete");

    let pile_indices: Vec<usize> = gd
        .locations
        .iter()
        .enumerate()
        .filter(|(_, l)| l.name == "TeamPile")
        .map(|(i, _)| i)
        .collect();
    assert_eq!(pile_indices.len(), 2, "one TeamPile per Red member");
    for member in gd
        .players
        .iter()
        .filter(|p| p.name == "P1" || p.name == "P2")
    {
        assert!(
            member
                .owner
                .locations
                .iter()
                .any(|i| pile_indices.contains(i)),
            "{} owns a TeamPile",
            member.name
        );
    }
    assert!(
        gd.players[2].owner.locations.is_empty(),
        "P3 (not in Red) owns no TeamPile"
    );

    // `TeamFlag is 5` during P1's turn: declared per-member slots are
    // ambiguous (P1_TeamFlag, P2_TeamFlag), so the write bridges to the
    // current player's slot (D-14 fallback); P2's slot keeps its default.
    assert_eq!(gd.get_memory("P1_TeamFlag"), Some(&MemoryValue::Int(5)));
    assert_eq!(
        gd.get_memory("P2_TeamFlag"),
        Some(&MemoryValue::Int(0)),
        "P2's declared slot is untouched"
    );
}
