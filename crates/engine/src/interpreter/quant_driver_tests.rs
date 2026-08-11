//! Unit tests for the quantifier resume state machine (`take_quant_resume`
//! and the per-`PendingKind` resume arms). These pin the I-19 (state-match)
//! and I-21 (stale-input) guards and the dispatch / re-prompt / error
//! contract of every resume arm, independent of the integration fixtures in
//! `tests/quantifier_test.rs`.

use crate::game_data::{Combo, GameData, Location, MemoryValue};
use crate::interpreter::types::{Input, InputKind, InputType, StepResult};
use crate::interpreter::Interpreter;
use crate::quantifier::{PendingKind, PendingQuant, SYNTH_MEMORY_KEY};
use crate::EngineError;
use front_end::ast::{
    ActionRule, AggregateFilter, AggregatePlayerCollection, CardSet, DealMove, FilterExpr,
    GameRule, Group, Groupable, MoveCardSet, MoveType, Owner, PlayerCollection, Quantifier,
    Quantity, SetUpRule, Status,
};
use front_end::ir::{Edge, Ir, LoweredPayLoad, Payload, StateID};
use std::collections::HashMap;

fn synth_state_id(n: u32) -> StateID {
    // `StateID(u32)` is `#[repr(transparent)]`; transmute is a test-only
    // shortcut (matches the helper in interpreter/tests.rs).
    unsafe { std::mem::transmute(n) }
}

fn loc_cs(name: &str) -> CardSet {
    CardSet::Group {
        group: Group::Groupable {
            groupable: Groupable::Location {
                name: name.to_string(),
            },
        },
    }
}

/// A `deal <quantity> from <from> private to <to>` edge with a concrete
/// (quantifier-free) destination — the shape every resume arm dispatches.
fn deal_edge(quantity: Quantity, from: CardSet, to: CardSet) -> Edge<LoweredPayLoad> {
    Edge {
        to: synth_state_id(1),
        payload: Payload::Action(GameRule::Action {
            action: ActionRule::Move {
                move_type: MoveType::Deal {
                    deal: DealMove::MoveCardSet {
                        deal_cs: MoveCardSet::MoveQuantity {
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

/// P1 + P2, a table `Stock` with `card_ids` (Ace, King) and a table `Hand`.
fn stock_hand_game() -> (GameData, Vec<usize>) {
    let mut gd = GameData::new();
    let p0 = gd.add_player("P1".to_string());
    let p1 = gd.add_player("P2".to_string());
    gd.turn_order = vec![p0, p1];
    let stock = gd.add_location(
        "Table".to_string(),
        Location {
            name: "Stock".to_string(),
            cards: vec![],
        },
    );
    gd.add_location(
        "Table".to_string(),
        Location {
            name: "Hand".to_string(),
            cards: vec![],
        },
    );
    let c0 = gd.add_card(
        stock,
        HashMap::from([("Rank".to_string(), "Ace".to_string())]),
    );
    let c1 = gd.add_card(
        stock,
        HashMap::from([("Rank".to_string(), "King".to_string())]),
    );
    gd.locations[stock].cards = vec![c0, c1];
    (gd, vec![c0, c1])
}

/// An `Interpreter` at `current_state = synth_state_id(10)` with one buffered
/// input and the given pending quantifier.
fn interp_with(gd: GameData, pending: PendingQuant, input: Input) -> Interpreter {
    Interpreter {
        ir: Ir::<LoweredPayLoad>::default(),
        game_data: gd,
        input_buffer: vec![input],
        current_state: synth_state_id(10),
        trace_sender: None,
        pending_overlay: HashMap::new(),
        next_synth: u32::MAX - 1,
        pending_quant: Some(pending),
    }
}

fn choice_input() -> Input {
    Input {
        player_id: "P1".into(),
        kind: InputKind::Choice { idx: 0 },
    }
}

fn choose_player_input(idx: usize) -> Input {
    Input {
        player_id: "P1".into(),
        kind: InputKind::ChoosePlayer { idx },
    }
}

fn choose_cards_input(selected: Vec<usize>) -> Input {
    Input {
        player_id: "P1".into(),
        kind: InputKind::ChooseCards { selected },
    }
}

fn number_input(value: i32) -> Input {
    Input {
        player_id: "P1".into(),
        kind: InputKind::Number { value },
    }
}

#[test]
fn take_quant_resume_returns_none_when_no_pending_quant() {
    let (gd, _) = stock_hand_game();
    let mut interp = Interpreter {
        ir: Ir::<LoweredPayLoad>::default(),
        game_data: gd,
        input_buffer: Vec::new(),
        current_state: synth_state_id(0),
        trace_sender: None,
        pending_overlay: HashMap::new(),
        next_synth: u32::MAX - 1,
        pending_quant: None,
    };
    let before_state = interp.current_state;
    let resumed = interp.take_quant_resume();
    assert!(resumed.is_none(), "no pending quant => None");
    assert_eq!(interp.current_state, before_state, "state unchanged");
}

/// I-19: the pending quantifier only resumes at the state it was issued from;
/// a state mismatch leaves both the pending request and the input untouched.
#[test]
fn resume_no_ops_when_pending_state_mismatches_current() {
    let (gd, _) = stock_hand_game();
    let pending = PendingQuant {
        state: synth_state_id(9),
        kind: PendingKind::CardsAnyOrRange {
            candidate_ids: vec![0],
            original: deal_edge(
                Quantity::Quantifier {
                    quantifier: Quantifier::Any,
                },
                loc_cs("Stock"),
                loc_cs("Hand"),
            ),
        },
    };
    let mut interp = interp_with(gd, pending, choice_input());
    let resumed = interp.take_quant_resume();
    assert!(resumed.is_none(), "state mismatch => None");
    assert_eq!(interp.input_buffer.len(), 1, "input left in place");
    assert!(interp.pending_quant.is_some(), "pending quant preserved");
}

/// I-21: a buffered input whose kind does not match the pending prompt is
/// discarded (preventing an infinite prompt loop); the pending quant survives.
#[test]
fn resume_pops_stale_input_and_preserves_pending() {
    let (gd, _) = stock_hand_game();
    let pending = PendingQuant {
        state: synth_state_id(10),
        kind: PendingKind::CardsAnyOrRange {
            candidate_ids: vec![0, 1],
            original: deal_edge(
                Quantity::Quantifier {
                    quantifier: Quantifier::Any,
                },
                loc_cs("Stock"),
                loc_cs("Hand"),
            ),
        },
    };
    let mut interp = interp_with(gd, pending, choice_input());
    let resumed = interp.take_quant_resume();
    assert!(resumed.is_none(), "kind mismatch => None");
    assert!(interp.input_buffer.is_empty(), "stale input popped");
    assert!(
        matches!(
            interp.pending_quant.as_ref().map(|p| &p.kind),
            Some(PendingKind::CardsAnyOrRange { .. })
        ),
        "pending quant preserved for a future matching input"
    );
}

/// `DestPlayerAny`: a valid index substitutes the player and dispatches
/// (overlay entry); an out-of-range index is a fatal error (I-8).
#[test]
fn resume_dest_player_any_dispatches_or_errors() {
    let (gd, _) = stock_hand_game();
    let edge = deal_edge(
        Quantity::Int {
            int: front_end::ast::IntExpr::Literal { int: 1 },
        },
        loc_cs("Stock"),
        loc_cs("Hand"),
    );
    let pending = PendingQuant {
        state: synth_state_id(10),
        kind: PendingKind::DestPlayerAny {
            candidates: vec!["P1".to_string(), "P2".to_string()],
            original: edge.clone(),
        },
    };
    let mut interp = interp_with(gd, pending, choose_player_input(1));
    match interp.take_quant_resume() {
        Some(StepResult::Ok) => {}
        other => panic!("expected Ok resume, got {other:?}"),
    }
    assert!(interp.pending_quant.is_none(), "pending cleared");
    assert!(interp.input_buffer.is_empty(), "input consumed");
    assert!(
        !interp.pending_overlay.is_empty(),
        "the substituted edge is queued for dispatch"
    );

    let mut interp = interp_with(
        stock_hand_game().0,
        PendingQuant {
            state: synth_state_id(10),
            kind: PendingKind::DestPlayerAny {
                candidates: vec!["P1".to_string(), "P2".to_string()],
                original: edge,
            },
        },
        choose_player_input(5),
    );
    match interp.take_quant_resume() {
        Some(StepResult::Error(e)) => {
            assert!(
                matches!(e, EngineError::ChoosePlayerIdxOutOfRange { .. }),
                "got {e}"
            );
        }
        other => panic!("expected out-of-range error, got {other:?}"),
    }
}

/// `SourcePlayerAny`: mirrors `DestPlayerAny` for the move's source owner.
#[test]
fn resume_source_player_any_dispatches() {
    let (gd, _) = stock_hand_game();
    let pending = PendingQuant {
        state: synth_state_id(10),
        kind: PendingKind::SourcePlayerAny {
            candidates: vec!["P1".to_string(), "P2".to_string()],
            original: deal_edge(
                Quantity::Int {
                    int: front_end::ast::IntExpr::Literal { int: 1 },
                },
                loc_cs("Stock"),
                loc_cs("Hand"),
            ),
        },
    };
    let mut interp = interp_with(gd, pending, choose_player_input(0));
    assert!(matches!(interp.take_quant_resume(), Some(StepResult::Ok)));
    assert!(interp.pending_quant.is_none());
}

/// `SetupAny`: the chosen player is substituted into every any-site of the
/// setup rule before dispatch — the next `step()` creates the location for
/// that player only (I-20).
#[test]
fn resume_setup_any_creates_location_for_chosen_player() {
    let (mut gd, _) = stock_hand_game();
    gd.players[0].owner.locations.clear();
    gd.players[1].owner.locations.clear();
    let setup_edge = Edge {
        to: synth_state_id(1),
        payload: Payload::Action(GameRule::SetUp {
            setup: SetUpRule::CreateLocation {
                locations: vec!["Hand".to_string()],
                owner: Owner::PlayerCollection {
                    player_collection: PlayerCollection::Aggregate {
                        aggregate: AggregatePlayerCollection::Quantifier {
                            quantifier: Quantifier::Any,
                        },
                    },
                },
            },
        }),
        meta: None,
    };
    let pending = PendingQuant {
        state: synth_state_id(10),
        kind: PendingKind::SetupAny {
            candidates: vec!["P1".to_string(), "P2".to_string()],
            original: setup_edge,
        },
    };
    let mut interp = interp_with(gd, pending, choose_player_input(1));
    assert!(matches!(interp.take_quant_resume(), Some(StepResult::Ok)));

    assert!(matches!(interp.step(), StepResult::Ok), "setup dispatches");
    let p2_hand = interp.game_data.players.get(1).and_then(|p| {
        p.owner
            .locations
            .iter()
            .find(|&&li| interp.game_data.locations[li].name == "Hand")
            .copied()
    });
    assert!(p2_hand.is_some(), "P2's Hand was created");
    let p1_hand = interp.game_data.players.first().and_then(|p| {
        p.owner
            .locations
            .iter()
            .find(|&&li| interp.game_data.locations[li].name == "Hand")
            .copied()
    });
    assert!(
        p1_hand.is_none(),
        "P1 got no Hand — the any-site resolved to P2"
    );
}

/// `CardsAnyOrRange`: a valid selection writes the synthetic memory slot and
/// dispatches; an out-of-range index is fatal (I-8).
#[test]
fn resume_cards_any_or_range_dispatches_with_synth_memory_or_errors() {
    let (gd, card_ids) = stock_hand_game();
    let edge = deal_edge(
        Quantity::Quantifier {
            quantifier: Quantifier::Any,
        },
        loc_cs("Stock"),
        loc_cs("Hand"),
    );
    let pending = PendingQuant {
        state: synth_state_id(10),
        kind: PendingKind::CardsAnyOrRange {
            candidate_ids: card_ids.clone(),
            original: edge.clone(),
        },
    };
    let mut interp = interp_with(gd, pending, choose_cards_input(vec![0]));
    assert!(matches!(interp.take_quant_resume(), Some(StepResult::Ok)));
    assert!(interp.pending_quant.is_none());
    assert_eq!(
        interp
            .game_data
            .memories
            .get(&format!("Table_{}", SYNTH_MEMORY_KEY)),
        Some(&MemoryValue::CardSet(vec![card_ids[0]])),
        "chosen ids travel through the synthetic slot"
    );

    // The next step dispatches the replacement edge: one card moves.
    assert!(matches!(interp.step(), StepResult::Ok));
    let hand = interp
        .game_data
        .locations
        .iter()
        .find(|l| l.name == "Hand")
        .unwrap();
    assert_eq!(hand.cards, vec![card_ids[0]]);

    let mut interp = interp_with(
        stock_hand_game().0,
        PendingQuant {
            state: synth_state_id(10),
            kind: PendingKind::CardsAnyOrRange {
                candidate_ids: card_ids,
                original: edge,
            },
        },
        choose_cards_input(vec![9]),
    );
    match interp.take_quant_resume() {
        Some(StepResult::Error(e)) => {
            assert!(
                matches!(e, EngineError::ChooseCardsIndexOutOfRange),
                "got {e}"
            );
        }
        other => panic!("expected index-out-of-range error, got {other:?}"),
    }
}

/// `CardsExactN` (`move N`): a wrong count re-prompts (pending restored);
/// an exact count dispatches with the synthetic slot.
#[test]
fn resume_cards_exact_n_reprompts_on_wrong_count_then_dispatches() {
    let (gd, card_ids) = stock_hand_game();
    let edge = deal_edge(
        Quantity::Int {
            int: front_end::ast::IntExpr::Literal { int: 2 },
        },
        loc_cs("Stock"),
        loc_cs("Hand"),
    );
    let pending = PendingQuant {
        state: synth_state_id(10),
        kind: PendingKind::CardsExactN {
            candidate_ids: card_ids.clone(),
            expected: 2,
            original: edge.clone(),
        },
    };
    let mut interp = interp_with(gd, pending, choose_cards_input(vec![0]));
    match interp.take_quant_resume() {
        Some(StepResult::NeedsInput(InputType::ChooseCards { min, max, .. })) => {
            assert_eq!((min, max), (2, 2), "re-prompt demands exactly 2");
        }
        other => panic!("expected exact-N re-prompt, got {other:?}"),
    }
    assert!(
        matches!(
            interp.pending_quant.as_ref().map(|p| &p.kind),
            Some(PendingKind::CardsExactN { .. })
        ),
        "pending restored for the re-prompt"
    );

    let mut interp = interp_with(
        stock_hand_game().0,
        PendingQuant {
            state: synth_state_id(10),
            kind: PendingKind::CardsExactN {
                candidate_ids: card_ids.clone(),
                expected: 1,
                original: edge,
            },
        },
        choose_cards_input(vec![0]),
    );
    assert!(matches!(interp.take_quant_resume(), Some(StepResult::Ok)));
    assert_eq!(
        interp
            .game_data
            .memories
            .get(&format!("Table_{}", SYNTH_MEMORY_KEY)),
        Some(&MemoryValue::CardSet(vec![card_ids[0]]))
    );
}

/// `DealCount`: an out-of-bounds count re-prompts (pending restored); an
/// in-range count substitutes the literal and deals automatically.
#[test]
fn resume_deal_count_reprompts_out_of_range_then_dispatches() {
    let (gd, card_ids) = stock_hand_game();
    let edge = deal_edge(
        Quantity::Quantifier {
            quantifier: Quantifier::Any,
        },
        loc_cs("Stock"),
        loc_cs("Hand"),
    );
    let pending = PendingQuant {
        state: synth_state_id(10),
        kind: PendingKind::DealCount {
            min: Some(1),
            max: Some(2),
            prompt: "how many?".to_string(),
            original: edge.clone(),
        },
    };
    let mut interp = interp_with(gd, pending, number_input(5));
    match interp.take_quant_resume() {
        Some(StepResult::NeedsInput(InputType::Number { min, max, .. })) => {
            assert_eq!((min, max), (Some(1), Some(2)), "bounds preserved");
        }
        other => panic!("expected count re-prompt, got {other:?}"),
    }
    assert!(
        matches!(
            interp.pending_quant.as_ref().map(|p| &p.kind),
            Some(PendingKind::DealCount { .. })
        ),
        "pending restored for the re-prompt"
    );

    let mut interp = interp_with(
        stock_hand_game().0,
        PendingQuant {
            state: synth_state_id(10),
            kind: PendingKind::DealCount {
                min: Some(1),
                max: Some(2),
                prompt: "how many?".to_string(),
                original: edge,
            },
        },
        number_input(2),
    );
    assert!(matches!(interp.take_quant_resume(), Some(StepResult::Ok)));
    assert!(matches!(interp.step(), StepResult::Ok));
    let hand = interp
        .game_data
        .locations
        .iter()
        .find(|l| l.name == "Hand")
        .unwrap();
    assert_eq!(hand.cards, card_ids, "top 2 dealt automatically");
}

/// `Combo` (lay-down): the chosen set is validated against the combo filter —
/// a matching set dispatches, a partial set re-prompts (0 = skip is handled
/// by the controller, not here).
#[test]
fn resume_combo_source_validates_against_filter() {
    // Two Aces so the Same-Rank filter can match a full selection.
    let mut gd = GameData::new();
    let p0 = gd.add_player("P1".to_string());
    let p1 = gd.add_player("P2".to_string());
    gd.turn_order = vec![p0, p1];
    let stock = gd.add_location(
        "Table".to_string(),
        Location {
            name: "Stock".to_string(),
            cards: vec![],
        },
    );
    gd.add_location(
        "Table".to_string(),
        Location {
            name: "Hand".to_string(),
            cards: vec![],
        },
    );
    let c0 = gd.add_card(
        stock,
        HashMap::from([("Rank".to_string(), "Ace".to_string())]),
    );
    let c1 = gd.add_card(
        stock,
        HashMap::from([("Rank".to_string(), "Ace".to_string())]),
    );
    gd.locations[stock].cards = vec![c0, c1];
    let card_ids = vec![c0, c1];
    gd.combos.push(Combo {
        name: "Pair".to_string(),
        filter: FilterExpr::Aggregate {
            aggregate: AggregateFilter::Same {
                key: "Rank".to_string(),
            },
        },
    });
    let edge = deal_edge(
        Quantity::Quantifier {
            quantifier: Quantifier::Any,
        },
        loc_cs("Stock"),
        loc_cs("Hand"),
    );
    let filter = gd.combos[0].filter.clone();

    // A partial selection (one card, but the filter needs a pair) re-prompts.
    let mut interp = interp_with(
        gd.clone(),
        PendingQuant {
            state: synth_state_id(10),
            kind: PendingKind::Combo {
                candidate_ids: card_ids.clone(),
                filter: filter.clone(),
                original: edge.clone(),
            },
        },
        choose_cards_input(vec![0]),
    );
    match interp.take_quant_resume() {
        Some(StepResult::NeedsInput(InputType::ChooseCards { .. })) => {}
        other => panic!("expected combo re-prompt, got {other:?}"),
    }
    assert!(
        matches!(
            interp.pending_quant.as_ref().map(|p| &p.kind),
            Some(PendingKind::Combo { .. })
        ),
        "pending restored"
    );

    // A full matching selection dispatches and the cards move on the next step.
    let mut interp = interp_with(
        gd,
        PendingQuant {
            state: synth_state_id(10),
            kind: PendingKind::Combo {
                candidate_ids: card_ids.clone(),
                filter,
                original: edge,
            },
        },
        choose_cards_input(vec![0, 1]),
    );
    assert!(matches!(interp.take_quant_resume(), Some(StepResult::Ok)));
    assert!(matches!(interp.step(), StepResult::Ok));
    let hand = interp
        .game_data
        .locations
        .iter()
        .find(|l| l.name == "Hand")
        .unwrap();
    assert_eq!(hand.cards, card_ids, "both cards laid down");
}

/// `DestAllThenCards` (`deal any … to Hand of all`): one prompt, then a
/// per-player fan-out chain whose edges all read the shared synthetic slot.
#[test]
fn resume_dest_all_then_cards_builds_fanout_chain() {
    let (gd, card_ids) = stock_hand_game();
    let edge = deal_edge(
        Quantity::Quantifier {
            quantifier: Quantifier::Any,
        },
        loc_cs("Stock"),
        loc_cs("Hand"),
    );
    let pending = PendingQuant {
        state: synth_state_id(10),
        kind: PendingKind::DestAllThenCards {
            player_names: vec!["P1".to_string(), "P2".to_string()],
            candidate_ids: card_ids.clone(),
            original: edge,
        },
    };
    let mut interp = interp_with(gd, pending, choose_cards_input(vec![0]));
    assert!(matches!(interp.take_quant_resume(), Some(StepResult::Ok)));
    assert!(interp.pending_quant.is_none());
    assert_eq!(
        interp.pending_overlay.len(),
        2,
        "one synthetic edge per player"
    );
    assert_eq!(
        interp
            .game_data
            .memories
            .get(&format!("Table_{}", SYNTH_MEMORY_KEY)),
        Some(&MemoryValue::CardSet(vec![card_ids[0]]))
    );
    assert!(
        interp.pending_overlay.keys().all(|s| s.raw() > 1000),
        "overlay keyed only by synthetic ids (I-17)"
    );
}
