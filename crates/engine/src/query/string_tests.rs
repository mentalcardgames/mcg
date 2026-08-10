use super::*;
use crate::game_data::{GameData, Location, MemoryValue};
use front_end::ast::{
    CardPosition, IntExpr, MultiOwner, PlayerCollection, PlayerExpr, QueryCardPosition,
    QueryString, SingleOwner, StringCollection, StringExpr, Types, UseSingleMemory,
};
use std::collections::HashMap;

fn one_card_gd() -> (GameData, usize, usize) {
    let mut gd = GameData::new();
    let pid = gd.add_player("P1".to_string());
    gd.turn_order = vec![pid];
    let stock = gd.add_location(
        "Table".to_string(),
        Location {
            name: "Stock".to_string(),
            cards: vec![],
        },
    );
    let card = gd.add_card(
        stock,
        HashMap::from([
            ("Rank".to_string(), "Ace".to_string()),
            ("Suit".to_string(), "Hearts".to_string()),
        ]),
    );
    gd.locations[stock].cards = vec![card];
    (gd, stock, card)
}

#[test]
fn eval_string_literal_returns_value() {
    let gd = GameData::new();
    let expr = StringExpr::Literal {
        value: "hello".to_string(),
    };
    assert_eq!(
        Evaluator::eval_string(&expr, &gd).unwrap(),
        "hello".to_string()
    );
}

#[test]
fn eval_string_query_key_of_present() {
    let (gd, _stock, _card) = one_card_gd();
    let expr = StringExpr::Query {
        query: QueryString::KeyOf {
            key: "Rank".to_string(),
            card_position: CardPosition::Query {
                query: QueryCardPosition::Top {
                    location: "Stock".to_string(),
                },
            },
        },
    };
    assert_eq!(
        Evaluator::eval_string(&expr, &gd).unwrap(),
        "Ace".to_string()
    );
}

#[test]
fn eval_string_query_key_of_missing_key() {
    let (gd, _stock, card) = one_card_gd();
    let expr = StringExpr::Query {
        query: QueryString::KeyOf {
            key: "Color".to_string(),
            card_position: CardPosition::Query {
                query: QueryCardPosition::Top {
                    location: "Stock".to_string(),
                },
            },
        },
    };
    assert_eq!(
        Evaluator::eval_string(&expr, &gd).unwrap_err().to_string(),
        format!("Key Color not found in card {}", card)
    );
}

#[test]
fn eval_string_query_key_of_missing_card() {
    let (mut gd, _stock, _card) = one_card_gd();
    gd.cards.clear();
    let expr = StringExpr::Query {
        query: QueryString::KeyOf {
            key: "Rank".to_string(),
            card_position: CardPosition::Query {
                query: QueryCardPosition::Top {
                    location: "Stock".to_string(),
                },
            },
        },
    };
    assert_eq!(
        Evaluator::eval_string(&expr, &gd).unwrap_err().to_string(),
        "Card 0 not found".to_string()
    );
}

#[test]
fn eval_string_query_string_collection_at_in_range() {
    let gd = GameData::new();
    let expr = StringExpr::Query {
        query: QueryString::StringCollectionAt {
            string_collection: StringCollection::Literal {
                strings: vec![
                    StringExpr::Literal {
                        value: "a".to_string(),
                    },
                    StringExpr::Literal {
                        value: "b".to_string(),
                    },
                    StringExpr::Literal {
                        value: "c".to_string(),
                    },
                ],
            },
            int_expr: IntExpr::Literal { int: 1 },
        },
    };
    assert_eq!(Evaluator::eval_string(&expr, &gd).unwrap(), "b".to_string());
}

#[test]
fn eval_string_query_string_collection_at_out_of_range() {
    let gd = GameData::new();
    let expr = StringExpr::Query {
        query: QueryString::StringCollectionAt {
            string_collection: StringCollection::Literal {
                strings: vec![
                    StringExpr::Literal {
                        value: "a".to_string(),
                    },
                    StringExpr::Literal {
                        value: "b".to_string(),
                    },
                    StringExpr::Literal {
                        value: "c".to_string(),
                    },
                ],
            },
            int_expr: IntExpr::Literal { int: 5 },
        },
    };
    assert_eq!(
        Evaluator::eval_string(&expr, &gd).unwrap_err().to_string(),
        "No string at index 5".to_string()
    );
}

#[test]
fn eval_string_memory_string() {
    let mut gd = GameData::new();
    gd.memories.insert(
        "Table_m".to_string(),
        MemoryValue::String("hello".to_string()),
    );
    let expr = StringExpr::Memory {
        memory: UseSingleMemory::WithOwner {
            memory: "m".to_string(),
            owner: Box::new(SingleOwner::Table),
        },
    };
    assert_eq!(
        Evaluator::eval_string(&expr, &gd).unwrap(),
        "hello".to_string()
    );
}

#[test]
fn eval_string_memory_wrong_type() {
    let mut gd = GameData::new();
    gd.memories
        .insert("Table_m".to_string(), MemoryValue::Int(0));
    let expr = StringExpr::Memory {
        memory: UseSingleMemory::WithOwner {
            memory: "m".to_string(),
            owner: Box::new(SingleOwner::Table),
        },
    };
    assert_eq!(
        Evaluator::eval_string(&expr, &gd).unwrap_err().to_string(),
        "Memory value is not a String".to_string()
    );
}

#[test]
fn eval_string_memory_missing() {
    let gd = GameData::new();
    let expr = StringExpr::Memory {
        memory: UseSingleMemory::WithOwner {
            memory: "m".to_string(),
            owner: Box::new(SingleOwner::Table),
        },
    };
    assert_eq!(
        Evaluator::eval_string(&expr, &gd).unwrap_err().to_string(),
        "Memory Table_m not found".to_string()
    );
}

#[test]
fn eval_string_memory_no_owner_error() {
    let gd = GameData::new();
    let expr = StringExpr::Memory {
        memory: UseSingleMemory::Memory {
            memory: "M".to_string(),
        },
    };
    assert_eq!(
        Evaluator::eval_string(&expr, &gd).unwrap_err().to_string(),
        "memory access requires an explicit owner; use &M:M of <owner>".to_string()
    );
}

#[test]
fn eval_string_collection_at_aggregate_memory() {
    // Implemented 2026-08: aggregates the slot across every owner.
    let mut gd = GameData::new();
    gd.add_player("P1".to_string());
    gd.memories
        .insert("P1_m".to_string(), MemoryValue::String("ace".to_string()));
    let expr = StringExpr::Query {
        query: QueryString::StringCollectionAt {
            string_collection: StringCollection::AggregateMemory {
                memory: "m".to_string(),
                multi: MultiOwner::PlayerCollection {
                    player_collection: Box::new(PlayerCollection::Literal {
                        players: vec![PlayerExpr::Literal {
                            name: "P1".to_string(),
                        }],
                    }),
                },
            },
            int_expr: IntExpr::Literal { int: 0 },
        },
    };
    assert_eq!(
        Evaluator::eval_string(&expr, &gd).unwrap(),
        "ace".to_string()
    );
}

#[test]
fn eval_string_collection_at_aggregate_memory_missing_errors() {
    let mut gd = GameData::new();
    gd.add_player("P1".to_string());
    let expr = StringExpr::Query {
        query: QueryString::StringCollectionAt {
            string_collection: StringCollection::AggregateMemory {
                memory: "m".to_string(),
                multi: MultiOwner::PlayerCollection {
                    player_collection: Box::new(PlayerCollection::Literal {
                        players: vec![PlayerExpr::Literal {
                            name: "P1".to_string(),
                        }],
                    }),
                },
            },
            int_expr: IntExpr::Literal { int: 0 },
        },
    };
    assert_eq!(
        Evaluator::eval_string(&expr, &gd).unwrap_err().to_string(),
        "Memory P1_m not found".to_string()
    );
}

#[test]
fn expand_types_single_axis() {
    let types = Types {
        types: vec![(
            "Rank".to_string(),
            vec!["Ace".to_string(), "Two".to_string()],
        )],
    };
    let cards = Evaluator::expand_types(&types);
    assert_eq!(cards.len(), 2);
    assert!(cards.iter().all(|c| c.contains_key("Rank")));
    let ranks: Vec<&str> = cards.iter().map(|c| c["Rank"].as_str()).collect();
    assert!(ranks.contains(&"Ace"));
    assert!(ranks.contains(&"Two"));
}

#[test]
fn expand_types_two_axis_cartesian() {
    let types = Types {
        types: vec![
            (
                "Rank".to_string(),
                vec!["Ace".to_string(), "Two".to_string()],
            ),
            (
                "Suit".to_string(),
                vec![
                    "Hearts".to_string(),
                    "Spades".to_string(),
                    "Clubs".to_string(),
                ],
            ),
        ],
    };
    let cards = Evaluator::expand_types(&types);
    assert_eq!(cards.len(), 6);
    for card in &cards {
        assert!(card.contains_key("Rank"));
        assert!(card.contains_key("Suit"));
    }
}

#[test]
fn expand_types_empty_returns_one_empty_card() {
    let types = Types { types: vec![] };
    let cards = Evaluator::expand_types(&types);
    assert_eq!(cards, vec![HashMap::new()]);
}
