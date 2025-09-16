use crate::game::screens::{ScreenDef, ScreenMetadata};
use crate::game::websocket::WebSocketConnection;
use crate::game::{AppInterface, ScreenWidget};
use crate::store::AppState;
use eframe::Frame;
use egui::{Context, RichText, Ui};
use mcg_shared::{PlayerAction, PlayerConfig};

use crate::qr_scanner::QrScannerPopup;

use super::connection_manager::ConnectionManager;
use super::player_manager::{render_player_setup, PlayerManager};

#[derive(Clone, Debug)]
struct BettingControls {
    /// Amount to raise BY (not total)
    raise_amount: u32,
    /// Amount to bet (when no current bet)
    bet_amount: u32,
    /// Minimum raise amount (calculated from game state)
    min_raise: u32,
    /// Maximum raise (player's stack)
    max_raise: u32,
    /// Whether to show the betting controls
    show_betting_controls: bool,
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

pub struct PokerOnlineScreen {
    conn: WebSocketConnection,
    scanner: QrScannerPopup,
    connection_manager: ConnectionManager,
    player_manager: PlayerManager,
    betting_controls: BettingControls,
}

impl PokerOnlineScreen {
    pub fn new() -> Self {
        let app_state = AppState::new();
        let settings = app_state.settings;

        Self {
            conn: WebSocketConnection::new(),
            scanner: QrScannerPopup::new(),
            connection_manager: ConnectionManager::new(settings.server_address.clone()),
            player_manager: PlayerManager::new(),
            betting_controls: BettingControls::default(),
        }
    }

    /// Update betting controls based on current game state and player
    fn update_betting_controls(
        &mut self,
        state: &mcg_shared::GameStatePublic,
        player_id: mcg_shared::PlayerId,
    ) {
        if let Some(player) = state.players.iter().find(|p| p.id == player_id) {
            self.betting_controls.max_raise = player.stack;
            self.betting_controls.min_raise = if state.current_bet == 0 {
                state.bb
            } else {
                state.min_raise
            };
            self.betting_controls.show_betting_controls = true;

            // Initialize betting amounts if not set
            if self.betting_controls.bet_amount == 0 {
                self.betting_controls.bet_amount = state.bb;
            }
            if self.betting_controls.raise_amount == 0 {
                self.betting_controls.raise_amount = self.betting_controls.min_raise;
            }
        } else {
            self.betting_controls.show_betting_controls = false;
        }
    }

    /// Calculate the call amount for a player
    fn calculate_call_amount(
        &self,
        state: &mcg_shared::GameStatePublic,
        player_id: mcg_shared::PlayerId,
    ) -> u32 {
        if let Some(player) = state.players.iter().find(|p| p.id == player_id) {
            state.current_bet.saturating_sub(player.bet_this_round)
        } else {
            0
        }
    }

    /// Render betting/raising controls for variable bet sizing
    fn render_betting_controls(
        &self,
        ui: &mut egui::Ui,
        state: &mcg_shared::GameStatePublic,
        player_id: mcg_shared::PlayerId,
        player: &mcg_shared::PlayerPublic,
    ) {
        ui.group(|ui| {
            ui.label("Betting:");

            let min_bet = if state.current_bet == 0 {
                state.bb
            } else {
                state.min_raise
            };
            let max_bet = player.stack;

            if state.current_bet == 0 {
                // No current bet - can open bet
                ui.horizontal(|ui| {
                    // Quick bet buttons
                    if ui.button("Min Bet").clicked() {
                        self.send(&mcg_shared::ClientMsg::Action {
                            player_id,
                            action: PlayerAction::Bet(state.bb),
                        });
                    }

                    let pot_third = (state.pot / 3).max(state.bb);
                    if ui.button("1/3 Pot").clicked() {
                        self.send(&mcg_shared::ClientMsg::Action {
                            player_id,
                            action: PlayerAction::Bet(pot_third.min(max_bet)),
                        });
                    }

                    let pot_half = (state.pot / 2).max(state.bb);
                    if ui.button("1/2 Pot").clicked() {
                        self.send(&mcg_shared::ClientMsg::Action {
                            player_id,
                            action: PlayerAction::Bet(pot_half.min(max_bet)),
                        });
                    }

                    if ui.button("Pot Size").clicked() {
                        self.send(&mcg_shared::ClientMsg::Action {
                            player_id,
                            action: PlayerAction::Bet(state.pot.max(state.bb).min(max_bet)),
                        });
                    }

                    if max_bet > 0 && ui.button("All-in").clicked() {
                        self.send(&mcg_shared::ClientMsg::Action {
                            player_id,
                            action: PlayerAction::Bet(max_bet),
                        });
                    }
                });
            } else {
                // Current bet exists - can raise
                ui.horizontal(|ui| {
                    // Quick raise buttons
                    if min_bet <= max_bet && ui.button("Min Raise").clicked() {
                        self.send(&mcg_shared::ClientMsg::Action {
                            player_id,
                            action: PlayerAction::Bet(min_bet),
                        });
                    }

                    let pot_third = (state.pot / 3).max(min_bet);
                    if pot_third <= max_bet && ui.button("Raise 1/3 Pot").clicked() {
                        self.send(&mcg_shared::ClientMsg::Action {
                            player_id,
                            action: PlayerAction::Bet(pot_third),
                        });
                    }

                    let pot_half = (state.pot / 2).max(min_bet);
                    if pot_half <= max_bet && ui.button("Raise 1/2 Pot").clicked() {
                        self.send(&mcg_shared::ClientMsg::Action {
                            player_id,
                            action: PlayerAction::Bet(pot_half),
                        });
                    }

                    let pot_size = state.pot.max(min_bet);
                    if pot_size <= max_bet && ui.button("Raise Pot").clicked() {
                        self.send(&mcg_shared::ClientMsg::Action {
                            player_id,
                            action: PlayerAction::Bet(pot_size),
                        });
                    }

                    if max_bet > 0 && ui.button("All-in").clicked() {
                        self.send(&mcg_shared::ClientMsg::Action {
                            player_id,
                            action: PlayerAction::Bet(max_bet),
                        });
                    }
                });
            }
        });
    }

    fn draw_error_popup(&mut self, app_state: &mut AppState, ctx: &Context) {
        if app_state.last_error.is_none() {
            return;
        }

        let mut open = true;
        let mut close_popup = false;
        egui::Window::new("Connection error")
            .collapsible(false)
            .resizable(false)
            .open(&mut open)
            .show(ctx, |ui| {
                if let Some(err) = &app_state.last_error {
                    ui.label(err);
                }
                ui.add_space(8.0);
                if ui.button("Close").clicked() {
                    close_popup = true;
                }
            });

        if !open || close_popup {
            app_state.last_error = None;
        }
    }

    fn connect(&mut self, app_state: &mut AppState, ctx: &Context) {
        self.connection_manager.connect(
            &mut self.conn,
            app_state,
            ctx,
            self.player_manager.get_players().clone(),
        );
    }

    fn disconnect(&mut self) {
        self.conn.close();
    }

    fn send(&self, msg: &mcg_shared::ClientMsg) {
        self.conn.send_msg(msg);
    }

    fn render_full_player_setup(&mut self, ui: &mut Ui, ctx: &Context) {
        render_player_setup(ui, ctx);

        // Add the player table and controls
        self.render_players_table(ui);
        ui.add_space(8.0);

        self.render_add_player_section(ui);
        ui.add_space(16.0);

        self.render_start_game_button(ui);
        self.add_game_instructions(ui);
    }

    fn render_players_table(&mut self, ui: &mut Ui) {
        ui.group(|ui| {
            ui.label(RichText::new("Players:").strong());
            ui.add_space(4.0);

            egui::Grid::new("players_grid")
                .num_columns(4)
                .spacing([8.0, 4.0])
                .striped(true)
                .show(ui, |ui| {
                    self.render_players_table_header(ui);
                    self.render_players_table_rows(ui);
                });
        });
    }

    fn render_players_table_header(&mut self, ui: &mut Ui) {
        ui.label(RichText::new("ID").strong());
        ui.label(RichText::new("Name").strong());
        ui.label(RichText::new("Bot").strong());
        ui.label(RichText::new("Actions").strong());
        ui.end_row();
    }

    fn render_players_table_rows(&mut self, ui: &mut Ui) {
        let mut to_remove = None;
        let mut to_rename = None;
        let mut bot_updates = Vec::new();

        let players_snapshot = self.player_manager.get_players().clone();
        for (idx, player) in players_snapshot.iter().enumerate() {
            self.render_player_row(
                ui,
                player,
                idx,
                &mut bot_updates,
                &mut to_remove,
                &mut to_rename,
            );
        }

        self.apply_player_updates(bot_updates, to_remove, to_rename);
    }

    fn render_player_row(
        &mut self,
        ui: &mut Ui,
        player: &PlayerConfig,
        idx: usize,
        bot_updates: &mut Vec<(usize, bool)>,
        to_remove: &mut Option<usize>,
        to_rename: &mut Option<usize>,
    ) {
        ui.label(format!("{}", player.id));
        ui.label(&player.name);

        let mut is_bot = player.is_bot;
        if ui.checkbox(&mut is_bot, "").changed() {
            bot_updates.push((idx, is_bot));
        }

        ui.horizontal(|ui| {
            self.render_player_actions(ui, player, idx, to_remove, to_rename);
        });
        ui.end_row();
    }

    fn render_player_actions(
        &mut self,
        ui: &mut Ui,
        player: &PlayerConfig,
        idx: usize,
        to_remove: &mut Option<usize>,
        to_rename: &mut Option<usize>,
    ) {
        // Radio toggle to select which player the frontend would like to control.
        // Bot players cannot be selected.
        if player.is_bot {
            ui.label("Bot");
        } else {
            ui.radio_value(
                self.player_manager.get_preferred_player_mut(),
                player.id,
                "Play as",
            )
            .on_hover_text("Select this player for this client");
        }

        if ui.button("✏").on_hover_text("Rename").clicked() {
            *to_rename = Some(idx);
        }
        if self.player_manager.get_players().len() > 1
            && ui.button("🗑").on_hover_text("Remove").clicked()
        {
            *to_remove = Some(idx);
        }
    }

    fn apply_player_updates(
        &mut self,
        bot_updates: Vec<(usize, bool)>,
        to_remove: Option<usize>,
        to_rename: Option<usize>,
    ) {
        // Apply bot status updates after iteration
        for (idx, is_bot) in bot_updates {
            if let Some(p) = self.player_manager.get_players_mut().get_mut(idx) {
                p.is_bot = is_bot;
            }
        }

        // Handle remove after iteration
        if let Some(idx) = to_remove {
            if idx < self.player_manager.get_players().len() {
                self.player_manager.get_players_mut().remove(idx);
            }
        }

        // Handle rename after iteration
        if let Some(idx) = to_rename {
            if let Some(player) = self.player_manager.get_players().get(idx) {
                // For now, just set the edit buffer to the current name
                // In a more complete implementation, you might want a popup
                *self.player_manager.get_new_player_name_mut() = player.name.clone();
            }
        }
    }

    fn render_add_player_section(&mut self, ui: &mut Ui) {
        ui.group(|ui| {
            ui.label(RichText::new("Add New Player:").strong());
            ui.add_space(4.0);

            ui.horizontal(|ui| {
                ui.label("Name:");
                ui.text_edit_singleline(self.player_manager.get_new_player_name_mut());

                if ui.button("Add Player").clicked() {
                    self.player_manager.add_new_player();
                }
            });
        });
    }

    fn render_start_game_button(&mut self, ui: &mut Ui) {
        if ui.button("Start Game").clicked() {
            self.send(&mcg_shared::ClientMsg::NewGame {
                players: self.player_manager.get_players().clone(),
            });
        }
    }

    fn add_game_instructions(&self, ui: &mut Ui) {
        ui.add_space(8.0);
        ui.label(
            "This will connect to the server and start a new game with the configured players.",
        );
    }
}

impl ScreenDef for PokerOnlineScreen {
    fn metadata() -> ScreenMetadata
    where
        Self: Sized,
    {
        ScreenMetadata {
            path: "/poker-online",
            display_name: "Poker Online",
            icon: "♠",
            description: "Play poker against bots or online",
            show_in_menu: true,
        }
    }

    fn create() -> Box<dyn ScreenWidget>
    where
        Self: Sized,
    {
        Box::new(Self::new())
    }
}

impl Default for PokerOnlineScreen {
    fn default() -> Self {
        Self::new()
    }
}

impl super::game_rendering::PokerScreenActions for PokerOnlineScreen {
    fn render_action_buttons(
        &self,
        ui: &mut egui::Ui,
        state: &mcg_shared::GameStatePublic,
        player_id: mcg_shared::PlayerId,
        enabled: bool,
    ) {
        // Update betting controls based on current state
        // Note: we can't modify self here due to &self, so we'll calculate values locally
        let call_amount = self.calculate_call_amount(state, player_id);
        let player = state.players.iter().find(|p| p.id == player_id);

        if let Some(player) = player {
            ui.vertical(|ui| {
                // First row: Check/Call and Fold buttons
                ui.horizontal(|ui| {
                    let check_call_label = if call_amount == 0 {
                        RichText::new("✔ Check").size(18.0)
                    } else {
                        RichText::new(format!("✔ Call {}", call_amount)).size(18.0)
                    };

                    if enabled {
                        if ui
                            .add(
                                egui::Button::new(check_call_label)
                                    .min_size(egui::vec2(120.0, 40.0)),
                            )
                            .clicked()
                        {
                            self.send(&mcg_shared::ClientMsg::Action {
                                player_id,
                                action: PlayerAction::CheckCall,
                            });
                        }
                    } else {
                        ui.add_enabled(
                            false,
                            egui::Button::new(check_call_label).min_size(egui::vec2(120.0, 40.0)),
                        );
                    }

                    let fold_label = RichText::new("✂ Fold").size(18.0);
                    if enabled {
                        if ui
                            .add(egui::Button::new(fold_label).min_size(egui::vec2(120.0, 40.0)))
                            .clicked()
                        {
                            self.send(&mcg_shared::ClientMsg::Action {
                                player_id,
                                action: PlayerAction::Fold,
                            });
                        }
                    } else {
                        ui.add_enabled(
                            false,
                            egui::Button::new(fold_label).min_size(egui::vec2(120.0, 40.0)),
                        );
                    }
                });

                if enabled {
                    ui.add_space(8.0);
                    // Second row: Betting/Raising controls
                    self.render_betting_controls(ui, state, player_id, player);
                }
            });
        }
    }

    fn render_action_row(
        &self,
        ui: &mut egui::Ui,
        state: &mcg_shared::GameStatePublic,
        player_id: mcg_shared::PlayerId,
        enabled: bool,
        show_next: bool,
    ) {
        ui.vertical(|ui| {
            if show_next {
                ui.horizontal(|ui| {
                    let next_label = RichText::new("▶ Next hand").size(16.0);
                    if ui
                        .add(egui::Button::new(next_label).min_size(egui::vec2(140.0, 40.0)))
                        .clicked()
                    {
                        self.send(&mcg_shared::ClientMsg::NextHand);
                    }
                });
                ui.add_space(6.0);
            }
            // Render the centralized action buttons (enabled or disabled)
            self.render_action_buttons(ui, state, player_id, enabled);
        });
    }

    fn send(&self, msg: &mcg_shared::ClientMsg) {
        self.conn.send_msg(msg);
    }
}

impl ScreenWidget for PokerOnlineScreen {
    fn ui(&mut self, app_interface: &mut AppInterface, ui: &mut egui::Ui, _frame: &mut Frame) {
        let ctx = ui.ctx().clone();
        let app_state = &mut app_interface.app_state;

        self.draw_error_popup(app_state, &ctx);

        // Check for button clicks
        let mut connect_clicked = false;
        let mut disconnect_clicked = false;

        // Render header
        self.render_header_with_controls(
            app_state,
            ui,
            &ctx,
            &mut connect_clicked,
            &mut disconnect_clicked,
        );

        // Handle button clicks
        if connect_clicked {
            self.connect(app_state, &ctx);
        }
        if disconnect_clicked {
            self.disconnect();
        }

        // Render main content from the latest snapshot
        if let Some(state) = &app_state.game_state {
            super::game_rendering::render_showdown_banner(
                ui,
                state,
                self.player_manager.get_preferred_player(),
            );
            super::game_rendering::render_panels(
                ui,
                state,
                self.player_manager.get_preferred_player(),
                self,
            );
        } else {
            ui.label("No state yet. Click Connect to start a session.");
        }
    }
}

impl PokerOnlineScreen {
    fn render_header_with_controls(
        &mut self,
        app_state: &mut AppState,
        ui: &mut Ui,
        ctx: &Context,
        connect_clicked: &mut bool,
        disconnect_clicked: &mut bool,
    ) {
        // Title row with current stage badge
        ui.horizontal(|ui| {
            ui.heading("Poker Online");
            ui.add_space(16.0);
            if let Some(s) = &app_state.game_state {
                ui.label(super::ui_components::stage_badge(s.stage));
                ui.add_space(8.0);
            }
        });

        // Collapsible connection & session controls
        let default_open = app_state.game_state.is_none();
        egui::CollapsingHeader::new("Connection & session")
            .default_open(default_open)
            .show(ui, |ui| {
                self.connection_manager.render_connection_controls(
                    app_state,
                    ui,
                    ctx,
                    connect_clicked,
                    disconnect_clicked,
                );
            });

        // Collapsible player setup section
        egui::CollapsingHeader::new("Player Setup")
            .default_open(false)
            .show(ui, |ui| {
                self.render_full_player_setup(ui, ctx);
            });

        if let Some(err) = &app_state.last_error {
            ui.colored_label(egui::Color32::RED, err);
        }
        if let Some(info) = &app_state.last_info {
            ui.label(RichText::new(info));
        }
        ui.separator();
    }
}
