//! Unit tests for the quantifier preprocessor helpers (see `crates/engine/src/quantifier.rs`).
//!
//! These tests construct `Edge` / `StateID` directly and exercise the pure
//! helper functions (`alloc_synth`, `scan_edge`, `substitute_*`,
//! `build_dest_all_chain`, `validate_int_range`, `setup_contains_any`).
//! End-to-end tests that drive `run_game` against `.cgdsl` fixtures live
//! in the separate integration test file `tests/quantifier_test.rs`.

use super::*;
use crate::game_data::MemoryValue;
use crate::query::Evaluator;
use front_end::ast::{
    ActionRule, AggregateFilter, ClassicMove, GameRule, Group, Groupable, IntCompare, MoveType,
    Status,
};
use front_end::ir::Ir;

/// A throwaway `StateID` for the `to` of hand-built test edges. `Ir::default`
/// has `entry == StateID(0)`, which is fine for unit tests (we never
/// dispatch these edges through the real interpreter).
fn dest_state() -> StateID {
    Ir::<LoweredPayLoad>::default().entry
}

fn loc_cardset(name: &str) -> CardSet {
    CardSet::Group {
        group: Group::Groupable {
            groupable: Groupable::Location {
                name: name.to_string(),
            },
        },
    }
}

fn groupowner_cardset(name: &str, owner: Owner) -> CardSet {
    CardSet::GroupOwner {
        group: Group::Groupable {
            groupable: Groupable::Location {
                name: name.to_string(),
            },
        },
        owner,
    }
}

fn aggregate_owner(quantifier: Quantifier) -> Owner {
    Owner::PlayerCollection {
        player_collection: PlayerCollection::Aggregate {
            aggregate: AggregatePlayerCollection::Quantifier { quantifier },
        },
    }
}

/// Build a `Classic`/`MoveQuantity` edge carrying the given quantity/from/to.
fn move_qty_edge(quantity: Quantity, from: CardSet, to: CardSet) -> Edge<LoweredPayLoad> {
    Edge {
        to: dest_state(),
        payload: Payload::Action(GameRule::Action {
            action: ActionRule::Move {
                move_type: MoveType::Classic {
                    classic: ClassicMove::MoveCardSet {
                        move_cs: MoveCardSet::MoveQuantity {
                            quantity,
                            from,
                            status: Status::Private,
                            to,
                        },
                    },
                },
            },
        }),
        meta: None,
    }
}

#[test]
fn alloc_synth_yields_valid_decreasing_stateids() {
    let mut counter = u32::MAX - 1;
    let mut prev_raw = u32::MAX;
    for _ in 0..1024 {
        let id = alloc_synth(&mut counter);
        let raw = id.raw();
        assert_ne!(raw, 0, "synthetic ids must never be 0");
        assert_eq!(raw, prev_raw - 1, "ids must decrease monotonically");
        prev_raw = raw;
    }
    assert_eq!(
        alloc_synth(&mut (u32::MAX - 1)).raw(),
        u32::MAX - 1,
        "first allocation from a fresh seed is u32::MAX - 1"
    );
}

#[test]
fn alloc_synth_wraps_without_panicking() {
    let mut counter = 0u32;
    let _ = alloc_synth(&mut counter);
    let _ = alloc_synth(&mut counter);
}

#[test]
fn scan_edge_dest_player_all() {
    let edge = move_qty_edge(
        Quantity::Int {
            int: IntExpr::Literal { int: 1 },
        },
        loc_cardset("Stock"),
        groupowner_cardset("Hand", aggregate_owner(Quantifier::All)),
    );
    assert!(matches!(scan_edge(&edge), QuantSite::DestPlayerAll { .. }));
}

#[test]
fn scan_edge_dest_player_any() {
    let edge = move_qty_edge(
        Quantity::Int {
            int: IntExpr::Literal { int: 1 },
        },
        loc_cardset("Stock"),
        groupowner_cardset("Hand", aggregate_owner(Quantifier::Any)),
    );
    assert!(matches!(scan_edge(&edge), QuantSite::DestPlayerAny { .. }));
}

#[test]
fn scan_edge_src_cards_any() {
    let edge = move_qty_edge(
        Quantity::Quantifier {
            quantifier: Quantifier::Any,
        },
        loc_cardset("Stock"),
        loc_cardset("Discard"),
    );
    assert!(matches!(
        scan_edge(&edge),
        QuantSite::SrcCardsAnyOrRange { .. }
    ));
}

#[test]
fn scan_edge_src_cards_int_range() {
    let range = IntRange {
        start: (IntCompare::Ge, IntExpr::Literal { int: 1 }),
        op_int: vec![(
            IntRangeOperator::And,
            IntCompare::Le,
            IntExpr::Literal { int: 3 },
        )],
    };
    let edge = move_qty_edge(
        Quantity::IntRange { int_range: range },
        loc_cardset("Stock"),
        loc_cardset("Discard"),
    );
    assert!(matches!(
        scan_edge(&edge),
        QuantSite::SrcCardsAnyOrRange { .. }
    ));
}

#[test]
fn scan_edge_none_for_concrete_move() {
    let edge = move_qty_edge(
        Quantity::Int {
            int: IntExpr::Literal { int: 1 },
        },
        loc_cardset("Stock"),
        loc_cardset("Discard"),
    );
    assert_eq!(scan_edge(&edge), QuantSite::None);
}

#[test]
fn scan_edge_precedence_all_over_card_any() {
    // `deal any from Stock to Hand of all` — both a dest-all site and a
    // card-any site. scan_edge must report DestPlayerAll (the resume
    // branch handles the card choice via card_site).
    let edge = move_qty_edge(
        Quantity::Quantifier {
            quantifier: Quantifier::Any,
        },
        loc_cardset("Stock"),
        groupowner_cardset("Hand", aggregate_owner(Quantifier::All)),
    );
    assert!(matches!(scan_edge(&edge), QuantSite::DestPlayerAll { .. }));
    assert!(
        card_site(&edge).is_some(),
        "card_site must still detect the any-qty"
    );
}

#[test]
fn substitute_dest_player_replaces_owner() {
    let edge = move_qty_edge(
        Quantity::Int {
            int: IntExpr::Literal { int: 1 },
        },
        loc_cardset("Stock"),
        groupowner_cardset("Hand", aggregate_owner(Quantifier::All)),
    );
    let repl = substitute_dest_player(&edge, "P2".to_string());
    let mcs = move_cardset_ref(&repl).expect("edge still a Move");
    match mcs_to_ref(mcs) {
        CardSet::GroupOwner {
            owner:
                Owner::Player {
                    player: PlayerExpr::Literal { name },
                },
            ..
        } => assert_eq!(name, "P2"),
        other => panic!("expected concrete Player owner, got {:?}", other),
    }
}

#[test]
fn substitute_cardset_memory_round_trips_through_eval_cardset() {
    let edge = move_qty_edge(
        Quantity::Quantifier {
            quantifier: Quantifier::Any,
        },
        loc_cardset("Stock"),
        loc_cardset("Discard"),
    );
    let repl = substitute_cardset_memory(&edge, &[5, 7]);
    let mcs = move_cardset_ref(&repl).expect("edge still a Move");

    let mut gd = GameData::new();
    gd.memories.insert(
        format!("Table_{}", SYNTH_MEMORY_KEY),
        MemoryValue::CardSet(vec![5, 7]),
    );
    let (loc_idx, card_ids) =
        Evaluator::eval_cardset(mcs_from_ref(mcs), &gd).expect("eval_cardset ok");
    assert_eq!(card_ids, vec![5, 7]);
    // No location holds card 5 in this empty GameData, so the fallback
    // sentinel loc_idx 0 is returned (invariant I-14).
    assert_eq!(loc_idx, 0);
}

#[test]
fn build_dest_all_chain_length_and_targets() {
    let edge = move_qty_edge(
        Quantity::Int {
            int: IntExpr::Literal { int: 1 },
        },
        loc_cardset("Stock"),
        groupowner_cardset("Hand", aggregate_owner(Quantifier::All)),
    );
    let mut next = u32::MAX - 1;
    let chain = build_dest_all_chain(
        &edge,
        vec!["P1".into(), "P2".into(), "P3".into()],
        &mut next,
    )
    .expect("chain builds");
    assert_eq!(chain.len(), 3);
    // Each per-player edge must target the next synth (or the original
    // `edge.to` for the last).
    assert_eq!(chain[1].0, chain[0].1.to, "edge 0 targets synth 1");
    assert_eq!(chain[2].0, chain[1].1.to, "edge 1 targets synth 2");
    assert_eq!(chain[2].1.to, edge.to, "last edge targets the original to");
    // Each per-player edge has a concrete Player owner.
    for (_, e) in &chain {
        let mcs = move_cardset_ref(e).expect("per-player edge is a Move");
        assert!(
            matches!(
                mcs_to_ref(mcs),
                CardSet::GroupOwner {
                    owner: Owner::Player { .. },
                    ..
                }
            ),
            "per-player edge must have a concrete Player owner"
        );
    }
}

#[test]
fn build_dest_all_chain_empty_is_noop() {
    let edge = move_qty_edge(
        Quantity::Int {
            int: IntExpr::Literal { int: 1 },
        },
        loc_cardset("Stock"),
        groupowner_cardset("Hand", aggregate_owner(Quantifier::All)),
    );
    let mut next = u32::MAX - 1;
    let chain = build_dest_all_chain(&edge, vec![], &mut next).expect("empty chain ok");
    assert!(chain.is_empty());
}

#[test]
fn build_dest_all_chain_errors_over_cap() {
    let edge = move_qty_edge(
        Quantity::Int {
            int: IntExpr::Literal { int: 1 },
        },
        loc_cardset("Stock"),
        groupowner_cardset("Hand", aggregate_owner(Quantifier::All)),
    );
    let mut next = u32::MAX - 1;
    let names: Vec<String> = (0..=FANOUT_CAP).map(|i| format!("P{i}")).collect();
    let result = build_dest_all_chain(&edge, names, &mut next);
    assert!(result.is_err(), "fan-out > cap must error");
}

#[test]
fn validate_int_range_accepts_in_range_count() {
    // `>= 1 and <= 3`
    let range = IntRange {
        start: (IntCompare::Ge, IntExpr::Literal { int: 1 }),
        op_int: vec![(
            IntRangeOperator::And,
            IntCompare::Le,
            IntExpr::Literal { int: 3 },
        )],
    };
    assert!(validate_int_range(&range, 1, 10).is_ok());
    assert!(validate_int_range(&range, 2, 10).is_ok());
    assert!(validate_int_range(&range, 3, 10).is_ok());
}

#[test]
fn validate_int_range_rejects_out_of_range_count() {
    let range = IntRange {
        start: (IntCompare::Ge, IntExpr::Literal { int: 1 }),
        op_int: vec![(
            IntRangeOperator::And,
            IntCompare::Le,
            IntExpr::Literal { int: 3 },
        )],
    };
    assert!(validate_int_range(&range, 0, 10).is_err(), "0 < 1");
    assert!(validate_int_range(&range, 4, 10).is_err(), "4 > 3");
    assert!(validate_int_range(&range, 100, 10).is_err(), "100 > 3");
}

#[test]
fn setup_contains_any_false_for_create_player() {
    let setup = SetUpRule::CreatePlayer {
        players: vec!["P1".into()],
    };
    assert!(!setup_contains_any(&setup));
}

#[test]
fn setup_contains_any_true_for_create_location_any() {
    let setup = SetUpRule::CreateLocation {
        locations: vec!["Hand".into()],
        owner: aggregate_owner(Quantifier::Any),
    };
    assert!(setup_contains_any(&setup));
}

#[test]
fn setup_contains_any_false_for_create_location_all() {
    let setup = SetUpRule::CreateLocation {
        locations: vec!["Hand".into()],
        owner: aggregate_owner(Quantifier::All),
    };
    assert!(!setup_contains_any(&setup));
}

#[test]
fn setup_contains_any_false_for_create_location_literal() {
    let setup = SetUpRule::CreateLocation {
        locations: vec!["Hand".into()],
        owner: Owner::PlayerCollection {
            player_collection: PlayerCollection::Literal {
                players: vec![front_end::ast::PlayerExpr::Literal { name: "P1".into() }],
            },
        },
    };
    assert!(!setup_contains_any(&setup));
}

#[test]
fn setup_contains_any_true_for_create_turnorder_any() {
    let setup = SetUpRule::CreateTurnorder {
        player_collection: PlayerCollection::Aggregate {
            aggregate: AggregatePlayerCollection::Quantifier {
                quantifier: Quantifier::Any,
            },
        },
    };
    assert!(setup_contains_any(&setup));
}

#[test]
fn setup_contains_any_true_for_create_teams_any_member() {
    let setup = SetUpRule::CreateTeams {
        teams: vec![(
            "T1".into(),
            PlayerCollection::Aggregate {
                aggregate: AggregatePlayerCollection::Quantifier {
                    quantifier: Quantifier::Any,
                },
            },
        )],
    };
    assert!(setup_contains_any(&setup));
}

#[test]
fn setup_contains_any_true_for_create_memory_any_owner() {
    let setup = SetUpRule::CreateMemory {
        memory: "M".into(),
        owner: aggregate_owner(Quantifier::Any),
    };
    assert!(setup_contains_any(&setup));
}

/// A `Hand where Rank is "Ace" of <owner>` cardset (where-filtered GroupOwner).
fn where_filtered_cardset(owner: Owner) -> CardSet {
    CardSet::GroupOwner {
        group: Group::Where {
            groupable: Groupable::Location {
                name: "Hand".to_string(),
            },
            filter: FilterExpr::Aggregate {
                aggregate: AggregateFilter::KeyIsString {
                    key: "Rank".to_string(),
                    string: Box::new(front_end::ast::StringExpr::Literal {
                        value: "Ace".to_string(),
                    }),
                },
            },
        },
        owner,
    }
}

#[test]
fn scan_edge_source_player_any() {
    let edge = move_qty_edge(
        Quantity::Int {
            int: IntExpr::Literal { int: 1 },
        },
        where_filtered_cardset(aggregate_owner(Quantifier::Any)),
        loc_cardset("Stock"),
    );
    assert!(matches!(
        scan_edge(&edge),
        QuantSite::SourcePlayerAny { .. }
    ));
}

#[test]
fn scan_edge_source_any_takes_precedence_over_card_quantity() {
    // `deal any from Hand of any …`: the source owner must be resolved first
    // (a multi-player owner cannot be evaluated), so the site is
    // `SourcePlayerAny`, not `SrcCardsAnyOrRange`.
    let edge = move_qty_edge(
        Quantity::Quantifier {
            quantifier: Quantifier::Any,
        },
        where_filtered_cardset(aggregate_owner(Quantifier::Any)),
        loc_cardset("Stock"),
    );
    assert!(matches!(
        scan_edge(&edge),
        QuantSite::SourcePlayerAny { .. }
    ));
}

#[test]
fn scan_edge_source_any_precedes_combo_source() {
    // `move Book in Hand of any …`: the from is *both* a combo group and
    // `any`-owned; the owner must be resolved before the combo filter can be
    // evaluated, so the site is `SourcePlayerAny`.
    let from = CardSet::GroupOwner {
        group: Group::Combo {
            combo: "Book".to_string(),
            groupable: Groupable::Location {
                name: "Hand".to_string(),
            },
        },
        owner: aggregate_owner(Quantifier::Any),
    };
    let edge = move_qty_edge(
        Quantity::Int {
            int: IntExpr::Literal { int: 1 },
        },
        from,
        loc_cardset("Books"),
    );
    assert!(matches!(
        scan_edge(&edge),
        QuantSite::SourcePlayerAny { .. }
    ));
}

#[test]
fn edge_source_any_detects_chained_source() {
    let edge = move_qty_edge(
        Quantity::Quantifier {
            quantifier: Quantifier::Any,
        },
        where_filtered_cardset(aggregate_owner(Quantifier::Any)),
        loc_cardset("Stock"),
    );
    assert!(crate::quantifier::edge_source_any(&edge).is_some());
    let clean = move_qty_edge(
        Quantity::Quantifier {
            quantifier: Quantifier::Any,
        },
        loc_cardset("Stock"),
        loc_cardset("Discard"),
    );
    assert!(crate::quantifier::edge_source_any(&clean).is_none());
}

/// A `SetUp` edge carrying the given rule.
fn setup_edge(setup: SetUpRule) -> Edge<LoweredPayLoad> {
    Edge {
        to: dest_state(),
        payload: Payload::Action(GameRule::SetUp { setup }),
        meta: None,
    }
}

#[test]
fn substitute_setup_any_replaces_location_owner() {
    let edge = setup_edge(SetUpRule::CreateLocation {
        locations: vec!["Hand".to_string()],
        owner: aggregate_owner(Quantifier::Any),
    });
    let repl = substitute_setup_any(&edge, "P2".to_string());
    let Payload::Action(GameRule::SetUp { setup }) = &repl.payload else {
        panic!("expected a setup payload");
    };
    let SetUpRule::CreateLocation { owner, .. } = setup else {
        panic!("expected CreateLocation");
    };
    assert_eq!(
        owner,
        &Owner::Player {
            player: PlayerExpr::Literal {
                name: "P2".to_string(),
            },
        }
    );
}

#[test]
fn substitute_setup_any_replaces_turnorder_collection() {
    let edge = setup_edge(SetUpRule::CreateTurnorder {
        player_collection: PlayerCollection::Aggregate {
            aggregate: AggregatePlayerCollection::Quantifier {
                quantifier: Quantifier::Any,
            },
        },
    });
    let repl = substitute_setup_any(&edge, "P3".to_string());
    let Payload::Action(GameRule::SetUp { setup }) = &repl.payload else {
        panic!("expected a setup payload");
    };
    let SetUpRule::CreateTurnorder { player_collection } = setup else {
        panic!("expected CreateTurnorder");
    };
    assert_eq!(
        player_collection,
        &PlayerCollection::Literal {
            players: vec![PlayerExpr::Literal {
                name: "P3".to_string(),
            }],
        }
    );
}

#[test]
fn substitute_setup_any_leaves_any_free_setups_untouched() {
    let edge = setup_edge(SetUpRule::CreatePlayer {
        players: vec!["P1".to_string()],
    });
    let repl = substitute_setup_any(&edge, "P2".to_string());
    let Payload::Action(GameRule::SetUp { setup }) = &repl.payload else {
        panic!("expected a setup payload");
    };
    let SetUpRule::CreatePlayer { players } = setup else {
        panic!("expected CreatePlayer");
    };
    assert_eq!(players, &vec!["P1".to_string()]);
}

#[test]
fn substitute_source_player_rewrites_from_owner() {
    let edge = move_qty_edge(
        Quantity::Int {
            int: IntExpr::Literal { int: 1 },
        },
        where_filtered_cardset(aggregate_owner(Quantifier::Any)),
        loc_cardset("Stock"),
    );
    let repl = substitute_source_player(&edge, "P2".to_string());
    let Payload::Action(GameRule::Action {
        action: ActionRule::Move { move_type },
    }) = &repl.payload
    else {
        panic!("expected a move payload");
    };
    let MoveType::Classic { classic } = move_type else {
        panic!("expected a classic move");
    };
    let front_end::ast::ClassicMove::MoveCardSet { move_cs } = classic;
    let front_end::ast::MoveCardSet::MoveQuantity { from, .. } = move_cs else {
        panic!("expected MoveQuantity");
    };
    assert_eq!(
        from,
        &where_filtered_cardset(Owner::Player {
            player: front_end::ast::PlayerExpr::Literal {
                name: "P2".to_string(),
            },
        }),
        "the from owner must be substituted with the chosen player"
    );
}
