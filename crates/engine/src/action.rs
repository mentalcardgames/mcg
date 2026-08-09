/*
The purpose of action.rs is to define all possible game data modifications based on edge payload type. The payload type tree is:
Payload
├── Condition { expr: L::BoolExpr, negated: bool }
├── EndCondition { expr: L::EndCondition, negated: bool }
│       EndCondition: UntilBool | UntilBoolRep | UntilRep | UntilEnd
├── Action(L::GameRule)
│   ├── SetUp { setup: L::SetUpRule }
│   │       SetUpRule: CreatePlayer | CreateTeam | CreateTurnorder | CreateTurnorderRandom
│   │                  CreateLocation | CreateCardOnLocation | CreateTokenOnLocation
│   │                  CreateCombo | CreateMemory | CreateMemoryWithMemoryType
│   │                  CreatePrecedence | CreatePointMap
│   ├── Action { action: L::ActionRule }
│   │   ├── FlipAction { card_set: L::CardSet, status: L::Status }
│   │   ├── ShuffleAction { card_set: L::CardSet }
│   │   ├── OutAction { players: L::Players, out_of: L::OutOf }
│   │   ├── SetMemory { memory: String, memory_type: L::MemoryType }
│   │   ├── ResetMemory { memory: String }
│   │   ├── CycleAction { player: L::PlayerExpr }
│   │   ├── BidAction { quantity: L::Quantity }
│   │   ├── BidMemoryAction { memory, quantity, owner }
│   │   ├── EndAction { end_type: L::EndType }
│   │   │       EndType: Turn | CurrentStage | Stage { stage } | GameWithWinner { players }
│   │   ├── DemandAction { demand_type: L::DemandType }
│   │   ├── DemandMemoryAction { demand_type, memory }
│   │   └── Move { move_type: L::MoveType }
│   │           MoveType: Deal | Exchange | Classic | Place
│   └── Scoring { scoring: L::ScoringRule }
│           ScoringRule: Score { int, players } | ScoreMemory { int, memory, players }
├── StageRoundCounter(String)
├── EndStage(String)          ← NOT emitted by IrBuilder (only Action→EndAction creates it via jump)
├── Choice                    ← no data (edge index is implicit)
├── Optional                  ← no data (accept/decline is edge order)
└── Trigger                   ← not implemented.

Each of the leaves of this payload tree should be accounted for in the execute_edge function, which takes a Payload and modifies the game state accordingly.
*/

use crate::game_data::{Combo, GameData, Location, PointMap, Precedence, Team};
use front_end::ast::{
    ActionRule, GameRule, MoveType, OutOf, Quantity, ScoringRule, SetUpRule, Status,
};
use front_end::ir::LoweredPayLoad;

#[allow(clippy::single_match)]
pub fn execute(payload: LoweredPayLoad, game_data: &mut GameData) {
    match payload {
        LoweredPayLoad::Action(gr) => match gr {
            GameRule::SetUp { setup } => execute_setup_rule(setup, game_data),
            GameRule::Action { action } => execute_action_rule(action, game_data),
            GameRule::Scoring { scoring } => execute_scoring_rule(scoring, game_data),
        },
        // StageRoundCounter and EndStage are applied by the interpreter's `step()`
        // (which mutates game_data before calling execute_edge). Re-applying them
        // here would double-increment the round counter and double-leave the stage
        // stack (see invariants I-5). They intentionally fall through to `_ => {}`.
        _ => {}
    }
}

pub fn execute_setup_rule(payload: SetUpRule, game_data: &mut GameData) {
    match payload {
        SetUpRule::CreatePlayer { players } => {
            for name in players {
                let idx = game_data.add_player(name);
                game_data.turn_order.push(idx);
            }
        }
        SetUpRule::CreateTeams { teams } => {
            for (team_name, player_collection) in teams {
                let player_indices =
                    crate::quantifier::resolve_player_candidates(&player_collection, game_data);
                game_data.teams.push(Team {
                    name: team_name,
                    players: player_indices,
                });
            }
        }
        SetUpRule::CreateTurnorder { player_collection } => {
            let indices =
                crate::quantifier::resolve_player_candidates(&player_collection, game_data);
            game_data.turn_order = indices;
        }
        SetUpRule::CreateTurnorderRandom { player_collection } => {
            use rand::seq::SliceRandom;
            let mut indices =
                crate::quantifier::resolve_player_candidates(&player_collection, game_data);
            indices.shuffle(&mut rand::thread_rng());
            game_data.turn_order = indices;
        }
        SetUpRule::CreateLocation { locations, owner } => {
            let owner_names = crate::query::Evaluator::resolve_owner_to_names(&owner, game_data)
                .unwrap_or_else(|e| {
                    panic!("CreateLocation: failed to resolve owner {:?}: {}", owner, e)
                });
            for owner_name in &owner_names {
                for loc_name in &locations {
                    game_data.add_location(
                        owner_name.clone(),
                        Location {
                            name: loc_name.clone(),
                            cards: vec![],
                        },
                    );
                }
            }
        }
        SetUpRule::CreateCardOnLocation { location, cards } => {
            let loc_idx = game_data
                .locations
                .iter()
                .position(|l| l.name == location)
                .unwrap_or_else(|| {
                    panic!("CreateCardOnLocation: location {:?} not found", location)
                });
            for type_expr in cards {
                let expanded_cards = crate::query::Evaluator::expand_types(&type_expr);
                for card in expanded_cards {
                    let card_id = game_data.add_card(loc_idx, card);
                    game_data.locations[loc_idx].cards.push(card_id);
                }
            }
        }
        SetUpRule::CreateTokenOnLocation { .. } => {}
        SetUpRule::CreateCombo { combo, filter } => {
            game_data.combos.push(Combo {
                name: combo,
                filter,
            });
        }
        SetUpRule::CreateMemory { memory, owner } => {
            let names = crate::query::Evaluator::resolve_owner_to_names(&owner, game_data)
                .unwrap_or_else(|e| {
                    panic!("CreateMemory: failed to resolve owner {:?}: {}", owner, e)
                });
            for name in names {
                let key = format!("{}_{}", name, memory);
                game_data.add_memory(key, owner.clone(), None);
            }
        }
        SetUpRule::CreateMemoryWithMemoryType {
            memory,
            owner,
            memory_type,
        } => {
            let names = crate::query::Evaluator::resolve_owner_to_names(&owner, game_data)
                .unwrap_or_else(|e| {
                    panic!(
                        "CreateMemoryWithMemoryType: failed to resolve owner {:?}: {}",
                        owner, e
                    )
                });
            for name in names {
                let key = format!("{}_{}", name, memory);
                game_data.add_memory(key, owner.clone(), Some(memory_type.clone()));
            }
        }
        SetUpRule::CreatePrecedence { precedence, kvs } => {
            game_data.precedences.push(Precedence {
                name: precedence,
                key: kvs[0].0.clone(),
                values: kvs.into_iter().map(|(_, v)| v).collect(),
            });
        }
        SetUpRule::CreatePointMap { pointmap, kvis } => {
            let mut map = std::collections::HashMap::new();
            for (key, value, int_expr) in kvis {
                let card_key = format!("{}:{}", key, value);
                let points = crate::query::Evaluator::eval_int(&int_expr, game_data)
                    .unwrap_or_else(|e| {
                        panic!(
                            "CreatePointMap: failed to eval int {:?} for key {}:{}: {}",
                            int_expr, key, value, e
                        )
                    });
                map.insert(card_key, points);
            }
            game_data.point_maps.push(PointMap {
                name: pointmap,
                map,
            });
        }
    }
}

pub(crate) fn execute_action_rule(action: ActionRule, game_data: &mut GameData) {
    match action {
        ActionRule::FlipAction {
            card_set: _,
            status: _,
        } => {}
        ActionRule::ShuffleAction { card_set } => {
            use rand::seq::SliceRandom;

            let card_set_clone = card_set.clone();
            let result = crate::query::Evaluator::eval_cardset(&card_set_clone, game_data);
            match result {
                Ok((location_idx, card_ids)) => {
                    if let Some(loc) = game_data.locations.get_mut(location_idx) {
                        // Only shuffle the *selected* cards, leaving any
                        // unselected cards in the location untouched
                        // (e.g. `shuffle top 3 of Deck` must not discard the
                        // rest of the pile).
                        let positions: Vec<usize> = loc
                            .cards
                            .iter()
                            .enumerate()
                            .filter(|(_, id)| card_ids.contains(id))
                            .map(|(i, _)| i)
                            .collect();
                        if positions.len() == loc.cards.len() {
                            loc.cards.shuffle(&mut rand::thread_rng());
                        } else if positions.len() > 1 {
                            let mut shuffled: Vec<usize> =
                                positions.iter().map(|&i| loc.cards[i]).collect();
                            shuffled.shuffle(&mut rand::thread_rng());
                            for (slot, id) in positions.into_iter().zip(shuffled) {
                                loc.cards[slot] = id;
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("ShuffleAction failed: {}", e);
                }
            }
        }
        ActionRule::OutAction { players, out_of } => {
            let player_indices = crate::query::Evaluator::resolve_players(&players, game_data);
            let current_stage = game_data.get_current_stage().unwrap_or_default();
            for pid in player_indices {
                match out_of {
                    OutOf::CurrentStage => {
                        game_data.set_player_stage_flag(pid, current_stage.clone(), false)
                    }
                    OutOf::Stage { ref name } => {
                        game_data.set_player_stage_flag(pid, name.clone(), false)
                    }
                    OutOf::Game | OutOf::GameSuccessful | OutOf::GameFail => {
                        game_data.set_player_out(pid)
                    }
                }
            }
        }
        ActionRule::SetMemory {
            memory,
            memory_type,
        } => {
            use crate::game_data::MemoryValue;
            use front_end::ast::MemoryType;
            let value: MemoryValue = match memory_type {
                MemoryType::Int { int } => {
                    let n = crate::query::Evaluator::eval_int(&int, game_data)
                        .unwrap_or_else(|e| panic!("SetMemory Int eval failed: {e}"));
                    MemoryValue::Int(n)
                }
                MemoryType::String { string } => {
                    let s = crate::query::Evaluator::eval_string(&string, game_data)
                        .unwrap_or_else(|e| panic!("SetMemory String eval failed: {e}"));
                    MemoryValue::String(s)
                }
                MemoryType::Player { player } => {
                    // Evaluator::eval_player returns a name; we store the name as
                    // a String memory so later reads as String succeed. (Storing
                    // a player index would require a new MemoryValue variant.)
                    let name = crate::query::Evaluator::eval_player(&player, game_data)
                        .unwrap_or_else(|e| panic!("SetMemory Player eval failed: {e}"));
                    MemoryValue::String(name)
                }
                MemoryType::Team { team } => {
                    let name = crate::query::Evaluator::eval_team(&team, game_data)
                        .unwrap_or_else(|e| panic!("SetMemory Team eval failed: {e}"));
                    MemoryValue::Team(name)
                }
                // TODO: evaluate the remaining variants when Evaluator gains
                // the corresponding helpers (see test-plan-4). For now, insert
                // a typed default so the key exists and the variant is not lost.
                MemoryType::PlayerCollection { .. } => MemoryValue::PlayerCollection(vec![]),
                MemoryType::StringCollection { .. } => MemoryValue::StringCollection(vec![]),
                MemoryType::TeamCollection { .. } => MemoryValue::Int(0),
                MemoryType::IntCollection { .. } => MemoryValue::IntCollection(vec![]),
                MemoryType::LocationCollection { .. } => MemoryValue::LocationCollection(vec![]),
                MemoryType::CardSet { .. } => MemoryValue::CardSet(vec![]),
            };
            // NOTE(grammar-gap): set_memory grammar has no owner clause.
            // Prefix key with current player name until grammar supports
            // explicit `of <owner>`.
            let key = match game_data.get_current_player() {
                Some(p) => format!("{}_{}", p.name, memory),
                None => {
                    panic!("SetMemory requires a current player")
                }
            };
            game_data.set_memory(key, value);
        }
        ActionRule::ResetMemory { memory } => {
            // NOTE(grammar-gap): reset_memory grammar has no owner clause.
            let key = match game_data.get_current_player() {
                Some(p) => format!("{}_{}", p.name, memory),
                None => {
                    panic!("ResetMemory requires a current player")
                }
            };
            game_data.reset_memory(&key);
        }
        ActionRule::CycleAction { player } => {
            let player_name = crate::query::Evaluator::eval_player(&player, game_data)
                .unwrap_or_else(|e| {
                    panic!("CycleAction: failed to eval player {:?}: {}", player, e)
                });
            let player_idx = game_data
                .players
                .iter()
                .position(|p| p.name == player_name)
                .unwrap_or_else(|| {
                    panic!(
                        "CycleAction: player {} not found in game_data.players",
                        player_name
                    )
                });
            let turn_idx = game_data
                .turn_order
                .iter()
                .position(|&idx| idx == player_idx)
                .unwrap_or_else(|| {
                    panic!(
                        "CycleAction: player_idx {} not in turn_order {:?}",
                        player_idx, game_data.turn_order
                    )
                });
            game_data.current_player = Some(turn_idx);
        }
        ActionRule::BidAction { quantitiy: _ } => {
            //TODO: not implemented yet.
        }
        ActionRule::BidMemoryAction {
            memory: _,
            quantity: _,
            owner: _,
        } => {
            // TODO: not implemented yet
        }
        ActionRule::EndAction { end_type } => match end_type {
            front_end::ast::EndType::Turn => game_data.next_player(),
            front_end::ast::EndType::CurrentStage => {
                if let Some(stage) = game_data.get_current_stage() {
                    game_data.leave_stage(stage);
                }
            }
            front_end::ast::EndType::Stage { stage } => game_data.leave_stage(stage.clone()),
            front_end::ast::EndType::GameWithWinner { players: _ } => {
                // TODO: not implemented
            }
        },
        ActionRule::DemandAction { demand_type: _ } => {
            // TODO: not implemented (and unsure as to how this should work)
        }
        ActionRule::DemandMemoryAction {
            demand_type: _,
            memory: _,
        } => {
            //TODO: same as DemandAction
        }
        ActionRule::Move { move_type } => {
            execute_move(move_type, game_data);
        }
    }
}

pub(crate) fn execute_scoring_rule(scoring: ScoringRule, game_data: &mut GameData) {
    match scoring {
        ScoringRule::ScoreRule { score_rule } => match score_rule {
            front_end::ast::ScoreRule::Score { int, players } => {
                let value = crate::query::Evaluator::eval_int(&int, game_data)
                    .unwrap_or_else(|e| panic!("Score: failed to eval int {:?}: {}", int, e));
                let indices = crate::query::Evaluator::resolve_players(&players, game_data);
                for idx in indices {
                    game_data.players[idx].score += value;
                }
            }
            front_end::ast::ScoreRule::ScoreMemory {
                int,
                memory,
                players,
            } => {
                let value = crate::query::Evaluator::eval_int(&int, game_data)
                    .unwrap_or_else(|e| panic!("ScoreMemory: failed to eval int {:?}: {}", int, e));
                let indices = crate::query::Evaluator::resolve_players(&players, game_data);
                for idx in indices {
                    let name = &game_data.players[idx].name;
                    let key = format!("{}_{}", name, memory);
                    game_data
                        .memories
                        .insert(key, crate::game_data::MemoryValue::Int(value));
                }
            }
        },
        ScoringRule::WinnerRule { winner_rule } => match winner_rule {
            front_end::ast::WinnerRule::Winner { players } => {
                let winner_indices = crate::query::Evaluator::resolve_players(&players, game_data);
                for i in 0..game_data.players.len() {
                    if !winner_indices.contains(&i) {
                        game_data.set_player_out(i);
                    }
                }
            }
            front_end::ast::WinnerRule::WinnerWith {
                extrema,
                winner_type,
            } => {
                // NOTE: Position = turn-order index (lower = earlier in turn).
                let values: Vec<(usize, usize)> = game_data
                    .players
                    .iter()
                    .enumerate()
                    .filter(|(_, p)| p.in_game)
                    .map(|(i, p)| {
                        let val = match &winner_type {
                            front_end::ast::WinnerType::Score => p.score as usize,
                            front_end::ast::WinnerType::Position => game_data
                                .turn_order
                                .iter()
                                .position(|&x| x == i)
                                .unwrap_or(usize::MAX),
                            front_end::ast::WinnerType::Memory { memory } => {
                                let key = format!("{}_{}", p.name, memory);
                                match game_data.get_memory(&key) {
                                    Some(crate::game_data::MemoryValue::Int(n)) => {
                                        (*n).max(0) as usize
                                    }
                                    _ => 0,
                                }
                            }
                        };
                        (i, val)
                    })
                    .collect();

                if values.is_empty() {
                    return;
                }

                let target = match extrema {
                    front_end::ast::Extrema::Max => {
                        values.iter().map(|(_, v)| v).max().copied().unwrap()
                    }
                    front_end::ast::Extrema::Min => {
                        values.iter().map(|(_, v)| v).min().copied().unwrap()
                    }
                };

                for (i, v) in &values {
                    if *v != target {
                        game_data.set_player_out(*i);
                    }
                }
            }
        },
    }
}

pub(crate) fn execute_move(move_type: MoveType, game_data: &mut GameData) {
    match move_type {
        MoveType::Deal { deal } => {
            let front_end::ast::DealMove::MoveCardSet { deal_cs } = deal;
            let (from, to, status, quantity) = match deal_cs {
                front_end::ast::MoveCardSet::Move { from, status, to } => (from, to, status, None),
                front_end::ast::MoveCardSet::MoveQuantity {
                    quantity,
                    from,
                    status,
                    to,
                } => (from, to, status, Some(quantity)),
            };
            execute_cardset_move(from, quantity, status, to, game_data);
        }
        MoveType::Exchange { exchange } => {
            let front_end::ast::ExchangeMove::MoveCardSet { exchange_cs } = exchange;
            let (from, to, status, quantity) = match exchange_cs {
                front_end::ast::MoveCardSet::Move { from, status, to } => (from, to, status, None),
                front_end::ast::MoveCardSet::MoveQuantity {
                    quantity,
                    from,
                    status,
                    to,
                } => (from, to, status, Some(quantity)),
            };
            execute_cardset_move(from, quantity, status, to, game_data);
        }
        MoveType::Classic { classic } => {
            let front_end::ast::ClassicMove::MoveCardSet { move_cs } = classic;
            let (from, to, status, quantity) = match move_cs {
                front_end::ast::MoveCardSet::Move { from, status, to } => (from, to, status, None),
                front_end::ast::MoveCardSet::MoveQuantity {
                    quantity,
                    from,
                    status,
                    to,
                } => (from, to, status, Some(quantity)),
            };
            execute_cardset_move(from, quantity, status, to, game_data);
        }
        MoveType::Place { token: _ } => {}
    }
}

pub(crate) fn execute_cardset_move(
    from: front_end::ast::CardSet,
    quantity: Option<Quantity>,
    _status: Status,
    to: front_end::ast::CardSet,
    game_data: &mut GameData,
) {
    let card_indices = crate::query::Evaluator::eval_cardset(&from, game_data)
        .unwrap_or_else(|e| {
            panic!(
                "execute_cardset_move: failed to eval from cardset {:?}: {}",
                from, e
            )
        })
        .1; // get only the indices, we don't care about the location for from
    let count = match quantity {
        Some(qty) => {
            crate::query::Evaluator::resolve_quantity(&qty, card_indices.len()).unwrap_or(1)
        }
        None => card_indices.len(),
    };

    let dest_loc_idx = crate::query::Evaluator::eval_cardset(&to, game_data)
        .unwrap_or_else(|e| {
            panic!(
                "execute_cardset_move: failed to eval dest cardset {:?}: {}",
                to, e
            )
        })
        .0;

    if dest_loc_idx >= game_data.locations.len() {
        panic!(
            "execute_cardset_move: dest_loc_idx {} >= locations.len() {} (cardset expr: {:?})",
            dest_loc_idx,
            game_data.locations.len(),
            to
        )
    }

    // for each card to move
    let take = count.min(card_indices.len());
    for &card_id in card_indices.iter().take(take) {
        // iterate through all locations in the game, and remove the card id from all of them -
        // later, a more elegant solution would be to resolve the card location somehow but this might just result in the same thing.
        for loc in game_data.locations.iter_mut() {
            loc.cards.retain(|&id| id != card_id);
        }

        // add the card to the new location
        game_data.locations[dest_loc_idx].cards.push(card_id);
    }
}

#[cfg(test)]
#[path = "action_tests.rs"]
mod tests;
