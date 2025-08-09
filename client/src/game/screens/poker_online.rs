use eframe::Frame;
use egui::{Color32, Context, RichText};
use mcg_shared::{ClientMsg, GameStatePublic, PlayerAction, ServerMsg, Stage};
use std::cell::RefCell;
use std::rc::Rc;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::closure::Closure;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;
#[cfg(target_arch = "wasm32")]
use web_sys::{CloseEvent, Event, MessageEvent, WebSocket};

use super::{AppInterface, ScreenWidget};

pub struct PokerOnlineScreen {
    #[cfg(target_arch = "wasm32")]
    ws: Option<WebSocket>,
    last_error: Option<String>,
    last_info: Option<String>,
    state: Option<GameStatePublic>,
    name: String,
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
            last_info: None,
            state: None,
            name: "Player".to_string(),
            inbox: Rc::new(RefCell::new(Vec::new())),
            error_inbox: Rc::new(RefCell::new(Vec::new())),
            show_error_popup: false,
        }
    }
    fn draw_error_popup(&mut self, ctx: &Context) {
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
                    ui.label("Failed to connect to server. Is it running on port 3000?");
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
        let window = web_sys::window().unwrap();
        let loc = window.location();
        let host = loc.hostname().unwrap_or("127.0.0.1".into());
        let ws_url = format!("ws://{host}:3000/ws");
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
                    Closure::<dyn FnMut(Event)>::new(move |_e: Event| {
                        err_inbox.borrow_mut().push(
                            "Failed to connect to server. Is it running on port 3000?".into(),
                        );
                        ctx.request_repaint();
                    })
                };
                ws.set_onerror(Some(onerror.as_ref().unchecked_ref()));
                onerror.forget();
                let onclose = {
                    let err_inbox = Rc::clone(&self.error_inbox);
                    let ctx = ctx.clone();
                    Closure::<dyn FnMut(CloseEvent)>::new(move |e: CloseEvent| {
                        let code = e.code();
                        let reason = e.reason();
                        let msg = if reason.is_empty() {
                            format!("Connection closed (code {}). Is the server running?", code)
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
    fn connect(&mut self, _ctx: &Context) {
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
    fn update(&mut self, _app_interface: &mut AppInterface, ctx: &Context, _frame: &mut Frame) {
        self.draw_error_popup(ctx);
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
                        ui.label(RichText::new("Recent actions:").strong());
                        egui::ScrollArea::vertical()
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
