use super::Evaluator;
use crate::game_data::{GameData, MemoryValue};
use front_end::ast::{
    AggregatePlayer, AggregateTeam, Extrema, Owner, PlayerCollection, PlayerExpr, Players,
    QueryPlayer, RuntimePlayer, TeamExpr, UseSingleMemory,
};

impl Evaluator {
    pub fn eval_player(expr: &PlayerExpr, game_data: &GameData) -> Result<String, String> {
        match expr {
            PlayerExpr::Literal { name } => Ok(name.clone()),
            PlayerExpr::Runtime { runtime } => match runtime {
                RuntimePlayer::Current => game_data
                    .get_current_player()
                    .map(|p| p.name.clone())
                    .ok_or("No current player".to_string()),
                RuntimePlayer::Next => {
                    let current_idx = game_data.current_player.ok_or("No current player")?;
                    let current_stage = game_data.get_current_stage().ok_or("No current stage")?;
                    let turn_len = game_data.turn_order.len();
                    for i in 1..turn_len {
                        let player_idx = game_data.turn_order[(current_idx + i) % turn_len];
                        if let Some(player) = game_data.players.get(player_idx) {
                            if player.in_game
                                && *player.in_stage.get(&current_stage).unwrap_or(&false)
                            {
                                return Ok(player.name.clone());
                            }
                        }
                    }
                    Err("No next player available".to_string())
                }
                RuntimePlayer::Previous => {
                    let current_idx = game_data.current_player.ok_or("No current player")?;
                    let turn_len = game_data.turn_order.len();
                    let prev_idx = (current_idx + turn_len - 1) % turn_len;
                    let player_idx = *game_data
                        .turn_order
                        .get(prev_idx)
                        .ok_or("Previous player not found")?;
                    game_data
                        .players
                        .get(player_idx)
                        .map(|p| p.name.clone())
                        .ok_or("Previous player not found".to_string())
                }
                RuntimePlayer::Competitor => {
                    let current = game_data.get_current_player().ok_or("No current player")?;
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
                    Err("No competitor found".to_string())
                }
            },
            PlayerExpr::Aggregate { aggregate } => match aggregate {
                AggregatePlayer::OwnerOfCardPostion { card_position } => {
                    let card_id = Self::eval_card_position(card_position, game_data)?;
                    for (loc_idx, loc) in game_data.locations.iter().enumerate() {
                        if loc.cards.contains(&card_id) {
                            if let Some(owner_loc_idx) =
                                game_data.table.locations.iter().find(|&&l| l == loc_idx)
                            {
                                return Ok("Table".to_string());
                            }
                            for (player_idx, player) in game_data.players.iter().enumerate() {
                                if player.owner.locations.contains(&loc_idx) {
                                    return Ok(player.name.clone());
                                }
                            }
                        }
                    }
                    Err("Owner of card position not found".to_string())
                }
                AggregatePlayer::OwnerOfMemory { extrema, memory } => {
                    let mem_key = memory;
                    let mut best_player: Option<String> = None;
                    let mut best_value: Option<i32> = None;
                    let mut found = false;
                    for player in &game_data.players {
                        if player.in_game {
                            found = true;
                            let mem_key_with_owner = format!("{}_{}", mem_key, player.name);
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
                    best_player.ok_or("No player found for OwnerOfMemory".to_string())
                }
            },
            PlayerExpr::Query { query } => match query {
                QueryPlayer::Turnorder { int } => {
                    let idx = Self::eval_int(int, game_data)? as usize;
                    let player_idx = *game_data
                        .turn_order
                        .get(idx)
                        .ok_or(format!("No player at turn order index {}", idx))?;
                    game_data
                        .players
                        .get(player_idx)
                        .map(|p| p.name.clone())
                        .ok_or(format!("Player at index {} not found", idx))
                }
                QueryPlayer::CollectionAt { players: pc, int } => {
                    let indices = Self::resolve_player_collection(pc, game_data);
                    let idx = Self::eval_int(int, game_data)? as usize;
                    let player_idx = *indices
                        .get(idx)
                        .ok_or(format!("No player at index {} in player collection", idx))?;
                    game_data
                        .players
                        .get(player_idx)
                        .map(|p| p.name.clone())
                        .ok_or(format!("Player at collection index {} not found", idx))
                }
            },
            PlayerExpr::Memory { memory } => {
                let key = match memory {
                    UseSingleMemory::Memory { memory: m } => m.clone(),
                    UseSingleMemory::WithOwner { memory: m, .. } => m.clone(),
                };
                match game_data.get_memory(&key) {
                    Some(MemoryValue::PlayerCollection(indices)) => {
                        if let Some(&idx) = indices.first() {
                            game_data
                                .players
                                .get(idx)
                                .map(|p| p.name.clone())
                                .ok_or(format!("Player at index {} not found", idx))
                        } else {
                            Err("PlayerCollection memory is empty".to_string())
                        }
                    }
                    Some(MemoryValue::String(s)) => Ok(s.clone()),
                    Some(_) => Err("Memory value is not a valid player".to_string()),
                    None => Err(format!("Memory {} not found", key)),
                }
            }
        }
    }

    pub fn eval_team(expr: &TeamExpr, game_data: &GameData) -> Result<String, String> {
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
                    Err(format!("Player {} not found in any team", player_name))
                }
            },
            TeamExpr::Memory { memory } => {
                let key = match memory {
                    UseSingleMemory::Memory { memory: m } => m.clone(),
                    UseSingleMemory::WithOwner { memory: m, .. } => m.clone(),
                };
                match game_data.get_memory(&key) {
                    Some(MemoryValue::Team(v)) => Ok(v.clone()),
                    Some(_) => Err("Memory value is not a Team".to_string()),
                    None => Err(format!("Memory {} not found", key)),
                }
            }
        }
    }

    pub fn resolve_players(players: &Players, game_data: &GameData) -> Vec<usize> {
        match players {
            Players::Player { player } => {
                let name = Self::eval_player(player, game_data).unwrap_or_else(|e| {
                    panic!("resolve_players: failed to eval player {:?}: {}", player, e)
                });
                vec![game_data
                    .players
                    .iter()
                    .position(|p| p.name == name)
                    .unwrap_or_else(|| {
                        panic!("resolve_players: player {} not found in game_data", name)
                    })]
            }
            Players::PlayerCollection { player_collection } => {
                Self::resolve_player_collection(player_collection, game_data)
            }
        }
    }

    pub fn resolve_player_collection(pc: &PlayerCollection, game_data: &GameData) -> Vec<usize> {
        match pc {
            PlayerCollection::Literal { players } => {
                let mut indices = vec![];
                for player_expr in players {
                    let name = Self::eval_player(player_expr, game_data).unwrap_or_else(|e| {
                        panic!(
                            "resolve_player_collection: failed to eval player {:?}: {}",
                            player_expr, e
                        )
                    });
                    if let Some(idx) = game_data.players.iter().position(|p| p.name == name) {
                        indices.push(idx);
                    }
                }
                indices
            }

            PlayerCollection::Aggregate { .. } => {
                todo!("PlayerCollection::Aggregate not yet implemented")
            }
            PlayerCollection::Runtime { runtime } => match runtime {
                front_end::ast::RuntimePlayerCollection::PlayersOut => game_data
                    .players
                    .iter()
                    .enumerate()
                    .filter(|(_, p)| !p.in_game)
                    .map(|(i, _)| i)
                    .collect(),
                front_end::ast::RuntimePlayerCollection::PlayersIn => game_data
                    .players
                    .iter()
                    .enumerate()
                    .filter(|(_, p)| p.in_game)
                    .map(|(i, _)| i)
                    .collect(),
                front_end::ast::RuntimePlayerCollection::Others => {
                    let current = game_data
                        .get_current_player()
                        .map(|p| p.name.clone())
                        .unwrap_or_default();
                    game_data
                        .players
                        .iter()
                        .enumerate()
                        .filter(|(_, p)| p.name != current && p.in_game)
                        .map(|(i, _)| i)
                        .collect()
                }
            },
            // TODO: memory not implemented
            PlayerCollection::AggregateMemory {
                memory: _,
                multi: _,
            } => vec![],
            PlayerCollection::Memory { memory: _ } => vec![],
        }
    }

    pub fn resolve_owner_to_name(owner: &Owner, game_data: &GameData) -> Result<String, String> {
        match owner {
            Owner::Table => Ok("Table".to_string()),
            Owner::Player { player } => Self::eval_player(player, game_data),
            Owner::Team { team } => Self::eval_team(team, game_data),
            Owner::PlayerCollection { .. } => Err(
                "resolve_owner_to_name: PlayerCollection cannot resolve to a single name"
                    .to_string(),
            ),
            Owner::TeamCollection { .. } => Err(
                "resolve_owner_to_name: TeamCollection cannot resolve to a single name".to_string(),
            ),
        }
    }

    pub fn resolve_owner_to_names(
        owner: &Owner,
        game_data: &GameData,
    ) -> Result<Vec<String>, String> {
        match owner {
            Owner::Table => Ok(vec!["Table".to_string()]),
            Owner::Player { player } => Ok(vec![Self::eval_player(player, game_data)?]),
            Owner::Team { team } => {
                let name = Self::eval_team(team, game_data)?;
                Err(format!(
                    "resolve_owner_to_names: team '{name}' cannot own a location or memory (team-owned locations are not in the data model)"
                ))
            }
            Owner::PlayerCollection {
                player_collection: pc,
            } => {
                let indices = crate::quantifier::resolve_player_candidates(pc, game_data);
                Ok(indices
                    .into_iter()
                    .map(|i| game_data.players[i].name.clone())
                    .collect())
            }
            Owner::TeamCollection { .. } => Err(
                "resolve_owner_to_names: TeamCollection cannot resolve to owner names".to_string(),
            ),
        }
    }
}

#[cfg(test)]
#[path = "player_tests.rs"]
mod tests;
