use super::*;
use crate::game_data::{Card, CardStatus, Location, MemoryValue};
use front_end::ast::{
    CardSet, Group, Groupable, IntCollection, IntExpr, LocationCollection, MemoryType, Owner,
    PlayerCollection, PlayerExpr, StringCollection, StringExpr, TeamCollection, TeamExpr,
};

#[test]
fn test_ensure_stage_entered_is_idempotent_and_sets_flags() {
    let mut gd = GameData::new();
    let p0 = gd.add_player("P1".to_string());
    let p1 = gd.add_player("P2".to_string());
    gd.turn_order = vec![p0, p1];

    assert_eq!(gd.get_current_stage(), None);
    assert!(gd.stage_stack.is_empty());

    gd.ensure_stage_entered("Play");
    assert_eq!(gd.get_current_stage(), Some("Play".to_string()));
    assert_eq!(gd.stage_stack.len(), 1);
    assert_eq!(gd.players[0].in_stage.get("Play"), Some(&true));
    assert_eq!(gd.players[1].in_stage.get("Play"), Some(&true));

    gd.ensure_stage_entered("Play");
    assert_eq!(
        gd.stage_stack.len(),
        1,
        "ensure_stage_entered must not push twice"
    );

    gd.ensure_stage_entered("Sub");
    assert_eq!(gd.stage_stack.len(), 2);
    assert_eq!(gd.get_current_stage(), Some("Sub".to_string()));
    assert_eq!(gd.players[0].in_stage.get("Sub"), Some(&true));
}

#[test]
fn set_memory_int_stores_evaluated_value_not_increment() {
    // Regression for invariant I-9: set_memory must store the value, not
    // increment the existing one by 1.
    let mut gd = GameData::new();
    gd.add_memory(
        "Table_counter".to_string(),
        Owner::Table,
        Some(MemoryType::Int {
            int: IntExpr::Literal { int: 0 },
        }),
    );
    // simulate the post-fix path: action.rs evaluated IntExpr::Literal{5}
    gd.set_memory("Table_counter".to_string(), MemoryValue::Int(5));
    assert_eq!(gd.get_memory("Table_counter"), Some(&MemoryValue::Int(5)));

    // second set overwrites, does not increment
    gd.set_memory("Table_counter".to_string(), MemoryValue::Int(10));
    assert_eq!(gd.get_memory("Table_counter"), Some(&MemoryValue::Int(10)));
}

#[test]
fn set_memory_overwrites_non_int_variant() {
    // Pre-fix, set_memory silently no-op'd on non-Int memories. Post-fix,
    // it must overwrite regardless of variant.
    let mut gd = GameData::new();
    gd.add_memory(
        "Table_label".to_string(),
        Owner::Table,
        Some(MemoryType::String {
            string: StringExpr::Literal {
                value: String::new(),
            },
        }),
    );
    gd.set_memory(
        "Table_label".to_string(),
        MemoryValue::String("hello".to_string()),
    );
    assert_eq!(
        gd.get_memory("Table_label"),
        Some(&MemoryValue::String("hello".to_string()))
    );
}

#[test]
fn new_yields_empty_store_with_current_player_some_0() {
    // I-2: GameData::new() sets current_player = Some(0) but turn_order
    // is empty. get_current_player() must return None (safe), but indexing
    // turn_order[0] would panic — so the safe accessor is the only path.
    let gd = GameData::new();
    assert_eq!(gd.current_player, Some(0));
    assert!(gd.turn_order.is_empty());
    assert!(
        gd.get_current_player().is_none(),
        "get_current_player must return None when turn_order is empty"
    );
}

#[test]
fn add_player_appends_and_returns_increasing_indices() {
    let mut gd = GameData::new();
    let p0 = gd.add_player("Alice".to_string());
    let p1 = gd.add_player("Bob".to_string());
    assert_eq!(p0, 0);
    assert_eq!(p1, 1);
    assert_eq!(gd.players.len(), 2);
    assert_eq!(gd.players[0].name, "Alice");
    assert_eq!(gd.players[1].name, "Bob");
    assert!(gd.players[0].in_game, "new players start in_game");
    assert_eq!(gd.players[0].score, 0, "new players start with score 0");
}

#[test]
fn add_location_to_table_registers_under_table_owner() {
    let mut gd = GameData::new();
    let loc = gd.add_location(
        "Table".to_string(),
        Location {
            name: "Stock".to_string(),
            cards: vec![],
        },
    );
    assert_eq!(loc, 0);
    assert_eq!(gd.locations.len(), 1);
    assert_eq!(gd.table.locations, vec![0]);
}

#[test]
fn add_location_to_player_registers_under_player_owner() {
    let mut gd = GameData::new();
    let _p0 = gd.add_player("Alice".to_string());
    let loc = gd.add_location(
        "Alice".to_string(),
        Location {
            name: "Hand".to_string(),
            cards: vec![],
        },
    );
    assert_eq!(loc, 0);
    assert_eq!(gd.players[0].owner.locations, vec![0]);
}

#[test]
#[should_panic(expected = "owner")]
fn add_location_panics_when_owner_name_missing() {
    let mut gd = GameData::new();
    let _ = gd.add_location(
        "Ghost".to_string(),
        Location {
            name: "Hand".to_string(),
            cards: vec![],
        },
    );
}

#[test]
fn add_card_ignores_location_param_and_returns_increasing_ids() {
    // I-6: add_card takes a _location_id it ignores; the caller is
    // responsible for pushing the id into locations[..].cards.
    let mut gd = GameData::new();
    let card: Card = std::collections::HashMap::from([("Rank".to_string(), "Ace".to_string())]);
    let id0 = gd.add_card(0, card.clone());
    let id1 = gd.add_card(0, card);
    assert_eq!(id0, 0);
    assert_eq!(id1, 1);
    assert_eq!(gd.cards.len(), 2);
}

#[test]
fn get_card_returns_none_for_out_of_range() {
    let gd = GameData::new();
    assert!(gd.get_card(0).is_none());
}

#[test]
fn find_location_of_card_scans_all_locations() {
    let mut gd = GameData::new();
    let _p = gd.add_player("Alice".to_string());
    let hand = gd.add_location(
        "Alice".to_string(),
        Location {
            name: "Hand".to_string(),
            cards: vec![],
        },
    );
    let stock = gd.add_location(
        "Table".to_string(),
        Location {
            name: "Stock".to_string(),
            cards: vec![],
        },
    );
    let card_id = gd.add_card(0, std::collections::HashMap::new());
    gd.locations[stock].cards.push(card_id);
    assert_eq!(gd.find_location_of_card(card_id), Some(stock));
    assert_eq!(gd.find_location_of_card(999), None);
    // move the card to hand and re-scan
    gd.locations[stock].cards.retain(|&c| c != card_id);
    gd.locations[hand].cards.push(card_id);
    assert_eq!(gd.find_location_of_card(card_id), Some(hand));
}

#[test]
fn increment_stage_counter_creates_and_increments() {
    let mut gd = GameData::new();
    assert_eq!(
        gd.get_stage_counter("Play".to_string()),
        0,
        "absent counter reads 0"
    );
    gd.increment_stage_counter("Play".to_string());
    assert_eq!(gd.get_stage_counter("Play".to_string()), 1);
    gd.increment_stage_counter("Play".to_string());
    assert_eq!(gd.get_stage_counter("Play".to_string()), 2);
}

#[test]
fn reset_stage_counter_zeros_existing() {
    let mut gd = GameData::new();
    gd.increment_stage_counter("Play".to_string());
    gd.increment_stage_counter("Play".to_string());
    gd.reset_stage_counter("Play".to_string());
    assert_eq!(gd.get_stage_counter("Play".to_string()), 0);
}

#[test]
fn reset_stage_counter_on_absent_inserts_zero() {
    let mut gd = GameData::new();
    gd.reset_stage_counter("Unused".to_string());
    assert_eq!(gd.get_stage_counter("Unused".to_string()), 0);
    assert!(gd.stage_counters.contains_key("Unused"));
}

#[test]
fn get_current_player_resolves_through_turn_order() {
    // I-1: current_player indexes turn_order, not players.
    let mut gd = GameData::new();
    let p0 = gd.add_player("Alice".to_string());
    let p1 = gd.add_player("Bob".to_string());
    gd.turn_order = vec![p1, p0]; // current_player=0 -> turn_order[0]=Bob
    gd.current_player = Some(0);
    assert_eq!(
        gd.get_current_player().map(|p| &p.name),
        Some(&"Bob".to_string())
    );
    gd.current_player = Some(1);
    assert_eq!(
        gd.get_current_player().map(|p| &p.name),
        Some(&"Alice".to_string())
    );
}

#[test]
fn next_player_advances_within_turn_order() {
    let mut gd = GameData::new();
    let p0 = gd.add_player("Alice".to_string());
    let p1 = gd.add_player("Bob".to_string());
    gd.turn_order = vec![p0, p1];
    gd.enter_stage(
        "Play".to_string(),
        vec!["Alice".to_string(), "Bob".to_string()],
    );
    gd.current_player = Some(0);
    gd.next_player();
    assert_eq!(gd.current_player, Some(1));
    gd.next_player();
    assert_eq!(gd.current_player, Some(0), "wraps around");
}

#[test]
fn next_player_skips_ineligible_player() {
    // I-13: a player with in_stage=false is skipped.
    let mut gd = GameData::new();
    let p0 = gd.add_player("Alice".to_string());
    let p1 = gd.add_player("Bob".to_string());
    let p2 = gd.add_player("Carol".to_string());
    gd.turn_order = vec![p0, p1, p2];
    gd.enter_stage(
        "Play".to_string(),
        vec!["Alice".to_string(), "Bob".to_string(), "Carol".to_string()],
    );
    gd.set_player_stage_flag(p1, "Play".to_string(), false); // Bob out of stage
    gd.current_player = Some(0);
    gd.next_player();
    assert_eq!(
        gd.current_player,
        Some(2),
        "skips Bob (idx 1), lands on Carol (idx 2)"
    );
}

#[test]
fn next_player_becomes_none_when_only_other_is_out_of_game() {
    // I-13: resolve_turn scans only the OTHER players (loop 1..len),
    // never the current one. In a 2-player game where the only other
    // player is out of game, no eligible OTHER is found, so
    // current_player becomes None (stuck) — even though the current
    // player is still eligible. This is the documented I-13 behavior;
    // pin it.
    let mut gd = GameData::new();
    let p0 = gd.add_player("Alice".to_string());
    let p1 = gd.add_player("Bob".to_string());
    gd.turn_order = vec![p0, p1];
    gd.enter_stage(
        "Play".to_string(),
        vec!["Alice".to_string(), "Bob".to_string()],
    );
    gd.set_player_out(p0); // Alice out of game
    gd.current_player = Some(1); // Bob is current
    gd.next_player();
    assert_eq!(
        gd.current_player, None,
        "I-13: only-other-player out of game => stuck (None), not wrap-to-current"
    );
}

#[test]
fn next_player_sets_none_when_no_one_eligible() {
    // I-13 stuck-game path: when no player is in_stage, current_player
    // becomes None and the game is effectively stuck (no Error raised).
    let mut gd = GameData::new();
    let p0 = gd.add_player("Alice".to_string());
    let p1 = gd.add_player("Bob".to_string());
    gd.turn_order = vec![p0, p1];
    gd.enter_stage(
        "Play".to_string(),
        vec!["Alice".to_string(), "Bob".to_string()],
    );
    gd.set_player_stage_flag(p0, "Play".to_string(), false);
    gd.set_player_stage_flag(p1, "Play".to_string(), false);
    gd.current_player = Some(0);
    gd.next_player();
    assert_eq!(gd.current_player, None, "no eligible player => None");
}

#[test]
fn next_player_no_op_when_current_player_none() {
    let mut gd = GameData::new();
    gd.add_player("Alice".to_string());
    gd.turn_order = vec![0];
    gd.current_player = None;
    gd.next_player();
    assert_eq!(gd.current_player, None);
}

#[test]
fn enter_stage_pushes_and_sets_flags_only_for_named_players() {
    let mut gd = GameData::new();
    let p0 = gd.add_player("Alice".to_string());
    let p1 = gd.add_player("Bob".to_string());
    gd.enter_stage("Play".to_string(), vec!["Alice".to_string()]);
    assert_eq!(gd.stage_stack, vec!["Play".to_string()]);
    assert_eq!(gd.get_current_stage(), Some("Play".to_string()));
    assert_eq!(gd.players[p0].in_stage.get("Play"), Some(&true));
    assert_eq!(
        gd.players[p1].in_stage.get("Play"),
        Some(&false),
        "Bob was not in players_in; his flag must be false"
    );
}

#[test]
fn enter_stage_nested_pushes_two() {
    let mut gd = GameData::new();
    gd.add_player("Alice".to_string());
    gd.enter_stage("Play".to_string(), vec!["Alice".to_string()]);
    gd.enter_stage("Sub".to_string(), vec!["Alice".to_string()]);
    assert_eq!(gd.stage_stack, vec!["Play".to_string(), "Sub".to_string()]);
    assert_eq!(gd.get_current_stage(), Some("Sub".to_string()));
}

#[test]
fn leave_stage_pops_until_named_stage_inclusive() {
    let mut gd = GameData::new();
    gd.add_player("Alice".to_string());
    gd.enter_stage("A".to_string(), vec!["Alice".to_string()]);
    gd.enter_stage("B".to_string(), vec!["Alice".to_string()]);
    gd.enter_stage("C".to_string(), vec!["Alice".to_string()]);
    gd.leave_stage("A".to_string()); // pop C, B, A
    assert!(gd.stage_stack.is_empty());
}

#[test]
fn leave_stage_pops_only_through_named_stage() {
    let mut gd = GameData::new();
    gd.add_player("Alice".to_string());
    gd.enter_stage("A".to_string(), vec!["Alice".to_string()]);
    gd.enter_stage("B".to_string(), vec!["Alice".to_string()]);
    gd.enter_stage("C".to_string(), vec!["Alice".to_string()]);
    gd.leave_stage("B".to_string()); // pop C, B; A stays
    assert_eq!(gd.stage_stack, vec!["A".to_string()]);
    assert_eq!(gd.get_current_stage(), Some("A".to_string()));
}

#[test]
fn leave_stage_drains_entire_stack_when_stage_absent() {
    // I-11: if the named stage is not on the stack, the ENTIRE stack is
    // drained. This is the current behavior — pin it.
    let mut gd = GameData::new();
    gd.add_player("Alice".to_string());
    gd.enter_stage("A".to_string(), vec!["Alice".to_string()]);
    gd.enter_stage("B".to_string(), vec!["Alice".to_string()]);
    gd.leave_stage("Ghost".to_string());
    assert!(
        gd.stage_stack.is_empty(),
        "named stage absent => entire stack drained"
    );
}

#[test]
fn leave_stage_on_empty_stack_is_noop() {
    let mut gd = GameData::new();
    gd.leave_stage("Anything".to_string());
    assert!(gd.stage_stack.is_empty());
}

#[test]
fn set_player_out_flips_in_game_false() {
    let mut gd = GameData::new();
    let p0 = gd.add_player("Alice".to_string());
    assert!(gd.players[p0].in_game);
    gd.set_player_out(p0);
    assert!(!gd.players[p0].in_game);
}

#[test]
fn set_player_out_on_missing_index_is_noop() {
    let mut gd = GameData::new();
    gd.set_player_out(99); // no panic
    assert!(gd.players.is_empty());
}

#[test]
fn set_player_stage_flag_inserts_or_overwrites() {
    let mut gd = GameData::new();
    let p0 = gd.add_player("Alice".to_string());
    gd.set_player_stage_flag(p0, "Play".to_string(), true);
    assert_eq!(gd.players[p0].in_stage.get("Play"), Some(&true));
    gd.set_player_stage_flag(p0, "Play".to_string(), false);
    assert_eq!(gd.players[p0].in_stage.get("Play"), Some(&false));
}

#[test]
fn set_player_stage_flag_on_missing_index_is_noop() {
    let mut gd = GameData::new();
    gd.set_player_stage_flag(99, "Play".to_string(), true); // no panic
    assert!(gd.players.is_empty());
}

#[test]
fn add_memory_int_initializes_to_zero() {
    let mut gd = GameData::new();
    gd.add_memory(
        "Table_counter".to_string(),
        Owner::Table,
        Some(MemoryType::Int {
            int: IntExpr::Literal { int: 0 },
        }),
    );
    assert_eq!(gd.get_memory("Table_counter"), Some(&MemoryValue::Int(0)));
}

#[test]
fn add_memory_string_initializes_to_empty() {
    let mut gd = GameData::new();
    gd.add_memory(
        "Table_label".to_string(),
        Owner::Table,
        Some(MemoryType::String {
            string: StringExpr::Literal {
                value: String::new(),
            },
        }),
    );
    assert_eq!(
        gd.get_memory("Table_label"),
        Some(&MemoryValue::String(String::new()))
    );
}

#[test]
fn add_memory_none_initializes_to_int_zero() {
    let mut gd = GameData::new();
    gd.add_memory("Table_anon".to_string(), Owner::Table, None);
    assert_eq!(gd.get_memory("Table_anon"), Some(&MemoryValue::Int(0)));
}

#[test]
fn add_memory_player_initializes_to_int_zero_mismatched() {
    // I-10: MemoryType::Player initializes to MemoryValue::Int(0), NOT
    // a player. Reads as Player will fail type checks until something
    // writes a correctly-typed value. Pin the mismatch.
    let mut gd = GameData::new();
    gd.add_memory(
        "Table_p".to_string(),
        Owner::Table,
        Some(MemoryType::Player {
            player: PlayerExpr::Literal {
                name: "Alice".to_string(),
            },
        }),
    );
    assert_eq!(
        gd.get_memory("Table_p"),
        Some(&MemoryValue::Int(0)),
        "I-10: Player memory inits to Int(0), not a player"
    );
}

#[test]
fn add_memory_team_collection_initializes_to_int_zero_mismatched() {
    // I-10: TeamCollection also inits to Int(0).
    let mut gd = GameData::new();
    gd.add_memory(
        "Table_tc".to_string(),
        Owner::Table,
        Some(MemoryType::TeamCollection {
            teams: TeamCollection::Literal { teams: vec![] },
        }),
    );
    assert_eq!(
        gd.get_memory("Table_tc"),
        Some(&MemoryValue::Int(0)),
        "I-10: TeamCollection memory inits to Int(0)"
    );
}

#[test]
fn add_memory_collection_variants_init_to_empty_vecs() {
    let mut gd = GameData::new();
    gd.add_memory(
        "Table_pcs".to_string(),
        Owner::Table,
        Some(MemoryType::PlayerCollection {
            players: PlayerCollection::Literal { players: vec![] },
        }),
    );
    assert_eq!(
        gd.get_memory("Table_pcs"),
        Some(&MemoryValue::PlayerCollection(vec![]))
    );
    gd.add_memory(
        "Table_scs".to_string(),
        Owner::Table,
        Some(MemoryType::StringCollection {
            strings: StringCollection::Literal { strings: vec![] },
        }),
    );
    assert_eq!(
        gd.get_memory("Table_scs"),
        Some(&MemoryValue::StringCollection(vec![]))
    );
    gd.add_memory(
        "Table_ics".to_string(),
        Owner::Table,
        Some(MemoryType::IntCollection {
            ints: IntCollection::Literal { ints: vec![] },
        }),
    );
    assert_eq!(
        gd.get_memory("Table_ics"),
        Some(&MemoryValue::IntCollection(vec![]))
    );
    gd.add_memory(
        "Table_lcs".to_string(),
        Owner::Table,
        Some(MemoryType::LocationCollection {
            locations: LocationCollection::Literal { locations: vec![] },
        }),
    );
    assert_eq!(
        gd.get_memory("Table_lcs"),
        Some(&MemoryValue::LocationCollection(vec![]))
    );
    gd.add_memory(
        "Table_cs".to_string(),
        Owner::Table,
        Some(MemoryType::CardSet {
            card_set: CardSet::Group {
                group: Group::Groupable {
                    groupable: Groupable::Location {
                        name: "X".to_string(),
                    },
                },
            },
        }),
    );
    assert_eq!(
        gd.get_memory("Table_cs"),
        Some(&MemoryValue::CardSet(vec![]))
    );
}

#[test]
fn add_memory_team_initializes_to_empty_team_string() {
    // NOTE: The plan asserted `MemoryValue::String(String::new())`, but
    // `add_memory` (game_data.rs:283) produces `MemoryValue::Team(String::new())`
    // for `MemoryType::Team`. I-10 does NOT list `Team` as mismatched, so
    // `Team` is the correct variant. Corrected here.
    let mut gd = GameData::new();
    gd.add_memory(
        "Table_t".to_string(),
        Owner::Table,
        Some(MemoryType::Team {
            team: TeamExpr::Literal {
                name: "T1".to_string(),
            },
        }),
    );
    assert_eq!(
        gd.get_memory("Table_t"),
        Some(&MemoryValue::Team(String::new()))
    );
}

#[test]
fn reset_memory_zeros_int() {
    let mut gd = GameData::new();
    gd.add_memory("Table_counter".to_string(), Owner::Table, None);
    gd.set_memory("Table_counter".to_string(), MemoryValue::Int(5));
    gd.reset_memory("Table_counter");
    assert_eq!(gd.get_memory("Table_counter"), Some(&MemoryValue::Int(0)));
}

#[test]
fn reset_memory_on_absent_is_noop() {
    let mut gd = GameData::new();
    gd.reset_memory("ghost");
    assert!(gd.get_memory("ghost").is_none());
}

#[test]
fn reset_memory_on_non_int_is_noop() {
    // Per current behavior, reset_memory only touches Int memories.
    let mut gd = GameData::new();
    gd.add_memory("Table_label".to_string(), Owner::Table, None);
    gd.set_memory(
        "Table_label".to_string(),
        MemoryValue::String("hello".to_string()),
    );
    gd.reset_memory("Table_label");
    assert_eq!(
        gd.get_memory("Table_label"),
        Some(&MemoryValue::String("hello".to_string())),
        "reset_memory must not touch non-Int memories"
    );
}

#[test]
fn get_memory_returns_none_for_absent() {
    let gd = GameData::new();
    assert!(gd.get_memory("ghost").is_none());
}

#[test]
fn card_status_defaults_to_face_up_and_is_settable() {
    // Card status is reserved for the card-encryption work; the slot must
    // exist and default to FaceUp without any engine behaviour reading it.
    let mut gd = GameData::new();
    let id = gd.add_card(0, Card::new());
    assert_eq!(gd.card_status(id), Some(CardStatus::FaceUp));
    gd.set_card_status(id, CardStatus::FaceDown);
    assert_eq!(gd.card_status(id), Some(CardStatus::FaceDown));
    assert_eq!(gd.card_status(99), None, "out-of-range id");
}
