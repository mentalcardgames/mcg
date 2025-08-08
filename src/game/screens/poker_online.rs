use eframe::Frame;
use egui::{Context, RichText, Color32};
use serde::{Deserialize, Serialize};
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use web_sys::{MessageEvent, WebSocket};

use super::{AppInterface, ScreenWidget};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum ClientMsg {
    Join { name: String },
    Action(PlayerAction),
    RequestState,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum ServerMsg {
    Welcome { you: usize },
    State(GameStatePublic),
    Error(String),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum PlayerAction {
    Fold,
    CheckCall,
    Bet(u32),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GameStatePublic {
    pub players: Vec<PlayerPublic>,
    pub community: Vec<u8>,
    pub pot: u32,
    pub to_act: usize,
    pub stage: Stage,
    pub you_id: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlayerPublic {
    pub id: usize,
    pub name: String,
    pub stack: u32,
    pub cards: Option<[u8; 2]>,
    pub has_folded: bool,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Stage {
    Preflop,
    Flop,
    Turn,
    River,
    Showdown,
}

pub struct PokerOnlineScreen {
    ws: Option<WebSocket>,
    last_error: Option<String>,
    last_info: Option<String>,
    state: Option<GameStatePublic>,
    name: String,
}

impl PokerOnlineScreen {
    pub fn new() -> Self {
        Self {
            ws: None,
            last_error: None,
            last_info: None,
            state: None,
            name: "Player".to_string(),
        }
    }

    fn connect(&mut self) {
        // Determine ws url: if hosted at http(s)://host:port, server is at ws://host:3000/ws
        let window = web_sys::window().unwrap();
        let loc = window.location();
        let host = loc.hostname().unwrap_or("127.0.0.1".into());
        let ws_url = format!("ws://{host}:3000/ws");
        match WebSocket::new(&ws_url) {
            Ok(ws) => {
                // Open handler: send Join
                let join = serde_json::to_string(&ClientMsg::Join { name: self.name.clone() }).unwrap();
                let ws_clone = ws.clone();
                let onopen = Closure::<dyn FnMut()>::new(move || {
                    let _ = ws_clone.send_with_str(&join);
                });
                ws.set_onopen(Some(onopen.as_ref().unchecked_ref()));
                onopen.forget();

                // Message handler: parse and store state
                let onmessage = {
                    let this_ptr = self as *mut Self;
                    Closure::<dyn FnMut(MessageEvent)>::new(move |e: MessageEvent| {
                        if let Ok(txt) = e.data().dyn_into::<js_sys::JsString>() {
                            if let Ok(msg) = serde_json::from_str::<ServerMsg>(&String::from(txt)) {
                                unsafe {
                                    let s = &mut *this_ptr;
                                    match msg {
                                        ServerMsg::Welcome { .. } => {
                                            s.last_info = Some("Connected".into());
                                        }
                                        ServerMsg::State(gs) => {
                                            s.state = Some(gs);
                                            s.last_info = None;
                                        }
                                        ServerMsg::Error(err) => {
                                            s.last_error = Some(err);
                                        }
                                    }
                                }
                            }
                        }
                    })
                };
                ws.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));
                onmessage.forget();

                self.ws = Some(ws);
            }
            Err(err) => {
                self.last_error = Some(format!("WS connect error: {err:?}"));
            }
        }
    }

    fn send(&self, msg: &ClientMsg) {
        if let Some(ws) = &self.ws {
            if let Ok(txt) = serde_json::to_string(msg) {
                let _ = ws.send_with_str(&txt);
            }
        }
    }
}

impl Default for PokerOnlineScreen {
    fn default() -> Self { Self::new() }
}

impl ScreenWidget for PokerOnlineScreen {
    fn update(&mut self, _app_interface: &mut AppInterface, ctx: &Context, _frame: &mut Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Poker Online");
                ui.add_space(16.0);
                if let Some(s) = &self.state {
                    ui.label(stage_badge(s.stage));
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Connect").clicked() { self.connect(); }
                    ui.text_edit_singleline(&mut self.name).on_hover_text("Your nickname");
                    ui.label("Name:");
                });
            });

            if let Some(err) = &self.last_error {
                ui.colored_label(Color32::RED, err);
            }
            if let Some(info) = &self.last_info {
                ui.label(RichText::new(info));
            }

            ui.separator();
            if let Some(state) = &self.state {
                ui.columns(2, |cols| {
                    // Left column: table/pot/board
                    cols[0].group(|ui| {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Pot:").strong());
                            ui.monospace(format!(" {}", state.pot));
                        });
                        ui.add_space(8.0);
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Board:").strong());
                            if state.community.is_empty() { ui.label("—"); }
                            for &c in &state.community {
                                card_chip(ui, c);
                            }
                        });
                    });

                    // Right column: players
                    cols[1].group(|ui| {
                        for p in &state.players {
                            let me = p.id == state.you_id;
                            ui.horizontal(|ui| {
                                if me { ui.colored_label(Color32::LIGHT_GREEN, "You"); }
                                ui.label(RichText::new(&p.name).strong());
                                if p.has_folded { ui.colored_label(Color32::LIGHT_RED, "(folded)"); }
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
                });

                ui.separator();
                ui.horizontal(|ui| {
                    if ui.add(egui::Button::new("Check / Call")).clicked() {
                        self.send(&ClientMsg::Action(PlayerAction::CheckCall));
                    }
                    if ui.add(egui::Button::new("Bet 10"))
                        .on_hover_text("Place a small bet")
                        .clicked()
                    {
                        self.send(&ClientMsg::Action(PlayerAction::Bet(10)));
                    }
                    if ui.add(egui::Button::new("Fold")).clicked() {
                        self.send(&ClientMsg::Action(PlayerAction::Fold));
                    }
                    ui.separator();
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

// Small helper to show a card as a colored chip with suit
fn card_chip(ui: &mut egui::Ui, c: u8) {
    let (text, color) = card_text_and_color(c);
    let b = egui::widgets::Button::new(RichText::new(text).color(color));
    ui.add(b);
}

fn card_text_and_color(c: u8) -> (String, Color32) {
    let rank_idx = (c % 13) as usize;
    let suit_idx = (c / 13) as usize;
    let ranks = ["A", "2", "3", "4", "5", "6", "7", "8", "9", "T", "J", "Q", "K"];    
    let suits = ['♣', '♦', '♥', '♠'];
    let text = format!("{}{}", ranks[rank_idx], suits[suit_idx]);
    let color = match suits[suit_idx] {
        '♦' | '♥' => Color32::from_rgb(220, 50, 50),
        _ => Color32::WHITE,
    };
    (text, color)
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
