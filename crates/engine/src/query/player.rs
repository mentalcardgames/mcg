use super::Evaluator;
use crate::error::EngineError;
use crate::game_data::{GameData, MemoryValue};
use front_end::ast::{
    AggregatePlayer, AggregateTeam, Extrema, MultiOwner, Owner, PlayerCollection, PlayerExpr,
    Players, QueryPlayer, RuntimePlayer, TeamExpr,
};

impl Evaluator {
    pub fn eval_player(expr: &PlayerExpr, game_data: &GameData) -> Result<String, EngineError> {
        match expr {
            PlayerExpr::Literal { name } => Ok(name.clone()),
            PlayerExpr::Runtime { runtime } => match runtime {
                RuntimePlayer::Current => game_data
                    .get_current_player()
                    .map(|p| p.name.clone())
                    .ok_or(EngineError::NoCurrentPlayer),
                RuntimePlayer::Next => {
                    let current_idx = game_data
                        .current_player
                        .ok_or(EngineError::NoCurrentPlayer)?;
                    let current_stage = game_data
                        .get_current_stage()
                        .ok_or(EngineError::NoCurrentStage)?;
                    let player_idx = game_data
                        .next_eligible_player(current_idx, &current_stage)
                        .ok_or(EngineError::NoNextPlayerAvailable)?;
                    game_data
                        .players
                        .get(player_idx)
                        .map(|p| p.name.clone())
                        .ok_or(EngineError::NoNextPlayerAvailable)
                }
                RuntimePlayer::Previous => {
                    let current_idx = game_data
                        .current_player
                        .ok_or(EngineError::NoCurrentPlayer)?;
                    let current_stage = game_data
                        .get_current_stage()
                        .ok_or(EngineError::NoCurrentStage)?;
                    // D-12 fix: `previous` skips ineligible players, mirroring
                    // `next`, and wraps onto the current player when it is the
                    // only eligible one.
                    let player_idx = game_data
                        .previous_eligible_player(current_idx, &current_stage)
                        .ok_or(EngineError::PreviousPlayerNotFound)?;
                    game_data
                        .players
                        .get(player_idx)
                        .map(|p| p.name.clone())
                        .ok_or(EngineError::PreviousPlayerNotFound)
                }
                RuntimePlayer::Competitor => {
                    let current = game_data
                        .get_current_player()
                        .ok_or(EngineError::NoCurrentPlayer)?;
                    for team in &game_data.teams {
                        if team.players.iter().any(|&idx| {
                            game_data.players.get(idx).map(|p| p.name.clone())
                                == Some(current.name.clone())
                        }) {
                            for &player_idx in &team.players {
                                if game_data.players.get(player_idx).map(|p| &p.name)
                                    != Some(&current.name)
                                {
                                    return Ok(game_data.players[player_idx].name.clone());
                                }
                            }
                        }
                    }
                    Err(EngineError::NoCompetitorFound)
                }
            },
            PlayerExpr::Aggregate { aggregate } => match aggregate {
                AggregatePlayer::OwnerOfCardPostion { card_position } => {
                    let card_id = Self::eval_card_position(card_position, game_data)?;
                    for (loc_idx, loc) in game_data.locations.iter().enumerate() {
                        if loc.cards.contains(&card_id) {
                            if let Some(_owner_loc_idx) =
                                game_data.table.locations.iter().find(|&&l| l == loc_idx)
                            {
                                return Ok("Table".to_string());
                            }
                            for team in game_data.teams.iter() {
                                if team.owner.locations.contains(&loc_idx) {
                                    return Ok(team.name.clone());
                                }
                            }
                            for player in game_data.players.iter() {
                                if player.owner.locations.contains(&loc_idx) {
                                    return Ok(player.name.clone());
                                }
                            }
                        }
                    }
                    Err(EngineError::CardOwnerNotFound)
                }
                AggregatePlayer::OwnerOfMemory { extrema, memory } => {
                    let mem_key = memory;
                    let mut best_player: Option<String> = None;
                    let mut best_value: Option<i32> = None;
                    let mut found = false;
                    for player in &game_data.players {
                        if player.in_game {
                            found = true;
                            let mem_key_with_owner = format!("{}_{}", player.name, mem_key);
                            if let Some(MemoryValue::Int(v)) =
                                game_data.get_memory(&mem_key_with_owner)
                            {
                                match extrema {
                                    Extrema::Min => {
                                        if best_value.is_none()
                                            || *v < *best_value.as_ref().unwrap()
                                        {
                                            best_value = Some(*v);
                                            best_player = Some(player.name.clone());
                                        }
                                    }
                                    Extrema::Max => {
                                        if best_value.is_none()
                                            || *v > *best_value.as_ref().unwrap()
                                        {
                                            best_value = Some(*v);
                                            best_player = Some(player.name.clone());
                                        }
                                    }
                                }
                            }
                        }
                    }
                    if !found {
                        best_player = game_data.get_current_player().map(|p| p.name.clone());
                    }
                    best_player.ok_or(EngineError::OwnerOfMemoryNoPlayer)
                }
            },
            PlayerExpr::Query { query } => match query {
                QueryPlayer::Turnorder { int } => {
                    let idx = Self::eval_int(int, game_data)? as usize;
                    let player_idx = *game_data
                        .turn_order
                        .get(idx)
                        .ok_or(EngineError::TurnOrderIndexOutOfRange { idx })?;
                    game_data
                        .players
                        .get(player_idx)
                        .map(|p| p.name.clone())
                        .ok_or(EngineError::PlayerIndexNotFound { idx })
                }
                QueryPlayer::CollectionAt { players: pc, int } => {
                    let indices = Self::resolve_player_collection(pc, game_data)?;
                    let idx = Self::eval_int(int, game_data)? as usize;
                    let player_idx = *indices
                        .get(idx)
                        .ok_or(EngineError::PlayerCollectionAtOutOfRange { idx })?;
                    game_data
                        .players
                        .get(player_idx)
                        .map(|p| p.name.clone())
                        .ok_or(EngineError::PlayerCollectionIndexNotFound { idx })
                }
            },
            PlayerExpr::Memory { memory } => {
                let key = Self::resolve_memory_key(memory, game_data)?;
                match game_data.get_memory(&key) {
                    Some(MemoryValue::PlayerCollection(indices)) => {
                        if let Some(&idx) = indices.first() {
                            game_data
                                .players
                                .get(idx)
                                .map(|p| p.name.clone())
                                .ok_or(EngineError::PlayerIndexNotFound { idx })
                        } else {
                            Err(EngineError::EmptyPlayerCollectionMemory)
                        }
                    }
                    Some(MemoryValue::String(s)) => Ok(s.clone()),
                    Some(_) => Err(EngineError::MemoryNotValidPlayer),
                    None => Err(EngineError::MemoryNotFound { key }),
                }
            }
        }
    }

    pub fn eval_team(expr: &TeamExpr, game_data: &GameData) -> Result<String, EngineError> {
        match expr {
            TeamExpr::Literal { name } => Ok(name.clone()),
            TeamExpr::Aggregate { aggregate } => match aggregate {
                AggregateTeam::TeamOf { player } => {
                    let player_name = Self::eval_player(player, game_data)?;
                    for team in &game_data.teams {
                        for &player_idx in &team.players {
                            if game_data.players.get(player_idx).map(|p| &p.name)
                                == Some(&player_name)
                            {
                                return Ok(team.name.clone());
                            }
                        }
                    }
                    Err(EngineError::PlayerNotInAnyTeam { name: player_name })
                }
            },
            TeamExpr::Memory { memory } => {
                let key = Self::resolve_memory_key(memory, game_data)?;
                match game_data.get_memory(&key) {
                    Some(MemoryValue::Team(v)) => Ok(v.clone()),
                    Some(MemoryValue::TeamCollection(v)) => {
                        // A team-collection memory cannot name a single team;
                        // surface the first entry if there is exactly one.
                        if v.len() == 1 {
                            Ok(v[0].clone())
                        } else {
                            Err(EngineError::MemoryNotTeam)
                        }
                    }
                    Some(_) => Err(EngineError::MemoryNotTeam),
                    None => Err(EngineError::MemoryNotFound { key }),
                }
            }
        }
    }

    /// Resolve a `Players` expression to concrete player indices. Fallible:
    /// player-expressions that cannot be evaluated (e.g. `next` with no
    /// eligible player) or that reference unknown players yield `Err` instead
    /// of panicking (recoverable in the action/condition paths).
    pub fn resolve_players(
        players: &Players,
        game_data: &GameData,
    ) -> Result<Vec<usize>, EngineError> {
        match players {
            Players::Player { player } => {
                let name = Self::eval_player(player, game_data)?;
                let idx = game_data
                    .players
                    .iter()
                    .position(|p| p.name == name)
                    .ok_or(EngineError::ResolvePlayersPlayerNotFound { name })?;
                Ok(vec![idx])
            }
            Players::PlayerCollection { player_collection } => {
                Self::resolve_player_collection(player_collection, game_data)
            }
        }
    }

    /// Resolve a `PlayerCollection` to concrete player indices. Fallible since
    /// 2026-08: literal eval failures, unknown literal names, and missing
    /// memory slots return `Err`; the `Aggregate`/`AggregateMemory`/`Memory`
    /// arms are fully implemented (multi-owner aggregation).
    pub fn resolve_player_collection(
        pc: &PlayerCollection,
        game_data: &GameData,
    ) -> Result<Vec<usize>, EngineError> {
        match pc {
            PlayerCollection::Literal { players } => {
                let mut indices = vec![];
                for player_expr in players {
                    let name = Self::eval_player(player_expr, game_data)?;
                    let idx = game_data
                        .players
                        .iter()
                        .position(|p| p.name == name)
                        .ok_or(EngineError::ResolvePlayerCollectionPlayerNotFound { name })?;
                    indices.push(idx);
                }
                Ok(indices)
            }

            // `all` / `any` resolve to every in-game player; the caller
            // distinguishes fan-out (`all`) from pick-one (`any`).
            PlayerCollection::Aggregate { aggregate } => match aggregate {
                front_end::ast::AggregatePlayerCollection::Quantifier { .. } => Ok(game_data
                    .players
                    .iter()
                    .enumerate()
                    .filter(|(_, p)| p.in_game)
                    .map(|(i, _)| i)
                    .collect()),
            },
            PlayerCollection::Runtime { runtime } => match runtime {
                front_end::ast::RuntimePlayerCollection::PlayersOut => Ok(game_data
                    .players
                    .iter()
                    .enumerate()
                    .filter(|(_, p)| !p.in_game)
                    .map(|(i, _)| i)
                    .collect()),
                front_end::ast::RuntimePlayerCollection::PlayersIn => Ok(game_data
                    .players
                    .iter()
                    .enumerate()
                    .filter(|(_, p)| p.in_game)
                    .map(|(i, _)| i)
                    .collect()),
                front_end::ast::RuntimePlayerCollection::Others => {
                    let current = game_data
                        .get_current_player()
                        .map(|p| p.name.clone())
                        .unwrap_or_default();
                    Ok(game_data
                        .players
                        .iter()
                        .enumerate()
                        .filter(|(_, p)| p.name != current && p.in_game)
                        .map(|(i, _)| i)
                        .collect())
                }
            },
            // Aggregate a memory slot across every owner in `multi`, e.g.
            // `(&P:M of all)` / `(&P:M of others)`.
            PlayerCollection::AggregateMemory { memory, multi } => {
                let names = Self::resolve_multi_owner_names(multi, game_data)?;
                let mut indices = vec![];
                for name in names {
                    let key = format!("{}_{}", name, memory);
                    match game_data.get_memory(&key) {
                        Some(MemoryValue::PlayerCollection(ids)) => {
                            indices.extend(ids.iter().copied())
                        }
                        Some(_) => {
                            return Err(EngineError::MemoryNotPlayerCollectionFor { key });
                        }
                        None => {
                            return Err(EngineError::MemoryNotFound { key });
                        }
                    }
                }
                Ok(indices)
            }
            PlayerCollection::Memory { memory } => {
                let key = Self::resolve_collection_memory_key(memory, game_data)?;
                match game_data.get_memory(&key) {
                    Some(MemoryValue::PlayerCollection(ids)) => Ok(ids.clone()),
                    Some(_) => Err(EngineError::MemoryNotPlayerCollection),
                    None => Err(EngineError::MemoryNotFound { key }),
                }
            }
        }
    }

    /// Resolve the owner collection of a multi-owner memory reference to
    /// concrete names: `MultiOwner::PlayerCollection` → player names,
    /// `MultiOwner::TeamCollection` → team names.
    pub(super) fn resolve_multi_owner_names(
        multi: &MultiOwner,
        game_data: &GameData,
    ) -> Result<Vec<String>, EngineError> {
        match multi {
            MultiOwner::PlayerCollection { player_collection } => {
                let indices = Self::resolve_player_collection(player_collection, game_data)?;
                Ok(indices
                    .into_iter()
                    .map(|i| {
                        game_data
                            .players
                            .get(i)
                            .map(|p| p.name.clone())
                            .unwrap_or_default()
                    })
                    .collect())
            }
            MultiOwner::TeamCollection { team_collection } => {
                Self::eval_team_collection(team_collection, game_data)
            }
        }
    }

    pub fn resolve_owner_to_name(
        owner: &Owner,
        game_data: &GameData,
    ) -> Result<String, EngineError> {
        match owner {
            Owner::Table => Ok("Table".to_string()),
            Owner::Player { player } => Self::eval_player(player, game_data),
            Owner::Team { team } => Self::eval_team(team, game_data),
            Owner::PlayerCollection { .. } => Err(EngineError::OwnerNameFromPlayerCollection),
            Owner::TeamCollection { .. } => Err(EngineError::OwnerNameFromTeamCollection),
        }
    }

    /// Resolves an `Owner` to the list of entity names that own the
    /// locations/memories being created. Team owners resolve to the **team
    /// name itself** — `location X on T:Red` creates ONE shared pile for the
    /// team, and `memory M on T:Red` creates the single slot `Red_M`
    /// (matching the read/write addressing, e.g. `(&I:M of T:Red)`).
    pub fn resolve_owner_to_names(
        owner: &Owner,
        game_data: &GameData,
    ) -> Result<Vec<String>, EngineError> {
        match owner {
            Owner::Table => Ok(vec!["Table".to_string()]),
            Owner::Player { player } => Ok(vec![Self::eval_player(player, game_data)?]),
            Owner::Team { team } => {
                let team_name = Self::eval_team(team, game_data)?;
                if game_data.teams.iter().any(|t| t.name == team_name) {
                    Ok(vec![team_name])
                } else {
                    Ok(vec![])
                }
            }
            Owner::PlayerCollection {
                player_collection: pc,
            } => {
                let indices = Self::resolve_player_collection(pc, game_data)?;
                Ok(indices
                    .into_iter()
                    .map(|i| game_data.players[i].name.clone())
                    .collect())
            }
            Owner::TeamCollection { team_collection } => {
                let team_names = Self::eval_team_collection(team_collection, game_data)?;
                Ok(team_names
                    .into_iter()
                    .filter(|n| game_data.teams.iter().any(|t| &t.name == n))
                    .collect())
            }
        }
    }
}

#[cfg(test)]
#[path = "player_tests.rs"]
mod tests;
