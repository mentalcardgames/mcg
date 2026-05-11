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
use front_end::ir::LoweredPayLoad;
use front_end::ast::SetUpRule;

//TODO: write this module.

pub fn execute(payload: LoweredPayLoad, _game_data: &mut GameData) {
    match payload {
        LoweredPayLoad::Action(_) => {
            // TODO: evaluate action and modify game state accordingly.
        }
        LoweredPayLoad::StageRoundCounter(_) => {
            // TODO: read id and increment counter in game data for that stage.
        }
        LoweredPayLoad::EndStage(_) => {
            // TODO: update current stage.
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
            // TODO
        }
        SetUpRule::CreateTurnorder { player_collection: _ } => {
            // TODO
        }
        SetUpRule::CreateTurnorderRandom { player_collection: _ } => {
            // TODO
        }
        SetUpRule::CreateLocation { owner: _, locations: _ } => {
            // TODO
        }
        SetUpRule::CreateCardOnLocation { location: _, cards: _ } => {
            // TODO
        }
        SetUpRule::CreateTokenOnLocation { location: _, int: _, token: _ } => {
            // TODO
        }
        SetUpRule::CreateCombo { combo: _, filter: _ } => {
            // TODO
        }
        SetUpRule::CreateMemory { memory: _, owner: _ } => {
            // TODO
        }
        SetUpRule::CreateMemoryWithMemoryType { memory: _, owner: _, memory_type: _ } => {
            // TODO
        }
        SetUpRule::CreatePrecedence { precedence: _, kvs: _ } => {
            // TODO
        }
        SetUpRule::CreatePointMap { pointmap: _, kvis: _ } => {
            // TODO
        }
    }
}
