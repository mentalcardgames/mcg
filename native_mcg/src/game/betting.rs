//! Betting and action handling for Game.

use crate::game::Game;
use anyhow::{bail, Result};
use mcg_shared::{ActionEvent, ActionKind, PlayerAction};

/// Compute the normalized add amount for an open bet (when current_bet == 0).
/// Ensures the total bet is at least the big blind and not more than
/// the player's total available chips (round contribution + stack).
fn compute_open_bet_add(game: &Game, actor: usize, desired_total: u32) -> (u32, u32) {
    let bet_to = desired_total
        .max(game.bb)
        .min(game.players[actor].stack + game.round_bets[actor]);
    let add = bet_to
        .saturating_sub(game.round_bets[actor])
        .min(game.players[actor].stack);
    (add, bet_to)
}

/// Internal outcome when attempting a raise over a non-zero current bet.
#[derive(Debug, Clone, Copy)]
enum RaiseOutcome {
    Call,
    Raise { add: u32, by: u32 },
}

/// Decide how to resolve a raise attempt over a non-zero current bet.
fn decide_raise_outcome(
    game: &Game,
    actor: usize,
    raise_by: u32,
    prev_current_bet: u32,
) -> RaiseOutcome {
    let need = game.current_bet.saturating_sub(game.round_bets[actor]);
    let target_to = game.current_bet + raise_by;
    let required = target_to.saturating_sub(game.round_bets[actor]);
    let add = required.min(game.players[actor].stack);

    if add <= need {
        return RaiseOutcome::Call;
    }

    let required_add = prev_current_bet.saturating_sub(game.round_bets[actor]) + game.min_raise;
    if add < required_add {
        return RaiseOutcome::Call;
    }

    let new_to = game.round_bets[actor] + add;
    let by = new_to.saturating_sub(prev_current_bet);
    RaiseOutcome::Raise { add, by }
}

impl Game {
    /// Helper to execute a check or call.
    /// Handles stack updates, pot contribution, all-in detection, and logging.
    fn do_call(&mut self, actor: usize) {
        let need = self.current_bet.saturating_sub(self.round_bets[actor]);
        if need == 0 {
            self.log(ActionEvent::player(
                mcg_shared::PlayerId(actor),
                ActionKind::Check,
            ));
        } else {
            let pay = need.min(self.players[actor].stack);
            self.players[actor].stack -= pay;
            self.round_bets[actor] += pay;
            self.pot += pay;
            // distinct from "pay < need" check elsewhere: if pay consumes entire stack, they are all-in?
            // "pay < need" implies they didn't have enough to cover the bet.
            // If they had exactly enough, stack becomes 0, are they all-in?
            // Usually yes if stack is 0.
            if self.players[actor].stack == 0 {
                self.players[actor].all_in = true;
            }
            self.log(ActionEvent::player(
                mcg_shared::PlayerId(actor),
                ActionKind::Call(pay),
            ));
        }
    }

    fn execute_fold(&mut self, actor: usize) {
        self.players[actor].has_folded = true;
        self.log(ActionEvent::player(
            mcg_shared::PlayerId(actor),
            ActionKind::Fold,
        ));
    }

    fn execute_check_call(&mut self, actor: usize) {
        self.do_call(actor);
    }

    fn execute_bet(&mut self, actor: usize, amount: u32) {
        let (add, _bet_to) = compute_open_bet_add(self, actor, amount);
        self.players[actor].stack -= add;
        self.round_bets[actor] += add;
        self.pot += add;
        self.current_bet = self.round_bets[actor];
        self.min_raise = add;
        if self.players[actor].stack == 0 {
            self.players[actor].all_in = true;
        }
        self.log(ActionEvent::player(
            mcg_shared::PlayerId(actor),
            ActionKind::Bet(add),
        ));
    }

    fn execute_raise(&mut self, actor: usize, add: u32, by: u32) {
        self.players[actor].stack -= add;
        self.round_bets[actor] += add;
        self.pot += add;
        self.current_bet = self.round_bets[actor];
        self.min_raise = by;
        if self.players[actor].stack == 0 {
            self.players[actor].all_in = true;
        }
        self.log(ActionEvent::player(
            mcg_shared::PlayerId(actor),
            ActionKind::Raise {
                to: self.current_bet,
                by,
            },
        ));
    }

    pub fn apply_player_action(&mut self, actor: usize, action: PlayerAction) -> Result<()> {
        if actor != self.to_act {
            bail!("Not your turn");
        }
        if self.players[actor].has_folded {
            bail!("You have already folded");
        }
        if self.players[actor].all_in {
            bail!("You are all-in");
        }

        let prev_current_bet = self.current_bet;
        match action {
            PlayerAction::Fold => {
                self.execute_fold(actor);
            }
            PlayerAction::CheckCall => {
                self.execute_check_call(actor);
            }
            PlayerAction::Bet(x) => {
                if x == 0 {
                    self.execute_check_call(actor);
                } else if self.current_bet == 0 {
                    self.execute_bet(actor, x);
                } else {
                    match decide_raise_outcome(self, actor, x, prev_current_bet) {
                        RaiseOutcome::Call => {
                            self.execute_check_call(actor);
                        }
                        RaiseOutcome::Raise { add, by } => {
                            self.execute_raise(actor, add, by);
                        }
                    }
                }
            }
        }

        self.post_action_update(actor, prev_current_bet)
    }
}
