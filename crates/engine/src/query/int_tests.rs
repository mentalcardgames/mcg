use super::*;
use crate::game_data::PointMap;
use front_end::ast::*;
use std::collections::HashMap;

fn lit(i: i32) -> IntExpr {
    IntExpr::Literal { int: i }
}

fn binexpr(left: IntExpr, op: IntOp, right: IntExpr) -> IntExpr {
    IntExpr::Binary {
        int: Box::new(left),
        op,
        int1: Box::new(right),
    }
}

fn size_of(collection: Collection) -> IntExpr {
    IntExpr::Aggregate {
        aggregate: AggregateInt::SizeOf { collection },
    }
}

fn int_collection_mem(key: &str) -> IntCollection {
    IntCollection::Memory {
        memory: UseMemory::WithOwner {
            memory: key.to_string(),
            owner: Box::new(Owner::Table),
        },
    }
}

// ── I-1 ──────────────────────────────────────────────────────────
#[test]
fn eval_int_literal() {
    assert_eq!(Evaluator::eval_int(&lit(42), &GameData::new()).unwrap(), 42);
}

// ── I-2 ──────────────────────────────────────────────────────────
#[test]
fn eval_int_binary_plus() {
    assert_eq!(
        Evaluator::eval_int(&binexpr(lit(2), IntOp::Plus, lit(3)), &GameData::new()).unwrap(),
        5
    );
}

#[test]
fn eval_int_binary_minus() {
    assert_eq!(
        Evaluator::eval_int(&binexpr(lit(2), IntOp::Minus, lit(3)), &GameData::new()).unwrap(),
        -1
    );
}

#[test]
fn eval_int_binary_mul() {
    assert_eq!(
        Evaluator::eval_int(&binexpr(lit(2), IntOp::Mul, lit(3)), &GameData::new()).unwrap(),
        6
    );
}

// ── I-3 ──────────────────────────────────────────────────────────
#[test]
fn eval_int_binary_div_by_zero() {
    assert_eq!(
        Evaluator::eval_int(&binexpr(lit(1), IntOp::Div, lit(0)), &GameData::new())
            .unwrap_err()
            .to_string(),
        "Division by zero".to_string()
    );
}

// ── I-4 ──────────────────────────────────────────────────────────
#[test]
fn eval_int_binary_div_nonzero() {
    assert_eq!(
        Evaluator::eval_int(&binexpr(lit(7), IntOp::Div, lit(2)), &GameData::new()).unwrap(),
        3
    );
}

#[test]
fn eval_int_binary_mod_nonzero() {
    assert_eq!(
        Evaluator::eval_int(&binexpr(lit(7), IntOp::Mod, lit(3)), &GameData::new()).unwrap(),
        1
    );
}

// ── I-5 ──────────────────────────────────────────────────────────
#[test]
fn eval_int_binary_mod_by_zero() {
    assert_eq!(
        Evaluator::eval_int(&binexpr(lit(1), IntOp::Mod, lit(0)), &GameData::new())
            .unwrap_err()
            .to_string(),
        "Modulo by zero".to_string()
    );
}

// ── I-6 ──────────────────────────────────────────────────────────
#[test]
fn eval_int_query_int_collection_at_in_range() {
    let ic = IntCollection::Literal {
        ints: vec![lit(10), lit(20), lit(30)],
    };
    let expr = IntExpr::Query {
        query: QueryInt::IntCollectionAt {
            int_collection: Box::new(ic),
            int_expr: Box::new(lit(1)),
        },
    };
    assert_eq!(Evaluator::eval_int(&expr, &GameData::new()).unwrap(), 20);
}

#[test]
fn eval_int_query_int_collection_at_out_of_range() {
    let ic = IntCollection::Literal {
        ints: vec![lit(10), lit(20), lit(30)],
    };
    let expr = IntExpr::Query {
        query: QueryInt::IntCollectionAt {
            int_collection: Box::new(ic),
            int_expr: Box::new(lit(5)),
        },
    };
    assert_eq!(
        Evaluator::eval_int(&expr, &GameData::new())
            .unwrap_err()
            .to_string(),
        "No int at index 5".to_string()
    );
}

// ── I-7 ──────────────────────────────────────────────────────────
#[test]
fn eval_int_size_of_int_collection_memory_present() {
    let mut gd = GameData::new();
    gd.set_memory(
        "Table_ic".to_string(),
        MemoryValue::IntCollection(vec![1, 2, 3]),
    );
    let expr = size_of(Collection::IntCollection {
        int: int_collection_mem("ic"),
    });
    assert_eq!(Evaluator::eval_int(&expr, &gd).unwrap(), 3);
}

#[test]
fn eval_int_size_of_int_collection_memory_missing() {
    let gd = GameData::new();
    let expr = size_of(Collection::IntCollection {
        int: int_collection_mem("nope"),
    });
    assert_eq!(
        Evaluator::eval_int(&expr, &gd).unwrap_err().to_string(),
        "Memory Table_nope not found".to_string()
    );
}

// ── I-8 ──────────────────────────────────────────────────────────
#[test]
fn eval_int_size_of_player_collection_memory() {
    let mut gd = GameData::new();
    gd.set_memory(
        "Table_pc".to_string(),
        MemoryValue::PlayerCollection(vec![0, 1]),
    );
    let expr = size_of(Collection::PlayerCollection {
        player: PlayerCollection::Memory {
            memory: UseMemory::WithOwner {
                memory: "pc".to_string(),
                owner: Box::new(Owner::Table),
            },
        },
    });
    // Implemented 2026-08: the PlayerCollection memory slot is read.
    assert_eq!(Evaluator::eval_int(&expr, &gd).unwrap(), 2);
}

// ── I-9 ──────────────────────────────────────────────────────────
#[test]
fn eval_int_aggregate_sum_of_int_collection() {
    let ic = IntCollection::Literal {
        ints: vec![lit(1), lit(2), lit(3)],
    };
    let expr = IntExpr::Aggregate {
        aggregate: AggregateInt::SumOfIntCollection { int_collection: ic },
    };
    assert_eq!(Evaluator::eval_int(&expr, &GameData::new()).unwrap(), 6);
}

#[test]
fn eval_int_aggregate_sum_of_int_collection_empty() {
    let ic = IntCollection::Literal { ints: vec![] };
    let expr = IntExpr::Aggregate {
        aggregate: AggregateInt::SumOfIntCollection { int_collection: ic },
    };
    assert_eq!(Evaluator::eval_int(&expr, &GameData::new()).unwrap(), 0);
}

// ── I-10 ─────────────────────────────────────────────────────────
fn card_with(map: &[(&str, &str)]) -> crate::game_data::Card {
    let mut c = HashMap::new();
    for (k, v) in map {
        c.insert(k.to_string(), v.to_string());
    }
    c
}

#[test]
fn eval_int_aggregate_sum_of_card_set_present() {
    let mut gd = GameData::new();
    let c0 = gd.add_card(0, card_with(&[("rank", "Ace")]));
    gd.set_memory("Table_cs".to_string(), MemoryValue::CardSet(vec![c0]));
    let mut pm = PointMap {
        name: "Pm".to_string(),
        map: HashMap::new(),
    };
    pm.map.insert("rank:Ace".to_string(), 10);
    gd.point_maps.push(pm);

    let expr = IntExpr::Aggregate {
        aggregate: AggregateInt::SumOfCardSet {
            card_set: Box::new(CardSet::Memory {
                memory: UseMemory::WithOwner {
                    memory: "cs".to_string(),
                    owner: Box::new(Owner::Table),
                },
            }),
            pointmap: "Pm".to_string(),
        },
    };
    assert_eq!(Evaluator::eval_int(&expr, &gd).unwrap(), 10);
}

#[test]
fn eval_int_aggregate_sum_of_card_set_missing_pointmap() {
    let mut gd = GameData::new();
    let c0 = gd.add_card(0, card_with(&[("rank", "Ace")]));
    gd.set_memory("Table_cs".to_string(), MemoryValue::CardSet(vec![c0]));

    let expr = IntExpr::Aggregate {
        aggregate: AggregateInt::SumOfCardSet {
            card_set: Box::new(CardSet::Memory {
                memory: UseMemory::WithOwner {
                    memory: "cs".to_string(),
                    owner: Box::new(Owner::Table),
                },
            }),
            pointmap: "Ghost".to_string(),
        },
    };
    assert_eq!(
        Evaluator::eval_int(&expr, &gd).unwrap_err().to_string(),
        "PointMap Ghost not found".to_string()
    );
}

// ── I-11 ─────────────────────────────────────────────────────────
fn extrema_cardset(extrema: Extrema, cs_key: &str, pointmap: &str) -> IntExpr {
    IntExpr::Aggregate {
        aggregate: AggregateInt::ExtremaCardset {
            extrema,
            card_set: Box::new(CardSet::Memory {
                memory: UseMemory::WithOwner {
                    memory: cs_key.to_string(),
                    owner: Box::new(Owner::Table),
                },
            }),
            pointmap: pointmap.to_string(),
        },
    }
}

#[test]
fn eval_int_aggregate_extrema_cardset_max() {
    let mut gd = GameData::new();
    let ace = gd.add_card(0, card_with(&[("rank", "Ace")]));
    let king = gd.add_card(0, card_with(&[("rank", "King")]));
    gd.set_memory(
        "Table_cs".to_string(),
        MemoryValue::CardSet(vec![ace, king]),
    );
    let mut pm = PointMap {
        name: "Pm".to_string(),
        map: HashMap::new(),
    };
    pm.map.insert("rank:Ace".to_string(), 10);
    pm.map.insert("rank:King".to_string(), 5);
    gd.point_maps.push(pm);

    assert_eq!(
        Evaluator::eval_int(&extrema_cardset(Extrema::Max, "cs", "Pm"), &gd).unwrap(),
        ace as i32
    );
}

#[test]
fn eval_int_aggregate_extrema_cardset_min() {
    let mut gd = GameData::new();
    let ace = gd.add_card(0, card_with(&[("rank", "Ace")]));
    let king = gd.add_card(0, card_with(&[("rank", "King")]));
    gd.set_memory(
        "Table_cs".to_string(),
        MemoryValue::CardSet(vec![ace, king]),
    );
    let mut pm = PointMap {
        name: "Pm".to_string(),
        map: HashMap::new(),
    };
    pm.map.insert("rank:Ace".to_string(), 10);
    pm.map.insert("rank:King".to_string(), 5);
    gd.point_maps.push(pm);

    assert_eq!(
        Evaluator::eval_int(&extrema_cardset(Extrema::Min, "cs", "Pm"), &gd).unwrap(),
        king as i32
    );
}

#[test]
fn eval_int_aggregate_extrema_cardset_empty() {
    let mut gd = GameData::new();
    gd.set_memory("Table_cs".to_string(), MemoryValue::CardSet(vec![]));
    let pm = PointMap {
        name: "Pm".to_string(),
        map: HashMap::new(),
    };
    gd.point_maps.push(pm);

    assert_eq!(
        Evaluator::eval_int(&extrema_cardset(Extrema::Max, "cs", "Pm"), &gd)
            .unwrap_err()
            .to_string(),
        "No card found for extrema".to_string()
    );
}

// ── I-12 ─────────────────────────────────────────────────────────
fn extrema_int_collection(extrema: Extrema, ints: Vec<IntExpr>) -> IntExpr {
    IntExpr::Aggregate {
        aggregate: AggregateInt::ExtremaIntCollection {
            extrema,
            int_collection: IntCollection::Literal { ints },
        },
    }
}

#[test]
fn eval_int_aggregate_extrema_int_collection_max() {
    assert_eq!(
        Evaluator::eval_int(
            &extrema_int_collection(Extrema::Max, vec![lit(5), lit(1), lit(3)]),
            &GameData::new()
        )
        .unwrap(),
        5
    );
}

#[test]
fn eval_int_aggregate_extrema_int_collection_min() {
    assert_eq!(
        Evaluator::eval_int(
            &extrema_int_collection(Extrema::Min, vec![lit(5), lit(1), lit(3)]),
            &GameData::new()
        )
        .unwrap(),
        1
    );
}

#[test]
fn eval_int_aggregate_extrema_int_collection_empty() {
    assert_eq!(
        Evaluator::eval_int(
            &extrema_int_collection(Extrema::Max, vec![]),
            &GameData::new()
        )
        .unwrap_err()
        .to_string(),
        "No value found in IntCollection".to_string()
    );
}

// ── I-13 ─────────────────────────────────────────────────────────
#[test]
fn eval_int_runtime_current_stage_round_counter() {
    let mut gd = GameData::new();
    gd.enter_stage("S1".to_string(), vec![]);
    gd.increment_stage_counter("S1".to_string());
    gd.increment_stage_counter("S1".to_string());
    let expr = IntExpr::Runtime {
        runtime: RuntimeInt::CurrentStageRoundCounter,
    };
    assert_eq!(Evaluator::eval_int(&expr, &gd).unwrap(), 2);
}

#[test]
fn eval_int_runtime_current_stage_round_counter_no_stage() {
    let gd = GameData::new();
    let expr = IntExpr::Runtime {
        runtime: RuntimeInt::CurrentStageRoundCounter,
    };
    assert_eq!(
        Evaluator::eval_int(&expr, &gd).unwrap_err().to_string(),
        "No current stage".to_string()
    );
}

// ── I-14 ─────────────────────────────────────────────────────────
#[test]
fn eval_int_runtime_stage_round_counter_named() {
    let mut gd = GameData::new();
    gd.stage_counters.insert("MyStage".to_string(), 7);
    let expr = IntExpr::Runtime {
        runtime: RuntimeInt::StageRoundCounter {
            stage: "MyStage".to_string(),
        },
    };
    assert_eq!(Evaluator::eval_int(&expr, &gd).unwrap(), 7);
}

#[test]
fn eval_int_runtime_stage_round_counter_absent() {
    let gd = GameData::new();
    let expr = IntExpr::Runtime {
        runtime: RuntimeInt::StageRoundCounter {
            stage: "Ghost".to_string(),
        },
    };
    assert_eq!(Evaluator::eval_int(&expr, &gd).unwrap(), 0);
}

// ── I-15 ─────────────────────────────────────────────────────────
fn memory_int(key: &str) -> IntExpr {
    IntExpr::Memory {
        memory: UseSingleMemory::WithOwner {
            memory: key.to_string(),
            owner: Box::new(SingleOwner::Table),
        },
    }
}

#[test]
fn eval_int_memory_int_present() {
    let mut gd = GameData::new();
    gd.set_memory("Table_k".to_string(), MemoryValue::Int(7));
    assert_eq!(Evaluator::eval_int(&memory_int("k"), &gd).unwrap(), 7);
}

#[test]
fn eval_int_memory_int_wrong_type() {
    let mut gd = GameData::new();
    gd.set_memory("Table_k".to_string(), MemoryValue::String("x".to_string()));
    assert_eq!(
        Evaluator::eval_int(&memory_int("k"), &gd)
            .unwrap_err()
            .to_string(),
        "Memory value is not an Int".to_string()
    );
}

#[test]
fn eval_int_memory_int_missing() {
    let gd = GameData::new();
    assert_eq!(
        Evaluator::eval_int(&memory_int("ghost"), &gd)
            .unwrap_err()
            .to_string(),
        "Memory Table_ghost not found".to_string()
    );
}

#[test]
fn eval_int_memory_no_owner_error() {
    let gd = GameData::new();
    let expr = IntExpr::Memory {
        memory: UseSingleMemory::Memory {
            memory: "M".to_string(),
        },
    };
    assert_eq!(
        Evaluator::eval_int(&expr, &gd).unwrap_err().to_string(),
        "memory access requires an explicit owner; use &M:M of <owner>".to_string()
    );
}

// ── I-16 ─────────────────────────────────────────────────────────
#[test]
fn eval_int_size_of_location_collection_memory_wrong_type() {
    let mut gd = GameData::new();
    gd.set_memory("Table_lc".to_string(), MemoryValue::Int(0));
    let expr = size_of(Collection::LocationCollection {
        location: LocationCollection::Memory {
            memory: UseMemory::WithOwner {
                memory: "lc".to_string(),
                owner: Box::new(Owner::Table),
            },
        },
    });
    assert_eq!(
        Evaluator::eval_int(&expr, &gd).unwrap_err().to_string(),
        "Memory value is not a LocationCollection".to_string()
    );
}

#[test]
fn eval_int_size_of_location_collection_memory_missing() {
    let gd = GameData::new();
    let expr = size_of(Collection::LocationCollection {
        location: LocationCollection::Memory {
            memory: UseMemory::WithOwner {
                memory: "ghost".to_string(),
                owner: Box::new(Owner::Table),
            },
        },
    });
    assert_eq!(
        Evaluator::eval_int(&expr, &gd).unwrap_err().to_string(),
        "Memory Table_ghost not found".to_string()
    );
}

// ── I-17 ─────────────────────────────────────────────────────────
#[test]
fn eval_int_size_of_team_collection_memory_wrong_type() {
    let mut gd = GameData::new();
    gd.set_memory("Table_tc".to_string(), MemoryValue::Int(0));
    let expr = size_of(Collection::TeamCollection {
        team: TeamCollection::Memory {
            memory: UseMemory::WithOwner {
                memory: "tc".to_string(),
                owner: Box::new(Owner::Table),
            },
        },
    });
    assert_eq!(
        Evaluator::eval_int(&expr, &gd).unwrap_err().to_string(),
        "Memory value is not a Team".to_string()
    );
}

#[test]
fn eval_int_size_of_team_collection_memory_missing() {
    let gd = GameData::new();
    let expr = size_of(Collection::TeamCollection {
        team: TeamCollection::Memory {
            memory: UseMemory::WithOwner {
                memory: "ghost".to_string(),
                owner: Box::new(Owner::Table),
            },
        },
    });
    assert_eq!(
        Evaluator::eval_int(&expr, &gd).unwrap_err().to_string(),
        "Memory Table_ghost not found".to_string()
    );
}

#[test]
fn eval_int_collection_memory_no_owner_error() {
    let gd = GameData::new();
    let expr = size_of(Collection::LocationCollection {
        location: LocationCollection::Memory {
            memory: UseMemory::Memory {
                memory: "M".to_string(),
            },
        },
    });
    assert_eq!(
        Evaluator::eval_int(&expr, &gd).unwrap_err().to_string(),
        "memory access requires an explicit owner; use &M:M of <owner>".to_string()
    );
}

// ── I-18 ─────────────────────────────────────────────────────────
#[test]
fn eval_int_size_of_int_collection_aggregate_memory() {
    // Implemented 2026-08: aggregates the slot across every owner.
    let mut gd = GameData::new();
    gd.add_player("P1".to_string());
    gd.add_player("P2".to_string());
    gd.memories.insert("P1_x".to_string(), MemoryValue::Int(5));
    gd.memories.insert("P2_x".to_string(), MemoryValue::Int(7));
    let expr = size_of(Collection::IntCollection {
        int: IntCollection::AggregateMemory {
            memory: "x".to_string(),
            multi: MultiOwner::PlayerCollection {
                player_collection: Box::new(PlayerCollection::Runtime {
                    runtime: RuntimePlayerCollection::PlayersIn,
                }),
            },
        },
    });
    assert_eq!(Evaluator::eval_int(&expr, &gd).unwrap(), 2);
}

#[test]
fn eval_int_sum_of_int_collection_aggregate_memory() {
    let mut gd = GameData::new();
    gd.add_player("P1".to_string());
    gd.add_player("P2".to_string());
    gd.memories.insert("P1_x".to_string(), MemoryValue::Int(5));
    gd.memories.insert("P2_x".to_string(), MemoryValue::Int(7));
    let expr = IntExpr::Aggregate {
        aggregate: AggregateInt::SumOfIntCollection {
            int_collection: IntCollection::AggregateMemory {
                memory: "x".to_string(),
                multi: MultiOwner::PlayerCollection {
                    player_collection: Box::new(PlayerCollection::Runtime {
                        runtime: RuntimePlayerCollection::PlayersIn,
                    }),
                },
            },
        },
    };
    assert_eq!(Evaluator::eval_int(&expr, &gd).unwrap(), 12);
}

#[test]
fn eval_int_size_of_int_collection_aggregate_memory_missing_errors() {
    let mut gd = GameData::new();
    gd.add_player("P1".to_string());
    let expr = size_of(Collection::IntCollection {
        int: IntCollection::AggregateMemory {
            memory: "x".to_string(),
            multi: MultiOwner::PlayerCollection {
                player_collection: Box::new(PlayerCollection::Runtime {
                    runtime: RuntimePlayerCollection::PlayersIn,
                }),
            },
        },
    });
    assert_eq!(
        Evaluator::eval_int(&expr, &gd).unwrap_err().to_string(),
        "Memory P1_x not found".to_string()
    );
}

// ── I-19 ─────────────────────────────────────────────────────────
#[test]
fn eval_int_size_of_team_collection_aggregate_memory() {
    // Implemented 2026-08: aggregates the slot across every owner.
    let mut gd = GameData::new();
    gd.add_player("P1".to_string());
    gd.add_player("P2".to_string());
    gd.memories
        .insert("P1_x".to_string(), MemoryValue::Team("T1".to_string()));
    gd.memories
        .insert("P2_x".to_string(), MemoryValue::Team("T2".to_string()));
    let expr = size_of(Collection::TeamCollection {
        team: TeamCollection::AggregateMemory {
            memory: "x".to_string(),
            multi: MultiOwner::PlayerCollection {
                player_collection: Box::new(PlayerCollection::Runtime {
                    runtime: RuntimePlayerCollection::PlayersIn,
                }),
            },
        },
    });
    assert_eq!(Evaluator::eval_int(&expr, &gd).unwrap(), 2);
}

// ── I-20 ─────────────────────────────────────────────────────────
#[test]
fn eval_int_size_of_string_collection_aggregate_memory() {
    // Implemented 2026-08: aggregates the slot across every owner.
    let mut gd = GameData::new();
    gd.add_player("P1".to_string());
    gd.add_player("P2".to_string());
    gd.memories
        .insert("P1_x".to_string(), MemoryValue::String("a".to_string()));
    gd.memories
        .insert("P2_x".to_string(), MemoryValue::String("b".to_string()));
    let expr = size_of(Collection::StringCollection {
        string: StringCollection::AggregateMemory {
            memory: "x".to_string(),
            multi: MultiOwner::PlayerCollection {
                player_collection: Box::new(PlayerCollection::Runtime {
                    runtime: RuntimePlayerCollection::PlayersIn,
                }),
            },
        },
    });
    assert_eq!(Evaluator::eval_int(&expr, &gd).unwrap(), 2);
}

// ── I-21 ─────────────────────────────────────────────────────────
#[test]
fn resolve_quantity_int() {
    let q = Quantity::Int { int: lit(3) };
    assert_eq!(
        Evaluator::resolve_quantity(&q, 10, &GameData::new()).unwrap(),
        3
    );
}

#[test]
fn resolve_quantity_int_clamps_to_available() {
    let q = Quantity::Int { int: lit(20) };
    assert_eq!(
        Evaluator::resolve_quantity(&q, 5, &GameData::new()).unwrap(),
        5
    );
}

#[test]
fn resolve_quantity_quantifier_all() {
    let q = Quantity::Quantifier {
        quantifier: Quantifier::All,
    };
    assert_eq!(
        Evaluator::resolve_quantity(&q, 5, &GameData::new()).unwrap(),
        5
    );
}

#[test]
fn resolve_quantity_quantifier_any() {
    let q = Quantity::Quantifier {
        quantifier: Quantifier::Any,
    };
    assert_eq!(
        Evaluator::resolve_quantity(&q, 5, &GameData::new()).unwrap(),
        1
    );
}

// ── I-22 ─────────────────────────────────────────────────────────
#[test]
fn resolve_quantity_int_range_start_satisfied() {
    let q = Quantity::IntRange {
        int_range: IntRange {
            start: (IntCompare::Ge, lit(3)),
            op_int: vec![],
        },
    };
    assert_eq!(
        Evaluator::resolve_quantity(&q, 5, &GameData::new()).unwrap(),
        5
    );
}

#[test]
fn resolve_quantity_int_range_start_not_satisfied() {
    let q = Quantity::IntRange {
        int_range: IntRange {
            start: (IntCompare::Ge, lit(10)),
            op_int: vec![],
        },
    };
    assert_eq!(
        Evaluator::resolve_quantity(&q, 5, &GameData::new()).unwrap(),
        0
    );
}

#[test]
fn resolve_quantity_int_range_and_chain() {
    let q = Quantity::IntRange {
        int_range: IntRange {
            start: (IntCompare::Ge, lit(3)),
            op_int: vec![(IntRangeOperator::And, IntCompare::Le, lit(10))],
        },
    };
    assert_eq!(
        Evaluator::resolve_quantity(&q, 5, &GameData::new()).unwrap(),
        5
    );
}

#[test]
fn resolve_quantity_int_range_and_fails() {
    let q = Quantity::IntRange {
        int_range: IntRange {
            start: (IntCompare::Ge, lit(3)),
            op_int: vec![(IntRangeOperator::And, IntCompare::Le, lit(4))],
        },
    };
    // available=5, 5 <= 4 is false - returns Ok(0)
    assert_eq!(
        Evaluator::resolve_quantity(&q, 5, &GameData::new()).unwrap(),
        0
    );
}

#[test]
fn resolve_quantity_int_range_or_chain() {
    let q = Quantity::IntRange {
        int_range: IntRange {
            start: (IntCompare::Ge, lit(3)),
            op_int: vec![(IntRangeOperator::Or, IntCompare::Eq, lit(7))],
        },
    };
    // available=5, 5 == 7 false - falls through to Ok(available)
    assert_eq!(
        Evaluator::resolve_quantity(&q, 5, &GameData::new()).unwrap(),
        5
    );
}

#[test]
fn resolve_quantity_int_range_or_satisfied() {
    let q = Quantity::IntRange {
        int_range: IntRange {
            start: (IntCompare::Ge, lit(3)),
            op_int: vec![(IntRangeOperator::Or, IntCompare::Eq, lit(5))],
        },
    };
    // available=5, 5 == 5 true - returns Ok(available)
    assert_eq!(
        Evaluator::resolve_quantity(&q, 5, &GameData::new()).unwrap(),
        5
    );
}

// ── I-23 ─────────────────────────────────────────────────────────
#[test]
fn resolve_quantity_runtime_memory_evaluated_live() {
    // Runtime-backed quantities are evaluated against the live GameData
    // (fixed 2026-08, was: empty GameData with a silent fallback of 1).
    let mut gd = GameData::new();
    gd.memories
        .insert("Table_secret".to_string(), MemoryValue::Int(3));
    let q = Quantity::Int {
        int: IntExpr::Memory {
            memory: UseSingleMemory::WithOwner {
                memory: "secret".to_string(),
                owner: Box::new(SingleOwner::Table),
            },
        },
    };
    assert_eq!(Evaluator::resolve_quantity(&q, 100, &gd).unwrap(), 3);
}

#[test]
fn resolve_quantity_runtime_memory_missing_errors() {
    let q = Quantity::Int {
        int: IntExpr::Memory {
            memory: UseSingleMemory::WithOwner {
                memory: "secret".to_string(),
                owner: Box::new(SingleOwner::Table),
            },
        },
    };
    // Missing memory now surfaces as an error instead of silently moving 1.
    assert_eq!(
        Evaluator::resolve_quantity(&q, 100, &GameData::new())
            .unwrap_err()
            .to_string(),
        "Memory Table_secret not found".to_string()
    );
}

// ── I-24 ─────────────────────────────────────────────────────────
#[test]
fn resolve_quantity_int_range_evaluated_live() {
    let mut gd = GameData::new();
    gd.memories
        .insert("Table_ghost".to_string(), MemoryValue::Int(2));
    let q = Quantity::IntRange {
        int_range: IntRange {
            start: (
                IntCompare::Ge,
                IntExpr::Memory {
                    memory: UseSingleMemory::WithOwner {
                        memory: "ghost".to_string(),
                        owner: Box::new(SingleOwner::Table),
                    },
                },
            ),
            op_int: vec![],
        },
    };
    // available=100 >= 2 - satisfied - Ok(100) (previously eval failed against
    // an empty GameData and fell back to Ok(0)).
    assert_eq!(Evaluator::resolve_quantity(&q, 100, &gd).unwrap(), 100);
}

#[test]
fn resolve_quantity_int_range_missing_memory_errors() {
    let q = Quantity::IntRange {
        int_range: IntRange {
            start: (
                IntCompare::Ge,
                IntExpr::Memory {
                    memory: UseSingleMemory::WithOwner {
                        memory: "ghost".to_string(),
                        owner: Box::new(SingleOwner::Table),
                    },
                },
            ),
            op_int: vec![],
        },
    };
    assert_eq!(
        Evaluator::resolve_quantity(&q, 100, &GameData::new())
            .unwrap_err()
            .to_string(),
        "Memory Table_ghost not found".to_string()
    );
}
