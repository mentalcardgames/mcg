use std::path::PathBuf;

use cgdsl_engine::{run_game, GameData, Input, InputKind, InputSource, InputType};
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

fn default_input() -> InputSource {
    InputSource::Player(Box::new(|_it: InputType| Input {
        player_id: "P1".into(),
        kind: InputKind::Choice { idx: 0 },
    }))
}

#[test]
fn shuffle_action_preserves_card_count() {
    let ir = load_game("shuffle_action.cgdsl");
    let gd =
        run_game(ir, GameData::new(), default_input(), None, None).expect("game should complete");

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
