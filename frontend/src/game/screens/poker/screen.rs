use crate::game::screens::{ScreenDef, ScreenMetadata};
use crate::game::websocket::WebSocketConnection;
use crate::game::{AppInterface, ScreenWidget};
use crate::store::AppState;
use eframe::Frame;
use egui::{Context, RichText, Ui};
use mcg_shared::{PlayerAction, PlayerConfig};

use crate::qr_scanner::QrScannerPopup;

use super::betting_controls::BettingControls;
use super::connection_manager::ConnectionManager;
use super::player_manager::{render_player_setup, PlayerManager};

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
        let mut apply_rename = false;
        let mut cancel_rename = false;

        let players_snapshot = self.player_manager.get_players().clone();
        for (idx, player) in players_snapshot.iter().enumerate() {
            self.render_player_row(
                ui,
                player,
                idx,
                &mut bot_updates,
                &mut to_remove,
                &mut to_rename,
                &mut apply_rename,
                &mut cancel_rename,
            );
        }

        self.apply_player_updates(bot_updates, to_remove, to_rename, apply_rename, cancel_rename);
    }

    fn render_player_row(
        &mut self,
        ui: &mut Ui,
        player: &PlayerConfig,
        idx: usize,
        bot_updates: &mut Vec<(usize, bool)>,
        to_remove: &mut Option<usize>,
        to_rename: &mut Option<usize>,
        apply_rename: &mut bool,
        cancel_rename: &mut bool,
    ) {
        ui.label(format!("{}", player.id));
        
        // Check if this player is being renamed
        if self.player_manager.is_renaming(player.id) {
            // Show text edit field for renaming
            let response = ui.text_edit_singleline(self.player_manager.get_rename_buffer_mut());
            
            // Auto-focus the text field when rename starts
            response.request_focus();
            
            // Check for Enter key to confirm rename
            if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                *apply_rename = true;
            } else if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                *cancel_rename = true;
            }
        } else {
            ui.label(&player.name);
        }

        let mut is_bot = player.is_bot;
        if ui.checkbox(&mut is_bot, "").changed() {
            bot_updates.push((idx, is_bot));
        }

        ui.horizontal(|ui| {
            self.render_player_actions(ui, player, idx, to_remove, to_rename, apply_rename, cancel_rename);
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
        apply_rename: &mut bool,
        cancel_rename: &mut bool,
    ) {
        // If this player is being renamed, show Save/Cancel buttons
        if self.player_manager.is_renaming(player.id) {
            if ui.button("Save").clicked() {
                *apply_rename = true;
            }
            if ui.button("Cancel").clicked() {
                *cancel_rename = true;
            }
        } else {
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
    }

    fn apply_player_updates(
        &mut self,
        bot_updates: Vec<(usize, bool)>,
        to_remove: Option<usize>,
        to_rename: Option<usize>,
        apply_rename: bool,
        cancel_rename: bool,
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

        // Handle rename mode toggle
        if let Some(idx) = to_rename {
            if let Some(player) = self.player_manager.get_players().get(idx) {
                self.player_manager.start_renaming(player.id);
            }
        }

        // Apply or cancel rename
        if apply_rename {
            self.player_manager.apply_rename();
        } else if cancel_rename {
            self.player_manager.cancel_rename();
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
        &mut self,
        ui: &mut egui::Ui,
        state: &mcg_shared::GameStatePublic,
        player_id: mcg_shared::PlayerId,
        enabled: bool,
    ) {
        let call_amount = BettingControls::calculate_call_amount(state, player_id);
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
                    // Second row: Betting/Raising controls with slider
                    self.betting_controls
                        .update_from_game_state(state, player_id);

                    self.betting_controls
                        .render_betting_controls(ui, state, player_id, player, &self.conn);
                }
            });
        }
    }

    fn render_action_row(
        &mut self,
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

        // Process any queued WebSocket messages first
        self.connection_manager.process_queued_messages(app_state);

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
