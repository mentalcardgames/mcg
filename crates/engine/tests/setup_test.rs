//! Integration tests for setup rules: created entries (combos, precedences,
//! point maps, memories) and quantifier resolution in setup (`on all`,
//! `on any` — I-20, `turnorder all`, `team T with all`).

mod common;

use std::sync::{Arc, Mutex};

use cgdsl_engine::{run_game_with, GameData, Input, InputKind, InputSource, InputType, RunOptions};

use common::{default_input, load_game, player_location};

#[test]
fn setup_create_combo_stores_entry() {
    let ir = load_game("setup_create_combo.cgdsl");
    let gd = run_game_with(ir, GameData::new(), default_input(), RunOptions::default())
        .expect("game should complete");

    assert!(!gd.combos.is_empty(), "combo should be stored");
    assert_eq!(gd.combos[0].name, "TwoOfAKind");
}

#[test]
fn setup_create_precedence_stores_entry() {
    let ir = load_game("setup_create_precedence.cgdsl");
    let gd = run_game_with(ir, GameData::new(), default_input(), RunOptions::default())
        .expect("game should complete");

    assert!(!gd.precedences.is_empty());
    assert_eq!(gd.precedences[0].key, "Rank");
}

#[test]
fn setup_create_pointmap_stores_entry() {
    let ir = load_game("setup_create_pointmap.cgdsl");
    let gd = run_game_with(ir, GameData::new(), default_input(), RunOptions::default())
        .expect("game should complete");

    assert!(!gd.point_maps.is_empty());
    assert_eq!(gd.point_maps[0].name, "Values");
    // Verify actual point values from the DSL (Ace: 1, Two: 2, Three: 3)
    assert_eq!(gd.point_maps[0].map.get("Rank:Ace"), Some(&1));
    assert_eq!(gd.point_maps[0].map.get("Rank:Two"), Some(&2));
    assert_eq!(gd.point_maps[0].map.get("Rank:Three"), Some(&3));
}

#[test]
fn setup_create_memory_initializes_slot() {
    let ir = load_game("setup_create_memory.cgdsl");
    let gd = run_game_with(ir, GameData::new(), default_input(), RunOptions::default())
        .expect("game should complete");

    use cgdsl_engine::game_data::MemoryValue;
    match gd.get_memory("Table_M") {
        Some(MemoryValue::Int(n)) => assert_eq!(*n, 0),
        other => panic!("expected Int(0), got {:?}", other),
    }
}

#[test]
fn setup_location_all_creates_per_player_hands() {
    let ir = load_game("setup_location_all.cgdsl");
    let gd = run_game_with(
        ir,
        GameData::new(),
        InputSource::Player(Box::new(|_it: InputType| Input {
            player_id: "P1".into(),
            kind: InputKind::Choice { idx: 0 },
        })),
        RunOptions::default(),
    )
    .expect("game should complete");

    assert!(
        player_location(&gd, 0, "Hand").is_some(),
        "P1 has a Hand location"
    );
    assert!(
        player_location(&gd, 1, "Hand").is_some(),
        "P2 has a Hand location"
    );
    assert!(
        player_location(&gd, 2, "Hand").is_some(),
        "P3 has a Hand location"
    );
}

#[test]
fn setup_location_literal_creates_per_player_hands() {
    let ir = load_game("setup_location_literal.cgdsl");
    let gd = run_game_with(
        ir,
        GameData::new(),
        InputSource::Player(Box::new(|_it: InputType| Input {
            player_id: "P1".into(),
            kind: InputKind::Choice { idx: 0 },
        })),
        RunOptions::default(),
    )
    .expect("game should complete");

    assert!(
        player_location(&gd, 0, "Hand").is_some(),
        "P1 has a Hand location"
    );
    assert!(
        player_location(&gd, 1, "Hand").is_some(),
        "P2 has a Hand location"
    );
    assert!(
        player_location(&gd, 2, "Hand").is_some(),
        "P3 has a Hand location"
    );
}

/// `location Hand on any`: setup-`Any` (I-20, relaxed) prompts for one player
/// and creates the location for the chosen player instead of erroring.
#[test]
fn setup_location_any_prompts_and_creates_for_chosen_player() {
    let ir = load_game("setup_location_any.cgdsl");
    let prompts = Arc::new(Mutex::new(0usize));
    let prompts_clone = prompts.clone();
    let gd = run_game_with(
        ir,
        GameData::new(),
        InputSource::Player(Box::new(move |it: InputType| match it {
            InputType::ChoosePlayer { .. } => {
                *prompts_clone.lock().unwrap() += 1;
                Input {
                    player_id: "P1".into(),
                    kind: InputKind::ChoosePlayer { idx: 1 }, // P2
                }
            }
            _ => Input {
                player_id: "P1".into(),
                kind: InputKind::Choice { idx: 0 },
            },
        })),
        RunOptions::default(),
    )
    .expect("game should complete");

    assert_eq!(
        *prompts.lock().unwrap(),
        1,
        "exactly one ChoosePlayer prompt"
    );
    assert_eq!(
        player_location(&gd, 1, "Hand").unwrap().cards.len(),
        1,
        "P2's Hand was created by the prompt and received the deal"
    );
    assert!(
        player_location(&gd, 0, "Hand").is_none(),
        "P1 has no Hand — the any-site resolved to P2 only"
    );
}

#[test]
fn setup_turnorder_all_resolves_to_in_game_players() {
    let ir = load_game("setup_turnorder_all.cgdsl");
    let gd = run_game_with(
        ir,
        GameData::new(),
        InputSource::Player(Box::new(|_it: InputType| Input {
            player_id: "P1".into(),
            kind: InputKind::Choice { idx: 0 },
        })),
        RunOptions::default(),
    )
    .expect("game should complete");

    assert_eq!(
        gd.turn_order,
        vec![0, 1, 2],
        "turn_order should be all in-game players in declaration order"
    );
}

#[test]
fn setup_teams_all_resolves_all_players() {
    let ir = load_game("setup_teams_all.cgdsl");
    let gd = run_game_with(
        ir,
        GameData::new(),
        InputSource::Player(Box::new(|_it: InputType| Input {
            player_id: "P1".into(),
            kind: InputKind::Choice { idx: 0 },
        })),
        RunOptions::default(),
    )
    .expect("game should complete");

    assert_eq!(
        gd.teams[0].players.len(),
        3,
        "team T1 should have all 3 players"
    );
}
