use super::*;
use crate::game_data::{Card, Combo, GameData, Location, MemoryValue, PointMap, Precedence};
use front_end::ast::{
    AggregateCardPosition, AggregateFilter, CardPosition, CardSet, Extrema, FilterExpr, Group,
    Groupable, IntCompare, IntExpr, Owner, QueryCardPosition, StringExpr, UseMemory,
};
use std::collections::HashMap;

fn make_card(attrs: Vec<(&str, &str)>) -> Card {
    let mut card = HashMap::new();
    for (k, v) in attrs {
        card.insert(k.to_string(), v.to_string());
    }
    card
}

fn basic_fixture() -> GameData {
    let mut gd = GameData::new();
    let p0 = gd.add_player("P1".to_string());
    gd.add_player("P2".to_string());
    gd.turn_order = vec![p0, 1];
    let stock = gd.add_location(
        "Table".to_string(),
        Location {
            name: "Stock".to_string(),
            cards: vec![],
        },
    );
    let c0 = gd.add_card(stock, make_card(vec![("Rank", "Ace"), ("Suit", "Hearts")]));
    let c1 = gd.add_card(stock, make_card(vec![("Rank", "King"), ("Suit", "Hearts")]));
    gd.locations[stock].cards = vec![c0, c1];
    gd
}

fn empty_location_fixture() -> GameData {
    let mut gd = GameData::new();
    let p0 = gd.add_player("P1".to_string());
    gd.turn_order = vec![p0];
    gd.add_location(
        "Table".to_string(),
        Location {
            name: "EmptyLoc".to_string(),
            cards: vec![],
        },
    );
    gd
}

#[test]
fn eval_cardset_group_location() {
    let gd = basic_fixture();
    let expr = CardSet::Group {
        group: Group::Groupable {
            groupable: Groupable::Location {
                name: "Stock".to_string(),
            },
        },
    };
    let result = Evaluator::eval_cardset(&expr, &gd);
    assert_eq!(result.unwrap(), (0, vec![0, 1]));
}

#[test]
fn eval_cardset_group_where_filter_size() {
    let gd = basic_fixture();
    let expr = CardSet::Group {
        group: Group::Where {
            groupable: Groupable::Location {
                name: "Stock".to_string(),
            },
            filter: FilterExpr::Aggregate {
                aggregate: AggregateFilter::Size {
                    cmp: IntCompare::Eq,
                    int_expr: Box::new(IntExpr::Literal { int: 2 }),
                },
            },
        },
    };
    let result = Evaluator::eval_cardset(&expr, &gd);
    assert_eq!(result.unwrap(), (0, vec![0, 1]));
}

#[test]
fn eval_cardset_group_where_filter_same() {
    let mut gd = GameData::new();
    let p0 = gd.add_player("P1".to_string());
    gd.turn_order = vec![p0];
    let stock = gd.add_location(
        "Table".to_string(),
        Location {
            name: "Stock".to_string(),
            cards: vec![],
        },
    );
    let c0 = gd.add_card(stock, make_card(vec![("Rank", "Ace"), ("Suit", "Hearts")]));
    let c1 = gd.add_card(stock, make_card(vec![("Rank", "Ace"), ("Suit", "Spades")]));
    let c2 = gd.add_card(stock, make_card(vec![("Rank", "King"), ("Suit", "Clubs")]));
    gd.locations[stock].cards = vec![c0, c1, c2];

    let expr = CardSet::Group {
        group: Group::Where {
            groupable: Groupable::Location {
                name: "Stock".to_string(),
            },
            filter: FilterExpr::Aggregate {
                aggregate: AggregateFilter::Same {
                    key: "Rank".to_string(),
                },
            },
        },
    };
    let result = Evaluator::eval_cardset(&expr, &gd);
    assert!(result.is_ok());
    let (loc, cards) = result.unwrap();
    assert_eq!(loc, stock);
    assert_eq!(cards.len(), 2);
    assert!(cards.contains(&c0) && cards.contains(&c1));
}

#[test]
fn eval_cardset_group_where_filter_distinct() {
    let mut gd = GameData::new();
    let p0 = gd.add_player("P1".to_string());
    gd.turn_order = vec![p0];
    let stock = gd.add_location(
        "Table".to_string(),
        Location {
            name: "Stock".to_string(),
            cards: vec![],
        },
    );
    let c0 = gd.add_card(stock, make_card(vec![("Rank", "Ace"), ("Suit", "Hearts")]));
    let c1 = gd.add_card(stock, make_card(vec![("Rank", "Ace"), ("Suit", "Spades")]));
    let c2 = gd.add_card(stock, make_card(vec![("Rank", "King"), ("Suit", "Clubs")]));
    gd.locations[stock].cards = vec![c0, c1, c2];

    let expr = CardSet::Group {
        group: Group::Where {
            groupable: Groupable::Location {
                name: "Stock".to_string(),
            },
            filter: FilterExpr::Aggregate {
                aggregate: AggregateFilter::Distinct {
                    key: "Rank".to_string(),
                },
            },
        },
    };
    let result = Evaluator::eval_cardset(&expr, &gd);
    assert!(result.is_ok());
    let (loc, cards) = result.unwrap();
    assert_eq!(loc, stock);
    assert_eq!(cards.len(), 1);
    assert!(cards.contains(&c2));
}

#[test]
fn eval_cardset_group_where_filter_key_is_string() {
    let gd = basic_fixture();
    let expr = CardSet::Group {
        group: Group::Where {
            groupable: Groupable::Location {
                name: "Stock".to_string(),
            },
            filter: FilterExpr::Aggregate {
                aggregate: AggregateFilter::KeyIsString {
                    key: "Rank".to_string(),
                    string: Box::new(StringExpr::Literal {
                        value: "Ace".to_string(),
                    }),
                },
            },
        },
    };
    let result = Evaluator::eval_cardset(&expr, &gd);
    assert!(result.is_ok());
    let (_, cards) = result.unwrap();
    assert_eq!(cards, vec![0]);
}

#[test]
fn eval_cardset_group_where_filter_key_is_not_string() {
    let gd = basic_fixture();
    let expr = CardSet::Group {
        group: Group::Where {
            groupable: Groupable::Location {
                name: "Stock".to_string(),
            },
            filter: FilterExpr::Aggregate {
                aggregate: AggregateFilter::KeyIsNotString {
                    key: "Rank".to_string(),
                    string: Box::new(StringExpr::Literal {
                        value: "Ace".to_string(),
                    }),
                },
            },
        },
    };
    let result = Evaluator::eval_cardset(&expr, &gd);
    assert!(result.is_ok());
    let (_, cards) = result.unwrap();
    assert_eq!(cards, vec![1]);
}

#[test]
fn eval_cardset_group_where_filter_adjacent_missing_precedence() {
    let gd = basic_fixture();
    let expr = CardSet::Group {
        group: Group::Where {
            groupable: Groupable::Location {
                name: "Stock".to_string(),
            },
            filter: FilterExpr::Aggregate {
                aggregate: AggregateFilter::Adjacent {
                    key: "Rank".to_string(),
                    precedence: "NoSuchPrecedence".to_string(),
                },
            },
        },
    };
    let result = Evaluator::eval_cardset(&expr, &gd);
    assert_eq!(
        result.unwrap_err().to_string(),
        "Precedence NoSuchPrecedence not found".to_string()
    );
}

#[test]
fn eval_cardset_group_where_filter_higher_missing_precedence() {
    let gd = basic_fixture();
    let expr = CardSet::Group {
        group: Group::Where {
            groupable: Groupable::Location {
                name: "Stock".to_string(),
            },
            filter: FilterExpr::Aggregate {
                aggregate: AggregateFilter::Higher {
                    key: "Rank".to_string(),
                    value: StringExpr::Literal {
                        value: "Ace".to_string(),
                    },
                    precedence: "NoSuchPrecedence".to_string(),
                },
            },
        },
    };
    let result = Evaluator::eval_cardset(&expr, &gd);
    assert_eq!(
        result.unwrap_err().to_string(),
        "Precedence NoSuchPrecedence not found".to_string()
    );
}

#[test]
fn eval_cardset_group_where_filter_lower_missing_precedence() {
    let gd = basic_fixture();
    let expr = CardSet::Group {
        group: Group::Where {
            groupable: Groupable::Location {
                name: "Stock".to_string(),
            },
            filter: FilterExpr::Aggregate {
                aggregate: AggregateFilter::Lower {
                    key: "Rank".to_string(),
                    value: StringExpr::Literal {
                        value: "Ace".to_string(),
                    },
                    precedence: "NoSuchPrecedence".to_string(),
                },
            },
        },
    };
    let result = Evaluator::eval_cardset(&expr, &gd);
    assert_eq!(
        result.unwrap_err().to_string(),
        "Precedence NoSuchPrecedence not found".to_string()
    );
}

/// Stock holding Ace, Queen, King, and a Joker (value not in the
/// precedence), plus a `RankOrder` precedence Ace < Queen < King.
fn ranked_fixture() -> GameData {
    let mut gd = GameData::new();
    let p0 = gd.add_player("P1".to_string());
    gd.turn_order = vec![p0];
    let stock = gd.add_location(
        "Table".to_string(),
        Location {
            name: "Stock".to_string(),
            cards: vec![],
        },
    );
    let c0 = gd.add_card(stock, make_card(vec![("Rank", "Ace"), ("Suit", "Hearts")]));
    let c1 = gd.add_card(
        stock,
        make_card(vec![("Rank", "Queen"), ("Suit", "Spades")]),
    );
    let c2 = gd.add_card(stock, make_card(vec![("Rank", "King"), ("Suit", "Clubs")]));
    let c3 = gd.add_card(stock, make_card(vec![("Rank", "Joker"), ("Suit", "Wild")]));
    gd.locations[stock].cards = vec![c0, c1, c2, c3];
    gd.precedences.push(Precedence {
        name: "RankOrder".to_string(),
        key: "Rank".to_string(),
        values: vec!["Ace".to_string(), "Queen".to_string(), "King".to_string()],
    });
    gd
}

#[test]
fn eval_cardset_group_where_filter_adjacent_matches_consecutive_values() {
    let gd = ranked_fixture();
    let expr = CardSet::Group {
        group: Group::Where {
            groupable: Groupable::Location {
                name: "Stock".to_string(),
            },
            filter: FilterExpr::Aggregate {
                aggregate: AggregateFilter::Adjacent {
                    key: "Rank".to_string(),
                    precedence: "RankOrder".to_string(),
                },
            },
        },
    };
    let result = Evaluator::eval_cardset(&expr, &gd);
    let (loc, cards) = result.unwrap();
    assert_eq!(loc, 0);
    // (Ace,Queen) and (Queen,King) are consecutive; Joker is not in the
    // precedence and is excluded. Pairs are deduplicated.
    assert_eq!(cards, vec![0, 1, 2]);
}

#[test]
fn eval_cardset_group_where_filter_higher_matches_above_target() {
    let gd = ranked_fixture();
    let expr = CardSet::Group {
        group: Group::Where {
            groupable: Groupable::Location {
                name: "Stock".to_string(),
            },
            filter: FilterExpr::Aggregate {
                aggregate: AggregateFilter::Higher {
                    key: "Rank".to_string(),
                    value: StringExpr::Literal {
                        value: "Ace".to_string(),
                    },
                    precedence: "RankOrder".to_string(),
                },
            },
        },
    };
    let result = Evaluator::eval_cardset(&expr, &gd);
    let (_, cards) = result.unwrap();
    assert_eq!(cards, vec![1, 2], "Queen and King are above Ace");
}

#[test]
fn eval_cardset_group_where_filter_lower_matches_below_target() {
    let gd = ranked_fixture();
    let expr = CardSet::Group {
        group: Group::Where {
            groupable: Groupable::Location {
                name: "Stock".to_string(),
            },
            filter: FilterExpr::Aggregate {
                aggregate: AggregateFilter::Lower {
                    key: "Rank".to_string(),
                    value: StringExpr::Literal {
                        value: "King".to_string(),
                    },
                    precedence: "RankOrder".to_string(),
                },
            },
        },
    };
    let result = Evaluator::eval_cardset(&expr, &gd);
    let (_, cards) = result.unwrap();
    assert_eq!(cards, vec![0, 1], "Ace and Queen are below King");
}

#[test]
fn eval_cardset_group_where_filter_size_mismatch_returns_empty() {
    let gd = basic_fixture();
    let expr = CardSet::Group {
        group: Group::Where {
            groupable: Groupable::Location {
                name: "Stock".to_string(),
            },
            filter: FilterExpr::Aggregate {
                aggregate: AggregateFilter::Size {
                    cmp: IntCompare::Eq,
                    int_expr: Box::new(IntExpr::Literal { int: 3 }),
                },
            },
        },
    };
    let result = Evaluator::eval_cardset(&expr, &gd);
    let (_, cards) = result.unwrap();
    assert!(cards.is_empty(), "size 3 does not match a 2-card pile");
}

fn combo_fixture() -> GameData {
    let mut gd = GameData::new();
    let p0 = gd.add_player("P1".to_string());
    gd.turn_order = vec![p0];
    let stock = gd.add_location(
        "Table".to_string(),
        Location {
            name: "Stock".to_string(),
            cards: vec![],
        },
    );
    let c0 = gd.add_card(stock, make_card(vec![("Rank", "Ace"), ("Suit", "Hearts")]));
    let c1 = gd.add_card(stock, make_card(vec![("Rank", "King"), ("Suit", "Spades")]));
    let c2 = gd.add_card(stock, make_card(vec![("Rank", "Ace"), ("Suit", "Clubs")]));
    gd.locations[stock].cards = vec![c0, c1, c2];

    gd.combos.push(Combo {
        name: "UniqueRank".to_string(),
        filter: FilterExpr::Aggregate {
            aggregate: AggregateFilter::Distinct {
                key: "Rank".to_string(),
            },
        },
    });
    gd
}

#[test]
fn eval_cardset_group_combo_filter() {
    let gd = combo_fixture();
    let expr = CardSet::Group {
        group: Group::Combo {
            combo: "UniqueRank".to_string(),
            groupable: Groupable::Location {
                name: "Stock".to_string(),
            },
        },
    };
    let result = Evaluator::eval_cardset(&expr, &gd);
    assert!(result.is_ok());
    let (_, cards) = result.unwrap();
    assert_eq!(cards, vec![1]);
}

#[test]
fn eval_cardset_group_not_combo() {
    let gd = combo_fixture();
    let expr = CardSet::Group {
        group: Group::NotCombo {
            combo: "UniqueRank".to_string(),
            groupable: Groupable::Location {
                name: "Stock".to_string(),
            },
        },
    };
    let result = Evaluator::eval_cardset(&expr, &gd);
    assert!(result.is_ok());
    let (_, cards) = result.unwrap();
    assert_eq!(cards, vec![0, 2]);
}

#[test]
fn eval_cardset_group_missing_combo() {
    let gd = combo_fixture();
    let expr = CardSet::Group {
        group: Group::Combo {
            combo: "NoSuchCombo".to_string(),
            groupable: Groupable::Location {
                name: "Stock".to_string(),
            },
        },
    };
    let result = Evaluator::eval_cardset(&expr, &gd);
    assert_eq!(
        result.unwrap_err().to_string(),
        "Combo NoSuchCombo not found".to_string()
    );
}

#[test]
fn eval_cardset_group_owner() {
    let mut gd = GameData::new();
    let p0 = gd.add_player("P1".to_string());
    gd.add_player("P2".to_string());
    gd.turn_order = vec![p0, 1];
    let stock = gd.add_location(
        "Table".to_string(),
        Location {
            name: "Stock".to_string(),
            cards: vec![],
        },
    );
    let c0 = gd.add_card(stock, make_card(vec![("Rank", "Ace")]));
    let c1 = gd.add_card(stock, make_card(vec![("Rank", "King")]));
    gd.locations[stock].cards = vec![c0, c1];

    let expr = CardSet::GroupOwner {
        group: Group::Groupable {
            groupable: Groupable::Location {
                name: "Stock".to_string(),
            },
        },
        owner: Owner::Table,
    };
    let result = Evaluator::eval_cardset(&expr, &gd);
    assert_eq!(result.unwrap(), (stock, vec![c0, c1]));
}

#[test]
fn eval_cardset_memory_cardset() {
    let mut gd = basic_fixture();
    gd.memories
        .insert("Table_mycs".to_string(), MemoryValue::CardSet(vec![0, 1]));
    let expr = CardSet::Memory {
        memory: UseMemory::WithOwner {
            memory: "mycs".to_string(),
            owner: Box::new(Owner::Table),
        },
    };
    let result = Evaluator::eval_cardset(&expr, &gd);
    assert!(result.is_ok());
    let (loc, cards) = result.unwrap();
    assert_eq!(cards, vec![0, 1]);
    assert_eq!(loc, 0);
}

#[test]
fn eval_cardset_memory_wrong_type() {
    let mut gd = basic_fixture();
    gd.memories
        .insert("Table_notcs".to_string(), MemoryValue::Int(0));
    let expr = CardSet::Memory {
        memory: UseMemory::WithOwner {
            memory: "notcs".to_string(),
            owner: Box::new(Owner::Table),
        },
    };
    let result = Evaluator::eval_cardset(&expr, &gd);
    assert_eq!(
        result.unwrap_err().to_string(),
        "Memory value is not a CardSet".to_string()
    );
}

#[test]
fn eval_cardset_memory_missing() {
    let gd = basic_fixture();
    let expr = CardSet::Memory {
        memory: UseMemory::WithOwner {
            memory: "ghost".to_string(),
            owner: Box::new(Owner::Table),
        },
    };
    let result = Evaluator::eval_cardset(&expr, &gd);
    assert_eq!(
        result.unwrap_err().to_string(),
        "Memory Table_ghost not found".to_string()
    );
}

#[test]
fn eval_cardset_memory_no_owner_error() {
    let gd = basic_fixture();
    let expr = CardSet::Memory {
        memory: UseMemory::Memory {
            memory: "M".to_string(),
        },
    };
    let result = Evaluator::eval_cardset(&expr, &gd);
    assert_eq!(
        result.unwrap_err().to_string(),
        "memory access requires an explicit owner; use &M:M of <owner>".to_string()
    );
}

#[test]
fn eval_cardset_memory_orphaned_cards_sentinel_location() {
    let mut gd = GameData::new();
    let p0 = gd.add_player("P1".to_string());
    gd.turn_order = vec![p0];
    let stock = gd.add_location(
        "Table".to_string(),
        Location {
            name: "Stock".to_string(),
            cards: vec![],
        },
    );
    let orphan = gd.add_card(stock, make_card(vec![("Rank", "Ace")]));
    gd.memories.insert(
        "Table_orphans".to_string(),
        MemoryValue::CardSet(vec![orphan]),
    );
    let expr = CardSet::Memory {
        memory: UseMemory::WithOwner {
            memory: "orphans".to_string(),
            owner: Box::new(Owner::Table),
        },
    };
    let result = Evaluator::eval_cardset(&expr, &gd);
    assert_eq!(result.unwrap(), (0, vec![orphan]));
}

#[test]
fn eval_cardset_no_card_position_not_found() {
    let gd = empty_location_fixture();
    let expr = CardSet::Group {
        group: Group::CardPosition {
            card_position: CardPosition::Query {
                query: QueryCardPosition::At {
                    location: "EmptyLoc".to_string(),
                    int_expr: IntExpr::Literal { int: 0 },
                },
            },
        },
    };
    let result = Evaluator::eval_cardset(&expr, &gd);
    assert_eq!(result.unwrap(), (0, vec![]));
}

#[test]
fn eval_card_position_at_in_range() {
    let gd = basic_fixture();
    let expr = CardPosition::Query {
        query: QueryCardPosition::At {
            location: "Stock".to_string(),
            int_expr: IntExpr::Literal { int: 0 },
        },
    };
    let result = Evaluator::eval_card_position(&expr, &gd);
    assert_eq!(result.unwrap(), 0);
}

#[test]
fn eval_card_position_at_out_of_range() {
    let gd = basic_fixture();
    let expr = CardPosition::Query {
        query: QueryCardPosition::At {
            location: "Stock".to_string(),
            int_expr: IntExpr::Literal { int: 99 },
        },
    };
    let result = Evaluator::eval_card_position(&expr, &gd);
    assert_eq!(
        result.unwrap_err().to_string(),
        "No card at index 99 in location Stock".to_string()
    );
}

#[test]
fn eval_card_position_at_missing_loc() {
    let gd = basic_fixture();
    let expr = CardPosition::Query {
        query: QueryCardPosition::At {
            location: "MissingLoc".to_string(),
            int_expr: IntExpr::Literal { int: 0 },
        },
    };
    let result = Evaluator::eval_card_position(&expr, &gd);
    assert_eq!(
        result.unwrap_err().to_string(),
        "Location MissingLoc not found".to_string()
    );
}

#[test]
fn eval_card_position_top_present() {
    let gd = basic_fixture();
    let expr = CardPosition::Query {
        query: QueryCardPosition::Top {
            location: "Stock".to_string(),
        },
    };
    let result = Evaluator::eval_card_position(&expr, &gd);
    assert_eq!(result.unwrap(), 0);
}

#[test]
fn eval_card_position_top_empty() {
    let gd = empty_location_fixture();
    let expr = CardPosition::Query {
        query: QueryCardPosition::Top {
            location: "EmptyLoc".to_string(),
        },
    };
    let result = Evaluator::eval_card_position(&expr, &gd);
    assert_eq!(
        result.unwrap_err().to_string(),
        "No card at top of location EmptyLoc".to_string()
    );
}

#[test]
fn eval_card_position_top_missing_loc() {
    let gd = empty_location_fixture();
    let expr = CardPosition::Query {
        query: QueryCardPosition::Top {
            location: "MissingLoc".to_string(),
        },
    };
    let result = Evaluator::eval_card_position(&expr, &gd);
    assert_eq!(
        result.unwrap_err().to_string(),
        "Location MissingLoc not found".to_string()
    );
}

#[test]
fn eval_card_position_bottom_present() {
    let gd = basic_fixture();
    let expr = CardPosition::Query {
        query: QueryCardPosition::Bottom {
            location: "Stock".to_string(),
        },
    };
    let result = Evaluator::eval_card_position(&expr, &gd);
    assert_eq!(result.unwrap(), 1);
}

#[test]
fn eval_card_position_bottom_empty() {
    let gd = empty_location_fixture();
    let expr = CardPosition::Query {
        query: QueryCardPosition::Bottom {
            location: "EmptyLoc".to_string(),
        },
    };
    let result = Evaluator::eval_card_position(&expr, &gd);
    assert_eq!(
        result.unwrap_err().to_string(),
        "No card at bottom of location EmptyLoc".to_string()
    );
}

#[test]
fn eval_card_position_bottom_missing_loc() {
    let gd = empty_location_fixture();
    let expr = CardPosition::Query {
        query: QueryCardPosition::Bottom {
            location: "MissingLoc".to_string(),
        },
    };
    let result = Evaluator::eval_card_position(&expr, &gd);
    assert_eq!(
        result.unwrap_err().to_string(),
        "Location MissingLoc not found".to_string()
    );
}

#[test]
fn infer_location_from_cards_infallible() {
    let mut gd = GameData::new();
    let p0 = gd.add_player("P1".to_string());
    gd.turn_order = vec![p0];
    let stock = gd.add_location(
        "Table".to_string(),
        Location {
            name: "Stock".to_string(),
            cards: vec![],
        },
    );
    let orphan = gd.add_card(stock, make_card(vec![("Rank", "Ace")]));
    gd.memories.insert(
        "Table_orphans".to_string(),
        MemoryValue::CardSet(vec![orphan]),
    );
    let expr = CardSet::Memory {
        memory: UseMemory::WithOwner {
            memory: "orphans".to_string(),
            owner: Box::new(Owner::Table),
        },
    };
    let result = Evaluator::eval_cardset(&expr, &gd);
    assert_eq!(result.unwrap(), (0, vec![orphan]));
}

fn pointmap_fixture() -> GameData {
    let mut gd = GameData::new();
    let p0 = gd.add_player("P1".to_string());
    gd.turn_order = vec![p0];
    let stock = gd.add_location(
        "Table".to_string(),
        Location {
            name: "Stock".to_string(),
            cards: vec![],
        },
    );
    let c0 = gd.add_card(stock, make_card(vec![("Rank", "Ace"), ("Suit", "Hearts")]));
    let c1 = gd.add_card(stock, make_card(vec![("Rank", "King"), ("Suit", "Spades")]));
    gd.locations[stock].cards = vec![c0, c1];

    let mut map = HashMap::new();
    map.insert("Rank:Ace".to_string(), 10);
    map.insert("Rank:King".to_string(), 5);
    gd.point_maps.push(PointMap {
        name: "MyPoints".to_string(),
        map,
    });
    gd
}

#[test]
fn eval_card_position_extrema_point_map_max() {
    let gd = pointmap_fixture();
    let expr = CardPosition::Aggregate {
        aggregate: AggregateCardPosition::ExtremaPointMap {
            extrema: Extrema::Max,
            card_set: Box::new(CardSet::Group {
                group: Group::Groupable {
                    groupable: Groupable::Location {
                        name: "Stock".to_string(),
                    },
                },
            }),
            pointmap: "MyPoints".to_string(),
        },
    };
    let result = Evaluator::eval_card_position(&expr, &gd);
    assert_eq!(result.unwrap(), 0);
}

#[test]
fn eval_card_position_extrema_point_map_min() {
    let gd = pointmap_fixture();
    let expr = CardPosition::Aggregate {
        aggregate: AggregateCardPosition::ExtremaPointMap {
            extrema: Extrema::Min,
            card_set: Box::new(CardSet::Group {
                group: Group::Groupable {
                    groupable: Groupable::Location {
                        name: "Stock".to_string(),
                    },
                },
            }),
            pointmap: "MyPoints".to_string(),
        },
    };
    let result = Evaluator::eval_card_position(&expr, &gd);
    assert_eq!(result.unwrap(), 1);
}

#[test]
fn eval_card_position_extrema_point_map_missing() {
    let gd = pointmap_fixture();
    let expr = CardPosition::Aggregate {
        aggregate: AggregateCardPosition::ExtremaPointMap {
            extrema: Extrema::Max,
            card_set: Box::new(CardSet::Group {
                group: Group::Groupable {
                    groupable: Groupable::Location {
                        name: "Stock".to_string(),
                    },
                },
            }),
            pointmap: "NoSuchPointMap".to_string(),
        },
    };
    let result = Evaluator::eval_card_position(&expr, &gd);
    assert_eq!(
        result.unwrap_err().to_string(),
        "PointMap NoSuchPointMap not found".to_string()
    );
}

#[test]
fn eval_card_position_extrema_point_map_empty() {
    let mut gd = GameData::new();
    let p0 = gd.add_player("P1".to_string());
    gd.turn_order = vec![p0];
    gd.add_location(
        "Table".to_string(),
        Location {
            name: "EmptyLoc".to_string(),
            cards: vec![],
        },
    );
    gd.memories
        .insert("Table_emptycs".to_string(), MemoryValue::CardSet(vec![]));
    let expr = CardPosition::Aggregate {
        aggregate: AggregateCardPosition::ExtremaPointMap {
            extrema: Extrema::Max,
            card_set: Box::new(CardSet::Memory {
                memory: UseMemory::WithOwner {
                    memory: "emptycs".to_string(),
                    owner: Box::new(Owner::Table),
                },
            }),
            pointmap: "NoSuchPointMap".to_string(),
        },
    };

    let result = Evaluator::eval_card_position(&expr, &gd);
    assert_eq!(
        result.unwrap_err().to_string(),
        "PointMap NoSuchPointMap not found".to_string()
    );
}

fn precedence_fixture() -> GameData {
    let mut gd = GameData::new();
    let p0 = gd.add_player("P1".to_string());
    gd.turn_order = vec![p0];
    let stock = gd.add_location(
        "Table".to_string(),
        Location {
            name: "Stock".to_string(),
            cards: vec![],
        },
    );
    let c0 = gd.add_card(stock, make_card(vec![("Rank", "Ace"), ("Suit", "Hearts")]));
    let c1 = gd.add_card(stock, make_card(vec![("Rank", "King"), ("Suit", "Spades")]));
    gd.locations[stock].cards = vec![c0, c1];

    gd.precedences.push(Precedence {
        name: "RankOrder".to_string(),
        key: "Rank".to_string(),
        values: vec!["Ace".to_string(), "Queen".to_string(), "King".to_string()],
    });
    gd
}

#[test]
fn eval_card_position_extrema_precedence_max() {
    let gd = precedence_fixture();
    let expr = CardPosition::Aggregate {
        aggregate: AggregateCardPosition::ExtremaPrecedence {
            extrema: Extrema::Max,
            card_set: Box::new(CardSet::Group {
                group: Group::Groupable {
                    groupable: Groupable::Location {
                        name: "Stock".to_string(),
                    },
                },
            }),
            precedence: "RankOrder".to_string(),
        },
    };
    let result = Evaluator::eval_card_position(&expr, &gd);
    assert_eq!(result.unwrap(), 1);
}

#[test]
fn eval_card_position_extrema_precedence_min() {
    let gd = precedence_fixture();
    let expr = CardPosition::Aggregate {
        aggregate: AggregateCardPosition::ExtremaPrecedence {
            extrema: Extrema::Min,
            card_set: Box::new(CardSet::Group {
                group: Group::Groupable {
                    groupable: Groupable::Location {
                        name: "Stock".to_string(),
                    },
                },
            }),
            precedence: "RankOrder".to_string(),
        },
    };
    let result = Evaluator::eval_card_position(&expr, &gd);
    assert_eq!(result.unwrap(), 0);
}

#[test]
fn eval_card_position_extrema_precedence_missing() {
    let gd = precedence_fixture();
    let expr = CardPosition::Aggregate {
        aggregate: AggregateCardPosition::ExtremaPrecedence {
            extrema: Extrema::Max,
            card_set: Box::new(CardSet::Group {
                group: Group::Groupable {
                    groupable: Groupable::Location {
                        name: "Stock".to_string(),
                    },
                },
            }),
            precedence: "NoSuchPrec".to_string(),
        },
    };
    let result = Evaluator::eval_card_position(&expr, &gd);
    assert_eq!(
        result.unwrap_err().to_string(),
        "Precedence NoSuchPrec not found".to_string()
    );
}

#[test]
fn eval_card_position_extrema_precedence_value_not_in() {
    let mut gd = precedence_fixture();
    let c2 = gd.add_card(
        gd.locations[0].cards[0],
        make_card(vec![("Rank", "Joker"), ("Suit", "Wild")]),
    );
    gd.locations[0].cards.push(c2);

    let expr = CardPosition::Aggregate {
        aggregate: AggregateCardPosition::ExtremaPrecedence {
            extrema: Extrema::Max,
            card_set: Box::new(CardSet::Group {
                group: Group::Groupable {
                    groupable: Groupable::Location {
                        name: "Stock".to_string(),
                    },
                },
            }),
            precedence: "RankOrder".to_string(),
        },
    };
    let result = Evaluator::eval_card_position(&expr, &gd);
    assert_eq!(result.unwrap(), 1);
}

#[test]
fn eval_card_position_extrema_precedence_empty() {
    let mut gd = GameData::new();
    let p0 = gd.add_player("P1".to_string());
    gd.turn_order = vec![p0];
    gd.add_location(
        "Table".to_string(),
        Location {
            name: "EmptyLoc".to_string(),
            cards: vec![],
        },
    );
    gd.memories
        .insert("Table_emptycs".to_string(), MemoryValue::CardSet(vec![]));
    gd.precedences.push(Precedence {
        name: "RankOrder".to_string(),
        key: "Rank".to_string(),
        values: vec!["Ace".to_string(), "King".to_string()],
    });
    let expr = CardPosition::Aggregate {
        aggregate: AggregateCardPosition::ExtremaPrecedence {
            extrema: Extrema::Max,
            card_set: Box::new(CardSet::Memory {
                memory: UseMemory::WithOwner {
                    memory: "emptycs".to_string(),
                    owner: Box::new(Owner::Table),
                },
            }),
            precedence: "RankOrder".to_string(),
        },
    };
    let result = Evaluator::eval_card_position(&expr, &gd);
    assert_eq!(
        result.unwrap_err().to_string(),
        "No card found for ExtremaPrecedence".to_string()
    );
}

#[test]
fn eval_cardset_card_position_variant() {
    let gd = basic_fixture();
    let expr = CardSet::Group {
        group: Group::CardPosition {
            card_position: CardPosition::Query {
                query: QueryCardPosition::At {
                    location: "Stock".to_string(),
                    int_expr: IntExpr::Literal { int: 0 },
                },
            },
        },
    };
    let result = Evaluator::eval_cardset(&expr, &gd);
    assert_eq!(result.unwrap(), (0, vec![0]));
}

#[test]
fn check_attr_value_in_cardset_present() {
    let gd = basic_fixture();
    let result = Evaluator::check_attr_value_in_cardset(&"Hearts".to_string(), &vec![0, 1], &gd);
    assert!(result);
}

#[test]
fn check_attr_value_in_cardset_absent() {
    let gd = basic_fixture();
    let result = Evaluator::check_attr_value_in_cardset(&"Diamonds".to_string(), &vec![0, 1], &gd);
    assert!(!result);
}

#[test]
fn check_attr_value_in_cardset_empty_set() {
    let gd = basic_fixture();
    let result = Evaluator::check_attr_value_in_cardset(&"Hearts".to_string(), &vec![], &gd);
    assert!(!result);
}
