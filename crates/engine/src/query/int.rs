use super::Evaluator;
use crate::game_data::{GameData, MemoryValue};
use front_end::ast::{
    AggregateInt, Collection, Extrema, IntCollection, IntExpr, IntOp, LocationCollection, Quantity,
    QueryInt, RuntimeInt, RuntimeTeamCollection, TeamCollection,
};

impl Evaluator {
    pub fn eval_int(expr: &IntExpr, game_data: &GameData) -> Result<i32, String> {
        match expr {
            IntExpr::Literal { int } => Ok(*int),
            IntExpr::Binary { int, op, int1 } => {
                let left = Self::eval_int(int, game_data)?;
                let right = Self::eval_int(int1, game_data)?;
                match op {
                    IntOp::Plus => Ok(left + right),
                    IntOp::Minus => Ok(left - right),
                    IntOp::Mul => Ok(left * right),
                    IntOp::Div => {
                        if right == 0 {
                            Err("Division by zero".to_string())
                        } else {
                            Ok(left / right)
                        }
                    }
                    IntOp::Mod => Ok(left % right),
                }
            }
            IntExpr::Query { query } => match query {
                QueryInt::IntCollectionAt {
                    int_collection,
                    int_expr,
                } => {
                    let ints = Self::eval_int_collection(int_collection, game_data)?;
                    let idx = Self::eval_int(int_expr, game_data)? as usize;
                    ints.get(idx)
                        .copied()
                        .ok_or(format!("No int at index {}", idx))
                }
            },
            IntExpr::Aggregate { aggregate } => match aggregate {
                AggregateInt::SizeOf { collection } => {
                    Self::eval_collection_size(collection, game_data)
                }
                AggregateInt::SumOfIntCollection { int_collection } => {
                    let ints = Self::eval_int_collection(int_collection, game_data)?;
                    Ok(ints.iter().sum())
                }
                AggregateInt::SumOfCardSet { card_set, pointmap } => {
                    let (_, card_ids) = Self::eval_cardset(card_set, game_data)?;
                    let point_map = game_data
                        .point_maps
                        .iter()
                        .find(|pm| pm.name == *pointmap)
                        .ok_or(format!("PointMap {} not found", pointmap))?;
                    let mut sum = 0;
                    for card_id in &card_ids {
                        if let Some(card) = game_data.get_card(*card_id) {
                            for (key, value) in card.iter() {
                                let map_key = format!("{}:{}", key, value);
                                if let Some(&points) = point_map.map.get(&map_key) {
                                    sum += points;
                                    break;
                                }
                            }
                        }
                    }
                    Ok(sum)
                }
                AggregateInt::ExtremaCardset {
                    extrema,
                    card_set,
                    pointmap,
                } => {
                    let (_, card_ids) = Self::eval_cardset(card_set, game_data)?;
                    let point_map = game_data
                        .point_maps
                        .iter()
                        .find(|pm| pm.name == *pointmap)
                        .ok_or(format!("PointMap {} not found", pointmap))?;
                    let mut best_card_id = None;
                    let mut best_value = None;
                    for card_id in &card_ids {
                        if let Some(card) = game_data.get_card(*card_id) {
                            let mut card_value = 0;
                            for (key, value) in card.iter() {
                                let map_key = format!("{}:{}", key, value);
                                if let Some(&points) = point_map.map.get(&map_key) {
                                    card_value = points;
                                    break;
                                }
                            }
                            match extrema {
                                Extrema::Min => {
                                    if best_value.is_none()
                                        || card_value < *best_value.as_ref().unwrap()
                                    {
                                        best_value = Some(card_value);
                                        best_card_id = Some(*card_id);
                                    }
                                }
                                Extrema::Max => {
                                    if best_value.is_none()
                                        || card_value > *best_value.as_ref().unwrap()
                                    {
                                        best_value = Some(card_value);
                                        best_card_id = Some(*card_id);
                                    }
                                }
                            }
                        }
                    }
                    best_card_id
                        .map(|id| id as i32)
                        .ok_or("No card found for extrema".to_string())
                }
                AggregateInt::ExtremaIntCollection {
                    extrema,
                    int_collection,
                } => {
                    let ints = Self::eval_int_collection(int_collection, game_data)?;
                    let mut best_value = None;
                    for &v in &ints {
                        match extrema {
                            Extrema::Min => {
                                if best_value.is_none() || v < *best_value.as_ref().unwrap() {
                                    best_value = Some(v);
                                }
                            }
                            Extrema::Max => {
                                if best_value.is_none() || v > *best_value.as_ref().unwrap() {
                                    best_value = Some(v);
                                }
                            }
                        }
                    }
                    best_value.ok_or("No value found in IntCollection".to_string())
                }
            },
            IntExpr::Runtime { runtime } => match runtime {
                RuntimeInt::CurrentStageRoundCounter => {
                    let stage = game_data.get_current_stage().ok_or("No current stage")?;
                    Ok(game_data.get_stage_counter(stage) as i32)
                }
                RuntimeInt::StageRoundCounter { stage } => {
                    Ok(game_data.get_stage_counter(stage.clone()) as i32)
                }
            },
            IntExpr::Memory { memory } => {
                let key = Self::resolve_memory_key(memory, game_data)?;
                match game_data.get_memory(&key) {
                    Some(MemoryValue::Int(v)) => Ok(*v),
                    Some(_) => Err("Memory value is not an Int".to_string()),
                    None => Err(format!("Memory {} not found", key)),
                }
            }
        }
    }

    fn eval_int_collection(col: &IntCollection, game_data: &GameData) -> Result<Vec<i32>, String> {
        match col {
            IntCollection::Literal { ints } => {
                let mut result = vec![];
                for i in ints {
                    result.push(Self::eval_int(i, game_data)?);
                }
                Ok(result)
            }
            IntCollection::AggregateMemory { memory: _, multi } => {
                todo!(
                    "IntCollection::AggregateMemory not yet implemented: {:?}",
                    multi
                )
            }
            IntCollection::Memory { memory } => {
                let key = Self::resolve_collection_memory_key(memory, game_data)?;
                match game_data.get_memory(&key) {
                    Some(MemoryValue::IntCollection(v)) => Ok(v.clone()),
                    Some(_) => Err("Memory value is not an IntCollection".to_string()),
                    None => Err(format!("Memory {} not found", key)),
                }
            }
        }
    }

    fn eval_collection_size(collection: &Collection, game_data: &GameData) -> Result<i32, String> {
        match collection {
            Collection::IntCollection { int: col } => {
                Self::eval_int_collection(col, game_data).map(|v| v.len() as i32)
            }
            Collection::StringCollection { string: col } => {
                Self::eval_string_collection(col, game_data).map(|v| v.len() as i32)
            }
            Collection::LocationCollection { location: col } => {
                Self::eval_location_collection(col, game_data).map(|v| v.len() as i32)
            }
            Collection::PlayerCollection { player: col } => {
                Ok(Self::resolve_player_collection(col, game_data).len() as i32)
            }
            Collection::TeamCollection { team: col } => {
                Self::eval_team_collection(col, game_data).map(|v| v.len() as i32)
            }
            Collection::CardSet { card_set: cs } => {
                Self::eval_cardset(cs, game_data).map(|(_, card_ids)| card_ids.len() as i32)
            }
        }
    }

    pub(super) fn eval_location_collection(
        col: &LocationCollection,
        game_data: &GameData,
    ) -> Result<Vec<String>, String> {
        match col {
            LocationCollection::Literal { locations } => Ok(locations.clone()),
            LocationCollection::Memory { memory } => {
                let key = Self::resolve_collection_memory_key(memory, game_data)?;
                match game_data.get_memory(&key) {
                    Some(MemoryValue::LocationCollection(v)) => Ok(v
                        .iter()
                        .map(|&idx| {
                            game_data
                                .locations
                                .get(idx)
                                .map(|l| l.name.clone())
                                .unwrap_or_default()
                        })
                        .collect()),
                    Some(_) => Err("Memory value is not a LocationCollection".to_string()),
                    None => Err(format!("Memory {} not found", key)),
                }
            }
        }
    }

    fn eval_team_collection(
        col: &TeamCollection,
        game_data: &GameData,
    ) -> Result<Vec<String>, String> {
        match col {
            TeamCollection::Literal { teams } => {
                let mut result = vec![];
                for t in teams {
                    result.push(Self::eval_team(t, game_data)?);
                }
                Ok(result)
            }
            TeamCollection::Runtime { runtime } => match runtime {
                RuntimeTeamCollection::OtherTeams => {
                    let mut result = vec![];
                    for team in &game_data.teams {
                        result.push(team.name.clone());
                    }
                    Ok(result)
                }
            },
            TeamCollection::AggregateMemory { memory: _, multi } => {
                todo!(
                    "TeamCollection::AggregateMemory not yet implemented: {:?}",
                    multi
                )
            }
            TeamCollection::Memory { memory } => {
                let key = Self::resolve_collection_memory_key(memory, game_data)?;
                match game_data.get_memory(&key) {
                    Some(MemoryValue::Team(v)) => Ok(vec![v.clone()]),
                    Some(_) => Err("Memory value is not a Team".to_string()),
                    None => Err(format!("Memory {} not found", key)),
                }
            }
        }
    }

    pub fn resolve_quantity(qty: &Quantity, available: usize) -> Result<usize, String> {
        match qty {
            Quantity::Int { int } => {
                let val = Self::eval_int(int, &GameData::new()).unwrap_or(1) as usize;
                Ok(val.min(available))
            }
            Quantity::Quantifier { quantifier } => match quantifier {
                front_end::ast::Quantifier::All => Ok(available),
                front_end::ast::Quantifier::Any => Ok(1),
            },
            Quantity::IntRange { int_range } => {
                let (start_cmp, start_expr) = &int_range.start;
                let start_satisfied = match Self::eval_int(start_expr, &GameData::new()) {
                    Ok(target) => Self::eval_int_compare(available as i32, start_cmp, target),
                    Err(_) => false,
                };
                if !start_satisfied {
                    return Ok(0);
                }
                for (op, cmp, int_expr) in &int_range.op_int {
                    let target = Self::eval_int(int_expr, &GameData::new()).unwrap_or(0);
                    let satisfied = Self::eval_int_compare(available as i32, cmp, target);
                    match op {
                        front_end::ast::IntRangeOperator::And => {
                            if !satisfied {
                                return Ok(0);
                            }
                        }
                        front_end::ast::IntRangeOperator::Or => {
                            if satisfied {
                                return Ok(available);
                            }
                        }
                    }
                }
                Ok(available)
            }
        }
    }
}

#[cfg(test)]
#[path = "int_tests.rs"]
mod tests;
