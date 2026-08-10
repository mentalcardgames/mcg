//! Integration tests for the 2026-08-10 verb-semantics pass: `deal` =
//! automatic from the top (with a count prompt for `any`/ranges), `move`/
//! `exchange` = the player picks the cards, positional sources = automatic.

mod common;

use cgdsl_engine::{run_game_with, GameData, InputSource, InputType, RunOptions};

use common::{load_game, player_location, table_location, test_file, total_cards};

/// A closure that panics if the engine asks for *any* input — the fixture
/// must run fully automatically.
fn no_prompts_allowed() -> InputSource {
    InputSource::Player(Box::new(|it: InputType| {
        panic!("unexpected prompt: {:?}", it);
    }))
}

#[test]
fn deal_range_and_any_prompt_for_the_count() {
    let ir = load_game("verb_deal_count.cgdsl");
    let gd = run_game_with(
        ir,
        GameData::new(),
        test_file("verb_deal_count.txt"),
        RunOptions::default(),
    )
    .expect("deal count prompts should complete");

    assert_eq!(player_location(&gd, 0, "Hand").unwrap().cards.len(), 5);
    assert_eq!(table_location(&gd, "Deck").unwrap().cards.len(), 0);
    assert_eq!(total_cards(&gd), 5);
}

#[test]
fn degenerate_deal_range_is_automatic() {
    let ir = load_game("verb_deal_range_automatic.cgdsl");
    let gd = run_game_with(
        ir,
        GameData::new(),
        no_prompts_allowed(),
        RunOptions::default(),
    )
    .expect("degenerate range must not prompt");

    assert_eq!(player_location(&gd, 0, "Hand").unwrap().cards.len(), 2);
}

#[test]
fn deal_count_chains_with_dest_all_fanout() {
    let ir = load_game("verb_deal_count_to_all.cgdsl");
    let gd = run_game_with(
        ir,
        GameData::new(),
        test_file("verb_deal_count_to_all.txt"),
        RunOptions::default(),
    )
    .expect("deal-count-to-all should complete");

    for i in 0..2 {
        assert_eq!(
            player_location(&gd, i, "Hand").unwrap().cards.len(),
            2,
            "each player receives the chosen count"
        );
    }
    assert_eq!(table_location(&gd, "Deck").unwrap().cards.len(), 0);
    assert_eq!(total_cards(&gd), 4);
}

#[test]
fn move_exact_n_prompts_and_reprompts_on_wrong_count() {
    let ir = load_game("verb_move_exact_n.cgdsl");
    let gd = run_game_with(
        ir,
        GameData::new(),
        test_file("verb_move_exact_n.txt"),
        RunOptions::default(),
    )
    .expect("exact-N pick should complete after the re-prompt");

    assert_eq!(
        table_location(&gd, "Discard").unwrap().cards.len(),
        2,
        "the valid 2-card selection moved"
    );
    assert_eq!(player_location(&gd, 0, "Hand").unwrap().cards.len(), 1);
    assert_eq!(total_cards(&gd), 5);
}

#[test]
fn move_exact_n_clamps_to_short_pile() {
    let ir = load_game("verb_move_exact_n_short_pile.cgdsl");
    let gd = run_game_with(
        ir,
        GameData::new(),
        test_file("verb_move_exact_n_short_pile.txt"),
        RunOptions::default(),
    )
    .expect("clamped exact-N pick should complete");

    assert_eq!(
        table_location(&gd, "Discard").unwrap().cards.len(),
        2,
        "both cards picked (prompt was clamped to min=max=2)"
    );
    assert_eq!(player_location(&gd, 0, "Hand").unwrap().cards.len(), 0);
    assert_eq!(total_cards(&gd), 2);
}

#[test]
fn positional_sources_are_automatic_for_any_verb() {
    let ir = load_game("verb_positional_automatic.cgdsl");
    let gd = run_game_with(
        ir,
        GameData::new(),
        no_prompts_allowed(),
        RunOptions::default(),
    )
    .expect("positional moves must not prompt");

    assert_eq!(player_location(&gd, 0, "Hand").unwrap().cards.len(), 1);
    assert_eq!(table_location(&gd, "Discard").unwrap().cards.len(), 1);
    assert_eq!(table_location(&gd, "Stock").unwrap().cards.len(), 1);
    assert_eq!(total_cards(&gd), 3);
}
