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
pub fn execute(payload: LoweredPayLoad, game_data: &mut GameData) -> Result<(), String> {
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
        _ => Ok(()),
    }
}

pub fn execute_setup_rule(payload: SetUpRule, game_data: &mut GameData) -> Result<(), String> {
    match payload {
        SetUpRule::CreatePlayer { players } => {
            for name in players {
                let idx = game_data.add_player(name);
                game_data.turn_order.push(idx);
            }
            Ok(())
        }
        SetUpRule::CreateTeams { teams } => {
            for (team_name, player_collection) in teams {
                let player_indices =
                    crate::quantifier::resolve_player_candidates(&player_collection, game_data)?;
                game_data.teams.push(Team {
                    name: team_name,
                    players: player_indices,
                });
            }
            Ok(())
        }
        SetUpRule::CreateTurnorder { player_collection } => {
            let indices =
                crate::quantifier::resolve_player_candidates(&player_collection, game_data)?;
            game_data.turn_order = indices;
            Ok(())
        }
        SetUpRule::CreateTurnorderRandom { player_collection } => {
            use rand::seq::SliceRandom;
            let mut indices =
                crate::quantifier::resolve_player_candidates(&player_collection, game_data)?;
            indices.shuffle(&mut rand::thread_rng());
            game_data.turn_order = indices;
            Ok(())
        }
        SetUpRule::CreateLocation { locations, owner } => {
            let owner_names = crate::query::Evaluator::resolve_owner_to_names(&owner, game_data)
                .map_err(|e| {
                    format!("CreateLocation: failed to resolve owner {:?}: {}", owner, e)
                })?;
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
            Ok(())
        }
        SetUpRule::CreateCardOnLocation { location, cards } => {
            let loc_idx = game_data
                .locations
                .iter()
                .position(|l| l.name == location)
                .ok_or_else(|| {
                    format!("CreateCardOnLocation: location {:?} not found", location)
                })?;
            for type_expr in cards {
                let expanded_cards = crate::query::Evaluator::expand_types(&type_expr);
                for card in expanded_cards {
                    let card_id = game_data.add_card(loc_idx, card);
                    game_data.locations[loc_idx].cards.push(card_id);
                }
            }
            Ok(())
        }
        SetUpRule::CreateTokenOnLocation { .. } => Ok(()),
        SetUpRule::CreateCombo { combo, filter } => {
            game_data.combos.push(Combo {
                name: combo,
                filter,
            });
            Ok(())
        }
        SetUpRule::CreateMemory { memory, owner } => {
            let names = crate::query::Evaluator::resolve_owner_to_names(&owner, game_data)
                .map_err(|e| format!("CreateMemory: failed to resolve owner {:?}: {}", owner, e))?;
            for name in names {
                let key = format!("{}_{}", name, memory);
                game_data.add_memory(key, owner.clone(), None);
            }
            Ok(())
        }
        SetUpRule::CreateMemoryWithMemoryType {
            memory,
            owner,
            memory_type,
        } => {
            let names = crate::query::Evaluator::resolve_owner_to_names(&owner, game_data)
                .map_err(|e| {
                    format!(
                        "CreateMemoryWithMemoryType: failed to resolve owner {:?}: {}",
                        owner, e
                    )
                })?;
            for name in names {
                let key = format!("{}_{}", name, memory);
                game_data.add_memory(key, owner.clone(), Some(memory_type.clone()));
            }
            Ok(())
        }
        SetUpRule::CreatePrecedence { precedence, kvs } => {
            game_data.precedences.push(Precedence {
                name: precedence,
                key: kvs[0].0.clone(),
                values: kvs.into_iter().map(|(_, v)| v).collect(),
            });
            Ok(())
        }
        SetUpRule::CreatePointMap { pointmap, kvis } => {
            let mut map = std::collections::HashMap::new();
            for (key, value, int_expr) in kvis {
                let card_key = format!("{}:{}", key, value);
                let points =
                    crate::query::Evaluator::eval_int(&int_expr, game_data).map_err(|e| {
                        format!(
                            "CreatePointMap: failed to eval int {:?} for key {}:{}: {}",
                            int_expr, key, value, e
                        )
                    })?;
                map.insert(card_key, points);
            }
            game_data.point_maps.push(PointMap {
                name: pointmap,
                map,
            });
            Ok(())
        }
    }
}

pub(crate) fn execute_action_rule(
    action: ActionRule,
    game_data: &mut GameData,
) -> Result<(), String> {
    match action {
        ActionRule::FlipAction {
            card_set: _,
            status: _,
        } => {
            // Intentionally unimplemented until card encryption lands: flipping
            // a card is (de)encrypting its face. The per-card status slot
            // (`GameData::card_statuses`) exists in the data model but is not
            // yet read or written. TODO(crypto): implement with card encryption.
            Ok(())
        }
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
                    Ok(())
                }
                Err(e) => Err(format!("ShuffleAction failed: {}", e)),
            }
        }
        ActionRule::OutAction { players, out_of } => {
            let player_indices = crate::query::Evaluator::resolve_players(&players, game_data)?;
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
            Ok(())
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
                        .map_err(|e| format!("SetMemory Int eval failed: {e}"))?;
                    MemoryValue::Int(n)
                }
                MemoryType::String { string } => {
                    let s = crate::query::Evaluator::eval_string(&string, game_data)
                        .map_err(|e| format!("SetMemory String eval failed: {e}"))?;
                    MemoryValue::String(s)
                }
                MemoryType::Player { player } => {
                    // Evaluator::eval_player returns a name; we store the name as
                    // a String memory so later reads as String succeed. (Storing
                    // a player index would require a new MemoryValue variant.)
                    let name = crate::query::Evaluator::eval_player(&player, game_data)
                        .map_err(|e| format!("SetMemory Player eval failed: {e}"))?;
                    MemoryValue::String(name)
                }
                MemoryType::Team { team } => {
                    let name = crate::query::Evaluator::eval_team(&team, game_data)
                        .map_err(|e| format!("SetMemory Team eval failed: {e}"))?;
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
                    return Err("SetMemory requires a current player".to_string());
                }
            };
            game_data.set_memory(key, value);
            Ok(())
        }
        ActionRule::ResetMemory { memory } => {
            // NOTE(grammar-gap): reset_memory grammar has no owner clause.
            let key = match game_data.get_current_player() {
                Some(p) => format!("{}_{}", p.name, memory),
                None => {
                    return Err("ResetMemory requires a current player".to_string());
                }
            };
            game_data.reset_memory(&key);
            Ok(())
        }
        ActionRule::CycleAction { player } => {
            let player_name = crate::query::Evaluator::eval_player(&player, game_data)
                .map_err(|e| format!("CycleAction: failed to eval player {:?}: {}", player, e))?;
            let player_idx = game_data
                .players
                .iter()
                .position(|p| p.name == player_name)
                .ok_or_else(|| {
                    format!(
                        "CycleAction: player {} not found in game_data.players",
                        player_name
                    )
                })?;
            let turn_idx = game_data
                .turn_order
                .iter()
                .position(|&idx| idx == player_idx)
                .ok_or_else(|| {
                    format!(
                        "CycleAction: player_idx {} not in turn_order {:?}",
                        player_idx, game_data.turn_order
                    )
                })?;
            game_data.current_player = Some(turn_idx);
            Ok(())
        }
        ActionRule::BidAction { quantitiy: _ } => {
            //TODO: not implemented yet.
            Ok(())
        }
        ActionRule::BidMemoryAction {
            memory: _,
            quantity: _,
            owner: _,
        } => {
            // TODO: not implemented yet
            Ok(())
        }
        ActionRule::EndAction { end_type } => {
            match end_type {
                front_end::ast::EndType::Turn => game_data.next_player(),
                front_end::ast::EndType::CurrentStage => {
                    if let Some(stage) = game_data.get_current_stage() {
                        game_data.leave_stage(stage);
                    }
                }
                front_end::ast::EndType::Stage { stage } => game_data.leave_stage(stage.clone()),
                front_end::ast::EndType::GameWithWinner { players: _ } => {
                    // TODO: not implemented (the IR jump to the goal state
                    // already ends the game; see engine-vs-design.md)
                }
            }
            Ok(())
        }
        ActionRule::DemandAction { demand_type: _ } => {
            // TODO: not implemented (and unsure as to how this should work)
            Ok(())
        }
        ActionRule::DemandMemoryAction {
            demand_type: _,
            memory: _,
        } => {
            //TODO: same as DemandAction
            Ok(())
        }
        ActionRule::Move { move_type } => execute_move(move_type, game_data),
    }
}

pub(crate) fn execute_scoring_rule(
    scoring: ScoringRule,
    game_data: &mut GameData,
) -> Result<(), String> {
    match scoring {
        ScoringRule::ScoreRule { score_rule } => match score_rule {
            front_end::ast::ScoreRule::Score { int, players } => {
                let value = crate::query::Evaluator::eval_int(&int, game_data)
                    .map_err(|e| format!("Score: failed to eval int {:?}: {}", int, e))?;
                let indices = crate::query::Evaluator::resolve_players(&players, game_data)?;
                for idx in indices {
                    game_data.players[idx].score += value;
                }
                Ok(())
            }
            front_end::ast::ScoreRule::ScoreMemory {
                int,
                memory,
                players,
            } => {
                let value = crate::query::Evaluator::eval_int(&int, game_data)
                    .map_err(|e| format!("ScoreMemory: failed to eval int {:?}: {}", int, e))?;
                let indices = crate::query::Evaluator::resolve_players(&players, game_data)?;
                for idx in indices {
                    let name = &game_data.players[idx].name;
                    let key = format!("{}_{}", name, memory);
                    game_data
                        .memories
                        .insert(key, crate::game_data::MemoryValue::Int(value));
                }
                Ok(())
            }
        },
        ScoringRule::WinnerRule { winner_rule } => match winner_rule {
            front_end::ast::WinnerRule::Winner { players } => {
                let winner_indices = crate::query::Evaluator::resolve_players(&players, game_data)?;
                for i in 0..game_data.players.len() {
                    if !winner_indices.contains(&i) {
                        game_data.set_player_out(i);
                    }
                }
                Ok(())
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
                    return Ok(());
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
                Ok(())
            }
        },
    }
}

pub(crate) fn execute_move(move_type: MoveType, game_data: &mut GameData) -> Result<(), String> {
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
            execute_cardset_move(from, quantity, status, to, game_data)
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
            execute_cardset_move(from, quantity, status, to, game_data)
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
            execute_cardset_move(from, quantity, status, to, game_data)
        }
        MoveType::Place { token: _ } => Ok(()),
    }
}

pub(crate) fn execute_cardset_move(
    from: front_end::ast::CardSet,
    quantity: Option<Quantity>,
    _status: Status,
    to: front_end::ast::CardSet,
    game_data: &mut GameData,
) -> Result<(), String> {
    let card_indices = crate::query::Evaluator::eval_cardset(&from, game_data)
        .map_err(|e| {
            format!(
                "execute_cardset_move: failed to eval from cardset {:?}: {}",
                from, e
            )
        })?
        .1; // get only the indices, we don't care about the location for from

    // Nothing to move: no-op (also avoids evaluating the destination, which
    // for an empty `where`-filtered set used to resolve to location 0 — see
    // engine-vs-design.md D-11).
    if card_indices.is_empty() {
        return Ok(());
    }

    let count = match quantity {
        Some(qty) => {
            crate::query::Evaluator::resolve_quantity(&qty, card_indices.len(), game_data)?
        }
        None => card_indices.len(),
    };

    let dest_loc_idx = crate::query::Evaluator::eval_cardset(&to, game_data)
        .map_err(|e| {
            format!(
                "execute_cardset_move: failed to eval dest cardset {:?}: {}",
                to, e
            )
        })?
        .0;

    if dest_loc_idx >= game_data.locations.len() {
        return Err(format!(
            "execute_cardset_move: dest_loc_idx {} >= locations.len() {} (cardset expr: {:?})",
            dest_loc_idx,
            game_data.locations.len(),
            to
        ));
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
    Ok(())
}

#[cfg(test)]
#[path = "action_tests.rs"]
mod tests;
