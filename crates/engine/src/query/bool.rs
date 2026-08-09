use super::Evaluator;
use crate::game_data::GameData;
use front_end::ast::{AggregateBool, BoolExpr, BoolOp, CompareBool, EndCondition, UnaryOp};

impl Evaluator {
    pub fn eval_bool(expr: &BoolExpr, game_data: &GameData) -> Result<bool, String> {
        match expr {
            BoolExpr::Binary {
                bool_expr,
                op,
                bool_expr1,
            } => {
                let left = Self::eval_bool(bool_expr, game_data)?;
                match op {
                    BoolOp::And => {
                        if !left {
                            return Ok(false);
                        }
                    }
                    BoolOp::Or => {
                        if left {
                            return Ok(true);
                        }
                    }
                }
                Self::eval_bool(bool_expr1, game_data)
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
                let player_indices = Self::resolve_players(players, game_data)?;
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
        stage_name: &str,
    ) -> Result<bool, String> {
        match end_condition {
            EndCondition::UntilEnd => Ok(false),
            EndCondition::UntilRep { reps } => {
                let current = game_data.get_stage_counter(stage_name.to_string());
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
                let current = game_data.get_stage_counter(stage_name.to_string());
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
}

#[cfg(test)]
#[path = "bool_tests.rs"]
mod tests;
