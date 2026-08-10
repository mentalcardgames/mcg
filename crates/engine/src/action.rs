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

use crate::error::EngineError;
use crate::game_data::{Combo, GameData, Location, MemoryValue, PointMap, Precedence, Team};
use front_end::ast::{
    ActionRule, GameRule, MemoryType, MoveType, OutOf, Quantity, ScoringRule, SetUpRule, Status,
};
use front_end::ir::LoweredPayLoad;

#[allow(clippy::single_match)]
pub fn execute(payload: LoweredPayLoad, game_data: &mut GameData) -> Result<(), EngineError> {
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

pub fn execute_setup_rule(payload: SetUpRule, game_data: &mut GameData) -> Result<(), EngineError> {
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
                let player_indices = crate::query::Evaluator::resolve_player_collection(
                    &player_collection,
                    game_data,
                )?;
                game_data.teams.push(Team {
                    name: team_name,
                    players: player_indices,
                });
            }
            Ok(())
        }
        SetUpRule::CreateTurnorder { player_collection } => {
            let indices =
                crate::query::Evaluator::resolve_player_collection(&player_collection, game_data)?;
            game_data.turn_order = indices;
            Ok(())
        }
        SetUpRule::CreateTurnorderRandom { player_collection } => {
            use rand::seq::SliceRandom;
            let mut indices =
                crate::query::Evaluator::resolve_player_collection(&player_collection, game_data)?;
            indices.shuffle(&mut rand::thread_rng());
            game_data.turn_order = indices;
            Ok(())
        }
        SetUpRule::CreateLocation { locations, owner } => {
            let owner_names = crate::query::Evaluator::resolve_owner_to_names(&owner, game_data)
                .map_err(|source| EngineError::CreateLocationOwnerResolution {
                    owner: Box::new(owner.clone()),
                    source: Box::new(source),
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
                .ok_or(EngineError::CreateCardOnLocationLocationNotFound {
                    location: location.clone(),
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
                .map_err(|source| EngineError::CreateMemoryOwnerResolution {
                    owner: Box::new(owner.clone()),
                    source: Box::new(source),
                })?;
            for name in &names {
                let key = format!("{}_{}", name, memory);
                game_data.add_memory(key, name, None, None);
            }
            Ok(())
        }
        SetUpRule::CreateMemoryWithMemoryType {
            memory,
            owner,
            memory_type,
        } => {
            let names = crate::query::Evaluator::resolve_owner_to_names(&owner, game_data)
                .map_err(|source| EngineError::CreateMemoryWithTypeOwnerResolution {
                    owner: Box::new(owner.clone()),
                    source: Box::new(source),
                })?;
            // Evaluate the declared type-expression at setup time (2026-08-10):
            // `memory Pot 100 on table` now really initialises `Table_pot` to
            // 100 instead of silently storing 0 (the previous behaviour).
            let initial = evaluate_memory_type(&memory_type, game_data).map_err(|source| {
                EngineError::CreateMemoryTypeEval {
                    memory_type: Box::new(memory_type.clone()),
                    source: Box::new(source),
                }
            })?;
            for name in &names {
                let key = format!("{}_{}", name, memory);
                // A player-owned slot of a Player-typed memory initialises to
                // its own owner (so `&P:X of P:Pi` reads the slot's player);
                // Table-owned slots use the evaluated player expression.
                let slot_initial = match &memory_type {
                    MemoryType::Player { player } if is_player_name(name, game_data) => {
                        MemoryValue::String(name.clone())
                    }
                    _ => initial.clone(),
                };
                game_data.add_memory(key, name, Some(memory_type.clone()), Some(slot_initial));
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
                    crate::query::Evaluator::eval_int(&int_expr, game_data).map_err(|source| {
                        EngineError::CreatePointMapIntEval {
                            int_expr: Box::new(int_expr.clone()),
                            key: key.clone(),
                            value: value.clone(),
                            source: Box::new(source),
                        }
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

/// Evaluates a `MemoryType` expression against the live state at setup time
/// (2026-08-10). Collections are evaluated to their real contents where the
/// evaluators exist; anything that cannot be evaluated meaningfully falls
/// back to the typed default (the caller passes the result as the memory's
/// initial value).
fn evaluate_memory_type(
    memory_type: &MemoryType,
    game_data: &GameData,
) -> Result<MemoryValue, EngineError> {
    use crate::query::Evaluator;
    Ok(match memory_type {
        MemoryType::Int { int } => MemoryValue::Int(Evaluator::eval_int(int, game_data)?),
        MemoryType::String { string } => {
            MemoryValue::String(Evaluator::eval_string(string, game_data)?)
        }
        MemoryType::Player { player } => {
            MemoryValue::String(Evaluator::eval_player(player, game_data)?)
        }
        MemoryType::Team { team } => MemoryValue::Team(Evaluator::eval_team(team, game_data)?),
        MemoryType::PlayerCollection { players } => {
            let indices = Evaluator::resolve_player_collection(players, game_data)?;
            MemoryValue::PlayerCollection(indices)
        }
        MemoryType::StringCollection { strings } => {
            MemoryValue::StringCollection(Evaluator::eval_string_collection(strings, game_data)?)
        }
        MemoryType::TeamCollection { teams } => {
            MemoryValue::TeamCollection(Evaluator::eval_team_collection(teams, game_data)?)
        }
        MemoryType::IntCollection { ints } => {
            MemoryValue::IntCollection(Evaluator::eval_int_collection(ints, game_data)?)
        }
        MemoryType::LocationCollection { locations } => {
            let names = Evaluator::eval_location_collection(locations, game_data)?;
            let mut out = Vec::new();
            for name in names {
                let idx = game_data
                    .locations
                    .iter()
                    .position(|l| l.name == name)
                    .ok_or(EngineError::LocationNotFound { name })?;
                out.push(idx);
            }
            MemoryValue::LocationCollection(out)
        }
        MemoryType::CardSet { card_set } => {
            let (_loc_idx, card_ids) = Evaluator::eval_cardset(card_set, game_data)?;
            MemoryValue::CardSet(card_ids)
        }
    })
}

fn is_player_name(name: &str, game_data: &GameData) -> bool {
    game_data.players.iter().any(|p| p.name == name)
}

pub(crate) fn execute_action_rule(
    action: ActionRule,
    game_data: &mut GameData,
) -> Result<(), EngineError> {
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
                Err(source) => Err(EngineError::ShuffleActionEval {
                    source: Box::new(source),
                }),
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
            let value: MemoryValue = match memory_type {
                MemoryType::Int { int } => {
                    let n =
                        crate::query::Evaluator::eval_int(&int, game_data).map_err(|source| {
                            EngineError::SetMemoryIntEval {
                                source: Box::new(source),
                            }
                        })?;
                    MemoryValue::Int(n)
                }
                MemoryType::String { string } => {
                    let s = crate::query::Evaluator::eval_string(&string, game_data).map_err(
                        |source| EngineError::SetMemoryStringEval {
                            source: Box::new(source),
                        },
                    )?;
                    MemoryValue::String(s)
                }
                MemoryType::Player { player } => {
                    // Evaluator::eval_player returns a name; we store the name as
                    // a String memory so later reads as String succeed. (Storing
                    // a player index would require a new MemoryValue variant.)
                    let name = crate::query::Evaluator::eval_player(&player, game_data).map_err(
                        |source| EngineError::SetMemoryPlayerEval {
                            source: Box::new(source),
                        },
                    )?;
                    MemoryValue::String(name)
                }
                MemoryType::Team { team } => {
                    let name =
                        crate::query::Evaluator::eval_team(&team, game_data).map_err(|source| {
                            EngineError::SetMemoryTeamEval {
                                source: Box::new(source),
                            }
                        })?;
                    MemoryValue::Team(name)
                }
                // Evaluated collections (2026-08-10): previously every
                // collection type inserted a typed empty default.
                MemoryType::PlayerCollection { players } => {
                    let indices =
                        crate::query::Evaluator::resolve_player_collection(&players, game_data)
                            .map_err(|source| EngineError::SetMemoryCollectionEval {
                                source: Box::new(source),
                            })?;
                    MemoryValue::PlayerCollection(indices)
                }
                MemoryType::StringCollection { strings } => {
                    let out = crate::query::Evaluator::eval_string_collection(&strings, game_data)
                        .map_err(|source| EngineError::SetMemoryCollectionEval {
                            source: Box::new(source),
                        })?;
                    MemoryValue::StringCollection(out)
                }
                MemoryType::TeamCollection { teams } => {
                    let names = crate::query::Evaluator::eval_team_collection(&teams, game_data)
                        .map_err(|source| EngineError::SetMemoryCollectionEval {
                            source: Box::new(source),
                        })?;
                    MemoryValue::TeamCollection(names)
                }
                MemoryType::IntCollection { ints } => {
                    let out = crate::query::Evaluator::eval_int_collection(&ints, game_data)
                        .map_err(|source| EngineError::SetMemoryCollectionEval {
                            source: Box::new(source),
                        })?;
                    MemoryValue::IntCollection(out)
                }
                MemoryType::LocationCollection { locations } => {
                    let names =
                        crate::query::Evaluator::eval_location_collection(&locations, game_data)
                            .map_err(|source| EngineError::SetMemoryCollectionEval {
                                source: Box::new(source),
                            })?;
                    let mut out = Vec::new();
                    for name in names {
                        let idx = game_data
                            .locations
                            .iter()
                            .position(|l| l.name == name)
                            .ok_or(EngineError::LocationNotFound { name })?;
                        out.push(idx);
                    }
                    MemoryValue::LocationCollection(out)
                }
                MemoryType::CardSet { card_set } => {
                    let (_loc_idx, card_ids) =
                        crate::query::Evaluator::eval_cardset(&card_set, game_data).map_err(
                            |source| EngineError::SetMemoryCollectionEval {
                                source: Box::new(source),
                            },
                        )?;
                    MemoryValue::CardSet(card_ids)
                }
            };
            // NOTE(grammar-gap): the write rules have no `of <owner>` clause.
            // Since 2026-08-10 the target owner is resolved like the reads:
            // the declared slot wins when exactly one exists (`memory pot on
            // table` → `pot is 5` writes `Table_pot`), otherwise the current
            // player's slot (D-14).
            let owner = game_data.memory_write_owner(
                &memory,
                game_data.get_current_player().map(|p| p.name.as_str()),
            );
            match owner {
                Some(owner) => {
                    let key = format!("{}_{}", owner, memory);
                    game_data.set_memory(key, value);
                }
                None => return Err(EngineError::SetMemoryNoCurrentPlayer),
            }
            Ok(())
        }
        ActionRule::ResetMemory { memory } => {
            // Same owner resolution as SetMemory (D-14).
            let owner = game_data.memory_write_owner(
                &memory,
                game_data.get_current_player().map(|p| p.name.as_str()),
            );
            match owner {
                Some(owner) => {
                    let key = format!("{}_{}", owner, memory);
                    game_data.reset_memory(&key);
                }
                None => return Err(EngineError::ResetMemoryNoCurrentPlayer),
            }
            Ok(())
        }
        ActionRule::CycleAction { player } => {
            let player_name =
                crate::query::Evaluator::eval_player(&player, game_data).map_err(|source| {
                    EngineError::CycleActionPlayerEval {
                        player: Box::new(player.clone()),
                        source: Box::new(source),
                    }
                });
            let player_name = match player_name {
                Ok(name) => name,
                // 2026-08-10: `cycle to next` with no eligible player at all
                // (not even the current one) is a no-op, not an error — the
                // stage's auto-end (no players in game / in stage) terminates
                // the game from the loop-back (D-1 / I-13 relaxed).
                Err(EngineError::CycleActionPlayerEval { source, .. })
                    if matches!(*source, EngineError::NoNextPlayerAvailable) =>
                {
                    return Ok(());
                }
                Err(e) => return Err(e),
            };
            let player_idx = game_data
                .players
                .iter()
                .position(|p| p.name == player_name)
                .ok_or(EngineError::CycleActionPlayerNotFound {
                    name: player_name.clone(),
                })?;
            let turn_idx = game_data
                .turn_order
                .iter()
                .position(|&idx| idx == player_idx)
                .ok_or(EngineError::CycleActionTurnOrderNotFound {
                    player_idx,
                    turn_order: game_data.turn_order.clone(),
                })?;
            game_data.current_player = Some(turn_idx);
            Ok(())
        }
        ActionRule::BidAction { quantitiy } => {
            // 2026-08-10: plain `bid <qty>` (no memory target) has no defined
            // semantics — surface a recoverable error instead of a silent
            // no-op (D-7). Use `bid <qty> on <memory> of <owner>`.
            Err(EngineError::BidWithoutMemoryTarget {
                quantity: Box::new(quantitiy.clone()),
            })
        }
        ActionRule::BidMemoryAction {
            memory,
            quantity,
            owner,
        } => {
            // 2026-08-10: `bid <qty> on <memory> of <owner>` = "prompt (or
            // take) a number and store it in the owner's memory slot".
            // Literal/int-expr quantities are resolved here; `any`/ranges
            // are intercepted by the interpreter as a `InputType::Number`
            // prompt and reach this arm with a literal substitution (D-7).
            let value = match quantity {
                Quantity::Int { int } => crate::query::Evaluator::eval_int(&int, game_data)
                    .map_err(|source| EngineError::BidQuantityEval {
                        source: Box::new(source),
                    })?,
                other => {
                    return Err(EngineError::BidQuantityMustBeLiteral {
                        quantity: Box::new(other.clone()),
                    });
                }
            };
            let owner_name = crate::query::Evaluator::resolve_owner_to_name(&owner, game_data)
                .map_err(|source| EngineError::BidOwnerResolution {
                    owner: Box::new(owner.clone()),
                    source: Box::new(source),
                })?;
            let key = format!("{}_{}", owner_name, memory);
            game_data.set_memory(key, MemoryValue::Int(value));
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
                front_end::ast::EndType::GameWithWinner { players } => {
                    // 2026-08-10: the declared winners eliminate everyone
                    // else (mirroring `winner is X`); the IR jump to the goal
                    // then ends the game. The in-game survivors ARE the
                    // winner set (`GameData::winner_names`).
                    let winner_indices =
                        crate::query::Evaluator::resolve_players(&players, game_data)?;
                    for i in 0..game_data.players.len() {
                        if !winner_indices.contains(&i) {
                            game_data.set_player_out(i);
                        }
                    }
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
) -> Result<(), EngineError> {
    match scoring {
        ScoringRule::ScoreRule { score_rule } => match score_rule {
            front_end::ast::ScoreRule::Score { int, players } => {
                let value =
                    crate::query::Evaluator::eval_int(&int, game_data).map_err(|source| {
                        EngineError::ScoreIntEval {
                            int_expr: Box::new(int.clone()),
                            source: Box::new(source),
                        }
                    })?;
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
                let value =
                    crate::query::Evaluator::eval_int(&int, game_data).map_err(|source| {
                        EngineError::ScoreMemoryIntEval {
                            int_expr: Box::new(int.clone()),
                            source: Box::new(source),
                        }
                    })?;
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
                // 2026-08-10: players missing from `turn_order` are excluded
                // from the comparison (D-10 — previously they scored
                // `usize::MAX`, letting a non-participant win `lowest
                // position`). Memory extrema skip players without the slot
                // and error on a non-Int value (D-13).
                let mut values: Vec<(usize, usize)> = Vec::new();
                for (i, p) in game_data.players.iter().enumerate() {
                    if !p.in_game {
                        continue;
                    }
                    let val = match &winner_type {
                        front_end::ast::WinnerType::Score => Some(p.score as usize),
                        front_end::ast::WinnerType::Position => {
                            game_data.turn_order.iter().position(|&x| x == i)
                        }
                        front_end::ast::WinnerType::Memory { memory } => {
                            let key = format!("{}_{}", p.name, memory);
                            match game_data.get_memory(&key) {
                                Some(crate::game_data::MemoryValue::Int(n)) => {
                                    Some((*n).max(0) as usize)
                                }
                                None => None,
                                Some(_) => {
                                    return Err(EngineError::WinnerMemoryNotInt {
                                        memory: memory.clone(),
                                        player: p.name.clone(),
                                    })
                                }
                            }
                        }
                    };
                    if let Some(v) = val {
                        values.push((i, v));
                    }
                }

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

pub(crate) fn execute_move(
    move_type: MoveType,
    game_data: &mut GameData,
) -> Result<(), EngineError> {
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
) -> Result<(), EngineError> {
    let card_indices = crate::query::Evaluator::eval_cardset(&from, game_data)
        .map_err(|source| EngineError::MoveFromCardsetEval {
            cardset: Box::new(from.clone()),
            source: Box::new(source),
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
        .map_err(|source| EngineError::MoveDestCardsetEval {
            cardset: Box::new(to.clone()),
            source: Box::new(source),
        })?
        .0;

    if dest_loc_idx >= game_data.locations.len() {
        return Err(EngineError::MoveDestLocationOutOfRange {
            dest_loc_idx,
            locations_len: game_data.locations.len(),
            cardset: Box::new(to),
        });
    }

    // for each card to move (count is always <= the source size: quantities
    // are clamped by resolve_quantity)
    for &card_id in card_indices.iter().take(count) {
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
