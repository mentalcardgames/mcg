use super::*;
use crate::game_data::{GameData, Location, MemoryValue, Team};
use front_end::ast::{
    CardPosition, Extrema, IntExpr, MemoryType, Owner, PlayerCollection, PlayerExpr, Players,
    QueryCardPosition, QueryPlayer, RuntimePlayer, RuntimePlayerCollection, TeamCollection,
    TeamExpr, UseSingleMemory,
};
use std::collections::HashMap;

// ── P-1 ────────────────────────────────────────────────────────────────

#[test]
fn eval_player_literal() {
    let gd = GameData::new();
    let expr = PlayerExpr::Literal {
        name: "P1".to_string(),
    };
    assert_eq!(Evaluator::eval_player(&expr, &gd), Ok("P1".to_string()));
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
    assert_eq!(Evaluator::eval_player(&expr, &gd), Ok("P1".to_string()));
}

#[test]
fn eval_player_runtime_current_unset() {
    let gd = GameData::new();
    let expr = PlayerExpr::Runtime {
        runtime: RuntimePlayer::Current,
    };
    assert_eq!(
        Evaluator::eval_player(&expr, &gd),
        Err("No current player".to_string())
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
    assert_eq!(Evaluator::eval_player(&expr, &gd), Ok("P2".to_string()));
}

#[test]
fn eval_player_runtime_next_none() {
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
        Evaluator::eval_player(&expr, &gd),
        Err("No next player available".to_string())
    );
}

// ── P-4 ────────────────────────────────────────────────────────────────

#[test]
fn eval_player_runtime_previous_wraps() {
    let mut gd = GameData::new();
    let p0 = gd.add_player("P1".to_string());
    let p1 = gd.add_player("P2".to_string());
    gd.turn_order = vec![p0, p1];
    gd.current_player = Some(0);
    let expr = PlayerExpr::Runtime {
        runtime: RuntimePlayer::Previous,
    };
    assert_eq!(Evaluator::eval_player(&expr, &gd), Ok("P2".to_string()));
}

#[test]
fn eval_player_runtime_previous_missing() {
    let mut gd = GameData::new();
    gd.add_player("P1".to_string());
    gd.turn_order = vec![0];
    gd.current_player = Some(0);
    gd.players.clear();
    let expr = PlayerExpr::Runtime {
        runtime: RuntimePlayer::Previous,
    };
    assert_eq!(
        Evaluator::eval_player(&expr, &gd),
        Err("Previous player not found".to_string())
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
    }];
    gd.current_player = Some(0);
    let expr = PlayerExpr::Runtime {
        runtime: RuntimePlayer::Competitor,
    };
    assert_eq!(Evaluator::eval_player(&expr, &gd), Ok("P2".to_string()));
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
        Evaluator::eval_player(&expr, &gd),
        Err("No competitor found".to_string())
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
    assert_eq!(Evaluator::eval_player(&expr, &gd), Ok("P1".to_string()));
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
        Evaluator::eval_player(&expr, &gd),
        Err("Location Hand not found".to_string())
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
        .insert("score_P1".to_string(), MemoryValue::Int(42));
    gd.memories
        .insert("score_P2".to_string(), MemoryValue::Int(7));
    let expr = PlayerExpr::Aggregate {
        aggregate: front_end::ast::AggregatePlayer::OwnerOfMemory {
            extrema: Extrema::Max,
            memory: "score".to_string(),
        },
    };
    assert_eq!(Evaluator::eval_player(&expr, &gd), Ok("P1".to_string()));
}

#[test]
fn eval_player_aggregate_owner_of_memory_min() {
    let mut gd = GameData::new();
    let p0 = gd.add_player("P1".to_string());
    let p1 = gd.add_player("P2".to_string());
    gd.turn_order = vec![p0, p1];
    gd.memories
        .insert("score_P1".to_string(), MemoryValue::Int(42));
    gd.memories
        .insert("score_P2".to_string(), MemoryValue::Int(7));
    let expr = PlayerExpr::Aggregate {
        aggregate: front_end::ast::AggregatePlayer::OwnerOfMemory {
            extrema: Extrema::Min,
            memory: "score".to_string(),
        },
    };
    assert_eq!(Evaluator::eval_player(&expr, &gd), Ok("P2".to_string()));
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
        Evaluator::eval_player(&expr, &gd),
        Err("No player found for OwnerOfMemory".to_string())
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
    assert_eq!(Evaluator::eval_player(&expr, &gd), Ok("P1".to_string()));
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
        Evaluator::eval_player(&expr, &gd),
        Err("No player at turn order index 99".to_string())
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
    assert_eq!(Evaluator::eval_player(&expr, &gd), Ok("P1".to_string()));
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
        Evaluator::eval_player(&expr, &gd),
        Err("No player at index 99 in player collection".to_string())
    );
}

// ── P-10 ────────────────────────────────────────────────────────────────

#[test]
fn eval_player_memory_string() {
    let mut gd = GameData::new();
    gd.memories
        .insert("name".to_string(), MemoryValue::String("Alice".to_string()));
    let expr = PlayerExpr::Memory {
        memory: UseSingleMemory::Memory {
            memory: "name".to_string(),
        },
    };
    assert_eq!(Evaluator::eval_player(&expr, &gd), Ok("Alice".to_string()));
}

#[test]
fn eval_player_memory_player_collection() {
    let mut gd = GameData::new();
    let p0 = gd.add_player("P1".to_string());
    gd.memories
        .insert("pc".to_string(), MemoryValue::PlayerCollection(vec![p0]));
    let expr = PlayerExpr::Memory {
        memory: UseSingleMemory::Memory {
            memory: "pc".to_string(),
        },
    };
    assert_eq!(Evaluator::eval_player(&expr, &gd), Ok("P1".to_string()));
}

#[test]
fn eval_player_memory_empty() {
    let mut gd = GameData::new();
    gd.memories
        .insert("pc".to_string(), MemoryValue::PlayerCollection(vec![]));
    let expr = PlayerExpr::Memory {
        memory: UseSingleMemory::Memory {
            memory: "pc".to_string(),
        },
    };
    assert_eq!(
        Evaluator::eval_player(&expr, &gd),
        Err("PlayerCollection memory is empty".to_string())
    );
}

#[test]
fn eval_player_memory_wrong_type() {
    let mut gd = GameData::new();
    gd.memories
        .insert("counter".to_string(), MemoryValue::Int(42));
    let expr = PlayerExpr::Memory {
        memory: UseSingleMemory::Memory {
            memory: "counter".to_string(),
        },
    };
    assert_eq!(
        Evaluator::eval_player(&expr, &gd),
        Err("Memory value is not a valid player".to_string())
    );
}

#[test]
fn eval_player_memory_missing() {
    let gd = GameData::new();
    let expr = PlayerExpr::Memory {
        memory: UseSingleMemory::Memory {
            memory: "ghost".to_string(),
        },
    };
    assert_eq!(
        Evaluator::eval_player(&expr, &gd),
        Err("Memory ghost not found".to_string())
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
        Evaluator::eval_player(&expr, &gd),
        Err("No current player".to_string())
    );
}

// ── P-12 ────────────────────────────────────────────────────────────────

#[test]
fn eval_player_i10_player_memory_mismatched_init() {
    let mut gd = GameData::new();
    gd.add_memory(
        "p".to_string(),
        Owner::Table,
        Some(MemoryType::Player {
            player: PlayerExpr::Literal {
                name: "Alice".to_string(),
            },
        }),
    );
    let expr = PlayerExpr::Memory {
        memory: UseSingleMemory::Memory {
            memory: "p".to_string(),
        },
    };
    assert_eq!(
        Evaluator::eval_player(&expr, &gd),
        Err("Memory value is not a valid player".to_string())
    );
}

// ── P-13 ────────────────────────────────────────────────────────────────

#[test]
fn eval_team_literal() {
    let gd = GameData::new();
    let expr = TeamExpr::Literal {
        name: "T1".to_string(),
    };
    assert_eq!(Evaluator::eval_team(&expr, &gd), Ok("T1".to_string()));
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
    }];
    let expr = TeamExpr::Aggregate {
        aggregate: front_end::ast::AggregateTeam::TeamOf {
            player: PlayerExpr::Literal {
                name: "P1".to_string(),
            },
        },
    };
    assert_eq!(Evaluator::eval_team(&expr, &gd), Ok("Red".to_string()));
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
        Evaluator::eval_team(&expr, &gd),
        Err("Player P1 not found in any team".to_string())
    );
}

// ── P-15 ────────────────────────────────────────────────────────────────

#[test]
fn eval_team_memory_team() {
    let mut gd = GameData::new();
    gd.memories
        .insert("t".to_string(), MemoryValue::Team("Red".to_string()));
    let expr = TeamExpr::Memory {
        memory: UseSingleMemory::Memory {
            memory: "t".to_string(),
        },
    };
    assert_eq!(Evaluator::eval_team(&expr, &gd), Ok("Red".to_string()));
}

#[test]
fn eval_team_memory_wrong_type() {
    let mut gd = GameData::new();
    gd.memories
        .insert("counter".to_string(), MemoryValue::Int(42));
    let expr = TeamExpr::Memory {
        memory: UseSingleMemory::Memory {
            memory: "counter".to_string(),
        },
    };
    assert_eq!(
        Evaluator::eval_team(&expr, &gd),
        Err("Memory value is not a Team".to_string())
    );
}

#[test]
fn eval_team_memory_missing() {
    let gd = GameData::new();
    let expr = TeamExpr::Memory {
        memory: UseSingleMemory::Memory {
            memory: "ghost".to_string(),
        },
    };
    assert_eq!(
        Evaluator::eval_team(&expr, &gd),
        Err("Memory ghost not found".to_string())
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
    assert_eq!(Evaluator::resolve_players(&players, &gd), vec![p0]);
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
    assert_eq!(Evaluator::resolve_players(&players, &gd), vec![p0, p1]);
}

// ── P-17 ────────────────────────────────────────────────────────────────

#[test]
#[should_panic(expected = "resolve_players: failed to eval player")]
fn resolve_players_eval_failure_panics() {
    let gd = GameData::new();
    let players = Players::Player {
        player: PlayerExpr::Runtime {
            runtime: RuntimePlayer::Current,
        },
    };
    let _ = Evaluator::resolve_players(&players, &gd);
}

// ── P-18 ────────────────────────────────────────────────────────────────

#[test]
#[should_panic(expected = "resolve_players: player Ghost not found in game_data")]
fn resolve_players_player_not_in_gamedata_panics() {
    let mut gd = GameData::new();
    let p0 = gd.add_player("P1".to_string());
    gd.turn_order = vec![p0];
    let players = Players::Player {
        player: PlayerExpr::Literal {
            name: "Ghost".to_string(),
        },
    };
    let _ = Evaluator::resolve_players(&players, &gd);
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
    assert_eq!(Evaluator::resolve_player_collection(&pc, &gd), vec![p0, p1]);
}

// ── P-20 ────────────────────────────────────────────────────────────────

#[test]
#[should_panic(expected = "resolve_player_collection: failed to eval player")]
fn resolve_player_collection_literal_eval_failure_panics() {
    let gd = GameData::new();
    let pc = PlayerCollection::Literal {
        players: vec![PlayerExpr::Runtime {
            runtime: RuntimePlayer::Current,
        }],
    };
    let _ = Evaluator::resolve_player_collection(&pc, &gd);
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
    assert_eq!(Evaluator::resolve_player_collection(&pc, &gd), vec![p0]);
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
    assert_eq!(Evaluator::resolve_player_collection(&pc, &gd), vec![p1]);
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
    assert_eq!(Evaluator::resolve_player_collection(&pc, &gd), vec![p1]);
}

// ── P-22 ────────────────────────────────────────────────────────────────

#[test]
fn resolve_player_collection_aggregate_memory_silent_empty() {
    let mut gd = GameData::new();
    let p0 = gd.add_player("P1".to_string());
    gd.turn_order = vec![p0];
    let pc = PlayerCollection::AggregateMemory {
        memory: "m".to_string(),
        multi: front_end::ast::MultiOwner::PlayerCollection {
            player_collection: Box::new(PlayerCollection::Literal { players: vec![] }),
        },
    };
    // Quirk: AggregateMemory returns empty vec (silent, no error)
    let result: Vec<usize> = Evaluator::resolve_player_collection(&pc, &gd);
    assert!(result.is_empty());
}

#[test]
fn resolve_player_collection_memory_silent_empty() {
    let mut gd = GameData::new();
    let p0 = gd.add_player("P1".to_string());
    gd.turn_order = vec![p0];
    let pc = PlayerCollection::Memory {
        memory: front_end::ast::UseMemory::Memory {
            memory: "m".to_string(),
        },
    };
    // Quirk: Memory returns empty vec (silent, no error)
    let result: Vec<usize> = Evaluator::resolve_player_collection(&pc, &gd);
    assert!(result.is_empty());
}

// ── P-23 ────────────────────────────────────────────────────────────────

#[test]
#[should_panic(expected = "PlayerCollection::Aggregate not yet implemented")]
fn resolve_player_collection_aggregate_panics() {
    let gd = GameData::new();
    let pc = PlayerCollection::Aggregate {
        aggregate: front_end::ast::AggregatePlayerCollection::Quantifier {
            quantifier: front_end::ast::Quantifier::All,
        },
    };
    let _ = Evaluator::resolve_player_collection(&pc, &gd);
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
        Evaluator::resolve_owner_to_name(&owner, &gd),
        Ok("P1".to_string())
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
        Evaluator::resolve_owner_to_name(&owner, &gd),
        Ok("Red".to_string())
    );
}

#[test]
fn resolve_owner_to_name_table() {
    let gd = GameData::new();
    let owner = Owner::Table;
    assert_eq!(
        Evaluator::resolve_owner_to_name(&owner, &gd),
        Ok("Table".to_string())
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
        Evaluator::resolve_owner_to_name(&owner, &gd),
        Err("resolve_owner_to_name: PlayerCollection cannot resolve to a single name".to_string())
    );
}

#[test]
fn resolve_owner_to_name_team_collection_err() {
    let gd = GameData::new();
    let owner = Owner::TeamCollection {
        team_collection: TeamCollection::Literal { teams: vec![] },
    };
    assert_eq!(
        Evaluator::resolve_owner_to_name(&owner, &gd),
        Err("resolve_owner_to_name: TeamCollection cannot resolve to a single name".to_string())
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
fn resolve_owner_to_names_team_err() {
    let gd = GameData::new();
    let owner = Owner::Team {
        team: TeamExpr::Literal {
            name: "Red".to_string(),
        },
    };
    assert_eq!(
        Evaluator::resolve_owner_to_names(&owner, &gd),
        Err("resolve_owner_to_names: team 'Red' cannot own a location or memory (team-owned locations are not in the data model)".to_string())
    );
}

#[test]
fn resolve_owner_to_names_team_collection_err() {
    let gd = GameData::new();
    let owner = Owner::TeamCollection {
        team_collection: TeamCollection::Literal { teams: vec![] },
    };
    assert_eq!(
        Evaluator::resolve_owner_to_names(&owner, &gd),
        Err("resolve_owner_to_names: TeamCollection cannot resolve to owner names".to_string())
    );
}
