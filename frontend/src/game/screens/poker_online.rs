use crate::effects::ConnectionEffect;
use crate::store::{bootstrap_state, SharedState};
use eframe::Frame;
use egui::{Color32, RichText};
use mcg_shared::{
    ActionEvent, ActionKind, BlindKind, ClientMsg, GameAction, GameStatePublic, PlayerAction,
    PlayerPublic, PlayerConfig, Stage,
};

use super::{AppInterface, ScreenDef, ScreenMetadata, ScreenWidget};
use crate::qr_scanner::QrScannerPopup;

pub struct PokerOnlineScreen {
    state: SharedState,
    conn_eff: ConnectionEffect,
    show_error_popup: bool,
    scanner: QrScannerPopup,
    // Local UI-only edit buffers (kept here so the immediate-mode UI can mutate them)
    edit_server_address: String,
    // Player configuration
    players: Vec<PlayerConfig>,
    next_player_id: usize,
    new_player_name: String,
    // Preferred player this frontend would like to control (None = not requested)
    preferred_player: Option<usize>,
}
impl PokerOnlineScreen {
    pub fn new() -> Self {
        // bootstrap shared state and effects
        let state = bootstrap_state();
        let snapshot = state.borrow().clone();
        let settings = snapshot.settings;
        let conn_eff = ConnectionEffect::new(state.clone());

        Self {
            state,
            conn_eff,
            show_error_popup: false,
            scanner: QrScannerPopup::new(),
            edit_server_address: settings.server_address,
            // Player configuration
            players: vec![
                PlayerConfig {
                    id: 0,
                    name: "You".to_string(),
                    is_bot: false,
                },
                PlayerConfig {
                    id: 1,
                    name: "Bot 1".to_string(),
                    is_bot: true,
                },
            ],
            next_player_id: 2,
            new_player_name: String::new(),
            preferred_player: None,
        }
    }

    fn draw_error_popup(&mut self, ctx: &egui::Context) {
        if !self.show_error_popup {
            return;
        }
        // read last_error from shared state snapshot
        let snapshot = self.state.borrow().clone();
        let mut open = true;
        egui::Window::new("Connection error")
            .collapsible(false)
            .resizable(false)
            .open(&mut open)
            .show(ctx, |ui| {
                if let Some(err) = &snapshot.last_error {
                    ui.label(err);
                } else {
                    ui.label(format!(
                        "Failed to connect to server at {}. Is it running?",
                        snapshot.settings.server_address
                    ));
                }
                ui.add_space(8.0);
                if ui.button("Close").clicked() {
                    self.show_error_popup = false;
                }
            });
        if !open {
            self.show_error_popup = false;
        }
    }

    // Incoming messages are handled by ConnectionEffect which polls the ConnectionService
    // and pushes ServerMsg events into the shared store. We no longer drain messages here.

    fn render_header(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        // Read a snapshot from the shared state for rendering
        let snapshot = self.state.borrow().clone();

        // Title row with current stage badge
        ui.horizontal(|ui| {
            ui.heading("Poker Online");
            ui.add_space(16.0);
            if let Some(s) = &snapshot.game_state {
                ui.label(stage_badge(s.stage));
                ui.add_space(8.0);
            }
        });

        // Collapsible connection & session controls
        let default_open = snapshot.game_state.is_none();
        egui::CollapsingHeader::new("Connection & session")
            .default_open(default_open)
            .show(ui, |ui| {
                Self::render_connection_controls(self, ui, ctx);
            });

        // Collapsible player setup section
        egui::CollapsingHeader::new("Player Setup")
            .default_open(false)
            .show(ui, |ui| {
                self.render_player_setup(ui, &ctx);
            });

        if let Some(err) = &snapshot.last_error {
            ui.colored_label(Color32::RED, err);
        }
        if let Some(info) = &snapshot.last_info {
            ui.label(RichText::new(info));
        }
        ui.separator();
    }

    fn render_connection_controls(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        let narrow = ui.available_width() < 900.0;
        if narrow {
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    if ui.button("Connect").clicked() {
                        self.connect(ctx);
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
                    self.connect(ctx);
                }
            });
        }
    }

    fn render_player_setup(&mut self, ui: &mut egui::Ui, _ctx: &egui::Context) {
        ui.heading("Player Setup");
        ui.add_space(8.0);

        // Players table
        ui.group(|ui| {
            ui.label(RichText::new("Players:").strong());
            ui.add_space(4.0);

            egui::Grid::new("players_grid")
                .num_columns(4)
                .spacing([8.0, 4.0])
                .striped(true)
                .show(ui, |ui| {
                    // Header
                    ui.label(RichText::new("ID").strong());
                    ui.label(RichText::new("Name").strong());
                    ui.label(RichText::new("Bot").strong());
                    ui.label(RichText::new("Actions").strong());
                    ui.end_row();

                    // Player rows
                    let mut to_remove = None;
                    let mut to_rename = None;
                    let mut bot_updates = Vec::new();

                    let players_snapshot = self.players.clone();
                    for (idx, player) in players_snapshot.iter().enumerate() {
                        ui.label(format!("{}", player.id));
                        ui.label(&player.name);

                        let mut is_bot = player.is_bot;
                        if ui.checkbox(&mut is_bot, "").changed() {
                            bot_updates.push((idx, is_bot));
                        }

                        ui.horizontal(|ui| {
                            // Radio toggle to select which player the frontend would like to control.
                            // Bot players cannot be selected.
                            if player.is_bot {
                                ui.label("Bot");
                            } else {
                                if ui
                                    .radio_value(&mut self.preferred_player, Some(player.id), "Play as")
                                    .on_hover_text("Select this player for this client")
                                    .changed()
                                {
                                    if let Some(pid) = self.preferred_player {
                                        self.send(&ClientMsg::RequestState { player_id: pid });
                                    }
                                }
                            }

                            if ui.button("✏").on_hover_text("Rename").clicked() {
                                to_rename = Some(idx);
                            }
                            if self.players.len() > 1 {
                                if ui.button("🗑").on_hover_text("Remove").clicked() {
                                    to_remove = Some(idx);
                                }
                            }
                        });
                        ui.end_row();
                    }

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
                });
        });

        ui.add_space(8.0);

        // Add new player section
        ui.group(|ui| {
            ui.label(RichText::new("Add New Player:").strong());
            ui.add_space(4.0);

            ui.horizontal(|ui| {
                ui.label("Name:");
                ui.text_edit_singleline(&mut self.new_player_name);

                if ui.button("Add Player").clicked() {
                    let player_name = if self.new_player_name.is_empty() {
                        self.generate_random_name()
                    } else {
                        self.new_player_name.clone()
                    };

                    self.players.push(PlayerConfig {
                        id: self.next_player_id,
                        name: player_name,
                        is_bot: false, // New players start as human by default
                    });
                    self.next_player_id += 1;
                    self.new_player_name.clear();
                }
            });
        });

        ui.add_space(16.0);

        // Connect button
        if ui.button("Start Game").clicked() {
            self.send(&ClientMsg::NewGame {
                players: self.players.clone(),
            });
        }

        ui.add_space(8.0);
        ui.label("This will connect to the server and start a new game with the configured players.");
    }

    fn render_showdown_banner(&self, ui: &mut egui::Ui, state: &GameStatePublic) {
        if state.stage == Stage::Showdown {
            let you_won = state.winner_ids.contains(&state.you_id);
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

    fn render_table_panel(ui: &mut egui::Ui, state: &GameStatePublic) {
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
                        let clip = format_game_for_clipboard(state);
                        ui.ctx().copy_text(clip);
                    }
                });
            });
            egui::ScrollArea::vertical()
                .id_salt("action_log_scroll")
                .max_height(200.0)
                .show(ui, |ui| {
                    for entry in state.action_log.iter().rev().take(100) {
                        log_entry_row(ui, entry, &state.players, state.you_id);
                    }
                });
        });
    }

    fn render_players_panel(&self, ui: &mut egui::Ui, state: &GameStatePublic) {
        ui.group(|ui| {
            for (idx, p) in state.players.iter().enumerate() {
                let me = p.id == state.you_id;
                ui.horizontal(|ui| {
                    if state.to_act == idx && state.stage != Stage::Showdown {
                        ui.colored_label(Color32::from_rgb(255, 215, 0), "●");
                    } else {
                        ui.label("  ");
                    }
                    if me {
                        ui.colored_label(Color32::LIGHT_GREEN, "You");
                    }
                    ui.label(RichText::new(&p.name).strong());
                    if p.has_folded {
                        ui.colored_label(Color32::LIGHT_RED, "(folded)");
                    }
                    if state.stage == Stage::Showdown && state.winner_ids.contains(&p.id) {
                        ui.colored_label(Color32::YELLOW, "WINNER");
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.monospace(format!("stack: {}", p.stack));
                    });
                });
                if me {
                    ui.vertical(|ui| {
                        // Render cards if available; otherwise still render spacer + separator
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
                            // No cards (e.g. after hand) — keep the same visual spacing
                            ui.add_space(6.0);
                            ui.separator();
                            ui.add_space(6.0);
                        }

                        // Action buttons placed below the player's cards
                        if state.to_act == idx && state.stage != Stage::Showdown {
                            // Normal active action buttons during the hand
                            self.render_action_row(ui, p.id, true, false);
                            ui.add_space(6.0);
                            ui.separator();
                        } else if me && (state.stage == Stage::Showdown || p.cards.is_none()) {
                            // After showdown or if there are no cards (hand finished), show a Next hand button
                            // and the action buttons in a disabled state so the area doesn't disappear.
                            self.render_action_row(ui, p.id, false, true);
                            ui.add_space(6.0);
                            ui.separator();
                        } else {
                            // keep space for alignment when not active
                            ui.add_space(8.0);
                        }
                    });
                }
                ui.add_space(8.0);
            }
        });
    }

    fn render_panels(&self, ui: &mut egui::Ui, state: &GameStatePublic) {
        let narrow = ui.available_width() < 900.0;
        if narrow {
            self.render_players_panel(ui, state);
            ui.add_space(8.0);
            Self::render_table_panel(ui, state);
        } else {
            ui.columns(2, |cols| {
                Self::render_table_panel(&mut cols[0], state);
                self.render_players_panel(&mut cols[1], state);
            });
        }
    }

    // Centralized action buttons for a given player id.
    // Callers should check whether the player is active and whether the stage allows actions.
    fn render_action_buttons(&self, ui: &mut egui::Ui, player_id: usize, enabled: bool) {
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
        player_id: usize,
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
                        self.send(&ClientMsg::NextHand { player_id });
                    }
                });
                ui.add_space(6.0);
            }
            // Render the centralized action buttons (enabled or disabled)
            self.render_action_buttons(ui, player_id, enabled);
        });
    }

    fn connect(&mut self, ctx: &egui::Context) {
        self.conn_eff
            .start_connect(ctx, &self.edit_server_address, self.players.clone());
        self.show_error_popup = false;
    }

    fn send(&self, msg: &ClientMsg) {
        self.conn_eff.send(msg);
    }

    // Generate a random name that doesn't conflict with existing player names
    fn generate_random_name(&self) -> String {
        use std::collections::HashSet;

        // Random name pool (defined here instead of in struct state)
        let random_names = [
            "Alice", "Bob", "Charlie", "Diana", "Eve", "Frank", "Grace", "Henry",
            "Iris", "Jack", "Kate", "Leo", "Mia", "Noah", "Olivia", "Peter",
            "Quinn", "Rose", "Sam", "Tina", "Uma", "Victor", "Wendy", "Xander",
            "Yara", "Zoe", "Alex", "Blake", "Casey", "Dylan", "Erin", "Finn",
            "Gabe", "Holly", "Ian", "Jade", "Kyle", "Luna", "Max", "Nora",
            "Owen", "Piper", "Ryan", "Sage", "Tyler", "Violet", "Wyatt", "Zara"
        ];

        // Create a set of existing names for quick lookup
        let existing_names: HashSet<&str> = self.players.iter()
            .map(|p| p.name.as_str())
            .collect();

        // Try to find a name that's not already used
        for &name in &random_names {
            if !existing_names.contains(name) {
                return name.to_string();
            }
        }

        // If all names are used, append a number to the first available name
        for &base_name in &random_names {
            for i in 2..100 { // Try numbers 2-99
                let candidate = format!("{} {}", base_name, i);
                if !existing_names.contains(candidate.as_str()) {
                    return candidate;
                }
            }
        }

        // Fallback: use a timestamp-based name
        format!("Player {}", std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs())
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
    fn ui(&mut self, _app_interface: &mut AppInterface, ui: &mut egui::Ui, _frame: &mut Frame) {
        let ctx = ui.ctx().clone();

        // Poll the connection effect to drain incoming messages into the store
        self.conn_eff.poll();

        self.draw_error_popup(&ctx);

        // Render header (it will read from the store snapshot internally)
        self.render_header(ui, &ctx);

        // Render main content from the latest snapshot
        let snapshot = self.state.borrow().clone();
        if let Some(state) = &snapshot.game_state {
            self.render_showdown_banner(ui, state);
            self.render_panels(ui, state);
        } else {
            ui.label("No state yet. Click Connect to start a session.");
        }
    }
}

fn card_chip(ui: &mut egui::Ui, c: u8) {
    let (text, color) = card_text_and_color(c);
    let b = egui::widgets::Button::new(RichText::new(text).color(color).size(28.0))
        .min_size(egui::vec2(48.0, 40.0));
    ui.add(b);
}

fn card_text_and_color(c: u8) -> (String, Color32) {
    let rank_idx = (c % 13) as usize;
    let suit_idx = (c / 13) as usize;
    let ranks = [
        "A", "2", "3", "4", "5", "6", "7", "8", "9", "T", "J", "Q", "K",
    ];
    let suits = ['♣', '♦', '♥', '♠'];
    let text = format!("{}{}", ranks[rank_idx], suits[suit_idx]);
    let color = match suits[suit_idx] {
        '♦' | '♥' => Color32::from_rgb(220, 50, 50),
        _ => Color32::WHITE,
    };
    (text, color)
}

fn action_kind_text(kind: &ActionKind) -> (String, Color32) {
    match kind {
        ActionKind::Fold => ("🟥 folds".into(), Color32::from_rgb(220, 80, 80)),
        ActionKind::Check => ("⏭ checks".into(), Color32::from_rgb(120, 160, 220)),
        ActionKind::Call(n) => (format!("📞 calls {}", n), Color32::from_rgb(120, 160, 220)),
        ActionKind::Bet(n) => (format!("💰 bets {}", n), Color32::from_rgb(240, 200, 80)),
        ActionKind::Raise { to, by } => (
            format!("▲ raises to {} (+{})", to, by),
            Color32::from_rgb(250, 160, 60),
        ),
        ActionKind::PostBlind { kind, amount } => match kind {
            BlindKind::SmallBlind => (
                format!("🟤 posts small blind {}", amount),
                Color32::from_rgb(170, 120, 60),
            ),
            BlindKind::BigBlind => (
                format!("⚫ posts big blind {}", amount),
                Color32::from_rgb(120, 120, 120),
            ),
        },
    }
}

fn category_text(cat: &mcg_shared::HandRankCategory) -> &'static str {
    match cat {
        mcg_shared::HandRankCategory::HighCard => "High Card",
        mcg_shared::HandRankCategory::Pair => "Pair",
        mcg_shared::HandRankCategory::TwoPair => "Two Pair",
        mcg_shared::HandRankCategory::ThreeKind => "Three of a Kind",
        mcg_shared::HandRankCategory::Straight => "Straight",
        mcg_shared::HandRankCategory::Flush => "Flush",
        mcg_shared::HandRankCategory::FullHouse => "Full House",
        mcg_shared::HandRankCategory::FourKind => "Four of a Kind",
        mcg_shared::HandRankCategory::StraightFlush => "Straight Flush",
    }
}

fn name_of(players: &[PlayerPublic], id: usize) -> String {
    players
        .iter()
        .find(|p| p.id == id)
        .map(|p| p.name.clone())
        .unwrap_or_else(|| format!("Player {}", id))
}

fn card_text(c: u8) -> String {
    card_text_and_color(c).0
}

fn stage_badge(stage: Stage) -> egui::WidgetText {
    let (txt, color) = match stage {
        Stage::Preflop => ("Preflop", Color32::from_rgb(100, 150, 255)),
        Stage::Flop => ("Flop", Color32::from_rgb(100, 200, 120)),
        Stage::Turn => ("Turn", Color32::from_rgb(230, 180, 80)),
        Stage::River => ("River", Color32::from_rgb(220, 120, 120)),
        Stage::Showdown => ("Showdown", Color32::from_rgb(180, 100, 220)),
    };
    RichText::new(txt).color(color).strong().into()
}

fn stage_to_str(stage: Stage) -> &'static str {
    match stage {
        Stage::Preflop => "Preflop",
        Stage::Flop => "Flop",
        Stage::Turn => "Turn",
        Stage::River => "River",
        Stage::Showdown => "Showdown",
    }
}

fn format_game_for_clipboard(state: &GameStatePublic) -> String {
    let mut out = String::new();
    // Summary
    out.push_str("Game summary\n");
    out.push_str(&format!("Stage: {}\n", stage_to_str(state.stage)));
    out.push_str(&format!("Pot: {}\n", state.pot));
    if let Some(p) = state.players.iter().find(|p| p.id == state.you_id) {
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

    // Players
    out.push_str("Players\n");
    for (idx, p) in state.players.iter().enumerate() {
        let you = if p.id == state.you_id { " (you)" } else { "" };
        let folded = if p.has_folded { ", folded:true" } else { "" };
        let to_act = if state.stage != Stage::Showdown && state.to_act == idx {
            ", to_act:true"
        } else {
            ""
        };
        out.push_str(&format!(
            "- id:{}, name:{}, stack:{}{}{}{}\n",
            p.id, p.name, p.stack, you, folded, to_act
        ));
        if p.id == state.you_id {
            if let Some(cards) = p.cards {
                out.push_str(&format!(
                    "  hole: {}, {}\n",
                    card_text(cards[0]),
                    card_text(cards[1])
                ));
            }
        }
    }
    out.push('\n');

    // Board
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

    // Action log (chronological)
    out.push_str("Action log (chronological)\n");
    for entry in &state.action_log {
        match entry {
            ActionEvent::PlayerAction { player_id, action } => {
                let who_name = name_of(&state.players, *player_id);
                match action {
                    ActionKind::Fold => out.push_str(&format!("- {} folds\n", who_name)),
                    ActionKind::Check => out.push_str(&format!("- {} checks\n", who_name)),
                    ActionKind::Call(n) => out.push_str(&format!("- {} calls {}\n", who_name, n)),
                    ActionKind::Bet(n) => out.push_str(&format!("- {} bets {}\n", who_name, n)),
                    ActionKind::Raise { to, by } => {
                        out.push_str(&format!("- {} raises to {} (+{})\n", who_name, to, by))
                    }
                    ActionKind::PostBlind { kind, amount } => match kind {
                        BlindKind::SmallBlind => {
                            out.push_str(&format!("- {} posts small blind {}\n", who_name, amount))
                        }
                        BlindKind::BigBlind => {
                            out.push_str(&format!("- {} posts big blind {}\n", who_name, amount))
                        }
                    },
                }
            }
            ActionEvent::GameAction(GameAction::StageChanged(s)) => {
                out.push_str(&format!("== Stage: {} ==\\n", stage_to_str(*s)));
            }
            ActionEvent::GameAction(GameAction::DealtHole { player_id }) => {
                let who = name_of(&state.players, *player_id);
                out.push_str(&format!("- Dealt hole cards to {}\n", who));
            }
            ActionEvent::GameAction(GameAction::DealtCommunity { cards }) => match cards.len() {
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
            },
            ActionEvent::GameAction(GameAction::Showdown { hand_results }) => {
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
            ActionEvent::GameAction(GameAction::PotAwarded { winners, amount }) => {
                let names = winners
                    .iter()
                    .map(|&id| name_of(&state.players, id))
                    .collect::<Vec<_>>()
                    .join(", ");
                out.push_str(&format!("- Pot {} awarded to {}\n", amount, names));
            }
        }
    }

    out
}

fn log_entry_row(ui: &mut egui::Ui, entry: &ActionEvent, players: &[PlayerPublic], you_id: usize) {
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
                format!("🂠 Dealt hole cards to {}", who),
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
