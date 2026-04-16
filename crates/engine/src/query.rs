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

use crate::game_data::GameData;
use front_end::ast::{
    AggregateBool, BoolExpr, BoolOp, CardPosition, CardSet, CompareBool, EndCondition, IntExpr,
    PlayerExpr, StringExpr, TeamExpr, UnaryOp,
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
                let cards = Self::eval_cardset(card_set, game_data)?;
                Ok(Self::check_attr_value_in_cardset(&s, &cards, game_data))
            }
            AggregateBool::StringNotInCardSet { string, card_set } => {
                let s = Self::eval_string(string, game_data)?;
                let cards = Self::eval_cardset(card_set, game_data)?;
                Ok(!Self::check_attr_value_in_cardset(&s, &cards, game_data))
            }
            AggregateBool::CardSetEmpty { card_set } => {
                let cards = Self::eval_cardset(card_set, game_data)?;
                Ok(cards.is_empty())
            }
            AggregateBool::CardSetNotEmpty { card_set } => {
                let cards = Self::eval_cardset(card_set, game_data)?;
                Ok(!cards.is_empty())
            }
            AggregateBool::OutOfPlayer { .. } => {
                Err("Evaluator::eval_aggregate OutOfPlayer not yet implemented".to_string())
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
                let target = Self::eval_int(&reps.times, game_data)? as u32;
                Ok(current >= target)
            }
            EndCondition::UntilBool { bool_expr } => Self::eval_bool(bool_expr, game_data),
            EndCondition::UntilBoolRep {
                bool_expr,
                logic,
                reps,
            } => {
                let bool_result = Self::eval_bool(bool_expr, game_data)?;
                let current = game_data.get_stage_counter(stage_name.clone());
                let target = Self::eval_int(&reps.times, game_data)? as u32;
                let rep_result = current >= target;
                match logic {
                    BoolOp::And => Ok(bool_result && rep_result),
                    BoolOp::Or => Ok(bool_result || rep_result),
                }
            }
        }
    }

    pub fn eval_int(_expr: &IntExpr, _game_data: &GameData) -> Result<i32, String> {
        Err("Evaluator::eval_int not yet implemented".to_string())
    }

    pub fn eval_string(_expr: &StringExpr, _game_data: &GameData) -> Result<String, String> {
        Err("Evaluator::eval_string not yet implemented".to_string())
    }

    pub fn eval_player(_expr: &PlayerExpr, _game_data: &GameData) -> Result<String, String> {
        Err("Evaluator::eval_player not yet implemented".to_string())
    }

    pub fn eval_team(_expr: &TeamExpr, _game_data: &GameData) -> Result<String, String> {
        Err("Evaluator::eval_team not yet implemented".to_string())
    }

    pub fn eval_cardset(_expr: &CardSet, _game_data: &GameData) -> Result<Vec<usize>, String> {
        Err("Evaluator::eval_cardset not yet implemented".to_string())
    }

    pub fn eval_card_position(
        _expr: &CardPosition,
        _game_data: &GameData,
    ) -> Result<usize, String> {
        Err("Evaluator::eval_card_position not yet implemented".to_string())
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
}
