//! Integration tests for the 2026-08-10 verb-semantics pass: `deal` =
//! automatic from the top (with a count prompt for `any`/ranges), `move`/
//! `exchange` = the player picks the cards, positional sources = automatic.

use std::path::PathBuf;

use cgdsl_engine::{run_game_with, GameData, InputSource, InputType, RunOptions};
use front_end::ir::{Ir, LoweredPayLoad};
use front_end::validation::parse_document;

fn load_game(name: &str) -> Ir<LoweredPayLoad> {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let path = manifest.join("test_games").join(name);
    let src =
        std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {}", path.display(), e));
    let game = parse_document(&src).unwrap_or_else(|e| panic!("parse {}: {}", path.display(), e));
    game.to_lowered_graph()
}

fn test_file(name: &str) -> InputSource {
    InputSource::TestFile(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("test_games")
            .join(name),
    )
}

/// A closure that panics if the engine asks for *any* input — the fixture
/// must run fully automatically.
fn no_prompts_allowed() -> InputSource {
    InputSource::Player(Box::new(|it: InputType| {
        panic!("unexpected prompt: {:?}", it);
    }))
}

fn player_location<'a>(
    gd: &'a GameData,
    idx: usize,
    name: &str,
) -> Option<&'a cgdsl_engine::Location> {
    let player = gd.players.get(idx)?;
    player
        .owner
        .locations
        .iter()
        .filter_map(|&l| gd.locations.get(l))
        .find(|l| l.name == name)
}

fn table_location<'a>(gd: &'a GameData, name: &str) -> Option<&'a cgdsl_engine::Location> {
    gd.table
        .locations
        .iter()
        .filter_map(|&l| gd.locations.get(l))
        .find(|l| l.name == name)
}

fn total_cards(gd: &GameData) -> usize {
    gd.locations.iter().map(|l| l.cards.len()).sum()
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
