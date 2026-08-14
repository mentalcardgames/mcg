//! Unit tests for `action::execute_*`. These build `ActionRule` / `SetUpRule`
//! payloads directly and call the `execute_*` functions, asserting on the
//! resulting `GameData` mutation. End-to-end `.cgdsl` fixture tests live in
//! `tests/action_test.rs`.

use super::*;
use crate::game_data::{GameData, Location, MemoryValue};
use front_end::ast::{
    ActionRule, CardSet, ClassicMove, DealMove, EndType, ExchangeMove, Extrema, Group, Groupable,
    IntExpr, MemoryType, MoveCardSet, MoveType, OutOf, Owner, PlayerExpr, Players, Quantity,
    ScoringRule, SetUpRule, Status, UseMemory, UseSingleMemory, WinnerType,
};
use std::collections::HashMap;

fn loc_cardset(name: &str) -> CardSet {
    CardSet::Group {
        group: Group::Groupable {
            groupable: Groupable::Location {
                name: name.to_string(),
            },
        },
    }
}

/// Build a small `GameData` with one Stock pile holding 3 cards and an empty
/// Hand owned by Alice. Returns `(gd, stock_idx, hand_idx, card_ids)`.
fn fixture_stock_to_hand() -> (GameData, usize, usize, Vec<usize>) {
    let mut gd = GameData::new();
    let _alice = gd.add_player("Alice".to_string());
    let stock_idx = gd.add_location(
        "Table".to_string(),
        Location {
            name: "Stock".to_string(),
            cards: vec![],
        },
    );
    let hand_idx = gd.add_location(
        "Alice".to_string(),
        Location {
            name: "Hand".to_string(),
            cards: vec![],
        },
    );
    let mut card_ids = vec![];
    for i in 0..3 {
        let id = gd.add_card(
            stock_idx,
            HashMap::from([("name".to_string(), format!("C{}", i))]),
        );
        gd.locations[stock_idx].cards.push(id);
        card_ids.push(id);
    }
    (gd, stock_idx, hand_idx, card_ids)
}

/// Comparable snapshot of `locations` (which doesn't `derive(PartialEq/Debug)`).
fn loc_snapshot(gd: &GameData) -> Vec<(String, Vec<usize>)> {
    gd.locations
        .iter()
        .map(|l| (l.name.clone(), l.cards.clone()))
        .collect()
}

/// Comparable snapshot of `players` for assertion purposes.
fn player_snapshot(gd: &GameData) -> Vec<(String, i32, bool)> {
    gd.players
        .iter()
        .map(|p| (p.name.clone(), p.score, p.in_game))
        .collect()
}

#[test]
fn bid_action_without_target_errors() {
    let mut gd = GameData::new();
    let before_players = player_snapshot(&gd);
    let before_memories = gd.memories.clone();
    // 2026-08-10: a bare `bid <qty>` (no memory target) is an error, not a
    // silent no-op (D-7) — use `bid <qty> on <memory> of <owner>`.
    let result = execute_action_rule(
        ActionRule::BidAction {
            quantitiy: Quantity::Int {
                int: IntExpr::Literal { int: 1 },
            },
        },
        &mut gd,
    );
    assert!(
        matches!(result, Err(EngineError::BidWithoutMemoryTarget { .. })),
        "bare bid must error with a clear message"
    );
    assert_eq!(player_snapshot(&gd), before_players);
    assert_eq!(gd.memories, before_memories);
}

#[test]
fn bid_memory_action_writes_literal_to_owner_slot() {
    // 2026-08-10: `bid <qty> on <memory> of <owner>` = "store the number in
    // the owner's memory slot". Literal quantities write directly; `any`/
    // ranges are prompted by the interpreter before dispatch.
    let mut gd = GameData::new();
    execute_action_rule(
        ActionRule::BidMemoryAction {
            memory: "bid".to_string(),
            quantity: Quantity::Int {
                int: IntExpr::Literal { int: 7 },
            },
            owner: Owner::Table,
        },
        &mut gd,
    )
    .unwrap();
    assert_eq!(
        gd.get_memory("Table_bid"),
        Some(&crate::game_data::MemoryValue::Int(7)),
        "bid writes the number into the owner's memory slot"
    );
}

#[test]
fn end_action_game_with_winner_eliminates_non_winners() {
    // 2026-08-10: `end game with winner X` eliminates everyone not named
    // (the IR jump to the goal then ends the game) — the in-game survivors
    // ARE the winner set.
    let mut gd = GameData::new();
    let p0 = gd.add_player("Alice".to_string());
    let p1 = gd.add_player("Bob".to_string());
    execute_action_rule(
        ActionRule::EndAction {
            end_type: EndType::GameWithWinner {
                players: Players::Player {
                    player: PlayerExpr::Literal {
                        name: "Alice".to_string(),
                    },
                },
            },
        },
        &mut gd,
    )
    .unwrap();
    assert!(gd.players[p0].in_game, "Alice is the declared winner");
    assert!(!gd.players[p1].in_game, "Bob is eliminated");
    assert_eq!(gd.winner_names(), vec!["Alice".to_string()]);
}

#[test]
fn scoring_rule_score_adds_to_player_score() {
    let mut gd = GameData::new();
    let p0 = gd.add_player("Alice".to_string());
    assert_eq!(gd.players[p0].score, 0);
    execute_scoring_rule(
        ScoringRule::ScoreRule {
            score_rule: front_end::ast::ScoreRule::Score {
                int: IntExpr::Literal { int: 10 },
                players: Players::Player {
                    player: PlayerExpr::Literal {
                        name: "Alice".to_string(),
                    },
                },
            },
        },
        &mut gd,
    )
    .unwrap();
    assert_eq!(gd.players[p0].score, 10, "Score should add 10 to Alice");
}

#[test]
fn scoring_rule_score_memory_writes_to_memory_slot() {
    let mut gd = GameData::new();
    gd.add_player("Alice".to_string());
    execute_scoring_rule(
        ScoringRule::ScoreRule {
            score_rule: front_end::ast::ScoreRule::ScoreMemory {
                int: IntExpr::Literal { int: 10 },
                memory: "m".to_string(),
                players: Players::Player {
                    player: PlayerExpr::Literal {
                        name: "Alice".to_string(),
                    },
                },
            },
        },
        &mut gd,
    )
    .unwrap();
    match gd.get_memory("Alice_m") {
        Some(crate::game_data::MemoryValue::Int(n)) => assert_eq!(*n, 10),
        other => panic!("expected Int(10), got {:?}", other),
    }
}

#[test]
fn scoring_rule_winner_eliminates_non_winners() {
    let mut gd = GameData::new();
    let p0 = gd.add_player("Alice".to_string());
    let p1 = gd.add_player("Bob".to_string());
    gd.turn_order = vec![p0, p1];
    assert!(gd.players[p0].in_game);
    assert!(gd.players[p1].in_game);
    execute_scoring_rule(
        ScoringRule::WinnerRule {
            winner_rule: front_end::ast::WinnerRule::Winner {
                players: Players::Player {
                    player: PlayerExpr::Literal {
                        name: "Alice".to_string(),
                    },
                },
            },
        },
        &mut gd,
    )
    .unwrap();
    assert!(gd.players[p0].in_game, "Alice should still be in game");
    assert!(!gd.players[p1].in_game, "Bob should be eliminated");
}

#[test]
fn scoring_rule_winner_with_eliminates_lowest_score() {
    let mut gd = GameData::new();
    let p0 = gd.add_player("Alice".to_string());
    let p1 = gd.add_player("Bob".to_string());
    gd.turn_order = vec![p0, p1];
    gd.players[p0].score = 10;
    gd.players[p1].score = 5;
    assert!(gd.players[p0].in_game);
    assert!(gd.players[p1].in_game);
    execute_scoring_rule(
        ScoringRule::WinnerRule {
            winner_rule: front_end::ast::WinnerRule::WinnerWith {
                extrema: Extrema::Max,
                winner_type: WinnerType::Score,
            },
        },
        &mut gd,
    )
    .unwrap();
    assert!(
        gd.players[p0].in_game,
        "Alice (score=10) should win on highest score"
    );
    assert!(
        !gd.players[p1].in_game,
        "Bob (score=5) should be eliminated"
    );
}

#[test]
fn scoring_rule_winner_with_min_eliminates_highest_score() {
    let mut gd = GameData::new();
    let p0 = gd.add_player("Alice".to_string());
    let p1 = gd.add_player("Bob".to_string());
    gd.turn_order = vec![p0, p1];
    gd.players[p0].score = 10;
    gd.players[p1].score = 5;
    assert!(gd.players[p0].in_game);
    assert!(gd.players[p1].in_game);
    execute_scoring_rule(
        ScoringRule::WinnerRule {
            winner_rule: front_end::ast::WinnerRule::WinnerWith {
                extrema: Extrema::Min,
                winner_type: WinnerType::Score,
            },
        },
        &mut gd,
    )
    .unwrap();
    assert!(
        !gd.players[p0].in_game,
        "Alice (score=10) should be eliminated"
    );
    assert!(
        gd.players[p1].in_game,
        "Bob (score=5) should win on lowest score"
    );
}

// ---------------------------------------------------------------------------
// Task 3 — OutAction (all 5 OutOf variants)
// ---------------------------------------------------------------------------

fn two_players_in_play_stage() -> GameData {
    let mut gd = GameData::new();
    let p0 = gd.add_player("Alice".to_string());
    let p1 = gd.add_player("Bob".to_string());
    gd.turn_order = vec![p0, p1];
    gd.enter_stage(
        "Play".to_string(),
        vec!["Alice".to_string(), "Bob".to_string()],
    );
    gd
}

#[test]
fn out_action_current_stage_clears_in_stage_flag() {
    let mut gd = two_players_in_play_stage();
    execute_action_rule(
        ActionRule::OutAction {
            players: Players::Player {
                player: PlayerExpr::Literal {
                    name: "Alice".to_string(),
                },
            },
            out_of: OutOf::CurrentStage,
        },
        &mut gd,
    )
    .unwrap();
    assert_eq!(
        gd.players[0].in_stage.get("Play"),
        Some(&false),
        "Alice out of current stage"
    );
    assert!(gd.players[0].in_game, "Alice still in game");
    assert_eq!(
        gd.players[1].in_stage.get("Play"),
        Some(&true),
        "Bob unaffected"
    );
}

#[test]
fn out_action_named_stage_clears_named_flag() {
    let mut gd = two_players_in_play_stage();
    gd.enter_stage(
        "Sub".to_string(),
        vec!["Alice".to_string(), "Bob".to_string()],
    );
    execute_action_rule(
        ActionRule::OutAction {
            players: Players::Player {
                player: PlayerExpr::Literal {
                    name: "Alice".to_string(),
                },
            },
            out_of: OutOf::Stage {
                name: "Sub".to_string(),
            },
        },
        &mut gd,
    )
    .unwrap();
    assert_eq!(gd.players[0].in_stage.get("Sub"), Some(&false));
    assert_eq!(
        gd.players[0].in_stage.get("Play"),
        Some(&true),
        "Play flag untouched"
    );
}

#[test]
fn out_action_game_sets_in_game_false() {
    let mut gd = two_players_in_play_stage();
    execute_action_rule(
        ActionRule::OutAction {
            players: Players::Player {
                player: PlayerExpr::Literal {
                    name: "Alice".to_string(),
                },
            },
            out_of: OutOf::Game,
        },
        &mut gd,
    )
    .unwrap();
    assert!(!gd.players[0].in_game, "Alice out of game");
    assert!(gd.players[1].in_game, "Bob still in game");
}

#[test]
fn out_action_game_successful_sets_in_game_false() {
    let mut gd = two_players_in_play_stage();
    execute_action_rule(
        ActionRule::OutAction {
            players: Players::Player {
                player: PlayerExpr::Literal {
                    name: "Alice".to_string(),
                },
            },
            out_of: OutOf::GameSuccessful,
        },
        &mut gd,
    )
    .unwrap();
    assert!(!gd.players[0].in_game);
}

#[test]
fn out_action_game_fail_sets_in_game_false() {
    let mut gd = two_players_in_play_stage();
    execute_action_rule(
        ActionRule::OutAction {
            players: Players::Player {
                player: PlayerExpr::Literal {
                    name: "Alice".to_string(),
                },
            },
            out_of: OutOf::GameFail,
        },
        &mut gd,
    )
    .unwrap();
    assert!(!gd.players[0].in_game);
}

// ---------------------------------------------------------------------------
// Task 4 — EndAction variants
// ---------------------------------------------------------------------------

#[test]
fn end_action_turn_advances_current_player() {
    let mut gd = two_players_in_play_stage();
    gd.current_player = Some(0);
    execute_action_rule(
        ActionRule::EndAction {
            end_type: EndType::Turn,
        },
        &mut gd,
    )
    .unwrap();
    assert_eq!(
        gd.current_player,
        Some(1),
        "EndAction::Turn calls next_player"
    );
}

#[test]
fn end_action_current_stage_leaves_stage() {
    let mut gd = two_players_in_play_stage();
    execute_action_rule(
        ActionRule::EndAction {
            end_type: EndType::CurrentStage,
        },
        &mut gd,
    )
    .unwrap();
    assert!(
        gd.stage_stack.is_empty(),
        "EndAction::CurrentStage pops the current stage"
    );
}

#[test]
fn end_action_named_stage_leaves_named_stage() {
    let mut gd = two_players_in_play_stage();
    gd.enter_stage(
        "Sub".to_string(),
        vec!["Alice".to_string(), "Bob".to_string()],
    );
    assert_eq!(gd.stage_stack.len(), 2);
    execute_action_rule(
        ActionRule::EndAction {
            end_type: EndType::Stage {
                stage: "Play".to_string(),
            },
        },
        &mut gd,
    )
    .unwrap();
    // leave_stage("Play") pops Sub and Play
    assert!(gd.stage_stack.is_empty());
}

// ---------------------------------------------------------------------------
// Task 5 — CycleAction + panic sites
// ---------------------------------------------------------------------------

#[test]
fn cycle_action_sets_current_player_to_named_player() {
    let mut gd = two_players_in_play_stage();
    gd.turn_order = vec![0, 1]; // Alice=0, Bob=1
    gd.current_player = Some(0);
    execute_action_rule(
        ActionRule::CycleAction {
            player: PlayerExpr::Literal {
                name: "Bob".to_string(),
            },
        },
        &mut gd,
    )
    .unwrap();
    assert_eq!(
        gd.current_player,
        Some(1),
        "CycleAction sets current_player to Bob's turn-order slot"
    );
}

#[test]
fn cycle_action_eval_failure_errors() {
    // Fallible since 2026-08: eval failures surface as Err, not panics.
    // The bare memory ref now resolves to the current player's slot
    // (D-14, 2026-08-10), so the failure is "memory not found" instead of
    // "requires an explicit owner" — still a recoverable error.
    let mut gd = two_players_in_play_stage();
    let expr = PlayerExpr::Memory {
        memory: UseSingleMemory::Memory {
            memory: "nonexistent".to_string(),
        },
    };
    let result = execute_action_rule(ActionRule::CycleAction { player: expr }, &mut gd);
    assert_eq!(result.unwrap_err().to_string(),
            "CycleAction: failed to eval player Memory { memory: Memory { memory: \"nonexistent\" } }: Memory Alice_nonexistent not found"
                .to_string()
        );
}

#[test]
fn cycle_action_unknown_player_errors() {
    let mut gd = two_players_in_play_stage();
    let result = execute_action_rule(
        ActionRule::CycleAction {
            player: PlayerExpr::Literal {
                name: "Ghost".to_string(),
            },
        },
        &mut gd,
    );
    assert_eq!(
        result.unwrap_err().to_string(),
        "CycleAction: player Ghost not found in game_data.players".to_string()
    );
}

#[test]
fn cycle_action_player_not_in_turn_order_errors() {
    let mut gd = GameData::new();
    let p0 = gd.add_player("Alice".to_string());
    let _p1 = gd.add_player("Bob".to_string());
    gd.turn_order = vec![p0]; // Bob is in players but NOT in turn_order
    let result = execute_action_rule(
        ActionRule::CycleAction {
            player: PlayerExpr::Literal {
                name: "Bob".to_string(),
            },
        },
        &mut gd,
    );
    assert_eq!(
        result.unwrap_err().to_string(),
        "CycleAction: player_idx 1 not in turn_order [0]".to_string()
    );
}

// ---------------------------------------------------------------------------
// Task 6 — ShuffleAction success & failure paths
// ---------------------------------------------------------------------------

#[test]
fn shuffle_action_preserves_card_set_membership() {
    let mut gd = GameData::new();
    let _p = gd.add_player("Alice".to_string());
    let stock = gd.add_location(
        "Table".to_string(),
        Location {
            name: "Stock".to_string(),
            cards: vec![],
        },
    );
    let mut ids = vec![];
    for i in 0..5 {
        let id = gd.add_card(
            stock,
            HashMap::from([("name".to_string(), format!("C{}", i))]),
        );
        gd.locations[stock].cards.push(id);
        ids.push(id);
    }
    ids.sort();

    execute_action_rule(
        ActionRule::ShuffleAction {
            card_set: loc_cardset("Stock"),
        },
        &mut gd,
    )
    .unwrap();

    let mut after = gd.locations[stock].cards.clone();
    after.sort();
    assert_eq!(after, ids, "shuffle preserves the set of card ids in Stock");
    assert_eq!(gd.locations[stock].cards.len(), 5);
}

#[test]
fn shuffle_action_on_missing_location_errors() {
    // ShuffleAction now surfaces eval failures as Err (recoverable) instead
    // of printing to stderr and continuing.
    let mut gd = GameData::new();
    let before = loc_snapshot(&gd);
    let result = execute_action_rule(
        ActionRule::ShuffleAction {
            card_set: loc_cardset("Ghost"),
        },
        &mut gd,
    );
    assert!(result.is_err(), "missing location must be an error");
    assert_eq!(
        loc_snapshot(&gd),
        before,
        "no locations mutated on eval failure"
    );
}

// ---------------------------------------------------------------------------
// Task 7 — SetMemory / ResetMemory
// ---------------------------------------------------------------------------

#[test]
fn set_memory_action_stores_evaluated_int() {
    let mut gd = GameData::new();
    let _alice = gd.add_player("Alice".to_string());
    gd.turn_order.push(_alice);
    gd.current_player = Some(0);
    execute_action_rule(
        ActionRule::SetMemory {
            memory: "counter".to_string(),
            memory_type: MemoryType::Int {
                int: IntExpr::Literal { int: 42 },
            },
        },
        &mut gd,
    )
    .unwrap();
    assert_eq!(gd.get_memory("Alice_counter"), Some(&MemoryValue::Int(42)));
}

#[test]
fn set_memory_action_stores_evaluated_string() {
    let mut gd = GameData::new();
    let _alice = gd.add_player("Alice".to_string());
    gd.turn_order.push(_alice);
    gd.current_player = Some(0);
    execute_action_rule(
        ActionRule::SetMemory {
            memory: "label".to_string(),
            memory_type: MemoryType::String {
                string: front_end::ast::StringExpr::Literal {
                    value: "Hello".to_string(),
                },
            },
        },
        &mut gd,
    )
    .unwrap();
    assert_eq!(
        gd.get_memory("Alice_label"),
        Some(&MemoryValue::String("Hello".to_string()))
    );
}

#[test]
fn set_memory_action_stores_evaluated_int_collection() {
    let mut gd = GameData::new();
    let _alice = gd.add_player("Alice".to_string());
    gd.turn_order.push(_alice);
    gd.current_player = Some(0);
    execute_action_rule(
        ActionRule::SetMemory {
            memory: "nums".to_string(),
            memory_type: MemoryType::IntCollection {
                ints: front_end::ast::IntCollection::Literal {
                    ints: vec![
                        IntExpr::Literal { int: 1 },
                        IntExpr::Literal { int: 2 },
                        IntExpr::Literal { int: 3 },
                    ],
                },
            },
        },
        &mut gd,
    )
    .unwrap();
    assert_eq!(
        gd.get_memory("Alice_nums"),
        Some(&MemoryValue::IntCollection(vec![1, 2, 3]))
    );
}

#[test]
fn set_memory_action_stores_evaluated_string_collection() {
    let mut gd = GameData::new();
    let _alice = gd.add_player("Alice".to_string());
    gd.turn_order.push(_alice);
    gd.current_player = Some(0);
    execute_action_rule(
        ActionRule::SetMemory {
            memory: "labels".to_string(),
            memory_type: MemoryType::StringCollection {
                strings: front_end::ast::StringCollection::Literal {
                    strings: vec![
                        front_end::ast::StringExpr::Literal {
                            value: "a".to_string(),
                        },
                        front_end::ast::StringExpr::Literal {
                            value: "b".to_string(),
                        },
                    ],
                },
            },
        },
        &mut gd,
    )
    .unwrap();
    assert_eq!(
        gd.get_memory("Alice_labels"),
        Some(&MemoryValue::StringCollection(vec![
            "a".to_string(),
            "b".to_string()
        ]))
    );
}

#[test]
fn set_memory_action_stores_evaluated_player_collection() {
    let mut gd = GameData::new();
    let p0 = gd.add_player("Alice".to_string());
    let p1 = gd.add_player("Bob".to_string());
    gd.turn_order = vec![p0, p1];
    gd.current_player = Some(0);
    execute_action_rule(
        ActionRule::SetMemory {
            memory: "team".to_string(),
            memory_type: MemoryType::PlayerCollection {
                players: front_end::ast::PlayerCollection::Literal {
                    players: vec![
                        PlayerExpr::Literal {
                            name: "Alice".to_string(),
                        },
                        PlayerExpr::Literal {
                            name: "Bob".to_string(),
                        },
                    ],
                },
            },
        },
        &mut gd,
    )
    .unwrap();
    assert_eq!(
        gd.get_memory("Alice_team"),
        Some(&MemoryValue::PlayerCollection(vec![0, 1]))
    );
}

#[test]
fn set_memory_action_stores_evaluated_team_collection() {
    let mut gd = GameData::new();
    let p0 = gd.add_player("Alice".to_string());
    gd.turn_order.push(p0);
    gd.current_player = Some(0);
    gd.teams.push(crate::game_data::Team {
        name: "Red".to_string(),
        players: vec![p0],
        owner: crate::game_data::OwnerData { locations: vec![] },
    });
    execute_action_rule(
        ActionRule::SetMemory {
            memory: "colors".to_string(),
            memory_type: MemoryType::TeamCollection {
                teams: front_end::ast::TeamCollection::Literal {
                    teams: vec![front_end::ast::TeamExpr::Literal {
                        name: "Red".to_string(),
                    }],
                },
            },
        },
        &mut gd,
    )
    .unwrap();
    assert_eq!(
        gd.get_memory("Alice_colors"),
        Some(&MemoryValue::TeamCollection(vec!["Red".to_string()]))
    );
}

#[test]
fn set_memory_action_stores_evaluated_location_collection() {
    let mut gd = GameData::new();
    let p0 = gd.add_player("Alice".to_string());
    gd.turn_order.push(p0);
    gd.current_player = Some(0);
    let stock = gd.add_location(
        "Table".to_string(),
        Location {
            name: "Stock".to_string(),
            cards: vec![],
        },
    );
    execute_action_rule(
        ActionRule::SetMemory {
            memory: "piles".to_string(),
            memory_type: MemoryType::LocationCollection {
                locations: front_end::ast::LocationCollection::Literal {
                    locations: vec!["Stock".to_string()],
                },
            },
        },
        &mut gd,
    )
    .unwrap();
    assert_eq!(
        gd.get_memory("Alice_piles"),
        Some(&MemoryValue::LocationCollection(vec![stock]))
    );
}

#[test]
fn set_memory_action_stores_evaluated_card_set() {
    let mut gd = GameData::new();
    let p0 = gd.add_player("Alice".to_string());
    gd.turn_order.push(p0);
    gd.current_player = Some(0);
    let stock = gd.add_location(
        "Table".to_string(),
        Location {
            name: "Stock".to_string(),
            cards: vec![],
        },
    );
    let c0 = gd.add_card(
        stock,
        HashMap::from([("Rank".to_string(), "Ace".to_string())]),
    );
    let c1 = gd.add_card(
        stock,
        HashMap::from([("Rank".to_string(), "King".to_string())]),
    );
    gd.locations[stock].cards = vec![c0, c1];
    execute_action_rule(
        ActionRule::SetMemory {
            memory: "cards".to_string(),
            memory_type: MemoryType::CardSet {
                card_set: loc_cardset("Stock"),
            },
        },
        &mut gd,
    )
    .unwrap();
    assert_eq!(
        gd.get_memory("Alice_cards"),
        Some(&MemoryValue::CardSet(vec![c0, c1]))
    );
}

#[test]
fn reset_memory_action_zeros_int() {
    let mut gd = GameData::new();
    let _alice = gd.add_player("Alice".to_string());
    gd.turn_order.push(_alice);
    gd.current_player = Some(0);
    gd.set_memory("Alice_counter".to_string(), MemoryValue::Int(7));
    execute_action_rule(
        ActionRule::ResetMemory {
            memory: "counter".to_string(),
        },
        &mut gd,
    )
    .unwrap();
    assert_eq!(gd.get_memory("Alice_counter"), Some(&MemoryValue::Int(0)));
}

// ---------------------------------------------------------------------------
// Task 8 — MoveType Deal / Exchange / Classic, Move vs MoveQuantity
// ---------------------------------------------------------------------------

#[test]
fn move_classic_moves_all_cards_when_no_quantity() {
    let (mut gd, stock, hand, _ids) = fixture_stock_to_hand();
    execute_move(
        MoveType::Classic {
            classic: ClassicMove::MoveCardSet {
                move_cs: MoveCardSet::Move {
                    from: loc_cardset("Stock"),
                    status: Status::Private,
                    to: loc_cardset("Hand"),
                },
            },
        },
        &mut gd,
    )
    .unwrap();
    assert_eq!(gd.locations[stock].cards.len(), 0, "Stock emptied");
    assert_eq!(gd.locations[hand].cards.len(), 3, "Hand got all 3");
}

#[test]
fn move_classic_move_quantity_moves_only_n_cards() {
    let (mut gd, stock, hand, _ids) = fixture_stock_to_hand();
    execute_move(
        MoveType::Classic {
            classic: ClassicMove::MoveCardSet {
                move_cs: MoveCardSet::MoveQuantity {
                    quantity: Quantity::Int {
                        int: IntExpr::Literal { int: 2 },
                    },
                    from: loc_cardset("Stock"),
                    status: Status::Private,
                    to: loc_cardset("Hand"),
                },
            },
        },
        &mut gd,
    )
    .unwrap();
    assert_eq!(gd.locations[stock].cards.len(), 1, "1 left in Stock");
    assert_eq!(gd.locations[hand].cards.len(), 2, "2 moved to Hand");
}

#[test]
fn move_deal_routes_to_execute_cardset_move() {
    let (mut gd, stock, hand, _ids) = fixture_stock_to_hand();
    execute_move(
        MoveType::Deal {
            deal: DealMove::MoveCardSet {
                deal_cs: MoveCardSet::Move {
                    from: loc_cardset("Stock"),
                    status: Status::Private,
                    to: loc_cardset("Hand"),
                },
            },
        },
        &mut gd,
    )
    .unwrap();
    assert_eq!(gd.locations[stock].cards.len(), 0);
    assert_eq!(gd.locations[hand].cards.len(), 3);
}

#[test]
fn move_exchange_routes_to_execute_cardset_move() {
    let (mut gd, stock, hand, _ids) = fixture_stock_to_hand();
    execute_move(
        MoveType::Exchange {
            exchange: ExchangeMove::MoveCardSet {
                exchange_cs: MoveCardSet::Move {
                    from: loc_cardset("Stock"),
                    status: Status::Private,
                    to: loc_cardset("Hand"),
                },
            },
        },
        &mut gd,
    )
    .unwrap();
    assert_eq!(gd.locations[stock].cards.len(), 0);
    assert_eq!(gd.locations[hand].cards.len(), 3);
}

// ---------------------------------------------------------------------------
// Task 9 — execute_cardset_move panic sites
// ---------------------------------------------------------------------------

#[test]
fn execute_cardset_move_from_eval_failure_errors() {
    let mut gd = GameData::new();
    let _p = gd.add_player("Alice".to_string());
    let _hand = gd.add_location(
        "Alice".to_string(),
        Location {
            name: "Hand".to_string(),
            cards: vec![],
        },
    );
    let from = CardSet::Memory {
        memory: UseMemory::Memory {
            memory: "ghost".to_string(),
        },
    };
    let to = loc_cardset("Hand");
    let result = execute_cardset_move(from, None, Status::Private, to, &mut gd);
    assert_eq!(result.unwrap_err().to_string(), "execute_cardset_move: failed to eval from cardset Memory { memory: Memory { memory: \"ghost\" } }: memory access requires an explicit owner; use &M:ghost of <owner>".to_string());
}

#[test]
fn execute_cardset_move_to_eval_failure_errors() {
    let mut gd = GameData::new();
    let _p = gd.add_player("Alice".to_string());
    let stock = gd.add_location(
        "Table".to_string(),
        Location {
            name: "Stock".to_string(),
            cards: vec![],
        },
    );
    let card_id = gd.add_card(stock, HashMap::new());
    gd.locations[stock].cards.push(card_id);
    let from = loc_cardset("Stock");
    let to = CardSet::Memory {
        memory: UseMemory::Memory {
            memory: "ghost".to_string(),
        },
    };
    let result = execute_cardset_move(from, None, Status::Private, to, &mut gd);
    assert_eq!(result.unwrap_err().to_string(), "execute_cardset_move: failed to eval dest cardset Memory { memory: Memory { memory: \"ghost\" } }: memory access requires an explicit owner; use &M:ghost of <owner>".to_string());
}

// ---------------------------------------------------------------------------
// Task 10 — SetUpRule panic sites
// ---------------------------------------------------------------------------

#[test]
fn create_location_owner_resolve_failure_errors() {
    // Use `PlayerExpr::Runtime { Next }` against an empty `GameData` so
    // `resolve_owner_to_names` returns `Err` (fallible since 2026-08).
    let mut gd = GameData::new();
    let result = execute_setup_rule(
        SetUpRule::CreateLocation {
            locations: vec!["Hand".to_string()],
            owner: Owner::Player {
                player: PlayerExpr::Runtime {
                    runtime: front_end::ast::RuntimePlayer::Next,
                },
            },
        },
        &mut gd,
    );
    assert_eq!(result.unwrap_err().to_string(), "CreateLocation: failed to resolve owner Player { player: Runtime { runtime: Next } }: No current stage".to_string());
}

#[test]
fn create_card_on_location_missing_location_errors() {
    let mut gd = GameData::new();
    let result = execute_setup_rule(
        SetUpRule::CreateCardOnLocation {
            location: "Ghost".to_string(),
            cards: vec![],
        },
        &mut gd,
    );
    assert_eq!(
        result.unwrap_err().to_string(),
        "CreateCardOnLocation: location \"Ghost\" not found".to_string()
    );
}
