use front_end::ast::{
    ActionRule, AggregatePlayerCollection, CardSet, ClassicMove, DealMove, ExchangeMove,
    GameRule, IntExpr, MoveCardSet, MoveType, Owner, PlayerCollection, Quantity, Quantifier,
    SetUpRule,
};
use front_end::ir::{Edge, Ir, StateID};

use crate::engine_payload::EnginePayload;
use crate::input_request::InputRequest;

pub fn reify_quantifiers(
    mut ir: Ir<EnginePayload>,
    _game_data: &crate::game_data::GameData,
) -> Ir<EnginePayload> {
    let mut new_states: Vec<(StateID, Vec<Edge<EnginePayload>>)> = Vec::new();

    for (state_id, edges) in ir.states.iter_mut() {
        let mut new_edges = Vec::new();

        for edge in edges.drain(..) {
            let injected = inject_quantifier_edges(edge);
            for e in injected {
                new_edges.push(e);
            }
        }

        new_states.push((*state_id, new_edges));
    }

    ir.states = new_states.into_iter().collect();
    ir
}

fn inject_quantifier_edges(
    edge: Edge<EnginePayload>,
) -> Vec<Edge<EnginePayload>> {
    match &edge.payload {
        EnginePayload::Action(GameRule::Action { action }) => {
            match action {
                ActionRule::Move { move_type } => {
                    if let Some((request, original_action)) = detect_move_quantifier(move_type) {
                        let needs_input_edge = Edge {
                            to: edge.to,
                            payload: EnginePayload::NeedsInput {
                                request,
                                original_action: GameRule::Action { action: original_action },
                            },
                            meta: None,
                        };
                        vec![needs_input_edge]
                    } else {
                        vec![edge]
                    }
                }
                _ => vec![edge],
            }
        }
        EnginePayload::Action(GameRule::SetUp { setup }) => {
            if let Some((request, resolved_setup)) = detect_owner_any_in_setup(setup) {
                let needs_input_edge = Edge {
                    to: edge.to,
                    payload: EnginePayload::NeedsInput {
                        request,
                        original_action: GameRule::SetUp { setup: resolved_setup },
                    },
                    meta: None,
                };
                vec![needs_input_edge]
            } else {
                vec![edge]
            }
        }
        _ => vec![edge],
    }
}

fn detect_move_quantifier(move_type: &MoveType) -> Option<(InputRequest, ActionRule)> {
    match move_type {
        MoveType::Deal { deal } => {
            let DealMove::MoveCardSet { deal_cs } = deal;
            if let MoveCardSet::MoveQuantity { quantity, from, status, to } = deal_cs {
                return detect_and_resolve_quantity(quantity, from.clone(), status.clone(), to.clone(), |qty, from, status, to| {
                    ActionRule::Move {
                        move_type: MoveType::Deal {
                            deal: DealMove::MoveCardSet {
                                deal_cs: MoveCardSet::MoveQuantity {
                                    quantity: qty,
                                    from,
                                    status,
                                    to,
                                },
                            },
                        },
                    }
                });
            }
            None
        }
        MoveType::Exchange { exchange } => {
            let ExchangeMove::MoveCardSet { exchange_cs } = exchange;
            if let MoveCardSet::MoveQuantity { quantity, from, status, to } = exchange_cs {
                return detect_and_resolve_quantity(quantity, from.clone(), status.clone(), to.clone(), |qty, from, status, to| {
                    ActionRule::Move {
                        move_type: MoveType::Exchange {
                            exchange: ExchangeMove::MoveCardSet {
                                exchange_cs: MoveCardSet::MoveQuantity {
                                    quantity: qty,
                                    from,
                                    status,
                                    to,
                                },
                            },
                        },
                    }
                });
            }
            None
        }
        MoveType::Classic { classic } => {
            let ClassicMove::MoveCardSet { move_cs } = classic;
            if let MoveCardSet::MoveQuantity { quantity, from, status, to } = move_cs {
                return detect_and_resolve_quantity(quantity, from.clone(), status.clone(), to.clone(), |qty, from, status, to| {
                    ActionRule::Move {
                        move_type: MoveType::Classic {
                            classic: ClassicMove::MoveCardSet {
                                move_cs: MoveCardSet::MoveQuantity {
                                    quantity: qty,
                                    from,
                                    status,
                                    to,
                                },
                            },
                        },
                    }
                });
            }
            None
        }
        MoveType::Place { .. } => None,
    }
}

fn detect_and_resolve_quantity<F>(
    quantity: &Quantity,
    from: CardSet,
    status: front_end::ast::Status,
    to: CardSet,
    make_action: F,
) -> Option<(InputRequest, ActionRule)>
where
    F: FnOnce(Quantity, CardSet, front_end::ast::Status, CardSet) -> ActionRule,
{
    match quantity {
        Quantity::Quantifier { quantifier } => match quantifier {
            Quantifier::All => None,
            Quantifier::Any => {
                let request = InputRequest::PickItems {
                    cardset: from.clone(),
                    min: 1,
                    max: 0,
                    context: "Select a card to move".to_string(),
                };
                let resolved = make_action(
                    Quantity::Int { int: IntExpr::Literal { int: 1 } },
                    from,
                    status,
                    to,
                );
                Some((request, resolved))
            }
        },
        Quantity::IntRange { int_range } => {
            let request = InputRequest::PickCount {
                int_range: int_range.clone(),
                context: "How many cards?".to_string(),
            };
            let resolved = make_action(
                Quantity::Int { int: IntExpr::Literal { int: 1 } },
                from,
                status,
                to,
            );
            Some((request, resolved))
        }
        Quantity::Int { .. } => None,
    }
}

fn detect_owner_any_in_setup(setup: &SetUpRule) -> Option<(InputRequest, SetUpRule)> {
    match setup {
        SetUpRule::CreateLocation { locations, owner } => {
            if let Owner::PlayerCollection { player_collection } = owner {
                if let PlayerCollection::Aggregate { aggregate } = player_collection {
                    if let AggregatePlayerCollection::Quantifier { quantifier } = aggregate {
                        if *quantifier == Quantifier::Any {
                            let players: Vec<String> = (0..4).map(|i| format!("Player{}", i)).collect();
                            let request = InputRequest::PickPlayer {
                                players,
                                min: 1,
                                max: 1,
                                context: "Select a player".to_string(),
                            };
                            let resolved = SetUpRule::CreateLocation {
                                locations: locations.clone(),
                                owner: Owner::Player {
                                    player: front_end::ast::PlayerExpr::Literal {
                                        name: "Player0".to_string(),
                                    },
                                },
                            };
                            return Some((request, resolved));
                        }
                    }
                }
            }
            None
        }
        _ => None,
    }
}