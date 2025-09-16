use crate::game::websocket::WebSocketConnection;
use crate::store::{AppState, ConnectionStatus};
use eframe::Frame;
use egui::{Color32, RichText};
use mcg_shared::{
    ActionEvent, ActionKind, BlindKind, Card, ClientMsg, GameAction, GameStatePublic, HandResult, PlayerAction,
    PlayerConfig, PlayerId, PlayerPublic, ServerMsg, Stage,
};

use super::{AppInterface, ScreenDef, ScreenMetadata, ScreenWidget};
use super::poker::ui_components::{action_kind_text, card_chip, card_text_and_color, category_text, name_of, card_text, stage_badge, stage_to_str};
use crate::qr_scanner::QrScannerPopup;

pub struct PokerOnlineScreen {
    conn: WebSocketConnection,
    scanner: QrScannerPopup,
    // Local UI-only edit buffers
    edit_server_address: String,
    // Player configuration
    players: Vec<PlayerConfig>,
    next_player_id: usize,
    new_player_name: String,
    // Preferred player this frontend would like to control
    preferred_player: PlayerId,
}

impl PokerOnlineScreen {
    pub fn new() -> Self {
        let app_state = AppState::new();
        let settings = app_state.settings;

        Self {
            conn: WebSocketConnection::new(),
            scanner: QrScannerPopup::new(),
            edit_server_address: settings.server_address,
            players: vec![
                PlayerConfig {
                    id: mcg_shared::PlayerId(0),
                    name: "You".to_string(),
                    is_bot: false,
                },
                PlayerConfig {
                    id: mcg_shared::PlayerId(1),
                    name: "Bot 1".to_string(),
                    is_bot: true,
                },
                PlayerConfig {
                    id: mcg_shared::PlayerId(2),
                    name: "Bot 2".to_string(),
                    is_bot: true,
                },
                PlayerConfig {
                    id: mcg_shared::PlayerId(3),
                    name: "Bot 3".to_string(),
                    is_bot: true,
                },
            ],
            next_player_id: 4,
            new_player_name: String::new(),
            preferred_player: PlayerId(0),
        }
    }

    fn draw_error_popup(&mut self, app_state: &mut AppState, ctx: &egui::Context) {
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

    fn render_header(&mut self, app_state: &mut AppState, ui: &mut egui::Ui, ctx: &egui::Context) {
        // Title row with current stage badge
        ui.horizontal(|ui| {
            ui.heading("Poker Online");
            ui.add_space(16.0);
            if let Some(s) = &app_state.game_state {
                ui.label(stage_badge(s.stage));
                ui.add_space(8.0);
            }
        });

        // Collapsible connection & session controls
        let default_open = app_state.game_state.is_none();
        egui::CollapsingHeader::new("Connection & session")
            .default_open(default_open)
            .show(ui, |ui| {
                self.render_connection_controls(app_state, ui, ctx);
            });

        // Collapsible player setup section
        egui::CollapsingHeader::new("Player Setup")
            .default_open(false)
            .show(ui, |ui| {
                self.render_player_setup(ui, ctx);
            });

        if let Some(err) = &app_state.last_error {
            ui.colored_label(Color32::RED, err);
        }
        if let Some(info) = &app_state.last_info {
            ui.label(RichText::new(info));
        }
        ui.separator();
    }

    fn render_connection_controls(
        &mut self,
        app_state: &mut AppState,
        ui: &mut egui::Ui,
        ctx: &egui::Context,
    ) {
        let narrow = ui.available_width() < 900.0;
        if narrow {
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    if ui.button("Connect").clicked() {
                        self.connect(app_state, ctx);
                    }
                    if ui.button("Disconnect").clicked() {
                        self.disconnect();
                    }
                });
                ui.horizontal(|ui| {
                    ui.label("Server:");
                    ui.text_edit_singleline(&mut self.edit_server_address)
                        .on_hover_text("Server address (IP:PORT)");
                    self.scanner
                        .button_and_popup(ui, ctx, &mut self.edit_server_address);
                });
            });
        } else {
            ui.horizontal(|ui| {
                ui.label("Server:");
                ui.text_edit_singleline(&mut self.edit_server_address)
                    .on_hover_text("Server address (IP:PORT)");
                self.scanner
                    .button_and_popup(ui, ctx, &mut self.edit_server_address);
                ui.add_space(12.0);
                if ui.button("Connect").clicked() {
                    self.connect(app_state, ctx);
                }
                if ui.button("Disconnect").clicked() {
                    self.disconnect();
                }
            });
        }
    }

    fn render_player_setup(&mut self, ui: &mut egui::Ui, _ctx: &egui::Context) {
        ui.heading("Player Setup");
        ui.add_space(8.0);

        self.render_players_table(ui);
        ui.add_space(8.0);

        self.render_add_player_section(ui);
        ui.add_space(16.0);

        self.render_start_game_button(ui);
        self.add_game_instructions(ui);
    }

    fn render_players_table(&mut self, ui: &mut egui::Ui) {
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

    //TODO: this does not need to be its own function
    fn render_players_table_header(&mut self, ui: &mut egui::Ui) {
        ui.label(RichText::new("ID").strong());
        ui.label(RichText::new("Name").strong());
        ui.label(RichText::new("Bot").strong());
        ui.label(RichText::new("Actions").strong());
        ui.end_row();
    }

    fn render_players_table_rows(&mut self, ui: &mut egui::Ui) {
        let mut to_remove = None;
        let mut to_rename = None;
        let mut bot_updates = Vec::new();

        let players_snapshot = self.players.clone();
        for (idx, player) in players_snapshot.iter().enumerate() {
            self.render_player_row(ui, &player, idx, &mut bot_updates, &mut to_remove, &mut to_rename);
        }

        self.apply_player_updates(bot_updates, to_remove, to_rename);
    }

    fn render_player_row(
        &mut self,
        ui: &mut egui::Ui,
        player: &PlayerConfig,
        idx: usize,
        bot_updates: &mut Vec<(usize, bool)>,
        to_remove: &mut Option<usize>,
        to_rename: &mut Option<usize>
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
        ui: &mut egui::Ui,
        player: &PlayerConfig,
        idx: usize,
        to_remove: &mut Option<usize>,
        to_rename: &mut Option<usize>
    ) {
        // Radio toggle to select which player the frontend would like to control.
        // Bot players cannot be selected.
        if player.is_bot {
            ui.label("Bot");
        } else {
            ui.radio_value(&mut self.preferred_player, player.id, "Play as")
                .on_hover_text("Select this player for this client");
        }

        if ui.button("✏").on_hover_text("Rename").clicked() {
            *to_rename = Some(idx);
        }
        if self.players.len() > 1
            && ui.button("🗑").on_hover_text("Remove").clicked()
        {
            *to_remove = Some(idx);
        }
    }

    fn apply_player_updates(
        &mut self,
        bot_updates: Vec<(usize, bool)>,
        to_remove: Option<usize>,
        to_rename: Option<usize>
    ) {
        // Apply bot status updates after iteration
        for (idx, is_bot) in bot_updates {
            if let Some(p) = self.players.get_mut(idx) {
                p.is_bot = is_bot;
            }
        }

        // Handle remove after iteration
        if let Some(idx) = to_remove {
            if idx < self.players.len() {
                self.players.remove(idx);
            }
        }

        // Handle rename after iteration
        if let Some(idx) = to_rename {
            if let Some(player) = self.players.get(idx) {
                // For now, just set the edit buffer to the current name
                // In a more complete implementation, you might want a popup
                self.new_player_name = player.name.clone();
            }
        }
    }

    fn render_add_player_section(&mut self, ui: &mut egui::Ui) {
        ui.group(|ui| {
            ui.label(RichText::new("Add New Player:").strong());
            ui.add_space(4.0);

            ui.horizontal(|ui| {
                ui.label("Name:");
                ui.text_edit_singleline(&mut self.new_player_name);

                if ui.button("Add Player").clicked() {
                    self.add_new_player();
                }
            });
        });
    }

    fn add_new_player(&mut self) {
        let player_name = if self.new_player_name.is_empty() {
            self.generate_random_name()
        } else {
            self.new_player_name.clone()
        };

        self.players.push(PlayerConfig {
            id: mcg_shared::PlayerId(self.next_player_id),
            name: player_name,
            is_bot: true, // New players start as bots by default
        });
        self.next_player_id += 1;
        self.new_player_name.clear();
    }

    fn render_start_game_button(&mut self, ui: &mut egui::Ui) {
        if ui.button("Start Game").clicked() {
            self.send(&ClientMsg::NewGame {
                players: self.players.clone(),
            });
        }
    }

    fn add_game_instructions(&self, ui: &mut egui::Ui) {
        ui.add_space(8.0);
        ui.label(
            "This will connect to the server and start a new game with the configured players.",
        );
    }

    fn render_showdown_banner(&self, ui: &mut egui::Ui, state: &GameStatePublic) {
        if state.stage == Stage::Showdown {
            let you_won = state.winner_ids.contains(&self.preferred_player);
            if you_won {
                ui.colored_label(Color32::LIGHT_GREEN, "You won!");
            } else {
                ui.colored_label(Color32::LIGHT_RED, "You lost.");
            }
            let winners: Vec<String> = state
                .players
                .iter()
                .filter(|p| state.winner_ids.contains(&p.id))
                .map(|p| p.name.clone())
                .collect();
            if !winners.is_empty() {
                ui.label(format!("Winners: {}", winners.join(", ")));
            }
            ui.add_space(8.0);
        }
    }

    fn render_table_panel(ui: &mut egui::Ui, state: &GameStatePublic, preferred_player: PlayerId) {
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("Pot:").strong());
                ui.monospace(format!(" {}", state.pot));
            });
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.label(RichText::new("Board:").strong());
                if state.community.is_empty() {
                    ui.label("—");
                }
                for &c in &state.community {
                    card_chip(ui, c);
                }
            });
            ui.add_space(8.0);
            ui.separator();
            ui.horizontal(|ui| {
                ui.label(RichText::new("Action log:").strong());
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .add(egui::Button::new("Copy to clipboard"))
                        .on_hover_text(
                            "Copy a structured summary of the current game and full action log",
                        )
                        .clicked()
                    {
                        let clip = format_game_for_clipboard(state, preferred_player);
                        ui.ctx().copy_text(clip);
                    }
                });
            });
            egui::ScrollArea::vertical()
                .id_salt("action_log_scroll")
                .max_height(200.0)
                .show(ui, |ui| {
                    for entry in state.action_log.iter().rev().take(100) {
                        log_entry_row(ui, entry, &state.players, preferred_player);
                    }
                });
        });
    }

    fn render_player_status_and_bet(
        &self,
        ui: &mut egui::Ui,
        state: &GameStatePublic,
        p: &PlayerPublic,
    ) {
        if p.id == state.to_act && state.stage != Stage::Showdown {
            ui.colored_label(Color32::from_rgb(255, 215, 0), "●");
        } else {
            ui.label("  ");
        }

        if p.id == self.preferred_player {
            ui.colored_label(Color32::LIGHT_GREEN, "You");
        }
        ui.label(RichText::new(&p.name).strong());

        if p.bet_this_round > 0 {
            ui.label(format!("Bet: {}", p.bet_this_round));
        }

        if p.has_folded {
            ui.colored_label(Color32::LIGHT_RED, "(folded)");
        }

        if state.stage == Stage::Showdown && state.winner_ids.contains(&p.id) {
            ui.colored_label(Color32::YELLOW, "WINNER");
        }

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.monospace(format!("stack: {}", p.stack));
        });
    }

    fn render_my_cards_and_actions(
        &self,
        ui: &mut egui::Ui,
        state: &GameStatePublic,
        p: &PlayerPublic,
    ) {
        ui.vertical(|ui| {
            if let Some(cards) = p.cards {
                ui.horizontal(|ui| {
                    ui.add_space(12.0);
                    card_chip(ui, cards[0]);
                    card_chip(ui, cards[1]);
                });
                ui.add_space(6.0);
                ui.separator();
                ui.add_space(6.0);
            } else {
                ui.add_space(6.0);
                ui.separator();
                ui.add_space(6.0);
            }

            if p.id == state.to_act && state.stage != Stage::Showdown {
                self.render_action_row(ui, p.id, true, false);
                ui.add_space(6.0);
                ui.separator();
            } else if p.id == self.preferred_player
                && (state.stage == Stage::Showdown || p.cards.is_none())
            {
                self.render_action_row(ui, p.id, false, true);
                ui.add_space(6.0);
                ui.separator();
            } else {
                ui.add_space(8.0);
            }
        });
    }

    fn render_player(&self, ui: &mut egui::Ui, state: &GameStatePublic, p: &PlayerPublic) {
        ui.horizontal(|ui| {
            self.render_player_status_and_bet(ui, state, p);
        });

        if p.id == self.preferred_player {
            self.render_my_cards_and_actions(ui, state, p);
        }
        ui.add_space(8.0);
    }

    fn render_players_panel(&self, ui: &mut egui::Ui, state: &GameStatePublic) {
        ui.group(|ui| {
            for p in state.players.iter() {
                self.render_player(ui, state, p);
            }
        });
    }

    fn render_panels(&self, ui: &mut egui::Ui, state: &GameStatePublic) {
        let narrow = ui.available_width() < 900.0;
        if narrow {
            self.render_players_panel(ui, state);
            ui.add_space(8.0);
            Self::render_table_panel(ui, state, self.preferred_player);
        } else {
            ui.columns(2, |cols| {
                Self::render_table_panel(&mut cols[0], state, self.preferred_player);
                self.render_players_panel(&mut cols[1], state);
            });
        }
    }

    // Centralized action buttons for a given player id.
    // Callers should check whether the player is active and whether the stage allows actions.
    fn render_action_buttons(
        &self,
        ui: &mut egui::Ui,
        player_id: mcg_shared::PlayerId,
        enabled: bool,
    ) {
        ui.horizontal(|ui| {
            let check_label = RichText::new("✔ Check / Call").size(18.0);
            if enabled {
                if ui
                    .add(egui::Button::new(check_label).min_size(egui::vec2(120.0, 40.0)))
                    .clicked()
                {
                    self.send(&ClientMsg::Action {
                        player_id,
                        action: PlayerAction::CheckCall,
                    });
                }
            } else {
                ui.add_enabled(
                    false,
                    egui::Button::new(check_label).min_size(egui::vec2(120.0, 40.0)),
                );
            }

            let bet_label = RichText::new("💰 Bet 10").size(18.0);
            if enabled {
                if ui
                    .add(egui::Button::new(bet_label).min_size(egui::vec2(120.0, 40.0)))
                    .on_hover_text("Place a small bet")
                    .clicked()
                {
                    self.send(&ClientMsg::Action {
                        player_id,
                        action: PlayerAction::Bet(10),
                    });
                }
            } else {
                ui.add_enabled(
                    false,
                    egui::Button::new(bet_label).min_size(egui::vec2(120.0, 40.0)),
                );
            }

            let fold_label = RichText::new("✂ Fold").size(18.0);
            if enabled {
                if ui
                    .add(egui::Button::new(fold_label).min_size(egui::vec2(120.0, 40.0)))
                    .clicked()
                {
                    self.send(&ClientMsg::Action {
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
    }
}

impl PokerOnlineScreen {
    fn render_action_row(
        &self,
        ui: &mut egui::Ui,
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
                        self.send(&ClientMsg::NextHand);
                    }
                });
                ui.add_space(6.0);
            }
            // Render the centralized action buttons (enabled or disabled)
            self.render_action_buttons(ui, player_id, enabled);
        });
    }

    fn connect(&mut self, app_state: &mut AppState, ctx: &egui::Context) {
        app_state.connection_status = ConnectionStatus::Connecting;
        app_state.last_error = None;
        app_state.last_info = Some(format!("Connecting to {}...", self.edit_server_address));
        app_state.settings.server_address = self.edit_server_address.clone();

        let ctx_for_callback = ctx.clone();
        let app_state_ptr = std::rc::Rc::new(std::cell::RefCell::new(app_state as *mut AppState));

        // Clone for each closure to avoid move issues
        let ctx_for_msg = ctx_for_callback.clone();
        let app_state_ptr_for_msg = app_state_ptr.clone();
        let ctx_for_error = ctx_for_callback.clone();
        let app_state_ptr_for_error = app_state_ptr.clone();
        let ctx_for_close = ctx_for_callback.clone();
        let app_state_ptr_for_close = app_state_ptr.clone();

        self.conn.connect(
            &self.edit_server_address,
            self.players.clone(),
            move |msg: ServerMsg| {
                // handle_msg - immediate processing
                unsafe {
                    if let Some(app_state) = app_state_ptr_for_msg.borrow_mut().as_mut() {
                        app_state.apply_server_msg(msg);
                        ctx_for_msg.request_repaint(); // immediate UI update
                    }
                }
            },
            move |error: String| {
                // handle error
                unsafe {
                    if let Some(app_state) = app_state_ptr_for_error.borrow_mut().as_mut() {
                        app_state.last_error = Some(error);
                        app_state.connection_status = ConnectionStatus::Disconnected;
                        ctx_for_error.request_repaint();
                    }
                }
            },
            move |reason: String| {
                // handle close
                unsafe {
                    if let Some(app_state) = app_state_ptr_for_close.borrow_mut().as_mut() {
                        app_state.connection_status = ConnectionStatus::Disconnected;
                        app_state.last_error = Some(reason);
                        ctx_for_close.request_repaint();
                    }
                }
            }
        );
    }

    fn disconnect(&mut self) {
        self.conn.close();
    }

    fn send(&self, msg: &ClientMsg) {
        self.conn.send_msg(msg);
    }

    // Generate a random name that doesn't conflict with existing player names
    fn generate_random_name(&self) -> String {
        let random_names = Self::get_random_name_pool();
        let existing_names: std::collections::HashSet<&str> = self.get_existing_names();

        // Try to find a name that's not already used
        if let Some(name) = self.find_unused_name(&random_names, &existing_names) {
            return name;
        }

        // If all names are used, append a number
        if let Some(name) = self.find_available_name_with_number(&random_names, &existing_names) {
            return name;
        }

        // Fallback: use a timestamp-based name
        self.generate_timestamp_name()
    }

    fn get_random_name_pool() -> [&'static str; 48] {
        [
            "Alice", "Bob", "Charlie", "Diana", "Eve", "Frank", "Grace", "Henry", "Iris", "Jack",
            "Kate", "Leo", "Mia", "Noah", "Olivia", "Peter", "Quinn", "Rose", "Sam", "Tina", "Uma",
            "Victor", "Wendy", "Xander", "Yara", "Zoe", "Alex", "Blake", "Casey", "Dylan", "Erin",
            "Finn", "Gabe", "Holly", "Ian", "Jade", "Kyle", "Luna", "Max", "Nora", "Owen", "Piper",
            "Ryan", "Sage", "Tyler", "Violet", "Wyatt", "Zara",
        ]
    }

    fn get_existing_names(&self) -> std::collections::HashSet<&str> {
        self.players.iter().map(|p| p.name.as_str()).collect()
    }

    fn find_unused_name(
        &self,
        random_names: &[&str],
        existing_names: &std::collections::HashSet<&str>
    ) -> Option<String> {
        for &name in random_names {
            if !existing_names.contains(name) {
                return Some(name.to_string());
            }
        }
        None
    }

    fn find_available_name_with_number(
        &self,
        random_names: &[&str],
        existing_names: &std::collections::HashSet<&str>
    ) -> Option<String> {
        for &base_name in random_names {
            for i in 2..100 {
                // Try numbers 2-99
                let candidate = format!("{} {}", base_name, i);
                if !existing_names.contains(candidate.as_str()) {
                    return Some(candidate);
                }
            }
        }
        None
    }

    fn generate_timestamp_name(&self) -> String {
        format!(
            "Player {}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
        )
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

impl ScreenWidget for PokerOnlineScreen {
    fn ui(&mut self, app_interface: &mut AppInterface, ui: &mut egui::Ui, _frame: &mut Frame) {
        let ctx = ui.ctx().clone();
        let app_state = &mut app_interface.app_state;

        self.draw_error_popup(app_state, &ctx);

        // Render header (it will read from the store snapshot internally)
        self.render_header(app_state, ui, &ctx);

        // Render main content from the latest snapshot
        if let Some(state) = &app_state.game_state {
            self.render_showdown_banner(ui, state);
            self.render_panels(ui, state);
        } else {
            ui.label("No state yet. Click Connect to start a session.");
        }
    }
}

fn format_game_for_clipboard(state: &GameStatePublic, you: PlayerId) -> String {
    let mut out = String::new();

    format_game_summary(&mut out, state, you);
    format_players_section(&mut out, state, you);
    format_board_section(&mut out, state);
    format_action_log(&mut out, state);

    out
}

fn format_game_summary(out: &mut String, state: &GameStatePublic, you: PlayerId) {
    out.push_str("Game summary\n");
    out.push_str(&format!("Stage: {}\n", stage_to_str(state.stage)));
    out.push_str(&format!("Pot: {}\n", state.pot));

    if let Some(p) = state.players.iter().find(|p| p.id == you) {
        if let Some(cards) = p.cards {
            out.push_str(&format!(
                "Your hole cards: {}, {}\n",
                card_text(cards[0]),
                card_text(cards[1])
            ));
        } else {
            out.push_str("Your hole cards: (hidden)\n");
        }
    }
    out.push('\n');
}

fn format_players_section(out: &mut String, state: &GameStatePublic, you: PlayerId) {
    out.push_str("Players\n");
    for p in state.players.iter() {
        format_player_entry(out, state, p, you);
    }
    out.push('\n');
}

fn format_player_entry(out: &mut String, state: &GameStatePublic, player: &PlayerPublic, you: PlayerId) {
    let you_str = if player.id == you { " (you)" } else { "" };
    let folded = if player.has_folded { ", folded:true" } else { "" };
    let to_act = if state.stage != Stage::Showdown && player.id == state.to_act {
        ", to_act:true"
    } else {
        ""
    };
    out.push_str(&format!(
        "- id:{}, name:{}, stack:{}{}{}{}\n",
        player.id, player.name, player.stack, you_str, folded, to_act
    ));

    if player.id == you {
        if let Some(cards) = player.cards {
            out.push_str(&format!(
                "  hole: {}, {}\n",
                card_text(cards[0]),
                card_text(cards[1])
            ));
        }
    }
}

fn format_board_section(out: &mut String, state: &GameStatePublic) {
    out.push_str("Board\n");
    if state.community.is_empty() {
        out.push_str("- (no community cards yet)\n");
    } else {
        let board = state
            .community
            .iter()
            .map(|&c| card_text(c))
            .collect::<Vec<_>>()
            .join(", ");
        out.push_str(&format!("- {}\n", board));
    }
    out.push('\n');
}

fn format_action_log(out: &mut String, state: &GameStatePublic) {
    out.push_str("Action log (chronological)\n");
    for entry in &state.action_log {
        format_action_log_entry(out, entry, state);
    }
}

fn format_action_log_entry(out: &mut String, entry: &ActionEvent, state: &GameStatePublic) {
    match entry {
        ActionEvent::PlayerAction { player_id, action } => {
            format_player_action_entry(out, *player_id, action, state);
        }
        ActionEvent::GameAction(game_action) => {
            format_game_action_entry(out, game_action, state);
        }
    }
}

fn format_player_action_entry(out: &mut String, player_id: PlayerId, action: &ActionKind, state: &GameStatePublic) {
    let who_name = name_of(&state.players, player_id);
    match action {
        ActionKind::Fold => out.push_str(&format!("- {} folds\n", who_name)),
        ActionKind::Check => out.push_str(&format!("- {} checks\n", who_name)),
        ActionKind::Call(n) => out.push_str(&format!("- {} calls {}\n", who_name, n)),
        ActionKind::Bet(n) => out.push_str(&format!("- {} bets {}\n", who_name, n)),
        ActionKind::Raise { to, by } => {
            out.push_str(&format!("- {} raises to {} (+{})\n", who_name, to, by))
        }
        ActionKind::PostBlind { kind, amount } => {
            format_blind_entry(out, &who_name, kind, amount);
        }
    }
}

fn format_blind_entry(out: &mut String, who_name: &str, kind: &BlindKind, amount: &u32) {
    match kind {
        BlindKind::SmallBlind => {
            out.push_str(&format!("- {} posts small blind {}\n", who_name, amount))
        }
        BlindKind::BigBlind => {
            out.push_str(&format!("- {} posts big blind {}\n", who_name, amount))
        }
    }
}

fn format_game_action_entry(out: &mut String, game_action: &GameAction, state: &GameStatePublic) {
    match game_action {
        GameAction::StageChanged(s) => {
            out.push_str(&format!("== Stage: {} ==\\n", stage_to_str(*s)));
        }
        GameAction::DealtHole { player_id } => {
            let who = name_of(&state.players, *player_id);
            out.push_str(&format!("- Dealt hole cards to {}\n", who));
        }
        GameAction::DealtCommunity { cards } => {
            format_community_cards_entry(out, cards);
        }
        GameAction::Showdown { hand_results } => {
            format_showdown_entry(out, hand_results, state);
        }
        GameAction::PotAwarded { winners, amount } => {
            format_pot_awarded_entry(out, winners, amount, state);
        }
    }
}

fn format_community_cards_entry(out: &mut String, cards: &[Card]) {
    match cards.len() {
        3 => out.push_str(&format!(
            "- Flop: {}, {}, {}\n",
            card_text(cards[0]),
            card_text(cards[1]),
            card_text(cards[2])
        )),
        4 => out.push_str(&format!("- Turn: {}\n", card_text(cards[3]))),
        5 => out.push_str(&format!("- River: {}\n", card_text(cards[4]))),
        _ => {
            let s = cards
                .iter()
                .map(|&c| card_text(c))
                .collect::<Vec<_>>()
                .join(", ");
            out.push_str(&format!("- Community: {}\n", s));
        }
    }
}

fn format_showdown_entry(out: &mut String, hand_results: &[HandResult], state: &GameStatePublic) {
    if hand_results.is_empty() {
        out.push_str("- Showdown\n");
    } else {
        for hr in hand_results {
            let who = name_of(&state.players, hr.player_id);
            let cat = category_text(&hr.rank.category);
            let best = hr
                .best_five
                .iter()
                .map(|&c| card_text(c))
                .collect::<Vec<_>>()
                .join(", ");
            out.push_str(&format!("- Showdown: {} -> {} [{}]\n", who, cat, best));
        }
    }
}

fn format_pot_awarded_entry(out: &mut String, winners: &[PlayerId], amount: &u32, state: &GameStatePublic) {
    let names = winners
        .iter()
        .map(|&id| name_of(&state.players, id))
        .collect::<Vec<_>>()
        .join(", ");
    out.push_str(&format!("- Pot {} awarded to {}\n", amount, names));
}

fn log_entry_row(
    ui: &mut egui::Ui,
    entry: &ActionEvent,
    players: &[PlayerPublic],
    you_id: mcg_shared::PlayerId,
) {
    match entry {
        ActionEvent::PlayerAction { player_id, action } => {
            let who_id = Some(*player_id);
            let who_name = name_of(players, *player_id);
            let (txt, color) = action_kind_text(action);
            let is_you = who_id == Some(you_id);
            let label = if is_you {
                RichText::new(format!("{} {}", who_name, txt))
                    .color(color)
                    .strong()
            } else {
                RichText::new(format!("{} {}", who_name, txt)).color(color)
            };
            ui.label(label);
        }
        ActionEvent::GameAction(GameAction::StageChanged(s)) => {
            ui.add_space(6.0);
            ui.separator();
            ui.horizontal(|ui| {
                ui.label(RichText::new("🕒").strong());
                ui.label(stage_badge(*s));
            });
            ui.separator();
            ui.add_space(6.0);
        }
        ActionEvent::GameAction(GameAction::DealtHole { player_id }) => {
            let who = name_of(players, *player_id);
            ui.colored_label(
                Color32::from_rgb(150, 150, 150),
                format!("♠ Dealt hole cards to {}", who),
            );
        }
        ActionEvent::GameAction(GameAction::DealtCommunity { cards }) => match cards.len() {
            3 => {
                ui.colored_label(
                    Color32::from_rgb(100, 200, 120),
                    format!(
                        "🃏 Flop: {} {} {}",
                        card_text(cards[0]),
                        card_text(cards[1]),
                        card_text(cards[2])
                    ),
                );
            }
            4 => {
                ui.colored_label(
                    Color32::from_rgb(230, 180, 80),
                    format!("🃏 Turn: {}", card_text(cards[3])),
                );
            }
            5 => {
                ui.colored_label(
                    Color32::from_rgb(220, 120, 120),
                    format!("🃏 River: {}", card_text(cards[4])),
                );
            }
            _ => {
                ui.colored_label(
                    Color32::from_rgb(120, 120, 120),
                    format!(
                        "🃏 Community: {}",
                        cards
                            .iter()
                            .map(|&c| card_text(c))
                            .collect::<Vec<_>>()
                            .join(" ")
                    ),
                );
            }
        },
        ActionEvent::GameAction(GameAction::Showdown { hand_results }) => {
            let mut parts = Vec::new();
            for hr in hand_results {
                let who = name_of(players, hr.player_id);
                let cat = category_text(&hr.rank.category);
                parts.push(format!("{}: {}", who, cat));
            }
            let text = if parts.is_empty() {
                "🏁 Showdown".to_string()
            } else {
                format!("🏁 Showdown — {}", parts.join(", "))
            };
            ui.colored_label(Color32::from_rgb(180, 100, 220), text);
        }
        ActionEvent::GameAction(GameAction::PotAwarded { winners, amount }) => {
            let names = winners
                .iter()
                .map(|&id| name_of(players, id))
                .collect::<Vec<_>>()
                .join(", ");
            ui.colored_label(
                Color32::from_rgb(240, 200, 80),
                format!("🏆 Pot {} awarded to {}", amount, names),
            );
        }
    }
}
