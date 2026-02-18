//! Betting controls and interface for poker games.

use crate::game::websocket::WebSocketConnection;
use egui::{RichText, Ui};
use mcg_shared::{ClientMsg, GameStatePublic, PlayerAction, PlayerId, PlayerPublic};

/// Manages betting controls state for the poker interface
#[derive(Clone, Debug)]
pub struct BettingControls {
    /// Amount to raise BY (not total)
    pub raise_amount: u32,
    /// Amount to bet (when no current bet)
    pub bet_amount: u32,
    /// Minimum raise amount (calculated from game state)
    pub min_raise: u32,
    /// Maximum raise (player's stack)
    pub max_raise: u32,
    /// Whether to show the betting controls
    pub show_betting_controls: bool,
}

impl Default for BettingControls {
    fn default() -> Self {
        Self {
            raise_amount: 0,
            bet_amount: 0,
            min_raise: 0,
            max_raise: 0,
            show_betting_controls: false,
        }
    }
}

impl BettingControls {
    /// Update betting controls based on current game state and player
    pub fn update_from_game_state(&mut self, state: &GameStatePublic, player_id: PlayerId) {
        if let Some(player) = state.players.iter().find(|p| p.id == player_id) {
            self.max_raise = player.stack;
            self.min_raise = if state.current_bet == 0 {
                state.bb
            } else {
                state.min_raise
            };
            self.show_betting_controls = true;

            // Initialize betting amounts if not set
            if self.bet_amount == 0 {
                self.bet_amount = state.bb;
            }
            if self.raise_amount == 0 {
                self.raise_amount = self.min_raise;
            }
        } else {
            self.show_betting_controls = false;
        }
    }

    /// Calculate the call amount for a player
    pub fn calculate_call_amount(state: &GameStatePublic, player_id: PlayerId) -> u32 {
        if let Some(player) = state.players.iter().find(|p| p.id == player_id) {
            state.current_bet.saturating_sub(player.bet_this_round)
        } else {
            0
        }
    }

    /// Render betting/raising controls with slider and preset buttons
    pub fn render_betting_controls(
        &mut self,
        ui: &mut Ui,
        state: &GameStatePublic,
        player_id: PlayerId,
        player: &PlayerPublic,
        conn: &WebSocketConnection,
    ) {
        ui.group(|ui| {
            ui.label(RichText::new("Betting Options:").strong());
            ui.add_space(4.0);

            let min_bet = if state.current_bet == 0 {
                state.bb
            } else {
                state.min_raise
            };
            let max_bet = player.stack;

            if state.current_bet == 0 {
                // No current bet - can open bet
                self.render_opening_bet_controls(ui, state, player_id, min_bet, max_bet, conn);
            } else {
                // Current bet exists - can raise
                self.render_raise_controls(ui, state, player_id, min_bet, max_bet, conn);
            }
        });
    }

    fn render_opening_bet_controls(
        &mut self,
        ui: &mut Ui,
        state: &GameStatePublic,
        player_id: PlayerId,
        min_bet: u32,
        max_bet: u32,
        conn: &WebSocketConnection,
    ) {
        ui.label("Open betting:");

        // Slider for custom bet amount
        ui.horizontal(|ui| {
            ui.label("Bet:");
            let mut bet_amount = self.bet_amount as f32;
            if ui
                .add(
                    egui::Slider::new(&mut bet_amount, min_bet as f32..=max_bet as f32)
                        .suffix(" chips")
                        .smart_aim(false),
                )
                .changed()
            {
                self.bet_amount = bet_amount as u32;
            }

            if ui.button("Bet").clicked() {
                conn.send_msg(&ClientMsg::Action {
                    player_id,
                    action: PlayerAction::Bet(self.bet_amount),
                });
            }
        });

        ui.add_space(4.0);
        ui.label("Quick bets:");

        // Quick bet buttons
        ui.horizontal(|ui| {
            if ui.button("Min Bet").clicked() {
                conn.send_msg(&ClientMsg::Action {
                    player_id,
                    action: PlayerAction::Bet(state.bb),
                });
            }

            let pot_third = (state.pot / 3).max(state.bb);
            if ui.button("1/3 Pot").clicked() {
                conn.send_msg(&ClientMsg::Action {
                    player_id,
                    action: PlayerAction::Bet(pot_third.min(max_bet)),
                });
            }

            let pot_half = (state.pot / 2).max(state.bb);
            if ui.button("1/2 Pot").clicked() {
                conn.send_msg(&ClientMsg::Action {
                    player_id,
                    action: PlayerAction::Bet(pot_half.min(max_bet)),
                });
            }
        });

        ui.horizontal(|ui| {
            if ui.button("Pot Size").clicked() {
                conn.send_msg(&ClientMsg::Action {
                    player_id,
                    action: PlayerAction::Bet(state.pot.max(state.bb).min(max_bet)),
                });
            }

            if max_bet > 0 && ui.button("All-in").clicked() {
                conn.send_msg(&ClientMsg::Action {
                    player_id,
                    action: PlayerAction::Bet(max_bet),
                });
            }
        });
    }

    fn render_raise_controls(
        &mut self,
        ui: &mut Ui,
        state: &GameStatePublic,
        player_id: PlayerId,
        min_bet: u32,
        max_bet: u32,
        conn: &WebSocketConnection,
    ) {
        ui.label("Raise betting:");

        // Slider for custom raise amount
        ui.horizontal(|ui| {
            ui.label("Raise:");
            let mut raise_amount = self.raise_amount as f32;
            if ui
                .add(
                    egui::Slider::new(&mut raise_amount, min_bet as f32..=max_bet as f32)
                        .suffix(" chips")
                        .smart_aim(false),
                )
                .changed()
            {
                self.raise_amount = raise_amount as u32;
            }

            if ui.button("Raise").clicked() {
                conn.send_msg(&ClientMsg::Action {
                    player_id,
                    action: PlayerAction::Bet(self.raise_amount),
                });
            }
        });

        ui.add_space(4.0);
        ui.label("Quick raises:");

        // Quick raise buttons
        ui.horizontal(|ui| {
            if min_bet <= max_bet && ui.button("Min Raise").clicked() {
                conn.send_msg(&ClientMsg::Action {
                    player_id,
                    action: PlayerAction::Bet(min_bet),
                });
            }

            let pot_third = (state.pot / 3).max(min_bet);
            if pot_third <= max_bet && ui.button("Raise 1/3 Pot").clicked() {
                conn.send_msg(&ClientMsg::Action {
                    player_id,
                    action: PlayerAction::Bet(pot_third),
                });
            }

            let pot_half = (state.pot / 2).max(min_bet);
            if pot_half <= max_bet && ui.button("Raise 1/2 Pot").clicked() {
                conn.send_msg(&ClientMsg::Action {
                    player_id,
                    action: PlayerAction::Bet(pot_half),
                });
            }
        });

        ui.horizontal(|ui| {
            let pot_size = state.pot.max(min_bet);
            if pot_size <= max_bet && ui.button("Raise Pot").clicked() {
                conn.send_msg(&ClientMsg::Action {
                    player_id,
                    action: PlayerAction::Bet(pot_size),
                });
            }

            if max_bet > 0 && ui.button("All-in").clicked() {
                conn.send_msg(&ClientMsg::Action {
                    player_id,
                    action: PlayerAction::Bet(max_bet),
                });
            }
        });
    }
}
