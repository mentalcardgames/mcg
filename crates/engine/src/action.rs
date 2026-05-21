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

use crate::game_data::{Card, Combo, GameData, Location, PointMap, Precedence, Team};
use front_end::ast::{
    ActionRule, GameRule, MoveType, OutOf, Quantity, ScoringRule, SetUpRule, Status,
};
use front_end::ir::LoweredPayLoad;

pub fn execute(payload: LoweredPayLoad, game_data: &mut GameData) {
    match payload {
        LoweredPayLoad::Action(gr) => match gr {
            GameRule::SetUp { setup } => execute_setup_rule(setup, game_data),
            GameRule::Action { action } => execute_action_rule(action, game_data),
            GameRule::Scoring { scoring } => execute_scoring_rule(scoring, game_data),
        },
        LoweredPayLoad::StageRoundCounter(stage_name) => {
            game_data.increment_stage_counter(stage_name);
        }
        LoweredPayLoad::EndStage(stage_name) => {
            game_data.leave_stage(stage_name);
        }
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
                    crate::query::Evaluator::resolve_player_collection(&player_collection, game_data);
                game_data.teams.push(Team {
                    name: team_name,
                    players: player_indices,
                });
            }
        }
        SetUpRule::CreateTurnorder { player_collection } => {
            let indices =
                crate::query::Evaluator::resolve_player_collection(&player_collection, game_data);
            game_data.turn_order = indices;
        }
        SetUpRule::CreateTurnorderRandom { player_collection } => {
            use rand::seq::SliceRandom;
            let mut indices =
                crate::query::Evaluator::resolve_player_collection(&player_collection, game_data);
            indices.shuffle(&mut rand::thread_rng());
            game_data.turn_order = indices;
        }
        SetUpRule::CreateLocation { locations, owner } => {
            let owner_name = crate::query::Evaluator::resolve_owner_to_name(&owner, game_data);
            for loc_name in locations {
                game_data.add_location(
                    owner_name.clone(),
                    Location {
                        name: loc_name,
                        cards: vec![],
                    },
                );
            }
        }
        SetUpRule::CreateCardOnLocation { location, cards } => {
            let loc_idx = game_data
                .locations
                .iter()
                .position(|l| l.name == location)
                .expect("Location not found");
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
            game_data.add_memory(memory, owner, None);
        }
        SetUpRule::CreateMemoryWithMemoryType {
            memory,
            owner,
            memory_type,
        } => {
            game_data.add_memory(memory, owner, Some(memory_type));
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
            for (key, value, _) in kvis {
                let card_key = format!("{}:{}", key, value);
                map.insert(card_key, 0);
            }
            game_data.point_maps.push(PointMap {
                name: pointmap,
                map,
            });
        }
    }
}

fn execute_action_rule(action: ActionRule, game_data: &mut GameData) {
    match action {
        ActionRule::FlipAction {
            card_set: _,
            status: _,
        } => {}
        ActionRule::ShuffleAction { card_set: _ } => {
            //TODO: implement card set shuffling
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
            game_data.set_memory(memory, memory_type);
        }
        ActionRule::ResetMemory { memory } => {
            game_data.reset_memory(&memory);
        }
        ActionRule::CycleAction { player } => {
            let player_name = crate::query::Evaluator::eval_player(&player, game_data)
                .expect("Failed to eval player");
            let player_idx = game_data
                .players
                .iter()
                .position(|p| p.name == player_name)
                .expect("Player not found");
            let turn_idx = game_data
                .turn_order
                .iter()
                .position(|&idx| idx == player_idx)
                .expect("Player not in turn order");
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

/// not yet implemented
fn execute_scoring_rule(scoring: ScoringRule, game_data: &mut GameData) {
    match scoring {
        ScoringRule::ScoreRule { score_rule } => match score_rule {
            front_end::ast::ScoreRule::Score { int: _, players: _ } => {
                //TODO: figure out what this should do
            }
            front_end::ast::ScoreRule::ScoreMemory {
                int: _,
                players: _,
                memory: _,
            } => {
                //TODO: figure out what this should do
            }
        },
        ScoringRule::WinnerRule { winner_rule: _ } => {
            // TODO: figure out what this should do
        }
    }
}

fn execute_move(move_type: MoveType, game_data: &mut GameData) {
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

fn execute_cardset_move(
    from: front_end::ast::CardSet,
    quantity: Option<Quantity>,
    _status: Status,
    to: front_end::ast::CardSet,
    game_data: &mut GameData,
) {
    let card_indices = crate::query::Evaluator::eval_cardset(&from, game_data)
        .expect("Failed to eval cardset")
        .1; // get only the indices, we don't care about the location for from
    let count = match quantity {
        Some(qty) => crate::query::Evaluator::resolve_quantity(&qty, card_indices.len()),
        None => card_indices.len(),
    };

    let dest_loc_idx = crate::query::Evaluator::eval_cardset(&to, game_data)
        .expect("Failed to eval dest")
        .0;

    if dest_loc_idx > game_data.locations.len() {
        // TODO: throw error since we could not resolve a location for the destination - destination must always resolve a single location, otherwise the rule is invalid.
        panic!("Could not resolve a destination for move action")
    }

    // for each card to move
    for i in 0..count.min(card_indices.len()) {
        // later, once we have implemented card encryption, this won't work (since we won't be able to lookup cards by encrypted indices)
        let card_id = card_indices[i];

        // iterate through all locations in the game, and remove the card id from all of them -
        // later, a more elegant solution would be to resolve the card location somehow but this might just result in the same thing.
        for loc in game_data.locations.iter_mut() {
            loc.cards.retain(|&id| id != card_id);
        }

        // add the card to the new location
        game_data.locations[dest_loc_idx].cards.push(card_id);
    }
}
