//! Integration tests for team-owned state: one shared pile per team, team
//! addressing (`X of T:Red`, bare name via the current player's team), and
//! the single team-keyed memory slot.

mod common;

use cgdsl_engine::game_data::MemoryValue;
use cgdsl_engine::{run_game_with, GameData, Input, InputKind, InputSource, RunOptions};

use common::{load_game, total_cards};

#[test]
fn team_pile_is_shared_and_addressable() {
    // The bare name `TeamPile` finds the current player's team's pile, the
    // explicit `TeamPile of T:Red` finds it by team, `owner of top(...)`
    // reports the team, and `(&I:TeamFlag of T:Red)` reads the single
    // team-keyed slot.
    let ir = load_game("team_pile_reads.cgdsl");
    let gd = run_game_with(
        ir,
        GameData::new(),
        InputSource::Player(Box::new(|_| Input {
            player_id: "P1".into(),
            kind: InputKind::Choice { idx: 0 },
        })),
        RunOptions::default(),
    )
    .expect("team-pile reads should complete");

    let pile = gd.locations.iter().find(|l| l.name == "TeamPile").unwrap();
    assert_eq!(
        pile.cards.len(),
        3,
        "all three deals went to ONE shared pile"
    );
    assert_eq!(
        gd.get_memory("Table_Who"),
        Some(&MemoryValue::String("Red".to_string())),
        "owner of top(TeamPile) reports the team"
    );
    assert_eq!(
        gd.players[0].score, 3,
        "size(cards TeamPile of T:Red) reads the team pile"
    );
    assert_eq!(
        gd.players[1].score, 7,
        "(&I:TeamFlag of T:Red) reads the team slot Red_TeamFlag"
    );
    assert_eq!(total_cards(&gd), 3);
}

#[test]
fn team_owned_locations_and_memories_are_one_per_team() {
    // `location X on T:Red` creates ONE shared pile for the team, and
    // `memory M on T:Red` the single slot `Red_M` — matching the read
    // addressing `(&I:M of T:Red)`.
    let ir = load_game("team_locations.cgdsl");
    let gd = run_game_with(
        ir,
        GameData::new(),
        InputSource::Player(Box::new(|_| Input {
            player_id: "P1".into(),
            kind: InputKind::Choice { idx: 0 },
        })),
        RunOptions::default(),
    )
    .expect("team-location game should complete");

    let pile_indices: Vec<usize> = gd
        .locations
        .iter()
        .enumerate()
        .filter(|(_, l)| l.name == "TeamPile")
        .map(|(i, _)| i)
        .collect();
    assert_eq!(
        pile_indices.len(),
        1,
        "one TeamPile for the whole team, not per member"
    );
    let team = gd.teams.iter().find(|t| t.name == "Red").unwrap();
    assert!(
        team.owner
            .locations
            .iter()
            .any(|i| pile_indices.contains(i)),
        "the TeamPile is owned by the team entity"
    );
    for p in &gd.players {
        assert!(
            !p.owner.locations.iter().any(|i| pile_indices.contains(i)),
            "no player owns the team pile"
        );
    }

    // `TeamFlag is 5` during P1's turn writes the single team slot
    // (`memory_write_owner` finds `Red_TeamFlag`).
    assert_eq!(gd.get_memory("Red_TeamFlag"), Some(&MemoryValue::Int(5)));
    assert!(gd.get_memory("P1_TeamFlag").is_none());
    assert!(gd.get_memory("P2_TeamFlag").is_none());
}
