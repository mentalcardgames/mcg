/*
1. BoolExpr
BoolExpr
├── Binary { bool_expr, op: And|Or, bool_expr1 }
├── Unary { op: Not, bool_expr }
└── Aggregate { aggregate: AggregateBool }
    ├── Compare { cmp_bool: CompareBool }
    │   ├── Int { int: IntExpr, cmp: Eq|Neq|Gt|Lt|Ge|Le, int1: IntExpr }
    │   ├── CardSet { card_set, cmp: Eq|Neq, card_set1 }
    │   ├── String { string, cmp: Eq|Neq, string1 }
    │   ├── Player { player, cmp: Eq|Neq, player1 }
    │   └── Team { team, cmp: Eq|Neq, team1 }
    ├── StringInCardSet { string: StringExpr, card_set: CardSet }
    ├── StringNotInCardSet { string: StringExpr, card_set: CardSet }
    ├── CardSetEmpty { card_set: CardSet }
    ├── CardSetNotEmpty { card_set: CardSet }
    └── OutOfPlayer { players: Players, out_of: OutOf }
            └── OutOf: CurrentStage | Stage { name } | Game | GameSuccessful | GameFail
2. EndCondition
EndCondition
├── UntilBool { bool_expr: BoolExpr }
├── UntilBoolRep { bool_expr: BoolExpr, logic: BoolOp, reps: Repititions }
│       └── Repititions: { times: IntExpr }
├── UntilRep { reps: Repititions }
└── UntilEnd
3. IntExpr
IntExpr
├── Literal { int: i32 }
├── Binary { int, op: Plus|Minus|Mul|Div|Mod, int1 }
├── Query { query: QueryInt }
│   └── IntCollectionAt { int_collection, int_expr }
├── Aggregate { aggregate: AggregateInt }
│   ├── SizeOf { collection: Collection }
│   ├── SumOfIntCollection { int_collection }
│   ├── SumOfCardSet { card_set, pointmap }
│   ├── ExtremaCardset { extrema: Min|Max, card_set, pointmap }
│   └── ExtremaIntCollection { extrema, int_collection }
├── Runtime { runtime: RuntimeInt }
│   ├── CurrentStageRoundCounter
│   └── StageRoundCounter { stage: String }
└── Memory { memory: UseSingleMemory }
3. StringExpr
StringExpr
├── Literal { value: String }
├── Query { query: QueryString }
│   ├── KeyOf { key: String, card_position: CardPosition }
│   └── StringCollectionAt { string_collection, int_expr }
└── Memory { memory: UseSingleMemory }
4. PlayerExpr
PlayerExpr
├── Literal { name: String }
├── Runtime { runtime: RuntimePlayer }
│   └── Current | Next | Previous | Competitor
├── Aggregate { aggregate: AggregatePlayer }
│   ├── OwnerOfCardPostion { card_position }
│   └── OwnerOfMemory { extrema: Min|Max, memory: String }
├── Query { query: QueryPlayer }
│   ├── Turnorder { int: IntExpr }
│   └── CollectionAt { players: PlayerCollection, int: IntExpr }
└── Memory { memory: UseSingleMemory }
5. TeamExpr
TeamExpr
├── Literal { name: String }
├── Aggregate { aggregate: AggregateTeam }
│   └── TeamOf { player: PlayerExpr }
└── Memory { memory: UseSingleMemory }
6. CardPosition
CardPosition
├── Query { query: QueryCardPosition }
│   ├── At { location: String, int_expr: IntExpr }
│   ├── Top { location: String }
│   └── Bottom { location: String }
└── Aggregate { aggregate: AggregateCardPosition }
    ├── ExtremaPointMap { extrema, card_set, pointmap }
    └── ExtremaPrecedence { extrema, card_set, precedence }
7. CardSet
CardSet
├── Group { group: Group }
├── GroupOwner { group: Group, owner: Owner }
└── Memory { memory: UseMemory }
Group
├── Groupable { groupable: Groupable }
│   └── Groupable: Location { name } | LocationCollection { locations }
├── Where { groupable, filter: FilterExpr }
├── NotCombo { combo: String, groupable }
├── Combo { combo: String, groupable }
└── CardPosition { card_position }
Owner
├── Player { player: PlayerExpr }
├── Team { team: TeamExpr }
├── Table
├── PlayerCollection { player_collection }
└── TeamCollection { team_collection }
8. FilterExpr
FilterExpr
├── Aggregate { aggregate: AggregateFilter }
│   ├── Size { cmp: IntCompare, int_expr }
│   ├── Same { key: String }           // same Rank
│   ├── Distinct { key: String }        // distinct Suit
│   ├── Adjacent { key, precedence }    // adjacent Rank using Precedence
│   ├── Higher { key, value: StringExpr, precedence }  // higher than "Ace"
│   ├── Lower { key, value: StringExpr, precedence }    // lower than "Ace"
│   ├── KeyIsString { key, string }    // Suite is "Hearts"
│   ├── KeyIsNotString { key, string } // Suite is not "Hearts"
│   ├── Combo { combo: String }        // matches combo
│   └── NotCombo { combo: String }     // doesn't match combo
└── Binary { filter, op: And|Or, filter1 }
9. Players
Players
├── Player { player: PlayerExpr }
└── PlayerCollection { player_collection }
PlayerCollection
├── Literal { players: Vec<PlayerExpr> }
├── Aggregate { aggregate: AggregatePlayerCollection }
│   └── Quantifier { quantifier: All|Any }
├── Runtime { runtime: RuntimePlayerCollection }
│   └── PlayersOut | PlayersIn | Others
├── AggregateMemory { memory: String, multi: MultiOwner }
└── Memory { memory: UseMemory }
10. MemoryType (all storable types)
MemoryType
├── Int { int: IntExpr }
├── String { string: StringExpr }
├── Player { player: PlayerExpr }
├── Team { team: TeamExpr }
├── CardSet { card_set: CardSet }
├── PlayerCollection { players }
├── StringCollection { strings }
├── TeamCollection { teams }
├── IntCollection { ints }
└── LocationCollection { locations }
Supporting Types
Collection
├── IntCollection { int }
├── StringCollection { string }
├── LocationCollection { location }
├── PlayerCollection { player }
├── TeamCollection { team }
└── CardSet { card_set }
IntCollection / StringCollection
├── Literal { ints/strings: Vec<...> }
├── AggregateMemory { memory, multi: MultiOwner }
└── Memory { memory: UseMemory }
LocationCollection
├── Literal { locations: Vec<String> }
└── Memory { memory: UseMemory }
TeamCollection
├── Literal { teams: Vec<TeamExpr> }
├── Runtime { runtime: OtherTeams }
├── AggregateMemory { memory, multi: MultiOwner }
└── Memory { memory: UseMemory }
MultiOwner
├── PlayerCollection { player_collection }
└── TeamCollection { team_collection }
UseMemory
├── Memory { memory: String }
└── WithOwner { memory: String, owner: Box<Owner> }
UseSingleMemory
├── Memory { memory: String }
└── WithOwner { memory: String, owner: Box<SingleOwner> }
SingleOwner
├── Player { player: PlayerExpr }
├── Team { team: TeamExpr }
└── Table
 */

use crate::game_data::{Card, GameData, MemoryValue};
use front_end::ast::{
    AggregateBool, AggregateFilter, AggregateInt, AggregatePlayer, AggregateTeam, BoolExpr, BoolOp,
    CardPosition, CardSet, Collection, CompareBool, EndCondition, Extrema, FilterExpr, Group,
    Groupable, IntCollection, IntExpr, IntOp, LocationCollection, Owner, PlayerCollection,
    PlayerExpr, Players, Quantity, QueryCardPosition, QueryInt, QueryPlayer, QueryString,
    RuntimeInt, RuntimePlayer, RuntimeTeamCollection, SingleOwner, StringCollection, StringExpr,
    TeamCollection, TeamExpr, Types, UnaryOp, UseMemory, UseSingleMemory,
};

pub struct Evaluator;

impl Evaluator {
    pub fn eval_bool(expr: &BoolExpr, game_data: &GameData) -> Result<bool, String> {
        match expr {
            BoolExpr::Binary {
                bool_expr,
                op,
                bool_expr1,
            } => {
                let left = Self::eval_bool(bool_expr, game_data)?;
                let right = Self::eval_bool(bool_expr1, game_data)?;
                match op {
                    BoolOp::And => Ok(left && right),
                    BoolOp::Or => Ok(left || right),
                }
            }
            BoolExpr::Unary { op, bool_expr } => {
                let inner = Self::eval_bool(bool_expr, game_data)?;
                match op {
                    UnaryOp::Not => Ok(!inner),
                }
            }
            BoolExpr::Aggregate { aggregate } => Self::eval_aggregate(aggregate, game_data),
        }
    }

    pub fn eval_aggregate(aggregate: &AggregateBool, game_data: &GameData) -> Result<bool, String> {
        match aggregate {
            AggregateBool::Compare { cmp_bool } => Self::eval_compare(cmp_bool, game_data),
            AggregateBool::StringInCardSet { string, card_set } => {
                let s = Self::eval_string(string, game_data)?;
                let cards = Self::eval_cardset(card_set, game_data)?.1;
                Ok(Self::check_attr_value_in_cardset(&s, &cards, game_data))
            }
            AggregateBool::StringNotInCardSet { string, card_set } => {
                let s = Self::eval_string(string, game_data)?;
                let cards = Self::eval_cardset(card_set, game_data)?.1;
                Ok(!Self::check_attr_value_in_cardset(&s, &cards, game_data))
            }
            AggregateBool::CardSetEmpty { card_set } => {
                let cards = Self::eval_cardset(card_set, game_data)?.1;
                Ok(cards.is_empty())
            }
            AggregateBool::CardSetNotEmpty { card_set } => {
                let cards = Self::eval_cardset(card_set, game_data)?.1;
                Ok(!cards.is_empty())
            }
            AggregateBool::OutOfPlayer { players, out_of } => {
                let player_indices = Self::resolve_players(players, game_data);
                let current_stage = game_data.get_current_stage().unwrap_or_default();
                match out_of {
                    front_end::ast::OutOf::CurrentStage => {
                        for &pid in &player_indices {
                            if let Some(player) = game_data.players.get(pid) {
                                if *player.in_stage.get(&current_stage).unwrap_or(&false) {
                                    return Ok(false);
                                }
                            }
                        }
                        Ok(true)
                    }
                    front_end::ast::OutOf::Stage { name } => {
                        for &pid in &player_indices {
                            if let Some(player) = game_data.players.get(pid) {
                                if *player.in_stage.get(name).unwrap_or(&false) {
                                    return Ok(false);
                                }
                            }
                        }
                        Ok(true)
                    }
                    front_end::ast::OutOf::Game => {
                        for &pid in &player_indices {
                            if let Some(player) = game_data.players.get(pid) {
                                if player.in_game {
                                    return Ok(false);
                                }
                            }
                        }
                        Ok(true)
                    }
                    front_end::ast::OutOf::GameSuccessful | front_end::ast::OutOf::GameFail => {
                        for &pid in &player_indices {
                            if let Some(player) = game_data.players.get(pid) {
                                if player.in_game {
                                    return Ok(false);
                                }
                            }
                        }
                        Ok(true)
                    }
                }
            }
        }
    }

    pub fn eval_compare(cmp_bool: &CompareBool, game_data: &GameData) -> Result<bool, String> {
        match cmp_bool {
            CompareBool::Int { int, cmp, int1 } => {
                let left = Self::eval_int(int, game_data)?;
                let right = Self::eval_int(int1, game_data)?;
                Ok(Self::eval_int_compare(left, cmp, right))
            }
            CompareBool::CardSet {
                card_set,
                cmp,
                card_set1,
            } => {
                let left = Self::eval_cardset(card_set, game_data)?;
                let right = Self::eval_cardset(card_set1, game_data)?;
                match cmp {
                    front_end::ast::CardSetCompare::Eq => Ok(left == right),
                    front_end::ast::CardSetCompare::Neq => Ok(left != right),
                }
            }
            CompareBool::String {
                string,
                cmp,
                string1,
            } => {
                let left = Self::eval_string(string, game_data)?;
                let right = Self::eval_string(string1, game_data)?;
                match cmp {
                    front_end::ast::StringCompare::Eq => Ok(left == right),
                    front_end::ast::StringCompare::Neq => Ok(left != right),
                }
            }
            CompareBool::Player {
                player,
                cmp,
                player1,
            } => {
                let left = Self::eval_player(player, game_data)?;
                let right = Self::eval_player(player1, game_data)?;
                match cmp {
                    front_end::ast::PlayerCompare::Eq => Ok(left == right),
                    front_end::ast::PlayerCompare::Neq => Ok(left != right),
                }
            }
            CompareBool::Team { team, cmp, team1 } => {
                let left = Self::eval_team(team, game_data)?;
                let right = Self::eval_team(team1, game_data)?;
                match cmp {
                    front_end::ast::TeamCompare::Eq => Ok(left == right),
                    front_end::ast::TeamCompare::Neq => Ok(left != right),
                }
            }
        }
    }

    pub fn eval_int_compare(left: i32, cmp: &front_end::ast::IntCompare, right: i32) -> bool {
        match cmp {
            front_end::ast::IntCompare::Eq => left == right,
            front_end::ast::IntCompare::Neq => left != right,
            front_end::ast::IntCompare::Gt => left > right,
            front_end::ast::IntCompare::Lt => left < right,
            front_end::ast::IntCompare::Ge => left >= right,
            front_end::ast::IntCompare::Le => left <= right,
        }
    }

    pub fn eval_end_condition(
        end_condition: &EndCondition,
        game_data: &GameData,
        stage_name: &String,
    ) -> Result<bool, String> {
        match end_condition {
            EndCondition::UntilEnd => Ok(false),
            EndCondition::UntilRep { reps } => {
                let current = game_data.get_stage_counter(stage_name.clone());
                // evaluate target and handle error propagation
                match Self::eval_int(&reps.times, game_data) {
                    Ok(target) => Ok(current >= target as u32),
                    Err(e) => Err(e),
                }
            }
            EndCondition::UntilBool { bool_expr } => Self::eval_bool(bool_expr, game_data),
            EndCondition::UntilBoolRep {
                bool_expr,
                logic,
                reps,
            } => {
                let bool_result = Self::eval_bool(bool_expr, game_data)?;
                let current = game_data.get_stage_counter(stage_name.clone());
                // evaluate target and handle error propagation
                let rep_result = match Self::eval_int(&reps.times, game_data) {
                    Ok(target) => current >= target as u32,
                    Err(e) => return Err(e),
                };
                match logic {
                    BoolOp::And => Ok(bool_result && rep_result),
                    BoolOp::Or => Ok(bool_result || rep_result),
                }
            }
        }
    }

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
                let key = match memory {
                    UseSingleMemory::Memory { memory: m } => m.clone(),
                    UseSingleMemory::WithOwner { memory: m, .. } => m.clone(),
                };
                match game_data.get_memory(&key) {
                    Some(MemoryValue::Int(v)) => Ok(v.clone()),
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
                let key = match memory {
                    UseMemory::Memory { memory: m } => m.clone(),
                    UseMemory::WithOwner { memory: m, .. } => m.clone(),
                };
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

    fn eval_location_collection(
        col: &LocationCollection,
        game_data: &GameData,
    ) -> Result<Vec<String>, String> {
        match col {
            LocationCollection::Literal { locations } => Ok(locations.clone()),
            LocationCollection::Memory { memory } => {
                let key = match memory {
                    UseMemory::Memory { memory: m } => m.clone(),
                    UseMemory::WithOwner { memory: m, .. } => m.clone(),
                };
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
                let key = match memory {
                    UseMemory::Memory { memory: m } => m.clone(),
                    UseMemory::WithOwner { memory: m, .. } => m.clone(),
                };
                match game_data.get_memory(&key) {
                    Some(MemoryValue::Team(v)) => Ok(vec![v.clone()]),
                    Some(_) => Err("Memory value is not a Team".to_string()),
                    None => Err(format!("Memory {} not found", key)),
                }
            }
        }
    }

    pub fn eval_string(expr: &StringExpr, game_data: &GameData) -> Result<String, String> {
        match expr {
            StringExpr::Literal { value } => Ok(value.clone()),
            StringExpr::Query { query } => match query {
                QueryString::KeyOf { key, card_position } => {
                    let card_id = Self::eval_card_position(card_position, game_data)?;
                    let card = game_data
                        .get_card(card_id)
                        .ok_or(format!("Card {} not found", card_id))?;
                    card.get(key)
                        .cloned()
                        .ok_or(format!("Key {} not found in card {}", key, card_id))
                }
                QueryString::StringCollectionAt {
                    string_collection,
                    int_expr,
                } => {
                    let strings = Self::eval_string_collection(string_collection, game_data)?;
                    let idx = Self::eval_int(int_expr, game_data)? as usize;
                    strings
                        .get(idx)
                        .cloned()
                        .ok_or(format!("No string at index {}", idx))
                }
            },
            StringExpr::Memory { memory } => {
                let key = match memory {
                    UseSingleMemory::Memory { memory: m } => m.clone(),
                    UseSingleMemory::WithOwner { memory: m, .. } => m.clone(),
                };
                match game_data.get_memory(&key) {
                    Some(MemoryValue::String(v)) => Ok(v.clone()),
                    Some(_) => Err("Memory value is not a String".to_string()),
                    None => Err(format!("Memory {} not found", key)),
                }
            }
        }
    }

    fn eval_string_collection(
        col: &StringCollection,
        game_data: &GameData,
    ) -> Result<Vec<String>, String> {
        match col {
            StringCollection::Literal { strings } => {
                let mut result = vec![];
                for s in strings {
                    result.push(Self::eval_string(s, game_data)?);
                }
                Ok(result)
            }
            StringCollection::AggregateMemory { memory: _, multi } => {
                todo!(
                    "StringCollection::AggregateMemory not yet implemented: {:?}",
                    multi
                )
            }
            StringCollection::Memory { memory } => {
                let key = match memory {
                    UseMemory::Memory { memory: m } => m.clone(),
                    UseMemory::WithOwner { memory: m, .. } => m.clone(),
                };
                match game_data.get_memory(&key) {
                    Some(MemoryValue::StringCollection(v)) => Ok(v.clone()),
                    Some(_) => Err("Memory value is not a StringCollection".to_string()),
                    None => Err(format!("Memory {} not found", key)),
                }
            }
        }
    }

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

    pub fn eval_cardset(
        expr: &CardSet,
        game_data: &GameData,
    ) -> Result<(usize, Vec<usize>), String> {
        match expr {
            CardSet::Group { group } => Self::eval_group(group, game_data),
            CardSet::GroupOwner { group, owner } => {
                let (loc_idx, card_ids) = Self::eval_group(group, game_data)?;
                let owner_name = Self::resolve_owner_to_name(owner, game_data)?;
                let owner_idx = game_data
                    .players
                    .iter()
                    .position(|p| p.name == owner_name)
                    .unwrap_or(usize::MAX);
                let filtered: Vec<usize> = card_ids
                    .into_iter()
                    .filter(|&card_id| {
                        for (loc_i, loc) in game_data.locations.iter().enumerate() {
                            if loc.cards.contains(&card_id) {
                                if game_data.table.locations.contains(&loc_i) {
                                    return owner_name == "Table";
                                }
                                if game_data.players[owner_idx]
                                    .owner
                                    .locations
                                    .contains(&loc_i)
                                {
                                    return true;
                                }
                            }
                        }
                        false
                    })
                    .collect();
                Ok((loc_idx, filtered))
            }
            CardSet::Memory { memory } => {
                let key = match memory {
                    UseMemory::Memory { memory: m } => m.clone(),
                    UseMemory::WithOwner { memory: m, .. } => m.clone(),
                };
                match game_data.get_memory(&key) {
                    Some(MemoryValue::CardSet(card_ids)) => {
                        if let Some(&first_card) = card_ids.first() {
                            for (loc_idx, loc) in game_data.locations.iter().enumerate() {
                                if loc.cards.contains(&first_card) {
                                    return Ok((loc_idx, card_ids.clone()));
                                }
                            }
                        }
                        Ok((0, card_ids.clone()))
                    }
                    Some(_) => Err("Memory value is not a CardSet".to_string()),
                    None => Err(format!("Memory {} not found", key)),
                }
            }
        }
    }

    fn eval_group(group: &Group, game_data: &GameData) -> Result<(usize, Vec<usize>), String> {
        match group {
            Group::Groupable { groupable } => Self::eval_groupable(groupable, game_data),
            Group::Where { groupable, filter } => {
                let (_, card_ids) = Self::eval_groupable(groupable, game_data)?;
                let filtered = Self::apply_filter(filter, &card_ids, game_data)?;
                let location_idx = Self::infer_location_from_cards(&filtered, game_data)?;
                Ok((location_idx, filtered))
            }
            Group::NotCombo { combo, groupable } => {
                let (loc_idx, card_ids) = Self::eval_groupable(groupable, game_data)?;
                let combo_filter = game_data
                    .combos
                    .iter()
                    .find(|c| c.name == *combo)
                    .map(|c| c.filter.clone())
                    .ok_or(format!("Combo {} not found", combo))?;
                let filtered: Vec<usize> = card_ids
                    .into_iter()
                    .filter(|&card_id| {
                        !Self::card_matches_filter(card_id, &combo_filter, game_data)
                    })
                    .collect();
                Ok((loc_idx, filtered))
            }
            Group::Combo { combo, groupable } => {
                let (loc_idx, card_ids) = Self::eval_groupable(groupable, game_data)?;
                let combo_filter = game_data
                    .combos
                    .iter()
                    .find(|c| c.name == *combo)
                    .map(|c| c.filter.clone())
                    .ok_or(format!("Combo {} not found", combo))?;
                let filtered: Vec<usize> = card_ids
                    .into_iter()
                    .filter(|&card_id| Self::card_matches_filter(card_id, &combo_filter, game_data))
                    .collect();
                Ok((loc_idx, filtered))
            }
            Group::CardPosition { card_position } => {
                let card_id = Self::eval_card_position(card_position, game_data)?;
                for (loc_idx, loc) in game_data.locations.iter().enumerate() {
                    if loc.cards.contains(&card_id) {
                        return Ok((loc_idx, vec![card_id]));
                    }
                }
                Err("Card position not found in any location".to_string())
            }
        }
    }

    fn eval_groupable(
        groupable: &Groupable,
        game_data: &GameData,
    ) -> Result<(usize, Vec<usize>), String> {
        match groupable {
            Groupable::Location { name } => {
                let loc_idx = game_data
                    .locations
                    .iter()
                    .position(|l| l.name == *name)
                    .ok_or(format!("Location {} not found", name))?;
                let card_ids = game_data
                    .locations
                    .get(loc_idx)
                    .map(|l| l.cards.clone())
                    .unwrap_or_default();
                Ok((loc_idx, card_ids))
            }
            Groupable::LocationCollection {
                location_collection,
            } => {
                let loc_names = Self::eval_location_collection(location_collection, game_data)?;
                let mut all_cards = vec![];
                let mut location_idx = 0;
                for name in &loc_names {
                    if let Some(idx) = game_data.locations.iter().position(|l| &l.name == name) {
                        if location_idx == 0 {
                            location_idx = idx;
                        }
                        if let Some(loc) = game_data.locations.get(idx) {
                            all_cards.extend_from_slice(&loc.cards);
                        }
                    }
                }
                if location_idx == 0 && !loc_names.is_empty() {
                    location_idx = game_data
                        .locations
                        .iter()
                        .position(|l| l.name == loc_names[0])
                        .unwrap_or(0);
                }
                Ok((location_idx, all_cards))
            }
        }
    }

    fn apply_filter(
        filter: &FilterExpr,
        card_ids: &[usize],
        game_data: &GameData,
    ) -> Result<Vec<usize>, String> {
        match filter {
            FilterExpr::Aggregate { aggregate } => match aggregate {
                AggregateFilter::Size { cmp, int_expr } => {
                    let target = Self::eval_int(int_expr, game_data)?;
                    let size = card_ids.len() as i32;
                    if Self::eval_int_compare(size, cmp, target) {
                        Ok(card_ids.to_vec())
                    } else {
                        Ok(vec![])
                    }
                }
                AggregateFilter::Same { key } => {
                    let mut groups: std::collections::HashMap<String, Vec<usize>> =
                        std::collections::HashMap::new();
                    for &card_id in card_ids {
                        if let Some(card) = game_data.get_card(card_id) {
                            if let Some(value) = card.get(key) {
                                groups.entry(value.clone()).or_default().push(card_id);
                            }
                        }
                    }
                    let mut result = vec![];
                    for (_, mut ids) in groups {
                        if ids.len() > 1 {
                            result.append(&mut ids);
                        }
                    }
                    Ok(result)
                }
                AggregateFilter::Distinct { key } => {
                    let mut groups: std::collections::HashMap<String, Vec<usize>> =
                        std::collections::HashMap::new();
                    for &card_id in card_ids {
                        if let Some(card) = game_data.get_card(card_id) {
                            if let Some(value) = card.get(key) {
                                groups.entry(value.clone()).or_default().push(card_id);
                            }
                        }
                    }
                    let mut result = vec![];
                    for (_, mut ids) in groups {
                        if ids.len() == 1 {
                            result.append(&mut ids);
                        }
                    }
                    Ok(result)
                }
                AggregateFilter::Adjacent { key, precedence } => {
                    let prec = game_data
                        .precedences
                        .iter()
                        .find(|p| p.name == *precedence)
                        .ok_or(format!("Precedence {} not found", precedence))?;
                    let mut cards_with_values: Vec<(usize, i32, String)> = vec![];
                    for &card_id in card_ids {
                        if let Some(card) = game_data.get_card(card_id) {
                            if let Some(value) = card.get(key) {
                                if let Some(idx) = prec.values.iter().position(|v| v == value) {
                                    cards_with_values.push((card_id, idx as i32, value.clone()));
                                }
                            }
                        }
                    }
                    cards_with_values.sort_by_key(|&(_, idx, _)| idx);
                    let mut result = vec![];
                    for window in cards_with_values.windows(2) {
                        let (id1, idx1, _) = window[0];
                        let (id2, idx2, _) = window[1];
                        if idx2 - idx1 == 1 {
                            if !result.contains(&id1) {
                                result.push(id1);
                            }
                            if !result.contains(&id2) {
                                result.push(id2);
                            }
                        }
                    }
                    Ok(result)
                }
                AggregateFilter::Higher {
                    key,
                    value,
                    precedence,
                } => {
                    let prec = game_data
                        .precedences
                        .iter()
                        .find(|p| p.name == *precedence)
                        .ok_or(format!("Precedence {} not found", precedence))?;
                    let target_value = Self::eval_string(value, game_data)?;
                    let target_idx =
                        prec.values
                            .iter()
                            .position(|v| v == &target_value)
                            .ok_or(format!(
                                "Value {} not found in precedence {}",
                                target_value, precedence
                            ))?;
                    let mut result = vec![];
                    for &card_id in card_ids {
                        if let Some(card) = game_data.get_card(card_id) {
                            if let Some(card_value) = card.get(key) {
                                if let Some(idx) = prec.values.iter().position(|v| v == card_value)
                                {
                                    if idx as i32 > target_idx as i32 {
                                        result.push(card_id);
                                    }
                                }
                            }
                        }
                    }
                    Ok(result)
                }
                AggregateFilter::Lower {
                    key,
                    value,
                    precedence,
                } => {
                    let prec = game_data
                        .precedences
                        .iter()
                        .find(|p| p.name == *precedence)
                        .ok_or(format!("Precedence {} not found", precedence))?;
                    let target_value = Self::eval_string(value, game_data)?;
                    let target_idx =
                        prec.values
                            .iter()
                            .position(|v| v == &target_value)
                            .ok_or(format!(
                                "Value {} not found in precedence {}",
                                target_value, precedence
                            ))?;
                    let mut result = vec![];
                    for &card_id in card_ids {
                        if let Some(card) = game_data.get_card(card_id) {
                            if let Some(card_value) = card.get(key) {
                                if let Some(idx) = prec.values.iter().position(|v| v == card_value)
                                {
                                    if (idx as i32) < (target_idx as i32) {
                                        result.push(card_id);
                                    }
                                }
                            }
                        }
                    }
                    Ok(result)
                }
                AggregateFilter::KeyIsString { key, string } => {
                    let target = Self::eval_string(string, game_data)?;
                    let result: Vec<usize> = card_ids
                        .iter()
                        .filter(|&&card_id| {
                            game_data
                                .get_card(card_id)
                                .and_then(|c| c.get(key))
                                .map(|v| v == &target)
                                .unwrap_or(false)
                        })
                        .copied()
                        .collect();
                    Ok(result)
                }
                AggregateFilter::KeyIsNotString { key, string } => {
                    let target = Self::eval_string(string, game_data)?;
                    let result: Vec<usize> = card_ids
                        .iter()
                        .filter(|&&card_id| {
                            game_data
                                .get_card(card_id)
                                .and_then(|c| c.get(key))
                                .map(|v| v != &target)
                                .unwrap_or(true)
                        })
                        .copied()
                        .collect();
                    Ok(result)
                }
                AggregateFilter::Combo { combo } => {
                    let combo_filter = game_data
                        .combos
                        .iter()
                        .find(|c| c.name == *combo)
                        .map(|c| c.filter.clone())
                        .ok_or(format!("Combo {} not found", combo))?;
                    let result: Vec<usize> = card_ids
                        .iter()
                        .filter(|&&card_id| {
                            Self::card_matches_filter(card_id, &combo_filter, game_data)
                        })
                        .copied()
                        .collect();
                    Ok(result)
                }
                AggregateFilter::NotCombo { combo } => {
                    let combo_filter = game_data
                        .combos
                        .iter()
                        .find(|c| c.name == *combo)
                        .map(|c| c.filter.clone())
                        .ok_or(format!("Combo {} not found", combo))?;
                    let result: Vec<usize> = card_ids
                        .iter()
                        .filter(|&&card_id| {
                            !Self::card_matches_filter(card_id, &combo_filter, game_data)
                        })
                        .copied()
                        .collect();
                    Ok(result)
                }
            },
            FilterExpr::Binary {
                filter: f1,
                op,
                filter1: f2,
            } => {
                let result1 = Self::apply_filter(f1, card_ids, game_data)?;
                let result2 = Self::apply_filter(f2, card_ids, game_data)?;
                match op {
                    front_end::ast::FilterOp::And => {
                        let combined: Vec<usize> = result1
                            .into_iter()
                            .filter(|id| result2.contains(id))
                            .collect();
                        Ok(combined)
                    }
                    front_end::ast::FilterOp::Or => {
                        let mut combined = result1;
                        for id in result2 {
                            if !combined.contains(&id) {
                                combined.push(id);
                            }
                        }
                        Ok(combined)
                    }
                }
            }
        }
    }

    fn card_matches_filter(card_id: usize, filter: &FilterExpr, game_data: &GameData) -> bool {
        match filter {
            FilterExpr::Aggregate { aggregate } => match aggregate {
                AggregateFilter::Size { cmp, int_expr } => {
                    let mut cards = vec![card_id];
                    if let Ok(target) = Self::eval_int(int_expr, game_data) {
                        let size = cards.len() as i32;
                        return Self::eval_int_compare(size, cmp, target);
                    }
                    false
                }
                AggregateFilter::Same { key } => {
                    if let Some(card) = game_data.get_card(card_id) {
                        if let Some(value) = card.get(key) {
                            return game_data
                                .cards
                                .iter()
                                .filter(|c| c.get(key) == Some(value))
                                .any(|c| std::ptr::eq(c, card));
                        }
                    }
                    false
                }
                AggregateFilter::Distinct { key } => {
                    if let Some(card) = game_data.get_card(card_id) {
                        if let Some(value) = card.get(key) {
                            return !game_data
                                .cards
                                .iter()
                                .filter(|c| c.get(key) == Some(value))
                                .any(|c| !std::ptr::eq(c, card));
                        }
                    }
                    false
                }
                _ => false,
            },
            FilterExpr::Binary {
                filter: f1,
                op,
                filter1: f2,
            } => {
                let m1 = Self::card_matches_filter(card_id, f1, game_data);
                let m2 = Self::card_matches_filter(card_id, f2, game_data);
                match op {
                    front_end::ast::FilterOp::And => m1 && m2,
                    front_end::ast::FilterOp::Or => m1 || m2,
                }
            }
        }
    }

    fn infer_location_from_cards(
        card_ids: &[usize],
        game_data: &GameData,
    ) -> Result<usize, String> {
        for (loc_idx, loc) in game_data.locations.iter().enumerate() {
            if card_ids.iter().all(|&id| loc.cards.contains(&id)) {
                return Ok(loc_idx);
            }
        }
        if let Some(&first_card) = card_ids.first() {
            for (loc_idx, loc) in game_data.locations.iter().enumerate() {
                if loc.cards.contains(&first_card) {
                    return Ok(loc_idx);
                }
            }
        }
        Ok(0)
    }

    pub fn eval_card_position(expr: &CardPosition, game_data: &GameData) -> Result<usize, String> {
        match expr {
            CardPosition::Query { query } => match query {
                QueryCardPosition::At { location, int_expr } => {
                    let loc_idx = game_data
                        .locations
                        .iter()
                        .position(|l| l.name == *location)
                        .ok_or(format!("Location {} not found", location))?;
                    let idx = Self::eval_int(int_expr, game_data)? as usize;
                    let card_id = *game_data
                        .locations
                        .get(loc_idx)
                        .and_then(|l| l.cards.get(idx))
                        .ok_or(format!("No card at index {} in location {}", idx, location))?;
                    Ok(card_id)
                }
                QueryCardPosition::Top { location } => {
                    let loc_idx = game_data
                        .locations
                        .iter()
                        .position(|l| l.name == *location)
                        .ok_or(format!("Location {} not found", location))?;
                    let card_id = *game_data
                        .locations
                        .get(loc_idx)
                        .and_then(|l| l.cards.first())
                        .ok_or(format!("No card at top of location {}", location))?;
                    Ok(card_id)
                }
                QueryCardPosition::Bottom { location } => {
                    let loc_idx = game_data
                        .locations
                        .iter()
                        .position(|l| l.name == *location)
                        .ok_or(format!("Location {} not found", location))?;
                    let card_id = *game_data
                        .locations
                        .get(loc_idx)
                        .and_then(|l| l.cards.last())
                        .ok_or(format!("No card at bottom of location {}", location))?;
                    Ok(card_id)
                }
            },
            CardPosition::Aggregate { aggregate } => match aggregate {
                front_end::ast::AggregateCardPosition::ExtremaPointMap {
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
                    for &card_id in &card_ids {
                        if let Some(card) = game_data.get_card(card_id) {
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
                                        best_card_id = Some(card_id);
                                    }
                                }
                                Extrema::Max => {
                                    if best_value.is_none()
                                        || card_value > *best_value.as_ref().unwrap()
                                    {
                                        best_value = Some(card_value);
                                        best_card_id = Some(card_id);
                                    }
                                }
                            }
                        }
                    }
                    best_card_id.ok_or("No card found for ExtremaPointMap".to_string())
                }
                front_end::ast::AggregateCardPosition::ExtremaPrecedence {
                    extrema,
                    card_set,
                    precedence,
                } => {
                    let (_, card_ids) = Self::eval_cardset(card_set, game_data)?;
                    let prec = game_data
                        .precedences
                        .iter()
                        .find(|p| p.name == *precedence)
                        .ok_or(format!("Precedence {} not found", precedence))?;
                    let mut best_card_id = None;
                    let mut best_idx = None;
                    for &card_id in &card_ids {
                        if let Some(card) = game_data.get_card(card_id) {
                            if let Some(value) = card.get(&prec.key) {
                                if let Some(idx) = prec.values.iter().position(|v| v == value) {
                                    match extrema {
                                        Extrema::Min => {
                                            if best_idx.is_none()
                                                || idx < *best_idx.as_ref().unwrap()
                                            {
                                                best_idx = Some(idx);
                                                best_card_id = Some(card_id);
                                            }
                                        }
                                        Extrema::Max => {
                                            if best_idx.is_none()
                                                || idx > *best_idx.as_ref().unwrap()
                                            {
                                                best_idx = Some(idx);
                                                best_card_id = Some(card_id);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    best_card_id.ok_or("No card found for ExtremaPrecedence".to_string())
                }
            },
        }
    }

    pub fn check_attr_value_in_cardset(
        attr_value: &String,
        card_set: &Vec<usize>,
        game_data: &GameData,
    ) -> bool {
        // This function checks if any card in the card set has an attribute value equal to the string.
        // For example, if the string is "Hearts" and the card set contains the indices of some cards, this function checks if any of those cards has the suit "Hearts".
        // This is used for the StringInCardSet and StringNotInCardSet aggregate bools.

        for card_id in card_set {
            if let Some(card) = game_data.get_card(*card_id) {
                if card.values().any(|v| v == attr_value) {
                    return true;
                }
            }
        }

        false
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
                let start_satisfied = match &int_range.start {
                    (cmp, int_expr) => match Self::eval_int(int_expr, &GameData::new()) {
                        Ok(target) => Self::eval_int_compare(available as i32, cmp, target),
                        Err(_) => false,
                    },
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

    pub fn expand_types(types: &Types) -> Vec<Card> {
        let mut result = vec![Card::new()];
        for (attr, values) in &types.types {
            let mut new_result = vec![];
            for card in result.clone() {
                for value in values {
                    let mut new_card = card.clone();
                    new_card.insert(attr.clone(), value.clone());
                    new_result.push(new_card);
                }
            }
            result = new_result;
        }
        result
    }
}
