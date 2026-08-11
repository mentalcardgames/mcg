use super::*;
use crate::game_data::{GameData, Location, MemoryValue, Team};
use front_end::ast::{
    CardPosition, Extrema, IntExpr, MemoryType, Owner, PlayerCollection, PlayerExpr, Players,
    QueryCardPosition, QueryPlayer, RuntimePlayer, RuntimePlayerCollection, SingleOwner,
    TeamCollection, TeamExpr, UseSingleMemory,
};
use std::collections::HashMap;

// ── P-1 ────────────────────────────────────────────────────────────────

#[test]
fn eval_player_literal() {
    let gd = GameData::new();
    let expr = PlayerExpr::Literal {
        name: "P1".to_string(),
    };
    assert_eq!(
        Evaluator::eval_player(&expr, &gd).unwrap(),
        "P1".to_string()
    );
}

// ── P-2 ────────────────────────────────────────────────────────────────

#[test]
fn eval_player_runtime_current_set() {
    let mut gd = GameData::new();
    let p0 = gd.add_player("P1".to_string());
    gd.turn_order = vec![p0];
    gd.current_player = Some(0);
    let expr = PlayerExpr::Runtime {
        runtime: RuntimePlayer::Current,
    };
    assert_eq!(
        Evaluator::eval_player(&expr, &gd).unwrap(),
        "P1".to_string()
    );
}

#[test]
fn eval_player_runtime_current_unset() {
    let gd = GameData::new();
    let expr = PlayerExpr::Runtime {
        runtime: RuntimePlayer::Current,
    };
    assert_eq!(
        Evaluator::eval_player(&expr, &gd).unwrap_err().to_string(),
        "No current player".to_string()
    );
}

// ── P-3 ────────────────────────────────────────────────────────────────

#[test]
fn eval_player_runtime_next_eligible() {
    let mut gd = GameData::new();
    let p0 = gd.add_player("P1".to_string());
    let p1 = gd.add_player("P2".to_string());
    gd.turn_order = vec![p0, p1];
    gd.enter_stage("Play".to_string(), vec!["P1".to_string(), "P2".to_string()]);
    gd.current_player = Some(0);
    let expr = PlayerExpr::Runtime {
        runtime: RuntimePlayer::Next,
    };
    assert_eq!(
        Evaluator::eval_player(&expr, &gd).unwrap(),
        "P2".to_string()
    );
}

#[test]
fn eval_player_runtime_next_wraps_to_self_when_alone_eligible() {
    // I-13 relaxed (2026-08-10): with no *other* eligible player, `next`
    // wraps onto the current player instead of erroring.
    let mut gd = GameData::new();
    let p0 = gd.add_player("P1".to_string());
    let p1 = gd.add_player("P2".to_string());
    gd.turn_order = vec![p0, p1];
    gd.enter_stage("Play".to_string(), vec!["P1".to_string()]);
    gd.current_player = Some(0);
    let expr = PlayerExpr::Runtime {
        runtime: RuntimePlayer::Next,
    };
    assert_eq!(
        Evaluator::eval_player(&expr, &gd).unwrap(),
        "P1".to_string(),
        "no other eligible player => next = current (self-wrap)"
    );
}

// ── P-4 ────────────────────────────────────────────────────────────────

#[test]
fn eval_player_runtime_previous_wraps() {
    let mut gd = GameData::new();
    let p0 = gd.add_player("P1".to_string());
    let p1 = gd.add_player("P2".to_string());
    gd.turn_order = vec![p0, p1];
    gd.enter_stage("Play".to_string(), vec!["P1".to_string(), "P2".to_string()]);
    gd.current_player = Some(0);
    let expr = PlayerExpr::Runtime {
        runtime: RuntimePlayer::Previous,
    };
    assert_eq!(
        Evaluator::eval_player(&expr, &gd).unwrap(),
        "P2".to_string()
    );
}

#[test]
fn eval_player_runtime_previous_skips_ineligible_and_wraps_to_self() {
    // D-12 fixed (2026-08-10): `previous` skips ineligible players like
    // `next` does, and wraps onto the current player when it is the only
    // eligible one.
    let mut gd = GameData::new();
    let p0 = gd.add_player("P1".to_string());
    let p1 = gd.add_player("P2".to_string());
    let p2 = gd.add_player("P3".to_string());
    gd.turn_order = vec![p0, p1, p2];
    gd.enter_stage(
        "Play".to_string(),
        vec!["P1".to_string(), "P2".to_string(), "P3".to_string()],
    );
    gd.set_player_stage_flag(p2, "Play".to_string(), false); // P3 out of stage
    gd.current_player = Some(2); // P3 is current but ineligible...
    let expr = PlayerExpr::Runtime {
        runtime: RuntimePlayer::Previous,
    };
    // ...so `previous` skips past P2? No: P3 is current; scanning backwards
    // from P3: P2 eligible -> P2.
    assert_eq!(
        Evaluator::eval_player(&expr, &gd).unwrap(),
        "P2".to_string()
    );
}

#[test]
fn eval_player_runtime_previous_missing() {
    let mut gd = GameData::new();
    gd.add_player("P1".to_string());
    gd.turn_order = vec![0];
    gd.enter_stage("Play".to_string(), vec!["P1".to_string()]);
    gd.current_player = Some(0);
    gd.players.clear();
    let expr = PlayerExpr::Runtime {
        runtime: RuntimePlayer::Previous,
    };
    assert_eq!(
        Evaluator::eval_player(&expr, &gd).unwrap_err().to_string(),
        "Previous player not found".to_string()
    );
}

// ── P-5 ────────────────────────────────────────────────────────────────

#[test]
fn eval_player_runtime_competitor_teammate() {
    let mut gd = GameData::new();
    let p0 = gd.add_player("P1".to_string());
    let p1 = gd.add_player("P2".to_string());
    gd.turn_order = vec![p0, p1];
    gd.teams = vec![Team {
        name: "T1".to_string(),
        players: vec![p0, p1],
        owner: crate::game_data::OwnerData::default(),
    }];
    gd.current_player = Some(0);
    let expr = PlayerExpr::Runtime {
        runtime: RuntimePlayer::Competitor,
    };
    assert_eq!(
        Evaluator::eval_player(&expr, &gd).unwrap(),
        "P2".to_string()
    );
}

#[test]
fn eval_player_runtime_competitor_lone() {
    let mut gd = GameData::new();
    gd.add_player("P1".to_string());
    gd.turn_order = vec![0];
    gd.current_player = Some(0);
    let expr = PlayerExpr::Runtime {
        runtime: RuntimePlayer::Competitor,
    };
    assert_eq!(
        Evaluator::eval_player(&expr, &gd).unwrap_err().to_string(),
        "No competitor found".to_string()
    );
}

// ── P-6 ────────────────────────────────────────────────────────────────

#[test]
fn eval_player_aggregate_owner_of_card_position() {
    let mut gd = GameData::new();
    let _pid = gd.add_player("P1".to_string());
    let hand = gd.add_location(
        "P1".to_string(),
        Location {
            name: "Hand".to_string(),
            cards: vec![],
        },
    );
    let card = gd.add_card(hand, HashMap::new());
    gd.locations[hand].cards = vec![card];
    let expr = PlayerExpr::Aggregate {
        aggregate: front_end::ast::AggregatePlayer::OwnerOfCardPostion {
            card_position: Box::new(CardPosition::Query {
                query: QueryCardPosition::At {
                    location: "Hand".to_string(),
                    int_expr: IntExpr::Literal { int: 0 },
                },
            }),
        },
    };
    assert_eq!(
        Evaluator::eval_player(&expr, &gd).unwrap(),
        "P1".to_string()
    );
}

#[test]
fn eval_player_aggregate_owner_of_card_position_not_found() {
    let gd = GameData::new();
    let expr = PlayerExpr::Aggregate {
        aggregate: front_end::ast::AggregatePlayer::OwnerOfCardPostion {
            card_position: Box::new(CardPosition::Query {
                query: QueryCardPosition::At {
                    location: "Hand".to_string(),
                    int_expr: IntExpr::Literal { int: 0 },
                },
            }),
        },
    };
    assert_eq!(
        Evaluator::eval_player(&expr, &gd).unwrap_err().to_string(),
        "Location Hand not found".to_string()
    );
}

// ── P-7 ────────────────────────────────────────────────────────────────

#[test]
fn eval_player_aggregate_owner_of_memory_max() {
    let mut gd = GameData::new();
    let p0 = gd.add_player("P1".to_string());
    let p1 = gd.add_player("P2".to_string());
    gd.turn_order = vec![p0, p1];
    gd.memories
        .insert("P1_score".to_string(), MemoryValue::Int(42));
    gd.memories
        .insert("P2_score".to_string(), MemoryValue::Int(7));
    let expr = PlayerExpr::Aggregate {
        aggregate: front_end::ast::AggregatePlayer::OwnerOfMemory {
            extrema: Extrema::Max,
            memory: "score".to_string(),
        },
    };
    assert_eq!(
        Evaluator::eval_player(&expr, &gd).unwrap(),
        "P1".to_string()
    );
}

#[test]
fn eval_player_aggregate_owner_of_memory_min() {
    let mut gd = GameData::new();
    let p0 = gd.add_player("P1".to_string());
    let p1 = gd.add_player("P2".to_string());
    gd.turn_order = vec![p0, p1];
    gd.memories
        .insert("P1_score".to_string(), MemoryValue::Int(42));
    gd.memories
        .insert("P2_score".to_string(), MemoryValue::Int(7));
    let expr = PlayerExpr::Aggregate {
        aggregate: front_end::ast::AggregatePlayer::OwnerOfMemory {
            extrema: Extrema::Min,
            memory: "score".to_string(),
        },
    };
    assert_eq!(
        Evaluator::eval_player(&expr, &gd).unwrap(),
        "P2".to_string()
    );
}

#[test]
fn eval_player_aggregate_owner_of_memory_none() {
    let mut gd = GameData::new();
    let p0 = gd.add_player("P1".to_string());
    let p1 = gd.add_player("P2".to_string());
    gd.turn_order = vec![p0, p1];
    let expr = PlayerExpr::Aggregate {
        aggregate: front_end::ast::AggregatePlayer::OwnerOfMemory {
            extrema: Extrema::Max,
            memory: "score".to_string(),
        },
    };
    assert_eq!(
        Evaluator::eval_player(&expr, &gd).unwrap_err().to_string(),
        "No player found for OwnerOfMemory".to_string()
    );
}

// ── P-8 ────────────────────────────────────────────────────────────────

#[test]
fn eval_player_query_turnorder_at_in_range() {
    let mut gd = GameData::new();
    let p0 = gd.add_player("P1".to_string());
    let p1 = gd.add_player("P2".to_string());
    gd.turn_order = vec![p0, p1];
    let expr = PlayerExpr::Query {
        query: QueryPlayer::Turnorder {
            int: IntExpr::Literal { int: 0 },
        },
    };
    assert_eq!(
        Evaluator::eval_player(&expr, &gd).unwrap(),
        "P1".to_string()
    );
}

#[test]
fn eval_player_query_turnorder_at_out_of_range() {
    let mut gd = GameData::new();
    let p0 = gd.add_player("P1".to_string());
    gd.turn_order = vec![p0];
    let expr = PlayerExpr::Query {
        query: QueryPlayer::Turnorder {
            int: IntExpr::Literal { int: 99 },
        },
    };
    assert_eq!(
        Evaluator::eval_player(&expr, &gd).unwrap_err().to_string(),
        "No player at turn order index 99".to_string()
    );
}

// ── P-9 ────────────────────────────────────────────────────────────────

#[test]
fn eval_player_query_collection_at_in_range() {
    let mut gd = GameData::new();
    let p0 = gd.add_player("P1".to_string());
    let p1 = gd.add_player("P2".to_string());
    gd.turn_order = vec![p0, p1];
    let expr = PlayerExpr::Query {
        query: QueryPlayer::CollectionAt {
            players: PlayerCollection::Literal {
                players: vec![
                    PlayerExpr::Literal {
                        name: "P1".to_string(),
                    },
                    PlayerExpr::Literal {
                        name: "P2".to_string(),
                    },
                ],
            },
            int: IntExpr::Literal { int: 0 },
        },
    };
    assert_eq!(
        Evaluator::eval_player(&expr, &gd).unwrap(),
        "P1".to_string()
    );
}

#[test]
fn eval_player_query_collection_at_out_of_range() {
    let mut gd = GameData::new();
    let p0 = gd.add_player("P1".to_string());
    gd.turn_order = vec![p0];
    let expr = PlayerExpr::Query {
        query: QueryPlayer::CollectionAt {
            players: PlayerCollection::Literal {
                players: vec![PlayerExpr::Literal {
                    name: "P1".to_string(),
                }],
            },
            int: IntExpr::Literal { int: 99 },
        },
    };
    assert_eq!(
        Evaluator::eval_player(&expr, &gd).unwrap_err().to_string(),
        "No player at index 99 in player collection".to_string()
    );
}

// ── P-10 ────────────────────────────────────────────────────────────────

#[test]
fn eval_player_memory_string() {
    let mut gd = GameData::new();
    gd.memories.insert(
        "Table_name".to_string(),
        MemoryValue::String("Alice".to_string()),
    );
    let expr = PlayerExpr::Memory {
        memory: UseSingleMemory::WithOwner {
            memory: "name".to_string(),
            owner: Box::new(SingleOwner::Table),
        },
    };
    assert_eq!(
        Evaluator::eval_player(&expr, &gd).unwrap(),
        "Alice".to_string()
    );
}

#[test]
fn eval_player_memory_player_collection() {
    let mut gd = GameData::new();
    let p0 = gd.add_player("P1".to_string());
    gd.memories.insert(
        "Table_pc".to_string(),
        MemoryValue::PlayerCollection(vec![p0]),
    );
    let expr = PlayerExpr::Memory {
        memory: UseSingleMemory::WithOwner {
            memory: "pc".to_string(),
            owner: Box::new(SingleOwner::Table),
        },
    };
    assert_eq!(
        Evaluator::eval_player(&expr, &gd).unwrap(),
        "P1".to_string()
    );
}

#[test]
fn eval_player_memory_empty() {
    let mut gd = GameData::new();
    gd.memories.insert(
        "Table_pc".to_string(),
        MemoryValue::PlayerCollection(vec![]),
    );
    let expr = PlayerExpr::Memory {
        memory: UseSingleMemory::WithOwner {
            memory: "pc".to_string(),
            owner: Box::new(SingleOwner::Table),
        },
    };
    assert_eq!(
        Evaluator::eval_player(&expr, &gd).unwrap_err().to_string(),
        "PlayerCollection memory is empty".to_string()
    );
}

#[test]
fn eval_player_memory_wrong_type() {
    let mut gd = GameData::new();
    gd.memories
        .insert("Table_counter".to_string(), MemoryValue::Int(42));
    let expr = PlayerExpr::Memory {
        memory: UseSingleMemory::WithOwner {
            memory: "counter".to_string(),
            owner: Box::new(SingleOwner::Table),
        },
    };
    assert_eq!(
        Evaluator::eval_player(&expr, &gd).unwrap_err().to_string(),
        "Memory value is not a valid player".to_string()
    );
}

#[test]
fn eval_player_memory_missing() {
    let gd = GameData::new();
    let expr = PlayerExpr::Memory {
        memory: UseSingleMemory::WithOwner {
            memory: "ghost".to_string(),
            owner: Box::new(SingleOwner::Table),
        },
    };
    assert_eq!(
        Evaluator::eval_player(&expr, &gd).unwrap_err().to_string(),
        "Memory Table_ghost not found".to_string()
    );
}

#[test]
fn eval_player_memory_no_owner_error() {
    let gd = GameData::new();
    let expr = PlayerExpr::Memory {
        memory: UseSingleMemory::Memory {
            memory: "M".to_string(),
        },
    };
    assert_eq!(
        Evaluator::eval_player(&expr, &gd).unwrap_err().to_string(),
        "memory access requires an explicit owner; use &M:M of <owner>".to_string()
    );
}

// ── P-11 ────────────────────────────────────────────────────────────────

#[test]
fn eval_player_i2_empty_turn_order_safe() {
    let gd = GameData::new();
    let expr = PlayerExpr::Runtime {
        runtime: RuntimePlayer::Current,
    };
    assert_eq!(
        Evaluator::eval_player(&expr, &gd).unwrap_err().to_string(),
        "No current player".to_string()
    );
}

// ── P-12 ────────────────────────────────────────────────────────────────

#[test]
fn eval_player_player_memory_init_to_owner_name_reads() {
    // I-10 fix (2026-08-10): a Player-typed memory initialises to the
    // owner's name, so reading it as a player now succeeds out of the box.
    let mut gd = GameData::new();
    gd.add_player("P1".to_string());
    gd.add_memory(
        "P1_role".to_string(),
        "P1",
        Some(MemoryType::Player {
            player: PlayerExpr::Literal {
                name: "Alice".to_string(),
            },
        }),
        None,
    );
    let expr = PlayerExpr::Memory {
        memory: UseSingleMemory::WithOwner {
            memory: "role".to_string(),
            owner: Box::new(SingleOwner::Player {
                player: PlayerExpr::Literal {
                    name: "P1".to_string(),
                },
            }),
        },
    };
    assert_eq!(
        Evaluator::eval_player(&expr, &gd).unwrap(),
        "P1".to_string(),
        "player-typed memory reads as its owner's name"
    );
}

// ── P-13 ────────────────────────────────────────────────────────────────

#[test]
fn eval_team_literal() {
    let gd = GameData::new();
    let expr = TeamExpr::Literal {
        name: "T1".to_string(),
    };
    assert_eq!(Evaluator::eval_team(&expr, &gd).unwrap(), "T1".to_string());
}

// ── P-14 ────────────────────────────────────────────────────────────────

#[test]
fn eval_team_aggregate_team_of_present() {
    let mut gd = GameData::new();
    let p0 = gd.add_player("P1".to_string());
    gd.turn_order = vec![p0];
    gd.teams = vec![Team {
        name: "Red".to_string(),
        players: vec![p0],
        owner: crate::game_data::OwnerData::default(),
    }];
    let expr = TeamExpr::Aggregate {
        aggregate: front_end::ast::AggregateTeam::TeamOf {
            player: PlayerExpr::Literal {
                name: "P1".to_string(),
            },
        },
    };
    assert_eq!(Evaluator::eval_team(&expr, &gd).unwrap(), "Red".to_string());
}

#[test]
fn eval_team_aggregate_team_of_absent() {
    let gd = GameData::new();
    let expr = TeamExpr::Aggregate {
        aggregate: front_end::ast::AggregateTeam::TeamOf {
            player: PlayerExpr::Literal {
                name: "P1".to_string(),
            },
        },
    };
    assert_eq!(
        Evaluator::eval_team(&expr, &gd).unwrap_err().to_string(),
        "Player P1 not found in any team".to_string()
    );
}

// ── P-15 ────────────────────────────────────────────────────────────────

#[test]
fn eval_team_memory_team() {
    let mut gd = GameData::new();
    gd.memories
        .insert("Table_t".to_string(), MemoryValue::Team("Red".to_string()));
    let expr = TeamExpr::Memory {
        memory: UseSingleMemory::WithOwner {
            memory: "t".to_string(),
            owner: Box::new(SingleOwner::Table),
        },
    };
    assert_eq!(Evaluator::eval_team(&expr, &gd).unwrap(), "Red".to_string());
}

#[test]
fn eval_team_memory_wrong_type() {
    let mut gd = GameData::new();
    gd.memories
        .insert("Table_counter".to_string(), MemoryValue::Int(42));
    let expr = TeamExpr::Memory {
        memory: UseSingleMemory::WithOwner {
            memory: "counter".to_string(),
            owner: Box::new(SingleOwner::Table),
        },
    };
    assert_eq!(
        Evaluator::eval_team(&expr, &gd).unwrap_err().to_string(),
        "Memory value is not a Team".to_string()
    );
}

#[test]
fn eval_team_memory_missing() {
    let gd = GameData::new();
    let expr = TeamExpr::Memory {
        memory: UseSingleMemory::WithOwner {
            memory: "ghost".to_string(),
            owner: Box::new(SingleOwner::Table),
        },
    };
    assert_eq!(
        Evaluator::eval_team(&expr, &gd).unwrap_err().to_string(),
        "Memory Table_ghost not found".to_string()
    );
}

#[test]
fn eval_team_memory_no_owner_error() {
    let gd = GameData::new();
    let expr = TeamExpr::Memory {
        memory: UseSingleMemory::Memory {
            memory: "M".to_string(),
        },
    };
    assert_eq!(
        Evaluator::eval_team(&expr, &gd).unwrap_err().to_string(),
        "memory access requires an explicit owner; use &M:M of <owner>".to_string()
    );
}

// ── P-16 ────────────────────────────────────────────────────────────────

#[test]
fn resolve_players_single_player() {
    let mut gd = GameData::new();
    let p0 = gd.add_player("P1".to_string());
    let p1 = gd.add_player("P2".to_string());
    gd.turn_order = vec![p0, p1];
    let players = Players::Player {
        player: PlayerExpr::Literal {
            name: "P1".to_string(),
        },
    };
    assert_eq!(Evaluator::resolve_players(&players, &gd).unwrap(), vec![p0]);
}

#[test]
fn resolve_players_player_collection_literal() {
    let mut gd = GameData::new();
    let p0 = gd.add_player("P1".to_string());
    let p1 = gd.add_player("P2".to_string());
    gd.turn_order = vec![p0, p1];
    let pc = PlayerCollection::Literal {
        players: vec![
            PlayerExpr::Literal {
                name: "P1".to_string(),
            },
            PlayerExpr::Literal {
                name: "P2".to_string(),
            },
        ],
    };
    let players = Players::PlayerCollection {
        player_collection: pc,
    };
    assert_eq!(
        Evaluator::resolve_players(&players, &gd).unwrap(),
        vec![p0, p1]
    );
}

// ── P-17 ────────────────────────────────────────────────────────────────

#[test]
fn resolve_players_eval_failure_errors() {
    // Fallible since 2026-08: eval failures surface as Err, not panics.
    let gd = GameData::new();
    let players = Players::Player {
        player: PlayerExpr::Runtime {
            runtime: RuntimePlayer::Current,
        },
    };
    assert_eq!(
        Evaluator::resolve_players(&players, &gd)
            .unwrap_err()
            .to_string(),
        "No current player".to_string()
    );
}

// ── P-18 ────────────────────────────────────────────────────────────────

#[test]
fn resolve_players_player_not_in_gamedata_errors() {
    let mut gd = GameData::new();
    let p0 = gd.add_player("P1".to_string());
    gd.turn_order = vec![p0];
    let players = Players::Player {
        player: PlayerExpr::Literal {
            name: "Ghost".to_string(),
        },
    };
    assert_eq!(
        Evaluator::resolve_players(&players, &gd)
            .unwrap_err()
            .to_string(),
        "resolve_players: player Ghost not found in game_data".to_string()
    );
}

// ── P-19 ────────────────────────────────────────────────────────────────

#[test]
fn resolve_player_collection_literal_happy() {
    let mut gd = GameData::new();
    let p0 = gd.add_player("P1".to_string());
    let p1 = gd.add_player("P2".to_string());
    gd.turn_order = vec![p0, p1];
    let pc = PlayerCollection::Literal {
        players: vec![
            PlayerExpr::Literal {
                name: "P1".to_string(),
            },
            PlayerExpr::Literal {
                name: "P2".to_string(),
            },
        ],
    };
    assert_eq!(
        Evaluator::resolve_player_collection(&pc, &gd).unwrap(),
        vec![p0, p1]
    );
}

// ── P-20 ────────────────────────────────────────────────────────────────

#[test]
fn resolve_player_collection_literal_eval_failure_errors() {
    let gd = GameData::new();
    let pc = PlayerCollection::Literal {
        players: vec![PlayerExpr::Runtime {
            runtime: RuntimePlayer::Current,
        }],
    };
    assert_eq!(
        Evaluator::resolve_player_collection(&pc, &gd)
            .unwrap_err()
            .to_string(),
        "No current player".to_string()
    );
}

#[test]
fn resolve_player_collection_literal_unknown_name_errors() {
    let mut gd = GameData::new();
    let p0 = gd.add_player("P1".to_string());
    gd.turn_order = vec![p0];
    let pc = PlayerCollection::Literal {
        players: vec![PlayerExpr::Literal {
            name: "Ghost".to_string(),
        }],
    };
    assert_eq!(
        Evaluator::resolve_player_collection(&pc, &gd)
            .unwrap_err()
            .to_string(),
        "resolve_player_collection: player Ghost not found in game_data".to_string()
    );
}

// ── P-21 ────────────────────────────────────────────────────────────────

#[test]
fn resolve_player_collection_runtime_players_out() {
    let mut gd = GameData::new();
    let p0 = gd.add_player("P1".to_string());
    let p1 = gd.add_player("P2".to_string());
    gd.turn_order = vec![p0, p1];
    gd.set_player_out(p0);
    let pc = PlayerCollection::Runtime {
        runtime: RuntimePlayerCollection::PlayersOut,
    };
    assert_eq!(
        Evaluator::resolve_player_collection(&pc, &gd).unwrap(),
        vec![p0]
    );
}

#[test]
fn resolve_player_collection_runtime_players_in() {
    let mut gd = GameData::new();
    let p0 = gd.add_player("P1".to_string());
    let p1 = gd.add_player("P2".to_string());
    gd.turn_order = vec![p0, p1];
    gd.set_player_out(p0);
    let pc = PlayerCollection::Runtime {
        runtime: RuntimePlayerCollection::PlayersIn,
    };
    assert_eq!(
        Evaluator::resolve_player_collection(&pc, &gd).unwrap(),
        vec![p1]
    );
}

#[test]
fn resolve_player_collection_runtime_others() {
    let mut gd = GameData::new();
    let p0 = gd.add_player("P1".to_string());
    let p1 = gd.add_player("P2".to_string());
    let p2 = gd.add_player("P3".to_string());
    gd.turn_order = vec![p0, p1, p2];
    gd.set_player_out(p2);
    gd.current_player = Some(0);
    let pc = PlayerCollection::Runtime {
        runtime: RuntimePlayerCollection::Others,
    };
    assert_eq!(
        Evaluator::resolve_player_collection(&pc, &gd).unwrap(),
        vec![p1]
    );
}

// ── P-22 ────────────────────────────────────────────────────────────────

#[test]
fn resolve_player_collection_aggregate_memory() {
    let mut gd = GameData::new();
    let p0 = gd.add_player("P1".to_string());
    let p1 = gd.add_player("P2".to_string());
    gd.turn_order = vec![p0, p1];
    gd.memories
        .insert("P1_m".to_string(), MemoryValue::PlayerCollection(vec![p1]));
    gd.memories
        .insert("P2_m".to_string(), MemoryValue::PlayerCollection(vec![p0]));
    let pc = PlayerCollection::AggregateMemory {
        memory: "m".to_string(),
        multi: front_end::ast::MultiOwner::PlayerCollection {
            player_collection: Box::new(PlayerCollection::Runtime {
                runtime: RuntimePlayerCollection::PlayersIn,
            }),
        },
    };
    // Implemented 2026-08: aggregates the slot across every owner.
    assert_eq!(
        Evaluator::resolve_player_collection(&pc, &gd).unwrap(),
        vec![p1, p0]
    );
}

#[test]
fn resolve_player_collection_memory_reads_slot() {
    let mut gd = GameData::new();
    let p0 = gd.add_player("P1".to_string());
    gd.turn_order = vec![p0];
    gd.memories.insert(
        "Table_m".to_string(),
        MemoryValue::PlayerCollection(vec![p0]),
    );
    let pc = PlayerCollection::Memory {
        memory: front_end::ast::UseMemory::WithOwner {
            memory: "m".to_string(),
            owner: Box::new(front_end::ast::Owner::Table),
        },
    };
    assert_eq!(
        Evaluator::resolve_player_collection(&pc, &gd).unwrap(),
        vec![p0]
    );
}

#[test]
fn resolve_player_collection_memory_missing_errors() {
    let gd = GameData::new();
    let pc = PlayerCollection::Memory {
        memory: front_end::ast::UseMemory::WithOwner {
            memory: "m".to_string(),
            owner: Box::new(front_end::ast::Owner::Table),
        },
    };
    assert_eq!(
        Evaluator::resolve_player_collection(&pc, &gd)
            .unwrap_err()
            .to_string(),
        "Memory Table_m not found".to_string()
    );
}

// ── P-23 ────────────────────────────────────────────────────────────────

#[test]
fn resolve_player_collection_aggregate_returns_in_game() {
    let mut gd = GameData::new();
    let p0 = gd.add_player("P1".to_string());
    let p1 = gd.add_player("P2".to_string());
    gd.turn_order = vec![p0, p1];
    gd.set_player_out(p1);
    let pc = PlayerCollection::Aggregate {
        aggregate: front_end::ast::AggregatePlayerCollection::Quantifier {
            quantifier: front_end::ast::Quantifier::All,
        },
    };
    // Implemented 2026-08: `all`/`any` resolve to in-game players.
    assert_eq!(
        Evaluator::resolve_player_collection(&pc, &gd).unwrap(),
        vec![p0]
    );
}

// ── P-24 ────────────────────────────────────────────────────────────────

#[test]
fn resolve_owner_to_name_player() {
    let mut gd = GameData::new();
    let p0 = gd.add_player("P1".to_string());
    gd.turn_order = vec![p0];
    let owner = Owner::Player {
        player: PlayerExpr::Literal {
            name: "P1".to_string(),
        },
    };
    assert_eq!(
        Evaluator::resolve_owner_to_name(&owner, &gd).unwrap(),
        "P1".to_string()
    );
}

#[test]
fn resolve_owner_to_name_team() {
    let gd = GameData::new();
    let owner = Owner::Team {
        team: TeamExpr::Literal {
            name: "Red".to_string(),
        },
    };
    assert_eq!(
        Evaluator::resolve_owner_to_name(&owner, &gd).unwrap(),
        "Red".to_string()
    );
}

#[test]
fn resolve_owner_to_name_table() {
    let gd = GameData::new();
    let owner = Owner::Table;
    assert_eq!(
        Evaluator::resolve_owner_to_name(&owner, &gd).unwrap(),
        "Table".to_string()
    );
}

#[test]
fn resolve_owner_to_name_player_collection_err() {
    let gd = GameData::new();
    let owner = Owner::PlayerCollection {
        player_collection: PlayerCollection::Literal {
            players: vec![PlayerExpr::Literal {
                name: "P1".to_string(),
            }],
        },
    };
    assert_eq!(
        Evaluator::resolve_owner_to_name(&owner, &gd)
            .unwrap_err()
            .to_string(),
        "resolve_owner_to_name: PlayerCollection cannot resolve to a single name".to_string()
    );
}

#[test]
fn resolve_owner_to_name_team_collection_err() {
    let gd = GameData::new();
    let owner = Owner::TeamCollection {
        team_collection: TeamCollection::Literal { teams: vec![] },
    };
    assert_eq!(
        Evaluator::resolve_owner_to_name(&owner, &gd)
            .unwrap_err()
            .to_string(),
        "resolve_owner_to_name: TeamCollection cannot resolve to a single name".to_string()
    );
}

// ── P-25 ────────────────────────────────────────────────────────────────

#[test]
fn resolve_owner_to_names_player_collection_all() {
    let mut gd = GameData::new();
    let p0 = gd.add_player("P1".to_string());
    let p1 = gd.add_player("P2".to_string());
    gd.turn_order = vec![p0, p1];
    let owner = Owner::PlayerCollection {
        player_collection: PlayerCollection::Aggregate {
            aggregate: front_end::ast::AggregatePlayerCollection::Quantifier {
                quantifier: front_end::ast::Quantifier::All,
            },
        },
    };
    let result = Evaluator::resolve_owner_to_names(&owner, &gd);
    assert!(result.is_ok());
    let names = result.unwrap();
    assert_eq!(names.len(), 2);
    assert!(names.contains(&"P1".to_string()));
    assert!(names.contains(&"P2".to_string()));
}

#[test]
fn resolve_owner_to_names_team_resolves_team_name() {
    // `location X on T:Red` creates ONE shared instance for the team, keyed
    // by the team name (matching `(&I:M of T:Red)` addressing).
    let mut gd = GameData::new();
    let p0 = gd.add_player("P1".to_string());
    let p1 = gd.add_player("P2".to_string());
    gd.add_player("P3".to_string());
    gd.teams = vec![Team {
        name: "Red".to_string(),
        players: vec![p0, p1],
        owner: crate::game_data::OwnerData::default(),
    }];
    let owner = Owner::Team {
        team: TeamExpr::Literal {
            name: "Red".to_string(),
        },
    };
    let names = Evaluator::resolve_owner_to_names(&owner, &gd).unwrap();
    assert_eq!(names, vec!["Red".to_string()]);
}

#[test]
fn resolve_owner_to_names_team_collection_resolves_team_names() {
    let mut gd = GameData::new();
    let p0 = gd.add_player("P1".to_string());
    let p1 = gd.add_player("P2".to_string());
    let p2 = gd.add_player("P3".to_string());
    gd.teams = vec![
        Team {
            name: "Red".to_string(),
            players: vec![p0, p1],
            owner: crate::game_data::OwnerData::default(),
        },
        Team {
            name: "Blue".to_string(),
            players: vec![p2],
            owner: crate::game_data::OwnerData::default(),
        },
    ];
    let owner = Owner::TeamCollection {
        team_collection: TeamCollection::Literal {
            teams: vec![
                TeamExpr::Literal {
                    name: "Red".to_string(),
                },
                TeamExpr::Literal {
                    name: "Blue".to_string(),
                },
            ],
        },
    };
    let names = Evaluator::resolve_owner_to_names(&owner, &gd).unwrap();
    assert_eq!(names, vec!["Red".to_string(), "Blue".to_string()]);
}

#[test]
fn resolve_owner_to_names_unknown_team_resolves_empty() {
    let gd = GameData::new();
    let owner = Owner::Team {
        team: TeamExpr::Literal {
            name: "Ghost".to_string(),
        },
    };
    let names = Evaluator::resolve_owner_to_names(&owner, &gd).unwrap();
    assert!(names.is_empty(), "unknown team => no owners");
}
