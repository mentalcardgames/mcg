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

use crate::game_data::GameData;
use front_end::ast::SetUpRule;
use front_end::ir::LoweredPayLoad;

//TODO: write this module.

pub fn execute(payload: LoweredPayLoad, _game_data: &mut GameData) {
    match payload {
        LoweredPayLoad::Action(_) => {
            // TODO: evaluate action and modify game state accordingly.
        }
        LoweredPayLoad::StageRoundCounter(_) => {
            // TODO: read stage id and increment stage round counter in game data for that stage.
        }
        LoweredPayLoad::EndStage(_) => {
            // TODO: update current stage in game data
        }
        _ => {}
    }
}

pub fn execute_setup_rule(payload: SetUpRule, game_data: &mut GameData) {
    match payload {
        SetUpRule::CreatePlayer { players } => {
            for player in players {
                game_data.add_player(player);
            }
            unimplemented!()
        }
        SetUpRule::CreateTeams { teams: _ } => {
            // TODO: create given teams in game data
        }
        SetUpRule::CreateTurnorder {
            player_collection: _,
        } => {
            // TODO: initalize turn order in game data
        }
        SetUpRule::CreateTurnorderRandom {
            player_collection: _,
        } => {
            // TODO: randomize order of player collection and initialze turn ordeer in game data
        }
        SetUpRule::CreateLocation {
            owner: _,
            locations: _,
        } => {
            // TODO: create locations in game data, use string names from locations argument only, ignore the Global part - this is handled differently in game data.
        }
        SetUpRule::CreateCardOnLocation {
            location: _,
            cards: _,
        } => {
            // TODO: create cards defined by adding them to the CardDB and then assigning their DB indices to the given location. Be careful here since cards: Vec<Types, Global> contains ast::Types, which should be unwrapped and expanded.
        }
        SetUpRule::CreateTokenOnLocation {
            location: _,
            int: _,
            token: _,
        } => {
            // TODO
        }
        SetUpRule::CreateCombo {
            combo: _,
            filter: _,
        } => {
            // TODO: add to game data
        }
        SetUpRule::CreateMemory {
            memory: _,
            owner: _,
        } => {
            // TODO: do not implement yet, Memory not implemented in current engine
        }
        SetUpRule::CreateMemoryWithMemoryType {
            memory: _,
            owner: _,
            memory_type: _,
        } => {
            // TODO: do not implement yet
        }
        SetUpRule::CreatePrecedence {
            precedence: _,
            kvs: _,
        } => {
            // TODO: create the precedence (just an ordered list of <attribute, value> pairs) in game data
        }
        SetUpRule::CreatePointMap {
            pointmap: _,
            kvis: _,
        } => {
            // TODO: create PointMap in game data using name pointmap and data kvis
        }
    }
}
