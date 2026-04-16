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
└── Trigger                   ← no data

Each of the leaves of this payload tree should be accounted for in the execute_edge function, which takes a Payload and modifies the game state accordingly.
*/

use crate::game_data::GameData;
use front_end::ir::LoweredPayLoad;

pub fn execute(payload: LoweredPayLoad, _game_data: &mut GameData) {
    match payload {
        LoweredPayLoad::Condition { .. } => {}
        LoweredPayLoad::EndCondition { .. } => {}
        LoweredPayLoad::Action(_) => {}
        LoweredPayLoad::StageRoundCounter(_) => {}
        LoweredPayLoad::EndStage(_) => {}
        LoweredPayLoad::Choice => {}
        LoweredPayLoad::Optional => {}
        LoweredPayLoad::Trigger => {}
    }
}
