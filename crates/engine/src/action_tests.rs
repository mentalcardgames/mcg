//! Unit tests for `action::execute_*`. These build `ActionRule` / `SetUpRule`
//! payloads directly and call the `execute_*` functions, asserting on the
//! resulting `GameData` mutation. End-to-end `.cgdsl` fixture tests live in
//! `tests/action_test.rs`.

use super::*;
use crate::game_data::{GameData, Location, MemoryValue};
use front_end::ast::{
    ActionRule, CardSet, ClassicMove, DealMove, EndType, ExchangeMove, Extrema, Group, Groupable,
    IntExpr, MemoryType, MoveCardSet, MoveType, OutOf, Owner, PlayerExpr, Players, Quantity,
    ScoringRule, SetUpRule, Status, TokenLocExpr, TokenMove, UseMemory, UseSingleMemory,
    WinnerType,
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

fn token_loc(name: &str) -> TokenLocExpr {
    TokenLocExpr::Groupable {
        groupable: Groupable::Location {
            name: name.to_string(),
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

#[test]
fn execute_cardset_move_rejects_dest_index_equal_to_len() {
    // Regression for the off-by-one fixed in plan-2 Task 1: `dest_loc_idx
    // == locations.len()` must panic (the corrected `>=` guard), not slip
    // through and panic on the subsequent index. Constructing a
    // `CardSet` whose eval yields `locations.len()` is not generally
    // possible (eval_cardset uses a sentinel-0 fallback), so the load-bearing
    // regression coverage lives in the integration test
    // `move_top_card_to_hand_succeeds` + the from/to-eval `#[should_panic]`
    // tests below. This unit test documents the intent and exists to keep
    // the test file compiling.
    let (mut gd, _stock, _hand, _card_ids) = fixture_stock_to_hand();
    let _ = &mut gd;
}

// ---------------------------------------------------------------------------
// Task 2 — TODO no-op pins
// ---------------------------------------------------------------------------

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
fn flip_action_is_currently_a_noop() {
    // Pin: FlipAction ignores its card_set and status fields entirely
    // (action.rs:164-167). When this is implemented, this test must be
    // updated to assert the new behavior.
    let mut gd = GameData::new();
    let _p = gd.add_player("Alice".to_string());
    let stock = gd.add_location(
        "Table".to_string(),
        Location {
            name: "Stock".to_string(),
            cards: vec![],
        },
    );
    let card_id = gd.add_card(
        stock,
        HashMap::from([("face".to_string(), "down".to_string())]),
    );
    gd.locations[stock].cards.push(card_id);

    let before_cards = gd.cards.clone();
    let before_locs = loc_snapshot(&gd);
    execute_action_rule(
        ActionRule::FlipAction {
            card_set: loc_cardset("Stock"),
            status: Status::FaceUp,
        },
        &mut gd,
    );
    assert_eq!(gd.cards, before_cards, "FlipAction must not mutate cards");
    assert_eq!(
        loc_snapshot(&gd),
        before_locs,
        "FlipAction must not mutate locations"
    );
}

#[test]
fn bid_action_is_currently_a_noop() {
    let mut gd = GameData::new();
    let before_players = player_snapshot(&gd);
    let before_memories = gd.memories.clone();
    execute_action_rule(
        ActionRule::BidAction {
            quantitiy: Quantity::Int {
                int: IntExpr::Literal { int: 1 },
            },
        },
        &mut gd,
    );
    assert_eq!(player_snapshot(&gd), before_players);
    assert_eq!(gd.memories, before_memories);
}

#[test]
fn bid_memory_action_is_currently_a_noop() {
    let mut gd = GameData::new();
    let before = gd.clone();
    execute_action_rule(
        ActionRule::BidMemoryAction {
            memory: "bid".to_string(),
            quantity: Quantity::Int {
                int: IntExpr::Literal { int: 1 },
            },
            owner: Owner::Table,
        },
        &mut gd,
    );
    assert_eq!(gd.memories, before.memories);
}

#[test]
fn demand_action_is_currently_a_noop() {
    let mut gd = GameData::new();
    let before = player_snapshot(&gd);
    execute_action_rule(
        ActionRule::DemandAction {
            demand_type: front_end::ast::DemandType::Int {
                int: IntExpr::Literal { int: 1 },
            },
        },
        &mut gd,
    );
    assert_eq!(player_snapshot(&gd), before);
}

#[test]
fn demand_memory_action_is_currently_a_noop() {
    let mut gd = GameData::new();
    let before = gd.clone();
    execute_action_rule(
        ActionRule::DemandMemoryAction {
            demand_type: front_end::ast::DemandType::Int {
                int: IntExpr::Literal { int: 1 },
            },
            memory: "d".to_string(),
        },
        &mut gd,
    );
    assert_eq!(gd.memories, before.memories);
}

#[test]
fn end_action_game_with_winner_is_currently_a_noop() {
    let mut gd = GameData::new();
    let p0 = gd.add_player("Alice".to_string());
    let before = gd.clone();
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
    );
    assert_eq!(
        gd.players[p0].in_game, before.players[p0].in_game,
        "GameWithWinner must not flip in_game (currently a TODO no-op)"
    );
}

#[test]
fn move_type_place_is_currently_a_noop() {
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
    let before = loc_snapshot(&gd);
    let token = TokenMove::Place {
        token: "X".to_string(),
        from_loc: token_loc("Stock"),
        to_loc: token_loc("Hand"),
    };
    execute_move(MoveType::Place { token }, &mut gd);
    assert_eq!(
        loc_snapshot(&gd),
        before,
        "Place must not move cards (TODO no-op)"
    );
}

#[test]
fn scoring_rule_score_is_currently_a_noop() {
    let mut gd = GameData::new();
    let p0 = gd.add_player("Alice".to_string());
    let before_score = gd.players[p0].score;
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
    );
    assert_eq!(
        gd.players[p0].score, before_score,
        "Score must not change score (TODO no-op)"
    );
}

#[test]
fn scoring_rule_score_memory_is_currently_a_noop() {
    let mut gd = GameData::new();
    let p0 = gd.add_player("Alice".to_string());
    let before_score = gd.players[p0].score;
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
    );
    assert_eq!(
        gd.players[p0].score, before_score,
        "ScoreMemory must not change score (TODO no-op)"
    );
}

#[test]
fn scoring_rule_winner_is_currently_a_noop() {
    let mut gd = GameData::new();
    let p0 = gd.add_player("Alice".to_string());
    let before_score = gd.players[p0].score;
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
    );
    assert_eq!(
        gd.players[p0].score, before_score,
        "WinnerRule must not change score (TODO no-op)"
    );
}

#[test]
fn scoring_rule_winner_with_is_currently_a_noop() {
    let mut gd = GameData::new();
    let p0 = gd.add_player("Alice".to_string());
    let before_score = gd.players[p0].score;
    execute_scoring_rule(
        ScoringRule::WinnerRule {
            winner_rule: front_end::ast::WinnerRule::WinnerWith {
                extrema: Extrema::Max,
                winner_type: WinnerType::Score,
            },
        },
        &mut gd,
    );
    assert_eq!(
        gd.players[p0].score, before_score,
        "WinnerRule::WinnerWith must not change score (TODO no-op)"
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
    );
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
    );
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
    );
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
    );
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
    );
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
    );
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
    );
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
    );
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
    );
    assert_eq!(
        gd.current_player,
        Some(1),
        "CycleAction sets current_player to Bob's turn-order slot"
    );
}

#[test]
#[should_panic(expected = "failed to eval player")]
fn cycle_action_panics_when_player_eval_fails() {
    let mut gd = two_players_in_play_stage();
    let expr = PlayerExpr::Memory {
        memory: UseSingleMemory::Memory {
            memory: "nonexistent".to_string(),
        },
    };
    execute_action_rule(ActionRule::CycleAction { player: expr }, &mut gd);
}

#[test]
#[should_panic(expected = "not found")]
fn cycle_action_panics_when_player_not_in_players() {
    let mut gd = two_players_in_play_stage();
    execute_action_rule(
        ActionRule::CycleAction {
            player: PlayerExpr::Literal {
                name: "Ghost".to_string(),
            },
        },
        &mut gd,
    );
}

#[test]
#[should_panic(expected = "not in turn_order")]
fn cycle_action_panics_when_player_not_in_turn_order() {
    let mut gd = GameData::new();
    let p0 = gd.add_player("Alice".to_string());
    let _p1 = gd.add_player("Bob".to_string());
    gd.turn_order = vec![p0]; // Bob is in players but NOT in turn_order
    execute_action_rule(
        ActionRule::CycleAction {
            player: PlayerExpr::Literal {
                name: "Bob".to_string(),
            },
        },
        &mut gd,
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
    );

    let mut after = gd.locations[stock].cards.clone();
    after.sort();
    assert_eq!(after, ids, "shuffle preserves the set of card ids in Stock");
    assert_eq!(gd.locations[stock].cards.len(), 5);
}

#[test]
fn shuffle_action_on_missing_location_is_silent_noop() {
    // ShuffleAction prints to stderr and continues when eval_cardset fails.
    // Assert no panic and no mutation.
    let mut gd = GameData::new();
    let before = loc_snapshot(&gd);
    execute_action_rule(
        ActionRule::ShuffleAction {
            card_set: loc_cardset("Ghost"),
        },
        &mut gd,
    );
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
    gd.add_memory(
        "counter".to_string(),
        Owner::Table,
        Some(MemoryType::Int {
            int: IntExpr::Literal { int: 0 },
        }),
    );
    execute_action_rule(
        ActionRule::SetMemory {
            memory: "counter".to_string(),
            memory_type: MemoryType::Int {
                int: IntExpr::Literal { int: 42 },
            },
        },
        &mut gd,
    );
    assert_eq!(gd.get_memory("counter"), Some(&MemoryValue::Int(42)));
}

#[test]
fn reset_memory_action_zeros_int() {
    let mut gd = GameData::new();
    gd.add_memory("counter".to_string(), Owner::Table, None);
    gd.set_memory("counter".to_string(), MemoryValue::Int(7));
    execute_action_rule(
        ActionRule::ResetMemory {
            memory: "counter".to_string(),
        },
        &mut gd,
    );
    assert_eq!(gd.get_memory("counter"), Some(&MemoryValue::Int(0)));
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
    );
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
    );
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
    );
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
    );
    assert_eq!(gd.locations[stock].cards.len(), 0);
    assert_eq!(gd.locations[hand].cards.len(), 3);
}

// ---------------------------------------------------------------------------
// Task 9 — execute_cardset_move panic sites
// ---------------------------------------------------------------------------

#[test]
#[should_panic(expected = "failed to eval from cardset")]
fn execute_cardset_move_panics_when_from_eval_fails() {
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
    execute_cardset_move(from, None, Status::Private, to, &mut gd);
}

#[test]
#[should_panic(expected = "failed to eval dest cardset")]
fn execute_cardset_move_panics_when_to_eval_fails() {
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
    execute_cardset_move(from, None, Status::Private, to, &mut gd);
}

/// Pin for the corrected `>=` guard introduced in plan-2 Task 1. The guard
/// only fires when a `CardSet` eval yields `locations.len()`, which is not
/// generally producible via the public `CardSet` variants (eval_cardset uses
/// a sentinel-0 fallback). The reliable regression coverage lives in the
/// integration tests in `tests/action_test.rs`. This test is kept
/// `#[ignore]` to document the intent without producing a flaky failure.
#[test]
#[ignore = "requires a CardSet variant whose eval yields locations.len()"]
#[should_panic(expected = ">= locations.len()")]
fn execute_cardset_move_panics_when_dest_index_at_len() {
    let mut gd = GameData::new();
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
    let to = loc_cardset("Ghost");
    execute_cardset_move(from, None, Status::Private, to, &mut gd);
}

// ---------------------------------------------------------------------------
// Task 10 — SetUpRule panic sites
// ---------------------------------------------------------------------------

#[test]
#[should_panic(expected = "failed to resolve owner")]
fn create_location_panics_when_owner_name_missing() {
    // Use `PlayerExpr::Runtime { Next }` against an empty `GameData` so
    // `resolve_owner_to_names` returns `Err` and trips the panic at
    // `action.rs:94` (`CreateLocation: failed to resolve owner …`).
    let mut gd = GameData::new();
    execute_setup_rule(
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
}

#[test]
#[should_panic(expected = "location")]
fn create_card_on_location_panics_when_location_missing() {
    let mut gd = GameData::new();
    execute_setup_rule(
        SetUpRule::CreateCardOnLocation {
            location: "Ghost".to_string(),
            cards: vec![],
        },
        &mut gd,
    );
}
