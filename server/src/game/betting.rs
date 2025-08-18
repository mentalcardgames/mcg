//! Betting and action handling for Game.

use anyhow::{Result, bail};
use mcg_shared::{ActionEvent, ActionKind, PlayerAction};
use crate::game::Game;

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
    Call { pay: u32 },
    Raise { add: u32, by: u32 },
}

/// Decide how to resolve a raise attempt over a non-zero current bet.
fn decide_raise_outcome(game: &Game, actor: usize, raise_by: u32, prev_current_bet: u32) -> RaiseOutcome {
    let need = game.current_bet.saturating_sub(game.round_bets[actor]);
    let target_to = game.current_bet + raise_by;
    let required = target_to.saturating_sub(game.round_bets[actor]);
    let add = required.min(game.players[actor].stack);

    if add <= need {
        let pay = need.min(game.players[actor].stack);
        return RaiseOutcome::Call { pay };
    }

    let required_add = prev_current_bet.saturating_sub(game.round_bets[actor]) + game.min_raise;
    if add < required_add {
        let pay = need.min(game.players[actor].stack);
        return RaiseOutcome::Call { pay };
    }

    let new_to = game.round_bets[actor] + add;
    let by = new_to.saturating_sub(prev_current_bet);
    RaiseOutcome::Raise { add, by }
}

impl Game {
    /// Apply a player action, enforcing betting rules and advancing the game flow.
    pub fn apply_player_action(
        &mut self,
        actor: usize,
        action: PlayerAction,
    ) -> Result<()> {
        if actor != self.to_act {
            bail!("Not your turn");
        }
        if self.players[actor].has_folded {
            bail!("You have already folded");
        }
        if self.players[actor].all_in {
            bail!("You are all-in");
        }

        println!(
            "[ACTION] {}: {:?} (stage: {:?})",
            self.players[actor].name, action, self.stage
        );
        self.recent_actions.push(ActionEvent {
            player_id: actor,
            action: action.clone(),
        });
        // cap recent_actions / action_log
        super::utils::cap_logs(self);

        let prev_current_bet = self.current_bet;
        match action {
            PlayerAction::Fold => {
                self.players[actor].has_folded = true;
                self.log(mcg_shared::LogEntry {
                    player_id: Some(actor),
                    event: mcg_shared::LogEvent::Action(ActionKind::Fold),
                });
            }
            PlayerAction::CheckCall => {
                let need = self.current_bet.saturating_sub(self.round_bets[actor]);
                if need == 0 {
                    self.log(mcg_shared::LogEntry {
                        player_id: Some(actor),
                        event: mcg_shared::LogEvent::Action(ActionKind::Check),
                    });
                } else {
                    let pay = need.min(self.players[actor].stack);
                    self.players[actor].stack -= pay;
                    self.round_bets[actor] += pay;
                    self.pot += pay;
                    if pay < need {
                        self.players[actor].all_in = true;
                    }
                    self.log(mcg_shared::LogEntry {
                        player_id: Some(actor),
                        event: mcg_shared::LogEvent::Action(ActionKind::Call(pay)),
                    });
                }
            }
            PlayerAction::Bet(x) => {
                if self.current_bet == 0 {
                    let (add, _bet_to) = compute_open_bet_add(self, actor, x);
                    self.players[actor].stack -= add;
                    self.round_bets[actor] += add;
                    self.pot += add;
                    self.current_bet = self.round_bets[actor];
                    self.min_raise = add;
                    if self.players[actor].stack == 0 {
                        self.players[actor].all_in = true;
                    }
                    self.log(mcg_shared::LogEntry {
                        player_id: Some(actor),
                        event: mcg_shared::LogEvent::Action(ActionKind::Bet(add)),
                    });
                } else {
                    match decide_raise_outcome(self, actor, x, prev_current_bet) {
                        RaiseOutcome::Call { pay } => {
                            let need = self.current_bet.saturating_sub(self.round_bets[actor]);
                            let pay = pay.min(need).min(self.players[actor].stack);
                            self.players[actor].stack -= pay;
                            self.round_bets[actor] += pay;
                            self.pot += pay;
                            if pay < need {
                                self.players[actor].all_in = true;
                            }
                            self.log(mcg_shared::LogEntry {
                                player_id: Some(actor),
                                event: mcg_shared::LogEvent::Action(ActionKind::Call(pay)),
                            });
                        }
                        RaiseOutcome::Raise { add, by } => {
                            self.players[actor].stack -= add;
                            self.round_bets[actor] += add;
                            self.pot += add;
                            let new_to = self.round_bets[actor];
                            self.current_bet = new_to;
                            self.min_raise = by;
                            if self.players[actor].stack == 0 {
                                self.players[actor].all_in = true;
                            }
                            self.log(mcg_shared::LogEntry {
                                player_id: Some(actor),
                                event: mcg_shared::LogEvent::Action(ActionKind::Raise {
                                    to: self.current_bet,
                                    by,
                                }),
                            });
                        }
                    }
                }
            }
        }

        // If bet/raise increased the current bet, rebuild the pending_to_act order
        if self.current_bet > prev_current_bet {
            let n = self.players.len();
            self.pending_to_act.clear();
            for i in 1..=n {
                let idx = (actor + i) % n;
                if !self.players[idx].has_folded
                    && !self.players[idx].all_in
                    && self.round_bets[idx] < self.current_bet
                {
                    self.pending_to_act.push(idx);
                }
            }
            if let Some(&nxt) = self.pending_to_act.first() {
                self.to_act = nxt;
            }
        }

        // Remove actor from pending list if matched requirement, folded, or all-in
        self.remove_from_pending(actor);

        // If only one player remains, end the hand
            if self.active_players().len() <= 1 {
                self.stage = mcg_shared::Stage::Showdown;
                // delegate to showdown module
                crate::game::showdown::finish_showdown(self);
                return Ok(());
            }

        // If betting round complete, advance stage
            if self.is_betting_round_complete() {
                self.advance_stage()?;
                if self.stage == mcg_shared::Stage::Showdown {
                    crate::game::showdown::finish_showdown(self);
                    return Ok(());
                }
                self.init_round_for_stage();
            } else if let Some(&nxt) = self.pending_to_act.first() {
                self.to_act = nxt;
            }
        Ok(())
    }
}
