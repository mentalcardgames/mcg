mod common;

use cgdsl_engine::{run_game_with, GameData, Input, InputKind, InputSource, InputType, RunOptions};

use common::{default_input, load_game};

#[test]
fn flow_if_true_executes_body() {
    let ir = load_game("flow_if_true.cgdsl");
    let gd = run_game_with(ir, GameData::new(), default_input(), RunOptions::default())
        .expect("game should complete");

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
    let gd = run_game_with(ir, GameData::new(), default_input(), RunOptions::default())
        .expect("game should complete");

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
    let gd = run_game_with(
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
        RunOptions::default(),
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
fn flow_optional_bust_eliminates_on_over_limit() {
    let ir = load_game("flow_optional_bust.cgdsl");
    let gd = run_game_with(
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
        RunOptions::default(),
    )
    .expect("game should complete");

    assert!(!gd.players[0].in_game, "P1 should be eliminated (bust)");
    assert!(gd.players[1].in_game, "P2 should still be in game");
}

#[test]
fn flow_compare_aggregate_true_branch_executes() {
    let ir = load_game("flow_compare_aggregate.cgdsl");
    let gd = run_game_with(ir, GameData::new(), default_input(), RunOptions::default())
        .expect("game should complete");

    assert_eq!(
        gd.players[0].score, 1,
        "P1 score should be 1 (if-true executed)"
    );
    assert_eq!(gd.players[1].score, 0, "P2 score unchanged");
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

#[test]
fn blackjack_runs_end_to_end() {
    let ir = load_game("blackjack.cgdsl");
    // P1 hits (accept optional), P2 stands (decline), P3 hits
    let calls = std::sync::Arc::new(std::sync::Mutex::new(0usize));
    let calls_clone = calls.clone();
    // Track the current player via the event_sender so every answer carries
    // the right player_id (validation I-23 rejects mismatched ids and would
    // otherwise re-prompt forever).
    let current: std::sync::Arc<std::sync::Mutex<Option<String>>> =
        std::sync::Arc::new(std::sync::Mutex::new(None));
    let current_writer = current.clone();
    let gd = run_game_with(
        ir,
        GameData::new(),
        InputSource::Player(Box::new(move |it: InputType| {
            let mut count = calls_clone.lock().unwrap();
            *count += 1;
            let n = *count;
            drop(count);
            let who = current.lock().unwrap().clone();
            match it {
                InputType::Optional { .. } => {
                    // P1 accepts (call 1), P2 declines (call 2), P3 accepts (call 3)
                    if n == 2 {
                        Input {
                            player_id: who.unwrap_or_else(|| "P2".into()),
                            kind: InputKind::OptionalDecline,
                        }
                    } else {
                        Input {
                            player_id: who.unwrap_or_else(|| "P1".into()),
                            kind: InputKind::OptionalAccept,
                        }
                    }
                }
                _ => Input {
                    player_id: who.unwrap_or_else(|| "P1".into()),
                    kind: InputKind::Choice { idx: 0 },
                },
            }
        })),
        RunOptions::new().with_event_sender(Box::new(move |gd: &GameData| {
            *current_writer.lock().unwrap() = gd.get_current_player().map(|p| p.name.clone());
        })),
    )
    .expect("blackjack should complete");

    // At least one player or the dealer should have cards
    let total_cards: usize = gd.locations.iter().map(|l| l.cards.len()).sum();
    assert!(total_cards > 0, "cards should be dealt");
    // The game completed without error — that's the property we verify
}
