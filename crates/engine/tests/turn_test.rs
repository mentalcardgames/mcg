mod common;

use cgdsl_engine::{run_game_with, GameData, RunOptions};

use common::{default_input, load_game};

#[test]
fn turn_end_turn_advances_current_player() {
    let ir = load_game("turn_end_turn.cgdsl");
    let gd = run_game_with(ir, GameData::new(), default_input(), RunOptions::default())
        .expect("game should complete");

    let current = gd.get_current_player().expect("should have current player");
    assert_eq!(current.name, "P2");
}

#[test]
fn turn_cycle_to_named_sets_player() {
    let ir = load_game("turn_cycle_to_named.cgdsl");
    let gd = run_game_with(ir, GameData::new(), default_input(), RunOptions::default())
        .expect("game should complete");

    let current = gd.get_current_player().expect("should have current player");
    assert_eq!(current.name, "P2");
}

#[test]
fn turn_end_stage_named_pops_stage_stack() {
    let ir = load_game("turn_end_stage_named.cgdsl");
    let gd = run_game_with(ir, GameData::new(), default_input(), RunOptions::default())
        .expect("game should complete");

    assert!(
        !gd.stage_stack.contains(&"Play".to_string()),
        "stage stack should not contain Play"
    );
}

#[test]
fn turn_stage_deal_all_in_counted_stage() {
    let ir = load_game("turn_stage_deal_all.cgdsl");
    let gd = run_game_with(ir, GameData::new(), default_input(), RunOptions::default())
        .expect("game should complete");

    for (i, player) in gd.players.iter().enumerate() {
        let hand_location = player
            .owner
            .locations
            .iter()
            .find_map(|&li| gd.locations.get(li).filter(|l| l.name == "Hand"));

        assert!(
            hand_location.is_some(),
            "Player {} should have a Hand location",
            i
        );
        assert_eq!(
            hand_location.unwrap().cards.len(),
            1,
            "Player {}'s Hand should have exactly 1 card",
            i
        );
    }
}
