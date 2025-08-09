use eframe::Frame;
use egui::{Color32, Context, RichText};
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ErrorKind {
    Connection,
    Game,
}

pub struct PokerOnlineScreen {
    #[cfg(target_arch = "wasm32")]
    ws: Option<WebSocket>,
    last_error: Option<String>,
    last_error_kind: Option<ErrorKind>,
    last_info: Option<String>,
    state: Option<GameStatePublic>,
    name: String,
    server_address: String,
    desired_bots: usize,
    inbox: Rc<RefCell<Vec<ServerMsg>>>,
    error_inbox: Rc<RefCell<Vec<String>>>,
    show_error_popup: bool,
}
impl PokerOnlineScreen {
    pub fn new() -> Self {
        Self {
            #[cfg(target_arch = "wasm32")]
            ws: None,
            last_error: None,
            last_error_kind: None,
            last_info: None,
            state: None,
            name: "Player".to_string(),
            #[cfg(target_arch = "wasm32")]
            server_address: {
                let window = web_sys::window().expect("no global window exists");
                let location = window.location();
                let hostname = location.hostname().unwrap_or("127.0.0.1".into());
                format!("{}:3000", hostname)
            },
            #[cfg(not(target_arch = "wasm32"))]
            server_address: "127.0.0.1:3000".to_string(),
            inbox: Rc::new(RefCell::new(Vec::new())),
            error_inbox: Rc::new(RefCell::new(Vec::new())),
            show_error_popup: false,
            desired_bots: 1,
        }
    }
    fn draw_error_popup(&mut self, ctx: &Context) {
        if !self.show_error_popup {
            return;
        }
        let mut open = true;
        let title = match self.last_error_kind {
            Some(ErrorKind::Game) => "Game error",
            Some(ErrorKind::Connection) => "Connection error",
            None => "Error",
        };
        egui::Window::new(title)
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
    #[cfg(target_arch = "wasm32")]
    fn connect(&mut self, ctx: &Context) {
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
        self.last_error_kind = Some(ErrorKind::Connection);
                self.show_error_popup = true;
            }
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    fn connect(&mut self, _ctx: &Context) {
    self.last_error = Some("Online mode is unavailable on native builds".into());
    self.last_error_kind = Some(ErrorKind::Game);
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
    fn update(&mut self, _app_interface: &mut AppInterface, ctx: &Context, _frame: &mut Frame) {
        self.draw_error_popup(ctx);
        {
            let mut msgs = self.inbox.borrow_mut();
            for msg in msgs.drain(..) {
                match msg {
                    ServerMsg::Welcome { .. } => {
                        self.last_info = Some("Connected".into());
                        self.last_error = None;
            self.last_error_kind = None;
                        self.show_error_popup = false;
                    }
                    ServerMsg::State(gs) => {
                        self.state = Some(gs);
                        self.last_info = None;
                    }
                    ServerMsg::Error(err) => {
                        self.last_error = Some(err);
            self.last_error_kind = Some(ErrorKind::Game);
                        self.show_error_popup = true;
                    }
                }
            }
        }
        {
            let mut errs = self.error_inbox.borrow_mut();
            for e in errs.drain(..) {
                self.last_error = Some(e);
        self.last_error_kind = Some(ErrorKind::Connection);
                self.show_error_popup = true;
            }
        }
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Poker Online");
                ui.add_space(16.0);
                if let Some(s) = &self.state {
                    ui.label(stage_badge(s.stage));
                    ui.add_space(8.0);
                    ui.label(format!("Bots: {}", s.bot_count));
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Connect").clicked() {
                        self.connect(ctx);
                    }
                    ui.text_edit_singleline(&mut self.name)
                        .on_hover_text("Your nickname");
                    ui.label("Name:");
                });
            });
            ui.horizontal(|ui| {
                ui.label("Server:");
                ui.text_edit_singleline(&mut self.server_address)
                    .on_hover_text("Server address (IP:PORT)");
                ui.separator();
                ui.label("Bots:");
                let mut bots_u32: u32 = self.desired_bots as u32;
                if ui.add(egui::DragValue::new(&mut bots_u32).range(0..=8)).changed() {
                    self.desired_bots = bots_u32 as usize;
                }
                if ui
                    .add(egui::Button::new("Reset Game"))
                    .on_hover_text("Reset the table with the chosen number of bots")
                    .clicked()
                {
                    self.send(&ClientMsg::ResetGame { bots: self.desired_bots });
                }
            });
            if let Some(err) = &self.last_error {
                ui.colored_label(Color32::RED, err);
            }
            if let Some(info) = &self.last_info {
                ui.label(RichText::new(info));
            }
            ui.separator();
            if let Some(state) = &self.state {
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
                ui.columns(2, |cols| {
                    cols[0].group(|ui| {
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
                        // Header + Copy button for Recent actions
                        let recent_copy_text: String = state
                            .recent_actions
                            .iter()
                            .rev()
                            .take(10)
                            .map(|ev| {
                                let name = state
                                    .players
                                    .iter()
                                    .find(|p| p.id == ev.player_id)
                                    .map(|p| p.name.as_str())
                                    .unwrap_or("Player");
                                let action_txt = action_text(&ev.action);
                                format!("{} {}", name, action_txt)
                            })
                            .collect::<Vec<_>>()
                            .join("\n");
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Recent actions:").strong());
                            if ui
                                .button("Copy")
                                .on_hover_text("Copy recent actions")
                                .clicked()
                            {
                                ui.ctx().copy_text(recent_copy_text.clone());
                            }
                        });
                        egui::ScrollArea::vertical()
                            .id_salt("recent_actions_scroll")
                            .max_height(120.0)
                            .show(ui, |ui| {
                                for ev in state.recent_actions.iter().rev().take(10) {
                                    let name = state
                                        .players
                                        .iter()
                                        .find(|p| p.id == ev.player_id)
                                        .map(|p| p.name.as_str())
                                        .unwrap_or("Player");
                                    let action_txt = action_text(&ev.action);
                                    ui.label(format!("{} {}", name, action_txt));
                                }
                            });
                        ui.add_space(8.0);
                        ui.separator();
                        // Header + Copy button for Action log
                        let action_log_copy_text: String = state
                            .action_log
                            .iter()
                            .rev()
                            .take(50)
                            .map(|entry| log_entry_text(entry, &state.players))
                            .collect::<Vec<_>>()
                            .join("\n");
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Action log:").strong());
                            if ui.button("Copy").on_hover_text("Copy full log").clicked() {
                                ui.ctx().copy_text(action_log_copy_text.clone());
                            }
                        });
                        egui::ScrollArea::vertical()
                            .id_salt("action_log_scroll")
                            .max_height(180.0)
                            .show(ui, |ui| {
                                for entry in state.action_log.iter().rev().take(50) {
                                    ui.label(log_entry_text(entry, &state.players));
                                }
                            });
                    });
                    cols[1].group(|ui| {
                        for p in &state.players {
                            let me = p.id == state.you_id;
                            ui.horizontal(|ui| {
                                if me {
                                    ui.colored_label(Color32::LIGHT_GREEN, "You");
                                }
                                ui.label(RichText::new(&p.name).strong());
                                if p.has_folded {
                                    ui.colored_label(Color32::LIGHT_RED, "(folded)");
                                }
                                if state.stage == Stage::Showdown
                                    && state.winner_ids.contains(&p.id)
                                {
                                    ui.colored_label(Color32::YELLOW, "WINNER");
                                }
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        ui.monospace(format!("stack: {}", p.stack));
                                    },
                                );
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
                });
                ui.separator();
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
            } else {
                ui.label("No state yet. Click Connect to start a session.");
            }
        });
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
fn action_text(a: &PlayerAction) -> String {
    match a {
        PlayerAction::Fold => "folds".into(),
        PlayerAction::CheckCall => "checks / calls".into(),
        PlayerAction::Bet(n) => format!("bets {}", n),
    }
}

fn action_kind_text(kind: &ActionKind) -> String {
    match kind {
        ActionKind::Fold => "folds".into(),
        ActionKind::Check => "checks".into(),
        ActionKind::Call(n) => format!("calls {}", n),
        ActionKind::Bet(n) => format!("bets {}", n),
        ActionKind::Raise { to, by } => format!("raises to {} (+{})", to, by),
        ActionKind::PostBlind { kind, amount } => match kind {
            BlindKind::SmallBlind => format!("posts small blind {}", amount),
            BlindKind::BigBlind => format!("posts big blind {}", amount),
        },
    }
}

fn stage_text(stage: Stage) -> &'static str {
    match stage {
        Stage::Preflop => "Preflop",
        Stage::Flop => "Flop",
        Stage::Turn => "Turn",
        Stage::River => "River",
        Stage::Showdown => "Showdown",
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

fn log_entry_text(entry: &LogEntry, players: &[PlayerPublic]) -> String {
    match &entry.event {
        LogEvent::Action(kind) => {
            let who = entry
                .player_id
                .map(|id| name_of(players, id))
                .unwrap_or_else(|| "Table".to_string());
            format!("{} {}", who, action_kind_text(kind))
        }
        LogEvent::StageChanged(s) => format!("Stage → {}", stage_text(*s)),
        LogEvent::DealtHole { player_id } => {
            let who = name_of(players, *player_id);
            format!("Dealt hole cards to {}", who)
        }
        LogEvent::DealtCommunity { cards } => match cards.len() {
            3 => format!(
                "Flop: {} {} {}",
                card_text(cards[0]),
                card_text(cards[1]),
                card_text(cards[2])
            ),
            4 => format!("Turn: {}", card_text(cards[3])),
            5 => format!("River: {}", card_text(cards[4])),
            _ => format!(
                "Community: {}",
                cards
                    .iter()
                    .map(|&c| card_text(c))
                    .collect::<Vec<_>>()
                    .join(" ")
            ),
        },
        LogEvent::Showdown { hand_results } => {
            let mut parts = Vec::new();
            for hr in hand_results {
                let who = name_of(players, hr.player_id);
                let cat = category_text(&hr.rank.category);
                parts.push(format!("{}: {}", who, cat));
            }
            if parts.is_empty() {
                "Showdown".to_string()
            } else {
                format!("Showdown — {}", parts.join(", "))
            }
        }
        LogEvent::PotAwarded { winners, amount } => {
            let names = winners
                .iter()
                .map(|&id| name_of(players, id))
                .collect::<Vec<_>>()
                .join(", ");
            format!("Pot {} awarded to {}", amount, names)
        }
    }
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
