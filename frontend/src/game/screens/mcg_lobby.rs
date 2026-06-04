use crate::game::{AppInterface, ScreenWidget};
use super::{ScreenDef, ScreenMetadata};
use egui::{vec2, ColorImage, Context, Image, TextureHandle, TextureOptions, RichText};
use image::{ImageBuffer, Luma};
use mcg_shared::{Frontend2BackendMsg, PlayerConfig, Backend2FrontendMsg};
use crate::sprintln;
use qrcode::QrCode;
use std::cell::RefCell;
use std::rc::Rc;

pub struct LobbyScreen {
    qr_payload: Rc<RefCell<Option<String>>>,
    players: Rc<RefCell<Vec<(String, bool)>>>, // (name, ready)
    our_name_pending: Rc<RefCell<Option<String>>>,
    initialized: bool,
    setup: bool,
}

impl Default for LobbyScreen {
    fn default() -> Self {
        Self {
            qr_payload: Rc::new(RefCell::new(None)),
            players: Rc::new(RefCell::new(Vec::new())),
            our_name_pending: Rc::new(RefCell::new(None)),
            initialized: false,
            setup: false,
        }
    }
}

impl ScreenWidget for LobbyScreen {
    fn ui(
        &mut self,
        app_interface: &mut AppInterface,
        ui: &mut egui::Ui,
        _frame: &mut eframe::Frame,
    ) {
        // Lazy connect / initialization: if not initialized, attempt to connect using central ws
        if !self.initialized {
            // Prepare closure state clones
            let payload = self.qr_payload.clone();
            let players = self.players.clone();
            let our_name_pending = self.our_name_pending.clone();

            let on_msg = move |x: Backend2FrontendMsg| match x {
                Backend2FrontendMsg::TicketValue(ticket) => {
                    sprintln!("Got a ticket value:\n\t- {:?}", ticket);
                    *payload.borrow_mut() = Some(ticket);
                }
                Backend2FrontendMsg::IPValue(ip) => {
                    sprintln!("Got an IP value:\n\t- {:?}", ip);
                    *payload.borrow_mut() = Some(ip);
                }
                Backend2FrontendMsg::NewPlayer(name) => {
                    // add player
                    if !players.borrow().iter().any(|(n, _)| n == &name) {
                        {
                            let mut p = players.borrow_mut();
                            p.push((name.clone(), false));
                        }
                        // Immediately inform backend about our current ready state (first entry = local)
                        let local_ready = players.borrow().first().map(|(_, r)| *r).unwrap_or(false);
                        let msg = Frontend2BackendMsg::ReadyUpdate(local_ready);
                        // send through central ws (can't access it here) -> rely on server behavior:
                        // clients typically send ReadyUpdate after connecting; we'll send explicitly below if already connected
                        sprintln!("Queued ready update after NewPlayer: {}", local_ready);
                    }
                }
                Backend2FrontendMsg::RemovePlayer(name) => {
                    sprintln!("Got a remove player message for: {}", name);
                    players.borrow_mut().retain(|(n, _)| n != &name);
                }
                Backend2FrontendMsg::PlayerReady(name, ready) => {
                    sprintln!("Got a ready update for player {}: {}", name, ready);
                    if let Some((_, r)) = players.borrow_mut().iter_mut().find(|(n, _)| n == &name) {
                        *r = ready;
                    }
                }
                Backend2FrontendMsg::OurName(name) => {
                    sprintln!("Got our player name from backend: {}", name);
                    // Update ourname if we got it from the backend (e.g. after being renamed by a peer)
                    // Only update the first entry which is reserved for the local player
                    if let Some((n, _)) = players.borrow_mut().first_mut() {
                        *n = name.clone();
                    }
                    *our_name_pending.borrow_mut() = Some(name);
                }
                _ => {
                    sprintln!("Got an unhandled message:\n\t- {:?}", x);
                }
            };

            let on_err = move |e: String| {
                sprintln!("Got an error:\n\t- {:?}", e);
            };
            let on_cls = move |c: String| {
                sprintln!("Got a close:\n\t- {:?}", c);
            };

            // attempt to connect using central connection
            {
                let mut players_cfg = Vec::new();
                let p = PlayerConfig {
                    id: mcg_shared::PlayerId::from(1337),
                    name: "Lobby".to_string(),
                    is_bot: false,
                };
                players_cfg.push(p);

                let server = app_interface.state().settings.server_address.clone();
                app_interface.ws.connect(&server, players_cfg, on_msg, on_err, on_cls);
            }
            self.initialized = true;
        }
        if !self.setup && app_interface.ws.is_connected() {
            // now connected: request our name and players explicitly
            let msg = Frontend2BackendMsg::GetOurName;
            app_interface.ws.send_msg(&msg);
            let msg = Frontend2BackendMsg::GetPlayers;
            app_interface.ws.send_msg(&msg);
            self.setup = true;
        }

        // If we got a name from the backend, apply it to our local player entry and settings.
        if let Some(new_name) = self.our_name_pending.borrow_mut().take() {
            app_interface.state().settings.name = new_name;
        }

        // If user set a name in the previous screen, apply it to the local player entry once.
        let chosen_name = app_interface.state().settings.name.clone();
        if !chosen_name.is_empty() {
            // Only add the chosen name once at the start if the list is currently empty
            let mut players_b = self.players.borrow_mut();
            if players_b.is_empty() {
                players_b.push((chosen_name.clone(), false));
            }
        }

        ui.heading("Card Game Lobby");
        ui.add_space(12.0);
        ui.group(|ui| {
            ui.label(RichText::new("Current Players:").strong());

            for (name, ready) in self.players.borrow().iter() {
                ui.horizontal(|ui| {
                    ui.label(name);
                    ui.label(
                        if *ready {
                            RichText::new("Ready").color(egui::Color32::GREEN)
                        } else {
                            RichText::new("Not Ready").color(egui::Color32::RED)
                        }
                    );
                });
            }
        });
        ui.add_space(12.0);
        {
            let is_ready = self.players
                .borrow()
                .iter()
                .find(|(name, _)| *name == chosen_name)
                .map(|(_, ready)| *ready)
                .unwrap_or(false);
            if ui.button(if is_ready { "Unready" } else { "Ready Up" }).clicked() {
                // Toggle ready state for the local player
                let mut players_b = self.players.borrow_mut();
                if let Some((_, ready)) = players_b.iter_mut().find(|(name, _)| *name == chosen_name) {
                    *ready = !*ready;
                    // Send ready state to backend so we can tell the other players
                    let msg = Frontend2BackendMsg::ReadyUpdate(*ready);
                    app_interface.ws.send_msg(&msg);
                }
            }
        }
        if self.players.borrow().len() < 2 {
            // Not enough players to start
            ui.add_space(4.0);
            ui.label(RichText::new("Need at least 2 players to start!").color(egui::Color32::RED));
        }
        else if self.players.borrow().iter().any(|(_, ready)| !*ready) {
            // Not all players are ready
            ui.add_space(4.0);
            ui.label(RichText::new("Waiting for all players to be ready...").color(egui::Color32::YELLOW));
        }
        else {
            // All players are ready, can start the game
            ui.add_space(12.0);
            if ui.button("Start Game").clicked() {
                //TODO
            }
            ui.add_space(4.0);
            ui.label(RichText::new("All players are ready! You can start the game.").color(egui::Color32::GREEN));
        }
        ui.add_space(12.0);
        ui.horizontal(|ui| {
            if ui.button("Generate QR Code and let others scan it to join!").clicked() {
                let msg = Frontend2BackendMsg::GetTicket;
                app_interface.ws.send_msg(&msg);
            }
        });
        ui.add_space(8.0);
        if let Ok(payload_ref) = self.qr_payload.try_borrow() {
            if let Some(payload) = payload_ref.as_ref() {
                let code = QrCode::new(payload.as_bytes()).unwrap();
                let image = code.render::<image::Luma<u8>>().build();
                let texture = egui::ColorImage::from_gray(
                    [image.width() as usize, image.height() as usize],
                    image.as_raw(),
                );
                let texture = ui.ctx().load_texture(
                    "qr_code",
                    texture,
                    TextureOptions::default(),
                );
                ui.image(&texture);
            }
        }
    }

    fn on_exit(&mut self, app_interface: &mut AppInterface) {
        // Disconnect when leaving this screen
        let msg = Frontend2BackendMsg::Disconnect;
        app_interface.ws.send_msg(&msg);
    }
}

impl ScreenDef for LobbyScreen {
    fn metadata() -> ScreenMetadata
    where
        Self: Sized,
    {
        ScreenMetadata {
            path: "/lobbyselect/lobby",
            display_name: "Card Game Lobby",
            icon: "🂱",
            description: "Lobby for online card games",
            show_in_menu: false,
        }
    }

    fn create() -> Box<dyn ScreenWidget>
    where
        Self: Sized,
    {
        let me = Self::default();
        // Do not connect here — connection is created lazily in ui() via AppInterface.ws
        Box::new(me)
    }
}