mod common;

use cgdsl_engine::{run_game_with, GameData, RunOptions};

use common::{default_input, load_game};

#[test]
fn shuffle_action_preserves_card_count() {
    let ir = load_game("shuffle_action.cgdsl");
    let gd = run_game_with(ir, GameData::new(), default_input(), RunOptions::default())
        .expect("game should complete");

    let stock = gd
        .locations
        .iter()
        .find(|l| l.name == "Stock")
        .expect("Stock location should exist");

    assert_eq!(
        stock.cards.len(),
        4,
        "Stock should still have 4 cards after shuffle"
    );
}
