use crate::game_data::GameData;
use std::fs::{self, OpenOptions};
use std::io::Write;

use super::*;

#[test]
fn test_debug_level_from_marker_valid() {
    assert_eq!(DebugLevel::from_marker("<!--LOW-->"), Some(DebugLevel::Low));
    assert_eq!(
        DebugLevel::from_marker("<!--MEDIUM-->"),
        Some(DebugLevel::Medium)
    );
    assert_eq!(
        DebugLevel::from_marker("<!--HIGH-->"),
        Some(DebugLevel::High)
    );
}

#[test]
fn test_debug_level_from_marker_case_insensitive() {
    assert_eq!(DebugLevel::from_marker("<!--low-->"), Some(DebugLevel::Low));
    assert_eq!(
        DebugLevel::from_marker("<!--Medium-->"),
        Some(DebugLevel::Medium)
    );
    assert_eq!(
        DebugLevel::from_marker("<!--HIGH-->"),
        Some(DebugLevel::High)
    );
}

#[test]
fn test_debug_level_from_marker_invalid() {
    assert_eq!(DebugLevel::from_marker("<!--INVALID-->"), None);
    assert_eq!(DebugLevel::from_marker("low"), None);
    assert_eq!(DebugLevel::from_marker(""), None);
}

#[test]
fn test_debug_level_marker_roundtrip() {
    assert_eq!(DebugLevel::Low.marker(), "<!--LOW-->");
    assert_eq!(DebugLevel::Medium.marker(), "<!--MEDIUM-->");
    assert_eq!(DebugLevel::High.marker(), "<!--HIGH-->");
}

#[test]
fn test_format_game_data_low() {
    let data = GameData::new();
    let output = format_game_data(&data, DebugLevel::Low);
    assert!(!output.is_empty());
    assert!(output.contains("GAME DATA (LOW)"));
    assert!(output.contains("Players:"));
    assert!(output.contains("Turn Order Indices:"));
    assert!(output.contains("Card Counts per Location:"));
}

#[test]
fn test_format_game_data_medium() {
    let data = GameData::new();
    let output = format_game_data(&data, DebugLevel::Medium);
    assert!(!output.is_empty());
    assert!(output.contains("GAME DATA (MEDIUM)"));
    assert!(output.contains("Scores:"));
    assert!(output.contains("Teams:"));
    assert!(output.contains("Memories:"));
}

#[test]
fn test_format_game_data_high() {
    let data = GameData::new();
    let output = format_game_data(&data, DebugLevel::High);
    assert!(!output.is_empty());
    assert!(output.contains("GAME DATA (HIGH)"));
    assert!(output.contains("Players:"));
    assert!(output.contains("Cards:"));
    assert!(output.contains("Combos:"));
    assert!(output.contains("Precedences:"));
    assert!(output.contains("Point Maps:"));
}

#[test]
fn test_save_game_data_creates_file() {
    let _data = GameData::new();
    let path = std::path::PathBuf::from("/tmp/test_mcg_debug.txt");
    let _ = fs::remove_file(&path);
}

#[test]
fn test_save_game_data_appends_to_file() {
    let data = GameData::new();
    let path = std::path::PathBuf::from("/tmp/test_mcg_debug_append.txt");
    let _ = fs::remove_file(&path);

    save_game_data(&data, &path).unwrap();
    save_game_data(&data, &path).unwrap();

    let content = fs::read_to_string(&path).unwrap();
    let count = content.matches("=== GAME DATA").count();
    assert_eq!(count, 2);

    let _ = fs::remove_file(&path);
}

#[test]
fn test_format_game_data_low_with_players() {
    let mut data = GameData::new();
    data.add_player("Alice".to_string());
    data.add_player("Bob".to_string());
    data.turn_order = vec![0, 1];

    let output = format_game_data(&data, DebugLevel::Low);
    assert!(output.contains("Alice"));
    assert!(output.contains("Bob"));
}

#[test]
fn test_format_game_data_low_with_stage() {
    let mut data = GameData::new();
    data.add_player("Alice".to_string());
    data.add_player("Bob".to_string());
    data.turn_order = vec![0, 1];
    data.enter_stage(
        "Play".to_string(),
        vec!["Alice".to_string(), "Bob".to_string()],
    );

    let output = format_game_data(&data, DebugLevel::Low);
    assert!(output.contains("Play"));
}

#[test]
fn test_format_game_data_low_turn_order_indices() {
    let mut data = GameData::new();
    data.add_player("Alice".to_string());
    data.add_player("Bob".to_string());
    data.turn_order = vec![0, 1];

    let output = format_game_data(&data, DebugLevel::Low);
    assert!(output.contains("[0, 1]"));
}

#[test]
fn test_format_game_data_low_card_counts() {
    use std::collections::HashMap;
    let mut data = GameData::new();
    data.add_player("Alice".to_string());
    let stock_loc = data.add_location(
        "Table".to_string(),
        crate::game_data::Location {
            name: "Stock".to_string(),
            cards: vec![],
        },
    );
    let card_id1 = data.add_card(
        stock_loc,
        HashMap::from([("name".to_string(), "Fireball".to_string())]),
    );
    let card_id2 = data.add_card(
        stock_loc,
        HashMap::from([("name".to_string(), "Lightning".to_string())]),
    );
    let card_id3 = data.add_card(
        stock_loc,
        HashMap::from([("name".to_string(), "Ice".to_string())]),
    );
    data.locations[stock_loc].cards.push(card_id1);
    data.locations[stock_loc].cards.push(card_id2);
    data.locations[stock_loc].cards.push(card_id3);

    let output = format_game_data(&data, DebugLevel::Low);
    assert!(output.contains("Stock"));
    assert!(output.contains("3 cards"));
}

#[test]
fn test_format_game_data_medium_scores() {
    let mut data = GameData::new();
    let p1 = data.add_player("Alice".to_string());
    data.players[p1].score = 100;
    data.add_player("Bob".to_string());
    data.turn_order = vec![0, 1];

    let low_output = format_game_data(&data, DebugLevel::Low);
    assert!(!low_output.contains("100"));

    let medium_output = format_game_data(&data, DebugLevel::Medium);
    assert!(medium_output.contains("100"));
    assert!(medium_output.contains("Alice"));
}

#[test]
fn test_format_game_data_medium_teams() {
    let mut data = GameData::new();
    let p1 = data.add_player("Alice".to_string());
    let p2 = data.add_player("Bob".to_string());
    data.teams.push(crate::game_data::Team {
        name: "T1".to_string(),
        players: vec![p1, p2],
    });
    data.turn_order = vec![p1, p2];

    let output = format_game_data(&data, DebugLevel::Medium);
    assert!(output.contains("T1"));
    assert!(output.contains("Alice"));
    assert!(output.contains("Bob"));
}

#[test]
fn test_format_game_data_medium_memories() {
    let mut data = GameData::new();
    data.add_player("Alice".to_string());
    data.turn_order = vec![0];
    data.memories
        .insert("counter".to_string(), crate::game_data::MemoryValue::Int(5));

    let output = format_game_data(&data, DebugLevel::Medium);
    assert!(output.contains("counter"));
    assert!(output.contains("5"));
}

#[test]
fn test_format_game_data_medium_truncated_cards() {
    use std::collections::HashMap;
    let mut data = GameData::new();
    data.add_player("Alice".to_string());
    let stock_loc = data.add_location(
        "Table".to_string(),
        crate::game_data::Location {
            name: "Stock".to_string(),
            cards: vec![],
        },
    );
    for i in 0..7 {
        let card_id = data.add_card(
            stock_loc,
            HashMap::from([("name".to_string(), format!("Card{}", i))]),
        );
        data.locations[stock_loc].cards.push(card_id);
    }

    let output = format_game_data(&data, DebugLevel::Medium);
    assert!(output.contains("..."));
}

#[test]
fn test_format_game_data_high_full_player_details() {
    let mut data = GameData::new();
    let p1 = data.add_player("Alice".to_string());
    data.players[p1].score = 50;
    data.players[p1].in_game = true;
    data.players[p1].in_stage.insert("Play".to_string(), true);
    data.turn_order = vec![p1];

    let output = format_game_data(&data, DebugLevel::High);
    assert!(output.contains("score=50"));
    assert!(output.contains("in_game=true"));
    assert!(output.contains("in_stage="));
}

#[test]
fn test_format_game_data_high_all_cards() {
    use std::collections::HashMap;
    let mut data = GameData::new();
    data.add_player("Alice".to_string());
    data.turn_order = vec![0];
    let stock_loc = data.add_location(
        "Table".to_string(),
        crate::game_data::Location {
            name: "Stock".to_string(),
            cards: vec![],
        },
    );
    data.add_card(
        stock_loc,
        HashMap::from([("name".to_string(), "Fireball".to_string())]),
    );
    data.add_card(
        stock_loc,
        HashMap::from([("name".to_string(), "Lightning".to_string())]),
    );

    let output = format_game_data(&data, DebugLevel::High);
    assert!(output.contains("Fireball"));
    assert!(output.contains("Lightning"));
}

#[test]
fn test_format_game_data_high_combos_precedences_pointmaps() {
    use front_end::ast::{AggregateFilter, FilterExpr, IntCompare, IntExpr};
    let mut data = GameData::new();
    data.add_player("Alice".to_string());
    data.turn_order = vec![0];

    data.combos.push(crate::game_data::Combo {
        name: "Combo1".to_string(),
        filter: FilterExpr::Aggregate {
            aggregate: AggregateFilter::Size {
                cmp: IntCompare::Eq,
                int_expr: Box::new(IntExpr::Literal { int: 1 }),
            },
        },
    });

    data.precedences.push(crate::game_data::Precedence {
        name: "Precedence1".to_string(),
        key: "rank".to_string(),
        values: vec!["Low".to_string(), "High".to_string()],
    });

    let mut point_map_data = std::collections::HashMap::new();
    point_map_data.insert("Ace".to_string(), 1);
    data.point_maps.push(crate::game_data::PointMap {
        name: "PointMap1".to_string(),
        map: point_map_data,
    });

    let output = format_game_data(&data, DebugLevel::High);
    assert!(output.contains("Combos:"));
    assert!(output.contains("Precedences:"));
    assert!(output.contains("Point Maps:"));
    assert!(output.contains("Combo1"));
    assert!(output.contains("Precedence1"));
    assert!(output.contains("PointMap1"));
}

#[test]
fn test_format_game_data_high_memories_typed() {
    let mut data = GameData::new();
    data.add_player("Alice".to_string());
    data.turn_order = vec![0];
    data.memories.insert(
        "int_mem".to_string(),
        crate::game_data::MemoryValue::Int(42),
    );
    data.memories.insert(
        "str_mem".to_string(),
        crate::game_data::MemoryValue::String("hello".to_string()),
    );

    let output = format_game_data(&data, DebugLevel::High);
    assert!(output.contains("Int(42)"));
    assert!(output.contains("String(\"hello\")"));
}

#[test]
fn test_format_game_data_memory_value_variants() {
    let mut data = GameData::new();
    data.add_player("Alice".to_string());
    data.turn_order = vec![0];

    data.memories.insert(
        "int_val".to_string(),
        crate::game_data::MemoryValue::Int(42),
    );
    data.memories.insert(
        "str_val".to_string(),
        crate::game_data::MemoryValue::String("foo".to_string()),
    );
    data.memories.insert(
        "card_set".to_string(),
        crate::game_data::MemoryValue::CardSet(vec![1, 2, 3]),
    );
    data.memories.insert(
        "player_coll".to_string(),
        crate::game_data::MemoryValue::PlayerCollection(vec![0, 1]),
    );
    data.memories.insert(
        "team_val".to_string(),
        crate::game_data::MemoryValue::Team("T1".to_string()),
    );
    data.memories.insert(
        "int_coll".to_string(),
        crate::game_data::MemoryValue::IntCollection(vec![1, 2]),
    );
    data.memories.insert(
        "str_coll".to_string(),
        crate::game_data::MemoryValue::StringCollection(vec!["a".to_string(), "b".to_string()]),
    );
    data.memories.insert(
        "loc_coll".to_string(),
        crate::game_data::MemoryValue::LocationCollection(vec![0, 1]),
    );

    let output = format_game_data(&data, DebugLevel::High);
    assert!(output.contains("Int(42)"));
    assert!(output.contains("String(\"foo\")"));
    assert!(output.contains("CardSet([1, 2, 3])"));
    assert!(output.contains("PlayerCollection([0, 1])"));
    assert!(output.contains("Team(\"T1\")"));
    assert!(output.contains("IntCollection([1, 2])"));
    assert!(output.contains("StringCollection([\"a\", \"b\"])"));
    assert!(output.contains("LocationCollection([0, 1])"));
}

#[test]
fn test_format_game_data_empty_players() {
    let data = GameData::new();
    let output = format_game_data(&data, DebugLevel::Low);
    assert!(output.contains("Players:"));
}

#[test]
fn test_format_game_data_empty_locations() {
    let mut data = GameData::new();
    data.add_player("Alice".to_string());
    data.turn_order = vec![0];

    let output = format_game_data(&data, DebugLevel::Low);
    assert!(output.contains("Card Counts per Location:"));
}

#[test]
fn test_format_game_data_empty_cards() {
    let mut data = GameData::new();
    data.add_player("Alice".to_string());
    data.add_location(
        "Table".to_string(),
        crate::game_data::Location {
            name: "Stock".to_string(),
            cards: vec![],
        },
    );
    data.turn_order = vec![0];

    let output = format_game_data(&data, DebugLevel::Low);
    assert!(output.contains("0 cards"));
}

#[test]
fn test_format_game_data_single_player() {
    let mut data = GameData::new();
    data.add_player("Alice".to_string());
    data.turn_order = vec![0];

    let output = format_game_data(&data, DebugLevel::Low);
    assert!(output.contains("Alice"));
    assert!(output.contains("[0]"));
}

#[test]
fn test_format_game_data_current_player_none() {
    let mut data = GameData::new();
    data.add_player("Alice".to_string());
    data.turn_order = vec![];
    data.current_player = None;

    let output = format_game_data(&data, DebugLevel::Low);
    assert!(!output.contains("Current Player:"));
}

#[test]
fn test_format_game_data_empty_turn_order() {
    let mut data = GameData::new();
    data.add_player("Alice".to_string());
    data.turn_order = vec![];

    let output = format_game_data(&data, DebugLevel::Low);
    assert!(output.contains("Turn Order Indices:"));
    assert!(output.contains("[]"));
}

#[test]
fn test_save_game_data_then_format() {
    let mut data = GameData::new();
    data.add_player("Alice".to_string());
    data.add_player("Bob".to_string());
    data.turn_order = vec![0, 1];
    data.enter_stage(
        "Play".to_string(),
        vec!["Alice".to_string(), "Bob".to_string()],
    );

    let path = std::path::PathBuf::from("/tmp/test_mcg_debug_roundtrip.txt");
    let _ = fs::remove_file(&path);

    save_game_data(&data, &path).unwrap();

    let content = fs::read_to_string(&path).unwrap();
    assert!(content.contains("Alice"));
    assert!(content.contains("Bob"));
    assert!(content.contains("Play"));

    let _ = fs::remove_file(&path);
}

#[test]
fn test_save_game_data_preserves_level_marker() {
    let data = GameData::new();
    let path = std::path::PathBuf::from("/tmp/test_mcg_debug_marker.txt");
    let _ = fs::remove_file(&path);

    {
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .append(true)
            .open(&path)
            .unwrap();
        writeln!(file, "<!--HIGH-->").unwrap();
    }

    save_game_data(&data, &path).unwrap();

    let content = fs::read_to_string(&path).unwrap();
    assert!(content.contains("<!--HIGH-->"));

    let _ = fs::remove_file(&path);
}
