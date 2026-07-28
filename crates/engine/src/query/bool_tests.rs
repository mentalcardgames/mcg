use super::*;
use crate::game_data::{Card, GameData, Location};
use front_end::ast::*;

fn true_expr() -> BoolExpr {
    BoolExpr::Aggregate {
        aggregate: AggregateBool::Compare {
            cmp_bool: CompareBool::Int {
                int: IntExpr::Literal { int: 1 },
                cmp: IntCompare::Eq,
                int1: IntExpr::Literal { int: 1 },
            },
        },
    }
}

fn false_expr() -> BoolExpr {
    BoolExpr::Aggregate {
        aggregate: AggregateBool::Compare {
            cmp_bool: CompareBool::Int {
                int: IntExpr::Literal { int: 1 },
                cmp: IntCompare::Neq,
                int1: IntExpr::Literal { int: 1 },
            },
        },
    }
}

fn div_by_zero_int() -> IntExpr {
    IntExpr::Binary {
        int: Box::new(IntExpr::Literal { int: 1 }),
        op: IntOp::Div,
        int1: Box::new(IntExpr::Literal { int: 0 }),
    }
}

fn div_by_zero_compare() -> CompareBool {
    CompareBool::Int {
        int: div_by_zero_int(),
        cmp: IntCompare::Eq,
        int1: IntExpr::Literal { int: 0 },
    }
}

fn div_by_zero_aggregate() -> AggregateBool {
    AggregateBool::Compare {
        cmp_bool: div_by_zero_compare(),
    }
}

fn div_by_zero_bool() -> BoolExpr {
    BoolExpr::Aggregate {
        aggregate: div_by_zero_aggregate(),
    }
}

fn gd_empty() -> GameData {
    let mut gd = GameData::new();
    gd.add_location(
        "Table".to_string(),
        Location {
            name: "EmptyLoc".to_string(),
            cards: vec![],
        },
    );
    gd
}

fn location_cardset(name: &str) -> CardSet {
    CardSet::Group {
        group: Group::Groupable {
            groupable: Groupable::Location {
                name: name.to_string(),
            },
        },
    }
}

fn ace_card() -> Card {
    let mut card = Card::new();
    card.insert("Rank".to_string(), "Ace".to_string());
    card
}

fn gd_with_ace_in_hand() -> GameData {
    let mut gd = GameData::new();
    gd.add_player("P1".to_string());
    let hand_loc = Location {
        name: "Hand".to_string(),
        cards: vec![],
    };
    let hand_idx = gd.add_location("P1".to_string(), hand_loc);
    let card = ace_card();
    let card_id = gd.add_card(hand_idx, card);
    gd.locations[hand_idx].cards.push(card_id);
    gd
}

fn gd_with_two_players() -> GameData {
    let mut gd = GameData::new();
    gd.add_player("P1".to_string());
    gd.add_player("P2".to_string());
    gd
}

fn int_compare_expr(left: i32, cmp: IntCompare, right: i32) -> BoolExpr {
    BoolExpr::Aggregate {
        aggregate: AggregateBool::Compare {
            cmp_bool: CompareBool::Int {
                int: IntExpr::Literal { int: left },
                cmp,
                int1: IntExpr::Literal { int: right },
            },
        },
    }
}

// --- B-1: AND short-circuits on false left ---
#[test]
fn eval_bool_binary_and_short_circuits_false_left() {
    let expr = BoolExpr::Binary {
        bool_expr: Box::new(false_expr()),
        op: BoolOp::And,
        bool_expr1: Box::new(div_by_zero_bool()),
    };
    assert_eq!(Evaluator::eval_bool(&expr, &gd_empty()), Ok(false));
}

// --- B-2: OR short-circuits on true left ---
#[test]
fn eval_bool_binary_or_short_circuits_true_left() {
    let expr = BoolExpr::Binary {
        bool_expr: Box::new(true_expr()),
        op: BoolOp::Or,
        bool_expr1: Box::new(div_by_zero_bool()),
    };
    assert_eq!(Evaluator::eval_bool(&expr, &gd_empty()), Ok(true));
}

// --- B-3: AND both true / false right ---
#[test]
fn eval_bool_binary_and_both_true() {
    let expr = BoolExpr::Binary {
        bool_expr: Box::new(true_expr()),
        op: BoolOp::And,
        bool_expr1: Box::new(true_expr()),
    };
    assert_eq!(Evaluator::eval_bool(&expr, &gd_empty()), Ok(true));
}

#[test]
fn eval_bool_binary_and_false_right() {
    let expr = BoolExpr::Binary {
        bool_expr: Box::new(true_expr()),
        op: BoolOp::And,
        bool_expr1: Box::new(false_expr()),
    };
    assert_eq!(Evaluator::eval_bool(&expr, &gd_empty()), Ok(false));
}

// --- B-4: Unary NOT ---
#[test]
fn eval_bool_unary_not_true() {
    let expr = BoolExpr::Unary {
        op: UnaryOp::Not,
        bool_expr: Box::new(true_expr()),
    };
    assert_eq!(Evaluator::eval_bool(&expr, &gd_empty()), Ok(false));
}

#[test]
fn eval_bool_unary_not_false() {
    let expr = BoolExpr::Unary {
        op: UnaryOp::Not,
        bool_expr: Box::new(false_expr()),
    };
    assert_eq!(Evaluator::eval_bool(&expr, &gd_empty()), Ok(true));
}

// --- B-5: AND with true left DOES evaluate right (div-by-zero) ---
#[test]
fn eval_bool_binary_forwards_div_by_zero() {
    let expr = BoolExpr::Binary {
        bool_expr: Box::new(true_expr()),
        op: BoolOp::And,
        bool_expr1: Box::new(div_by_zero_bool()),
    };
    assert_eq!(
        Evaluator::eval_bool(&expr, &gd_empty()),
        Err("Division by zero".to_string())
    );
}

// --- B-6: Aggregate Int comparisons ---
#[test]
fn eval_aggregate_compare_int_eq() {
    let expr = int_compare_expr(2, IntCompare::Eq, 3);
    assert_eq!(Evaluator::eval_bool(&expr, &gd_empty()), Ok(false));

    let expr2 = int_compare_expr(3, IntCompare::Eq, 3);
    assert_eq!(Evaluator::eval_bool(&expr2, &gd_empty()), Ok(true));
}

#[test]
fn eval_aggregate_compare_int_neq() {
    let expr = int_compare_expr(2, IntCompare::Neq, 3);
    assert_eq!(Evaluator::eval_bool(&expr, &gd_empty()), Ok(true));

    let expr2 = int_compare_expr(3, IntCompare::Neq, 3);
    assert_eq!(Evaluator::eval_bool(&expr2, &gd_empty()), Ok(false));
}

#[test]
fn eval_aggregate_compare_int_gt() {
    let expr = int_compare_expr(3, IntCompare::Gt, 2);
    assert_eq!(Evaluator::eval_bool(&expr, &gd_empty()), Ok(true));

    let expr2 = int_compare_expr(2, IntCompare::Gt, 3);
    assert_eq!(Evaluator::eval_bool(&expr2, &gd_empty()), Ok(false));
}

#[test]
fn eval_aggregate_compare_int_lt() {
    let expr = int_compare_expr(2, IntCompare::Lt, 3);
    assert_eq!(Evaluator::eval_bool(&expr, &gd_empty()), Ok(true));

    let expr2 = int_compare_expr(3, IntCompare::Lt, 2);
    assert_eq!(Evaluator::eval_bool(&expr2, &gd_empty()), Ok(false));
}

#[test]
fn eval_aggregate_compare_int_ge() {
    let expr = int_compare_expr(3, IntCompare::Ge, 2);
    assert_eq!(Evaluator::eval_bool(&expr, &gd_empty()), Ok(true));

    let expr2 = int_compare_expr(3, IntCompare::Ge, 3);
    assert_eq!(Evaluator::eval_bool(&expr2, &gd_empty()), Ok(true));

    let expr3 = int_compare_expr(2, IntCompare::Ge, 3);
    assert_eq!(Evaluator::eval_bool(&expr3, &gd_empty()), Ok(false));
}

#[test]
fn eval_aggregate_compare_int_le() {
    let expr = int_compare_expr(2, IntCompare::Le, 3);
    assert_eq!(Evaluator::eval_bool(&expr, &gd_empty()), Ok(true));

    let expr2 = int_compare_expr(3, IntCompare::Le, 3);
    assert_eq!(Evaluator::eval_bool(&expr2, &gd_empty()), Ok(true));

    let expr3 = int_compare_expr(3, IntCompare::Le, 2);
    assert_eq!(Evaluator::eval_bool(&expr3, &gd_empty()), Ok(false));
}

#[test]
fn eval_aggregate_compare_int_div_by_zero() {
    let expr = BoolExpr::Aggregate {
        aggregate: AggregateBool::Compare {
            cmp_bool: CompareBool::Int {
                int: IntExpr::Literal { int: 2 },
                cmp: IntCompare::Eq,
                int1: div_by_zero_int(),
            },
        },
    };
    assert_eq!(
        Evaluator::eval_bool(&expr, &gd_empty()),
        Err("Division by zero".to_string())
    );
}

// --- B-7: CardSet comparison ---
#[test]
fn eval_aggregate_compare_cardset_eq_and_neq() {
    let gd = gd_empty();

    let hand = location_cardset("EmptyLoc");
    let hand2 = location_cardset("EmptyLoc");

    let expr_eq = BoolExpr::Aggregate {
        aggregate: AggregateBool::Compare {
            cmp_bool: CompareBool::CardSet {
                card_set: hand.clone(),
                cmp: CardSetCompare::Eq,
                card_set1: hand2.clone(),
            },
        },
    };
    assert_eq!(Evaluator::eval_bool(&expr_eq, &gd), Ok(true));

    let expr_neq = BoolExpr::Aggregate {
        aggregate: AggregateBool::Compare {
            cmp_bool: CompareBool::CardSet {
                card_set: hand.clone(),
                cmp: CardSetCompare::Neq,
                card_set1: hand2.clone(),
            },
        },
    };
    assert_eq!(Evaluator::eval_bool(&expr_neq, &gd), Ok(false));
}

#[test]
fn eval_aggregate_compare_cardset_neq_different() {
    let mut gd = gd_empty();
    gd.add_location(
        "Table".to_string(),
        Location {
            name: "Other".to_string(),
            cards: vec![],
        },
    );

    let hand = location_cardset("EmptyLoc");
    let other = location_cardset("Other");

    let expr_neq = BoolExpr::Aggregate {
        aggregate: AggregateBool::Compare {
            cmp_bool: CompareBool::CardSet {
                card_set: hand,
                cmp: CardSetCompare::Neq,
                card_set1: other,
            },
        },
    };
    assert_eq!(Evaluator::eval_bool(&expr_neq, &gd), Ok(true));
}

// --- B-8: String comparison ---
#[test]
fn eval_aggregate_compare_string_eq_and_neq() {
    let gd = gd_empty();

    let hello = StringExpr::Literal {
        value: "hello".to_string(),
    };
    let hello2 = StringExpr::Literal {
        value: "hello".to_string(),
    };
    let world = StringExpr::Literal {
        value: "world".to_string(),
    };

    let expr_eq = BoolExpr::Aggregate {
        aggregate: AggregateBool::Compare {
            cmp_bool: CompareBool::String {
                string: hello.clone(),
                cmp: StringCompare::Eq,
                string1: hello2.clone(),
            },
        },
    };
    assert_eq!(Evaluator::eval_bool(&expr_eq, &gd), Ok(true));

    let expr_neq = BoolExpr::Aggregate {
        aggregate: AggregateBool::Compare {
            cmp_bool: CompareBool::String {
                string: hello.clone(),
                cmp: StringCompare::Neq,
                string1: world.clone(),
            },
        },
    };
    assert_eq!(Evaluator::eval_bool(&expr_neq, &gd), Ok(true));
}

#[test]
fn eval_aggregate_compare_string_missing_memory() {
    let gd = gd_empty();

    let expr = BoolExpr::Aggregate {
        aggregate: AggregateBool::Compare {
            cmp_bool: CompareBool::String {
                string: StringExpr::Memory {
                    memory: UseSingleMemory::WithOwner {
                        memory: "nonexistent".to_string(),
                        owner: Box::new(SingleOwner::Table),
                    },
                },
                cmp: StringCompare::Eq,
                string1: StringExpr::Literal {
                    value: "hello".to_string(),
                },
            },
        },
    };
    assert_eq!(
        Evaluator::eval_bool(&expr, &gd),
        Err("Memory Table_nonexistent not found".to_string())
    );
}

#[test]
fn eval_aggregate_compare_string_no_owner_error() {
    let gd = gd_empty();

    let expr = BoolExpr::Aggregate {
        aggregate: AggregateBool::Compare {
            cmp_bool: CompareBool::String {
                string: StringExpr::Memory {
                    memory: UseSingleMemory::Memory {
                        memory: "M".to_string(),
                    },
                },
                cmp: StringCompare::Eq,
                string1: StringExpr::Literal {
                    value: "hello".to_string(),
                },
            },
        },
    };
    assert_eq!(
        Evaluator::eval_bool(&expr, &gd),
        Err("memory access requires an explicit owner; use &M:M of <owner>".to_string())
    );
}

// --- B-9: Player comparison ---
#[test]
fn eval_aggregate_compare_player_eq_and_neq() {
    let mut gd = GameData::new();
    gd.add_player("Alice".to_string());

    let alice = PlayerExpr::Literal {
        name: "Alice".to_string(),
    };
    let alice2 = PlayerExpr::Literal {
        name: "Alice".to_string(),
    };
    let bob = PlayerExpr::Literal {
        name: "Bob".to_string(),
    };

    let expr_eq = BoolExpr::Aggregate {
        aggregate: AggregateBool::Compare {
            cmp_bool: CompareBool::Player {
                player: alice.clone(),
                cmp: PlayerCompare::Eq,
                player1: alice2.clone(),
            },
        },
    };
    assert_eq!(Evaluator::eval_bool(&expr_eq, &gd), Ok(true));

    let expr_neq = BoolExpr::Aggregate {
        aggregate: AggregateBool::Compare {
            cmp_bool: CompareBool::Player {
                player: alice.clone(),
                cmp: PlayerCompare::Neq,
                player1: bob.clone(),
            },
        },
    };
    assert_eq!(Evaluator::eval_bool(&expr_neq, &gd), Ok(true));
}

// --- B-10: Team comparison ---
#[test]
fn eval_aggregate_compare_team_eq_and_neq() {
    let gd = gd_empty();

    let t1 = TeamExpr::Literal {
        name: "T1".to_string(),
    };
    let t1_copy = TeamExpr::Literal {
        name: "T1".to_string(),
    };
    let t2 = TeamExpr::Literal {
        name: "T2".to_string(),
    };

    let expr_eq = BoolExpr::Aggregate {
        aggregate: AggregateBool::Compare {
            cmp_bool: CompareBool::Team {
                team: t1.clone(),
                cmp: TeamCompare::Eq,
                team1: t1_copy.clone(),
            },
        },
    };
    assert_eq!(Evaluator::eval_bool(&expr_eq, &gd), Ok(true));

    let expr_neq = BoolExpr::Aggregate {
        aggregate: AggregateBool::Compare {
            cmp_bool: CompareBool::Team {
                team: t1.clone(),
                cmp: TeamCompare::Neq,
                team1: t2.clone(),
            },
        },
    };
    assert_eq!(Evaluator::eval_bool(&expr_neq, &gd), Ok(true));
}

// --- B-11: StringInCardSet ---
#[test]
fn eval_aggregate_string_in_cardset_present_and_absent() {
    let gd = gd_with_ace_in_hand();

    let hand = location_cardset("Hand");

    let expr_ace = BoolExpr::Aggregate {
        aggregate: AggregateBool::StringInCardSet {
            string: StringExpr::Literal {
                value: "Ace".to_string(),
            },
            card_set: hand.clone(),
        },
    };
    assert_eq!(Evaluator::eval_bool(&expr_ace, &gd), Ok(true));

    let expr_king = BoolExpr::Aggregate {
        aggregate: AggregateBool::StringInCardSet {
            string: StringExpr::Literal {
                value: "King".to_string(),
            },
            card_set: hand.clone(),
        },
    };
    assert_eq!(Evaluator::eval_bool(&expr_king, &gd), Ok(false));
}

#[test]
fn eval_aggregate_string_in_cardset_empty() {
    let gd = gd_empty();

    let empty = location_cardset("EmptyLoc");

    let expr = BoolExpr::Aggregate {
        aggregate: AggregateBool::StringInCardSet {
            string: StringExpr::Literal {
                value: "Ace".to_string(),
            },
            card_set: empty,
        },
    };
    assert_eq!(Evaluator::eval_bool(&expr, &gd), Ok(false));
}

// --- B-12: StringNotInCardSet ---
#[test]
fn eval_aggregate_string_not_in_cardset() {
    let gd = gd_with_ace_in_hand();

    let hand = location_cardset("Hand");

    let expr = BoolExpr::Aggregate {
        aggregate: AggregateBool::StringNotInCardSet {
            string: StringExpr::Literal {
                value: "King".to_string(),
            },
            card_set: hand,
        },
    };
    assert_eq!(Evaluator::eval_bool(&expr, &gd), Ok(true));
}

// --- B-13: CardSetEmpty / CardSetNotEmpty ---
#[test]
fn eval_aggregate_cardset_empty_and_not_empty() {
    let gd = gd_empty();

    let empty = location_cardset("EmptyLoc");

    let expr_empty = BoolExpr::Aggregate {
        aggregate: AggregateBool::CardSetEmpty {
            card_set: empty.clone(),
        },
    };
    assert_eq!(Evaluator::eval_bool(&expr_empty, &gd), Ok(true));

    let expr_not = BoolExpr::Aggregate {
        aggregate: AggregateBool::CardSetNotEmpty {
            card_set: empty.clone(),
        },
    };
    assert_eq!(Evaluator::eval_bool(&expr_not, &gd), Ok(false));
}

// --- B-14: OutOfPlayer::Game ---
#[test]
fn eval_aggregate_out_of_player_game_true_and_false() {
    let mut gd = gd_with_two_players();
    // P1 is index 0, P2 is index 1
    gd.set_player_out(0);

    let p1 = PlayerExpr::Literal {
        name: "P1".to_string(),
    };
    let p2 = PlayerExpr::Literal {
        name: "P2".to_string(),
    };

    let expr_out = BoolExpr::Aggregate {
        aggregate: AggregateBool::OutOfPlayer {
            players: Players::Player { player: p1.clone() },
            out_of: OutOf::Game,
        },
    };
    assert_eq!(Evaluator::eval_bool(&expr_out, &gd), Ok(true));

    let expr_in = BoolExpr::Aggregate {
        aggregate: AggregateBool::OutOfPlayer {
            players: Players::Player { player: p2.clone() },
            out_of: OutOf::Game,
        },
    };
    assert_eq!(Evaluator::eval_bool(&expr_in, &gd), Ok(false));
}

// --- B-15: IntCompare table-driven ---
#[test]
fn eval_int_compare_table() {
    let cases = [
        (
            IntCompare::Eq,
            &[(2, 2, true), (2, 1, false), (1, 2, false)] as &[(i32, i32, bool)],
        ),
        (
            IntCompare::Neq,
            &[(2, 2, false), (2, 1, true), (1, 2, true)],
        ),
        (
            IntCompare::Gt,
            &[(2, 2, false), (2, 1, true), (1, 2, false)],
        ),
        (
            IntCompare::Lt,
            &[(2, 2, false), (2, 1, false), (1, 2, true)],
        ),
        (IntCompare::Ge, &[(2, 2, true), (2, 1, true), (1, 2, false)]),
        (IntCompare::Le, &[(2, 2, true), (2, 1, false), (1, 2, true)]),
    ];
    for (cmp, pairs) in &cases {
        for &(left, right, expected) in *pairs {
            assert_eq!(
                Evaluator::eval_int_compare(left, cmp, right),
                expected,
                "eval_int_compare({}, {:?}, {}) should be {}",
                left,
                cmp,
                right,
                expected
            );
        }
    }
}

// --- B-16: EndCondition::UntilEnd ---
#[test]
fn eval_end_condition_until_end_reached_and_not_reached() {
    let gd = gd_empty();
    let stage_name = "Play".to_string();

    assert_eq!(
        Evaluator::eval_end_condition(&EndCondition::UntilEnd, &gd, &stage_name),
        Ok(false)
    );
}

// --- B-17: EndCondition variants ---
#[test]
fn eval_end_condition_variants() {
    let stage_name = "Play".to_string();

    // UntilEnd: always false
    assert_eq!(
        Evaluator::eval_end_condition(&EndCondition::UntilEnd, &gd_empty(), &stage_name),
        Ok(false)
    );

    // UntilBool with true condition
    assert_eq!(
        Evaluator::eval_end_condition(
            &EndCondition::UntilBool {
                bool_expr: true_expr()
            },
            &gd_empty(),
            &stage_name,
        ),
        Ok(true)
    );

    // UntilBool with false condition
    assert_eq!(
        Evaluator::eval_end_condition(
            &EndCondition::UntilBool {
                bool_expr: false_expr()
            },
            &gd_empty(),
            &stage_name,
        ),
        Ok(false)
    );

    // UntilRep: counter not reached
    let mut gd = gd_empty();
    assert_eq!(
        Evaluator::eval_end_condition(
            &EndCondition::UntilRep {
                reps: Repititions {
                    times: IntExpr::Literal { int: 5 },
                },
            },
            &gd,
            &stage_name,
        ),
        Ok(false)
    );

    // UntilRep: counter reached
    gd.stage_counters.insert("Play".to_string(), 10);
    assert_eq!(
        Evaluator::eval_end_condition(
            &EndCondition::UntilRep {
                reps: Repititions {
                    times: IntExpr::Literal { int: 5 },
                },
            },
            &gd,
            &stage_name,
        ),
        Ok(true)
    );

    // UntilBoolRep: both not met
    let mut gd2 = gd_empty();
    assert_eq!(
        Evaluator::eval_end_condition(
            &EndCondition::UntilBoolRep {
                bool_expr: false_expr(),
                logic: BoolOp::And,
                reps: Repititions {
                    times: IntExpr::Literal { int: 5 },
                },
            },
            &gd2,
            &stage_name,
        ),
        Ok(false)
    );

    // UntilBoolRep: bool met, rep not (And)
    gd2.stage_counters.insert("Play".to_string(), 0);
    assert_eq!(
        Evaluator::eval_end_condition(
            &EndCondition::UntilBoolRep {
                bool_expr: true_expr(),
                logic: BoolOp::And,
                reps: Repititions {
                    times: IntExpr::Literal { int: 5 },
                },
            },
            &gd2,
            &stage_name,
        ),
        Ok(false)
    );

    // UntilBoolRep: bool met, rep not (Or)
    assert_eq!(
        Evaluator::eval_end_condition(
            &EndCondition::UntilBoolRep {
                bool_expr: true_expr(),
                logic: BoolOp::Or,
                reps: Repititions {
                    times: IntExpr::Literal { int: 5 },
                },
            },
            &gd2,
            &stage_name,
        ),
        Ok(true)
    );
}

// --- B-18: EndCondition with no current stage (error propagation) ---
#[test]
fn eval_end_condition_no_current_stage() {
    let gd = GameData::new();
    let stage_name = "Play".to_string();

    let result = Evaluator::eval_end_condition(
        &EndCondition::UntilBool {
            bool_expr: BoolExpr::Aggregate {
                aggregate: AggregateBool::Compare {
                    cmp_bool: CompareBool::Int {
                        int: IntExpr::Runtime {
                            runtime: RuntimeInt::CurrentStageRoundCounter,
                        },
                        cmp: IntCompare::Eq,
                        int1: IntExpr::Literal { int: 0 },
                    },
                },
            },
        },
        &gd,
        &stage_name,
    );
    assert_eq!(result, Err("No current stage".to_string()));
}
