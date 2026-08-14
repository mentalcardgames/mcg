use super::*;
use front_end::ast::{ActionRule, GameRule, ScoringRule, SetUpRule};

// We test only the subtype string (the first element of the returned
// tuple). The detail string is the Debug format of the rule and is not
// contractually stable.

fn subtype(rule: &GameRule) -> String {
    rule_signature(rule).0
}

fn make_loc_cardset() -> front_end::ast::CardSet {
    use front_end::ast::{CardSet, Group, Groupable};
    CardSet::Group {
        group: Group::Groupable {
            groupable: Groupable::Location {
                name: "X".to_string(),
            },
        },
    }
}

// --- ActionRule subtypes ---

#[test]
fn rule_signature_flip_action() {
    let rule = GameRule::Action {
        action: ActionRule::FlipAction {
            card_set: make_loc_cardset(),
            status: front_end::ast::Status::FaceUp,
        },
    };
    assert_eq!(subtype(&rule), "Action:FlipAction");
}

#[test]
fn rule_signature_shuffle_action() {
    let rule = GameRule::Action {
        action: ActionRule::ShuffleAction {
            card_set: make_loc_cardset(),
        },
    };
    assert_eq!(subtype(&rule), "Action:ShuffleAction");
}

#[test]
fn rule_signature_out_action() {
    use front_end::ast::{OutOf, PlayerExpr, Players};
    let rule = GameRule::Action {
        action: ActionRule::OutAction {
            players: Players::Player {
                player: PlayerExpr::Literal {
                    name: "P1".to_string(),
                },
            },
            out_of: OutOf::Game,
        },
    };
    assert_eq!(subtype(&rule), "Action:OutAction");
}

#[test]
fn rule_signature_set_memory() {
    use front_end::ast::{IntExpr, MemoryType};
    let rule = GameRule::Action {
        action: ActionRule::SetMemory {
            memory: "m".to_string(),
            memory_type: MemoryType::Int {
                int: IntExpr::Literal { int: 0 },
            },
        },
    };
    assert_eq!(subtype(&rule), "Action:SetMemory");
}

#[test]
fn rule_signature_reset_memory() {
    let rule = GameRule::Action {
        action: ActionRule::ResetMemory {
            memory: "m".to_string(),
        },
    };
    assert_eq!(subtype(&rule), "Action:ResetMemory");
}

#[test]
fn rule_signature_cycle_action() {
    use front_end::ast::PlayerExpr;
    let rule = GameRule::Action {
        action: ActionRule::CycleAction {
            player: PlayerExpr::Literal {
                name: "P1".to_string(),
            },
        },
    };
    assert_eq!(subtype(&rule), "Action:CycleAction");
}

#[test]
fn rule_signature_bid_action() {
    use front_end::ast::{IntExpr, Quantity};
    let rule = GameRule::Action {
        action: ActionRule::BidAction {
            quantitiy: Quantity::Int {
                int: IntExpr::Literal { int: 1 },
            },
        },
    };
    assert_eq!(subtype(&rule), "Action:BidAction");
}

#[test]
fn rule_signature_bid_memory_action() {
    use front_end::ast::{IntExpr, Owner, Quantity};
    let rule = GameRule::Action {
        action: ActionRule::BidMemoryAction {
            memory: "m".to_string(),
            quantity: Quantity::Int {
                int: IntExpr::Literal { int: 1 },
            },
            owner: Owner::Table,
        },
    };
    assert_eq!(subtype(&rule), "Action:BidMemoryAction");
}

#[test]
fn rule_signature_end_action() {
    use front_end::ast::EndType;
    let rule = GameRule::Action {
        action: ActionRule::EndAction {
            end_type: EndType::Turn,
        },
    };
    assert_eq!(subtype(&rule), "Action:EndAction");
}

#[test]
fn rule_signature_demand_action() {
    use front_end::ast::{DemandType, IntExpr};
    let rule = GameRule::Action {
        action: ActionRule::DemandAction {
            demand_type: DemandType::Int {
                int: IntExpr::Literal { int: 0 },
            },
        },
    };
    assert_eq!(subtype(&rule), "Action:DemandAction");
}

#[test]
fn rule_signature_demand_memory_action() {
    use front_end::ast::{DemandType, IntExpr};
    let rule = GameRule::Action {
        action: ActionRule::DemandMemoryAction {
            demand_type: DemandType::Int {
                int: IntExpr::Literal { int: 0 },
            },
            memory: "m".to_string(),
        },
    };
    assert_eq!(subtype(&rule), "Action:DemandMemoryAction");
}

#[test]
fn rule_signature_move() {
    use front_end::ast::{ClassicMove, MoveCardSet, MoveType, Status};
    let rule = GameRule::Action {
        action: ActionRule::Move {
            move_type: MoveType::Classic {
                classic: ClassicMove::MoveCardSet {
                    move_cs: MoveCardSet::Move {
                        from: make_loc_cardset(),
                        status: Status::Private,
                        to: make_loc_cardset(),
                    },
                },
            },
        },
    };
    assert_eq!(subtype(&rule), "Action:Move");
}

// --- SetUpRule subtypes ---

#[test]
fn rule_signature_create_player() {
    let rule = GameRule::SetUp {
        setup: SetUpRule::CreatePlayer {
            players: vec!["P1".to_string()],
        },
    };
    assert_eq!(subtype(&rule), "SetUp:CreatePlayer");
}

#[test]
fn rule_signature_create_teams() {
    use front_end::ast::PlayerCollection;
    let rule = GameRule::SetUp {
        setup: SetUpRule::CreateTeams {
            teams: vec![(
                "T1".to_string(),
                PlayerCollection::Literal { players: vec![] },
            )],
        },
    };
    assert_eq!(subtype(&rule), "SetUp:CreateTeams");
}

#[test]
fn rule_signature_create_turnorder() {
    use front_end::ast::PlayerCollection;
    let rule = GameRule::SetUp {
        setup: SetUpRule::CreateTurnorder {
            player_collection: PlayerCollection::Literal { players: vec![] },
        },
    };
    assert_eq!(subtype(&rule), "SetUp:CreateTurnorder");
}

#[test]
fn rule_signature_create_turnorder_random() {
    use front_end::ast::PlayerCollection;
    let rule = GameRule::SetUp {
        setup: SetUpRule::CreateTurnorderRandom {
            player_collection: PlayerCollection::Literal { players: vec![] },
        },
    };
    assert_eq!(subtype(&rule), "SetUp:CreateTurnorderRandom");
}

#[test]
fn rule_signature_create_location() {
    use front_end::ast::Owner;
    let rule = GameRule::SetUp {
        setup: SetUpRule::CreateLocation {
            locations: vec!["Hand".to_string()],
            owner: Owner::Table,
        },
    };
    assert_eq!(subtype(&rule), "SetUp:CreateLocation");
}

#[test]
fn rule_signature_create_card_on_location() {
    let rule = GameRule::SetUp {
        setup: SetUpRule::CreateCardOnLocation {
            location: "Stock".to_string(),
            cards: vec![],
        },
    };
    assert_eq!(subtype(&rule), "SetUp:CreateCardOnLocation");
}

#[test]
fn rule_signature_create_token_on_location() {
    use front_end::ast::IntExpr;
    let rule = GameRule::SetUp {
        setup: SetUpRule::CreateTokenOnLocation {
            int: IntExpr::Literal { int: 0 },
            token: "t".to_string(),
            location: "Stock".to_string(),
        },
    };
    assert_eq!(subtype(&rule), "SetUp:CreateTokenOnLocation");
}

#[test]
fn rule_signature_create_combo() {
    use front_end::ast::{AggregateFilter, FilterExpr, IntCompare, IntExpr};
    let rule = GameRule::SetUp {
        setup: SetUpRule::CreateCombo {
            combo: "C1".to_string(),
            filter: FilterExpr::Aggregate {
                aggregate: AggregateFilter::Size {
                    cmp: IntCompare::Eq,
                    int_expr: Box::new(IntExpr::Literal { int: 1 }),
                },
            },
        },
    };
    assert_eq!(subtype(&rule), "SetUp:CreateCombo");
}

#[test]
fn rule_signature_create_memory() {
    use front_end::ast::Owner;
    let rule = GameRule::SetUp {
        setup: SetUpRule::CreateMemory {
            memory: "m".to_string(),
            owner: Owner::Table,
        },
    };
    assert_eq!(subtype(&rule), "SetUp:CreateMemory");
}

#[test]
fn rule_signature_create_memory_with_memory_type() {
    use front_end::ast::{IntExpr, MemoryType, Owner};
    let rule = GameRule::SetUp {
        setup: SetUpRule::CreateMemoryWithMemoryType {
            memory: "m".to_string(),
            owner: Owner::Table,
            memory_type: MemoryType::Int {
                int: IntExpr::Literal { int: 0 },
            },
        },
    };
    assert_eq!(subtype(&rule), "SetUp:CreateMemoryWithMemoryType");
}

#[test]
fn rule_signature_create_precedence() {
    let rule = GameRule::SetUp {
        setup: SetUpRule::CreatePrecedence {
            precedence: "p".to_string(),
            kvs: vec![("k".to_string(), "v".to_string())],
        },
    };
    assert_eq!(subtype(&rule), "SetUp:CreatePrecedence");
}

#[test]
fn rule_signature_create_point_map() {
    use front_end::ast::IntExpr;
    let rule = GameRule::SetUp {
        setup: SetUpRule::CreatePointMap {
            pointmap: "pm".to_string(),
            kvis: vec![(
                "k".to_string(),
                "v".to_string(),
                IntExpr::Literal { int: 0 },
            )],
        },
    };
    assert_eq!(subtype(&rule), "SetUp:CreatePointMap");
}

// --- ScoringRule subtypes ---

#[test]
fn rule_signature_scoring_score_rule() {
    use front_end::ast::{IntExpr, PlayerExpr, Players, ScoreRule};
    let rule = GameRule::Scoring {
        scoring: ScoringRule::ScoreRule {
            score_rule: ScoreRule::Score {
                int: IntExpr::Literal { int: 1 },
                players: Players::Player {
                    player: PlayerExpr::Literal {
                        name: "P1".to_string(),
                    },
                },
            },
        },
    };
    assert_eq!(subtype(&rule), "Scoring:Score");
}

#[test]
fn rule_signature_scoring_score_memory() {
    use front_end::ast::{IntExpr, PlayerExpr, Players, ScoreRule};
    let rule = GameRule::Scoring {
        scoring: ScoringRule::ScoreRule {
            score_rule: ScoreRule::ScoreMemory {
                int: IntExpr::Literal { int: 1 },
                memory: "m".to_string(),
                players: Players::Player {
                    player: PlayerExpr::Literal {
                        name: "P1".to_string(),
                    },
                },
            },
        },
    };
    assert_eq!(subtype(&rule), "Scoring:ScoreMemory");
}

#[test]
fn rule_signature_scoring_winner_rule() {
    use front_end::ast::{PlayerExpr, Players, WinnerRule};
    let rule = GameRule::Scoring {
        scoring: ScoringRule::WinnerRule {
            winner_rule: WinnerRule::Winner {
                players: Players::Player {
                    player: PlayerExpr::Literal {
                        name: "P1".to_string(),
                    },
                },
            },
        },
    };
    assert_eq!(subtype(&rule), "Scoring:Winner");
}

#[test]
fn rule_signature_scoring_winner_with() {
    use front_end::ast::{Extrema, WinnerRule, WinnerType};
    let rule = GameRule::Scoring {
        scoring: ScoringRule::WinnerRule {
            winner_rule: WinnerRule::WinnerWith {
                extrema: Extrema::Max,
                winner_type: WinnerType::Score,
            },
        },
    };
    assert_eq!(subtype(&rule), "Scoring:WinnerWith");
}
