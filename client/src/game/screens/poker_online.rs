use eframe::Frame;
use egui::{Color32, RichText};
use mcg_shared::{
    ActionKind, BlindKind, ClientMsg, GameStatePublic, LogEntry, LogEvent, PlayerAction,
    PlayerPublic, ServerMsg, Stage,
};
use std::cell::RefCell;
use std::rc::Rc;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::closure::Closure;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;
#[cfg(target_arch = "wasm32")]
use web_sys::{CloseEvent, Event, MessageEvent, WebSocket};

use super::{AppInterface, ScreenWidget};
use crate::qr_scanner::QrScannerPopup;

pub struct PokerOnlineScreen {
    #[cfg(target_arch = "wasm32")]
    ws: Option<WebSocket>,
    last_error: Option<String>,
    last_info: Option<String>,
    state: Option<GameStatePublic>,
    name: String,
    server_address: String,
    bots: usize,
    inbox: Rc<RefCell<Vec<ServerMsg>>>,
    error_inbox: Rc<RefCell<Vec<String>>>,
    show_error_popup: bool,
    scanner: QrScannerPopup,
}
impl PokerOnlineScreen {
    pub fn new() -> Self {
        Self {
            #[cfg(target_arch = "wasm32")]
            ws: None,
            last_error: None,
            last_info: None,
            state: None,
            name: "Player".to_string(),
            #[cfg(target_arch = "wasm32")]
            server_address: {
                let window = web_sys::window().expect("no global window exists");
                let location = window.location();
                let hostname = location.hostname().unwrap_or("127.0.0.1".into());
                let port = location.port().unwrap_or("3000".into());
                format!("{}:{}", hostname, port)
            },
            #[cfg(not(target_arch = "wasm32"))]
            server_address: "127.0.0.1:3000".to_string(),
            bots: 1,
            inbox: Rc::new(RefCell::new(Vec::new())),
            error_inbox: Rc::new(RefCell::new(Vec::new())),
            show_error_popup: false,
            scanner: QrScannerPopup::new(),
        }
    }

    fn draw_error_popup(&mut self, ctx: &egui::Context) {
        if !self.show_error_popup {
            return;
        }
        let mut open = true;
        egui::Window::new("Connection error")
            .collapsible(false)
            .resizable(false)
            .open(&mut open)
            .show(ctx, |ui| {
                if let Some(err) = &self.last_error {
                    ui.label(err);
                } else {
                    ui.label(format!(
                        "Failed to connect to server at {}. Is it running?",
                        self.server_address
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

    fn process_incoming_messages(&mut self) {
        {
            let mut msgs = self.inbox.borrow_mut();
            for msg in msgs.drain(..) {
                match msg {
                    ServerMsg::Welcome { .. } => {
                        self.last_info = Some("Connected".into());
                        self.last_error = None;
                        self.show_error_popup = false;
                    }
                    ServerMsg::State(gs) => {
                        self.state = Some(gs);
                        self.last_info = None;
                    }
                    ServerMsg::Error(err) => {
                        self.last_error = Some(err);
                        self.show_error_popup = true;
                    }
                }
            }
        }
        {
            let mut errs = self.error_inbox.borrow_mut();
            for e in errs.drain(..) {
                self.last_error = Some(e);
                self.show_error_popup = true;
            }
        }
    }

    fn render_header(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        // Title row with current stage badge
        ui.horizontal(|ui| {
            ui.heading("Poker Online");
            ui.add_space(16.0);
            if let Some(s) = &self.state {
                ui.label(stage_badge(s.stage));
                ui.add_space(8.0);
                ui.label(format!("Bots: {}", s.bot_count));
            }
        });

        // Collapsible connection & session controls
        let default_open = self.state.is_none();
        egui::CollapsingHeader::new("Connection & session")
            .default_open(default_open)
            .show(ui, |ui| {
                Self::render_connection_controls(self, ui, ctx);
            });

        if let Some(err) = &self.last_error {
            ui.colored_label(Color32::RED, err);
        }
        if let Some(info) = &self.last_info {
            ui.label(RichText::new(info));
        }
        ui.separator();
    }

    fn render_connection_controls(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        let narrow = ui.available_width() < 700.0;
        if narrow {
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label("Name:");
                    ui.text_edit_singleline(&mut self.name).on_hover_text("Your nickname");
                    if ui.button("Connect").clicked() {
                        self.connect(ctx);
                    }
                });
                ui.horizontal(|ui| {
                    ui.label("Server:");
                    ui.text_edit_singleline(&mut self.server_address)
                        .on_hover_text("Server address (IP:PORT)");
                    self.scanner.button_and_popup(ui, ctx, &mut self.server_address);
                });
                ui.horizontal(|ui| {
                    ui.label("Bots:");
                    ui.add(
                        egui::DragValue::new(&mut self.bots)
                            .range(0..=8)
                            .speed(0.1)
                            .suffix(" bots"),
                    );
                    if ui
                        .add(egui::Button::new("Reset Game"))
                        .on_hover_text("Reset the game with the chosen number of bots")
                        .clicked()
                    {
                        self.send(&ClientMsg::ResetGame { bots: self.bots });
                        self.last_info = Some(format!("Reset requested ({} bots)", self.bots));
                        self.last_error = None;
                    }
                });
            });
        } else {
            ui.horizontal(|ui| {
                ui.label("Name:");
                ui.text_edit_singleline(&mut self.name).on_hover_text("Your nickname");
                ui.add_space(12.0);
                ui.label("Server:");
                ui.text_edit_singleline(&mut self.server_address)
                    .on_hover_text("Server address (IP:PORT)");
                self.scanner.button_and_popup(ui, ctx, &mut self.server_address);
                ui.add_space(12.0);
                ui.label("Bots:");
                ui.add(
                    egui::DragValue::new(&mut self.bots)
                        .range(0..=8)
                        .speed(0.1)
                        .suffix(" bots"),
                );
                if ui
                    .add(egui::Button::new("Reset Game"))
                    .on_hover_text("Reset the game with the chosen number of bots")
                    .clicked()
                {
                    self.send(&ClientMsg::ResetGame { bots: self.bots });
                    self.last_info = Some(format!("Reset requested ({} bots)", self.bots));
                    self.last_error = None;
                }
                if ui.button("Connect").clicked() {
                    self.connect(ctx);
                }
            });
        }
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
                        .on_hover_text("Copy a structured summary of the current game and full action log")
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

    fn render_players_panel(ui: &mut egui::Ui, state: &GameStatePublic) {
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
                    if let Some(cards) = p.cards {
                        ui.horizontal(|ui| {
                            ui.add_space(12.0);
                            card_chip(ui, cards[0]);
                            card_chip(ui, cards[1]);
                        });
                    }
                }
                ui.add_space(8.0);
            }
        });
    }

    fn render_panels(ui: &mut egui::Ui, state: &GameStatePublic) {
        let narrow = ui.available_width() < 700.0;
        if narrow {
            Self::render_table_panel(ui, state);
            ui.add_space(8.0);
            Self::render_players_panel(ui, state);
        } else {
            ui.columns(2, |cols| {
                Self::render_table_panel(&mut cols[0], state);
                Self::render_players_panel(&mut cols[1], state);
            });
        }
    }

    fn render_action_row(&self, ui: &mut egui::Ui, state: &GameStatePublic) {
        ui.horizontal(|ui| {
            if ui.add(egui::Button::new("Check / Call")).clicked() {
                self.send(&ClientMsg::Action(PlayerAction::CheckCall));
            }
            if ui
                .add(egui::Button::new("Bet 10"))
                .on_hover_text("Place a small bet")
                .clicked()
            {
                self.send(&ClientMsg::Action(PlayerAction::Bet(10)));
            }
            if ui.add(egui::Button::new("Fold")).clicked() {
                self.send(&ClientMsg::Action(PlayerAction::Fold));
            }
            ui.separator();
            if state.stage == Stage::Showdown {
                if ui.add(egui::Button::new("Next Hand")).clicked() {
                    self.send(&ClientMsg::NextHand);
                }
                ui.add_space(8.0);
            }
            if ui.add(egui::Button::new("Refresh")).clicked() {
                self.send(&ClientMsg::RequestState);
            }
        });
    }

    #[cfg(target_arch = "wasm32")]
    fn connect(&mut self, ctx: &egui::Context) {
        let ws_url = format!("ws://{}/ws", self.server_address);
        match WebSocket::new(&ws_url) {
            Ok(ws) => {
                let join = serde_json::to_string(&ClientMsg::Join {
                    name: self.name.clone(),
                })
                .unwrap();
                let ws_clone = ws.clone();
                let onopen = Closure::<dyn FnMut()>::new(move || {
                    let _ = ws_clone.send_with_str(&join);
                });
                ws.set_onopen(Some(onopen.as_ref().unchecked_ref()));
                onopen.forget();
                let onmessage = {
                    let inbox = Rc::clone(&self.inbox);
                    Closure::<dyn FnMut(MessageEvent)>::new(move |e: MessageEvent| {
                        if let Ok(txt) = e.data().dyn_into::<js_sys::JsString>() {
                            if let Ok(msg) = serde_json::from_str::<ServerMsg>(&String::from(txt)) {
                                inbox.borrow_mut().push(msg);
                            }
                        }
                    })
                };
                ws.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));
                onmessage.forget();
                let onerror = {
                    let err_inbox = Rc::clone(&self.error_inbox);
                    let ctx = ctx.clone();
                    let server_address = self.server_address.clone();
                    Closure::<dyn FnMut(Event)>::new(move |_e: Event| {
                        err_inbox.borrow_mut().push(format!(
                            "Failed to connect to server at {}.",
                            server_address
                        ));
                        ctx.request_repaint();
                    })
                };
                ws.set_onerror(Some(onerror.as_ref().unchecked_ref()));
                onerror.forget();
                let onclose = {
                    let err_inbox = Rc::clone(&self.error_inbox);
                    let ctx = ctx.clone();
                    let server_address = self.server_address.clone();
                    Closure::<dyn FnMut(CloseEvent)>::new(move |e: CloseEvent| {
                        let code = e.code();
                        let reason = e.reason();
                        let msg = if reason.is_empty() {
                            format!(
                                "Connection closed (code {}). Is the server running at {}?",
                                code, server_address
                            )
                        } else {
                            format!("Connection closed (code {}): {}", code, reason)
                        };
                        err_inbox.borrow_mut().push(msg);
                        ctx.request_repaint();
                    })
                };
                ws.set_onclose(Some(onclose.as_ref().unchecked_ref()));
                onclose.forget();
                self.ws = Some(ws);
                self.last_error = None;
                self.show_error_popup = false;
            }
            Err(err) => {
                self.last_error = Some(format!("WS connect error: {err:?}"));
                self.show_error_popup = true;
            }
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn connect(&mut self, _ctx: &egui::Context) {
        self.last_error = Some("Online mode is unavailable on native builds".into());
    }

    #[cfg(target_arch = "wasm32")]
    fn send(&self, msg: &ClientMsg) {
        if let Some(ws) = &self.ws {
            if let Ok(txt) = serde_json::to_string(msg) {
                let _ = ws.send_with_str(&txt);
            }
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn send(&self, _msg: &ClientMsg) {}
}

impl Default for PokerOnlineScreen {
    fn default() -> Self {
        Self::new()
    }
}

impl ScreenWidget for PokerOnlineScreen {
    fn ui(&mut self, _app_interface: &mut AppInterface, ui: &mut egui::Ui, _frame: &mut Frame) {
        let ctx = ui.ctx().clone();
        self.draw_error_popup(&ctx);
        self.process_incoming_messages();

        self.render_header(ui, &ctx);

        if let Some(state) = &self.state {
            self.render_showdown_banner(ui, state);
            Self::render_panels(ui, state);
            ui.separator();
            self.render_action_row(ui, state);
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
            out.push_str(&format!("Your hole cards: {}, {}\n", card_text(cards[0]), card_text(cards[1])));
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
        let to_act = if state.stage != Stage::Showdown && state.to_act == idx { ", to_act:true" } else { "" };
        out.push_str(&format!("- id:{}, name:{}, stack:{}{}{}{}\n", p.id, p.name, p.stack, you, folded, to_act));
        if p.id == state.you_id {
            if let Some(cards) = p.cards {
                out.push_str(&format!("  hole: {}, {}\n", card_text(cards[0]), card_text(cards[1])));
            }
        }
    }
    out.push('\n');

    // Board
    out.push_str("Board\n");
    if state.community.is_empty() {
        out.push_str("- (no community cards yet)\n");
    } else {
        let board = state.community.iter().map(|&c| card_text(c)).collect::<Vec<_>>().join(", ");
        out.push_str(&format!("- {}\n", board));
    }
    out.push('\n');

    // Action log (chronological)
    out.push_str("Action log (chronological)\n");
    for entry in &state.action_log {
        match &entry.event {
            LogEvent::Action(kind) => {
                let who_name = entry.player_id.map(|id| name_of(&state.players, id)).unwrap_or_else(|| "Table".to_string());
                match kind {
                    ActionKind::Fold => out.push_str(&format!("- {} folds\n", who_name)),
                    ActionKind::Check => out.push_str(&format!("- {} checks\n", who_name)),
                    ActionKind::Call(n) => out.push_str(&format!("- {} calls {}\n", who_name, n)),
                    ActionKind::Bet(n) => out.push_str(&format!("- {} bets {}\n", who_name, n)),
                    ActionKind::Raise { to, by } => out.push_str(&format!("- {} raises to {} (+{})\n", who_name, to, by)),
                    ActionKind::PostBlind { kind, amount } => match kind {
                        BlindKind::SmallBlind => out.push_str(&format!("- {} posts small blind {}\n", who_name, amount)),
                        BlindKind::BigBlind => out.push_str(&format!("- {} posts big blind {}\n", who_name, amount)),
                    },
                }
            }
            LogEvent::StageChanged(s) => {
                out.push_str(&format!("== Stage: {} ==\n", stage_to_str(*s)));
            }
            LogEvent::DealtHole { player_id } => {
                let who = name_of(&state.players, *player_id);
                out.push_str(&format!("- Dealt hole cards to {}\n", who));
            }
            LogEvent::DealtCommunity { cards } => {
                match cards.len() {
                    3 => out.push_str(&format!("- Flop: {}, {}, {}\n", card_text(cards[0]), card_text(cards[1]), card_text(cards[2]))),
                    4 => out.push_str(&format!("- Turn: {}\n", card_text(cards[3]))),
                    5 => out.push_str(&format!("- River: {}\n", card_text(cards[4]))),
                    _ => {
                        let s = cards.iter().map(|&c| card_text(c)).collect::<Vec<_>>().join(", ");
                        out.push_str(&format!("- Community: {}\n", s));
                    }
                }
            }
            LogEvent::Showdown { hand_results } => {
                if hand_results.is_empty() {
                    out.push_str("- Showdown\n");
                } else {
                    for hr in hand_results {
                        let who = name_of(&state.players, hr.player_id);
                        let cat = category_text(&hr.rank.category);
                        let best = hr.best_five.iter().map(|&c| card_text(c)).collect::<Vec<_>>().join(", ");
                        out.push_str(&format!("- Showdown: {} -> {} [{}]\n", who, cat, best));
                    }
                }
            }
            LogEvent::PotAwarded { winners, amount } => {
                let names = winners.iter().map(|&id| name_of(&state.players, id)).collect::<Vec<_>>().join(", ");
                out.push_str(&format!("- Pot {} awarded to {}\n", amount, names));
            }
        }
    }

    out
}

fn log_entry_row(ui: &mut egui::Ui, entry: &LogEntry, players: &[PlayerPublic], you_id: usize) {
    match &entry.event {
        LogEvent::Action(kind) => {
            let who_id = entry.player_id;
            let who_name = who_id.map(|id| name_of(players, id)).unwrap_or_else(|| "Table".to_string());
            let (txt, color) = action_kind_text(kind);
            let is_you = who_id == Some(you_id);
            let label = if is_you {
                RichText::new(format!("{} {}", who_name, txt)).color(color).strong()
            } else {
                RichText::new(format!("{} {}", who_name, txt)).color(color)
            };
            ui.label(label);
        }
        LogEvent::StageChanged(s) => {
            ui.add_space(6.0);
            ui.separator();
            ui.horizontal(|ui| {
                ui.label(RichText::new("🕒").strong());
                ui.label(stage_badge(*s));
            });
            ui.separator();
            ui.add_space(6.0);
        }
        LogEvent::DealtHole { player_id } => {
            let who = name_of(players, *player_id);
            ui.colored_label(Color32::from_rgb(150, 150, 150), format!("🂠 Dealt hole cards to {}", who));
        }
        LogEvent::DealtCommunity { cards } => {
            match cards.len() {
                3 => {
                    ui.colored_label(
                        Color32::from_rgb(100, 200, 120),
                        format!("🃏 Flop: {} {} {}", card_text(cards[0]), card_text(cards[1]), card_text(cards[2])),
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
            }
        },
        LogEvent::Showdown { hand_results } => {
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
        LogEvent::PotAwarded { winners, amount } => {
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
