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

mod bool;
mod cardset;
mod int;
mod player;
mod string;

use crate::game_data::GameData;
use front_end::ast::{UseMemory, UseSingleMemory};

pub struct Evaluator;

impl Evaluator {
    /// Resolves a `UseSingleMemory` to a prefixed HashMap key.
    /// `WithOwner` uses the explicit owner; `Memory` (no owner) returns
    /// an error — the engine requires explicit memory ownership.
    pub fn resolve_memory_key(
        use_single: &UseSingleMemory,
        game_data: &GameData,
    ) -> Result<String, crate::error::EngineError> {
        match use_single {
            UseSingleMemory::WithOwner { memory, owner } => {
                let name = match owner.as_ref() {
                    front_end::ast::SingleOwner::Player { player } => {
                        Self::eval_player(player, game_data)?
                    }
                    front_end::ast::SingleOwner::Team { team } => Self::eval_team(team, game_data)?,
                    front_end::ast::SingleOwner::Table => "Table".to_string(),
                };
                Ok(format!("{}_{}", name, memory))
            }
            UseSingleMemory::Memory { memory } => {
                // NOTE(grammar-gap): memory references without an
                // explicit owner should not be reachable once the
                // grammar enforces `of <owner>` everywhere.
                Err(crate::error::EngineError::MemoryRequiresExplicitOwner {
                    key: memory.clone(),
                })
            }
        }
    }

    /// Resolves a `UseMemory` (multi-owner collection memory) to a
    /// prefixed HashMap key.
    pub fn resolve_collection_memory_key(
        use_mem: &UseMemory,
        game_data: &GameData,
    ) -> Result<String, crate::error::EngineError> {
        match use_mem {
            UseMemory::WithOwner { memory, owner } => {
                let name = Self::resolve_owner_to_name(owner.as_ref(), game_data)?;
                Ok(format!("{}_{}", name, memory))
            }
            UseMemory::Memory { memory } => {
                Err(crate::error::EngineError::MemoryRequiresExplicitOwner {
                    key: memory.clone(),
                })
            }
        }
    }
}
