use crate::app::FrontendInterface;
use crate::sprintln;
use egui::{RichText, TextureOptions};
use mcg_shared::{Backend2FrontendMsg, Frontend2BackendMsg};
use qrcode::QrCode;
use std::cell::RefCell;
use std::rc::Rc;
use crate::widgets::screen::{ScreenDef, ScreenMetadata, ScreenWidget};

pub struct LobbyScreen {
    qr_payload: Rc<RefCell<Option<String>>>,
    players: Rc<RefCell<Vec<(String, bool)>>>, // (name, ready)
    our_name_pending: Rc<RefCell<Option<String>>>,
    initialized: bool,
    setup: bool,
    ready_sync: Rc<RefCell<bool>>,
}

impl Default for LobbyScreen {
    fn default() -> Self {
        Self {
            qr_payload: Rc::new(RefCell::new(None)),
            players: Rc::new(RefCell::new(Vec::new())),
            our_name_pending: Rc::new(RefCell::new(None)),
            initialized: false,
            setup: false,
            ready_sync: Rc::new(RefCell::new(false)),
        }
    }
}

impl ScreenWidget for LobbyScreen {
    fn ui(
        &mut self,
        app_interface: &mut FrontendInterface,
        ui: &mut egui::Ui,
        _frame: &mut eframe::Frame,
    ) {
        // Lazy init: connect through the application-owned WebSocket.
        if !self.initialized {
            let server = app_interface.state().server_address.clone();
            if !app_interface.is_connected() {
                app_interface.connect(&server);
            }

            self.initialized = true;
        }

        // When the screen is visible and connected, ensure we have requested initial state
        if !self.setup && app_interface.is_connected() {
            // now connected: request our name and players explicitly
            let msg = Frontend2BackendMsg::GetOurName;
            app_interface.send_msg(msg);
            let msg = Frontend2BackendMsg::GetPlayers;
            app_interface.send_msg(msg);
            self.setup = true;
        }
        if *self.ready_sync.borrow() && app_interface.is_connected() {
            // If we just got a new player and are syncing ready state, send our current ready state to backend
            let local_ready = self
                .players
                .borrow()
                .first()
                .map(|(_, r)| *r)
                .unwrap_or(false);
            let msg = Frontend2BackendMsg::ReadyUpdate(local_ready);
            app_interface.send_msg(msg);
            *self.ready_sync.borrow_mut() = false; // Reset the flag after syncing
        }

        // If we got a name from the backend, apply it to our local player entry and settings.
        if let Some(new_name) = self.our_name_pending.borrow_mut().take() {
            app_interface.state_mut().name = new_name;
        }

        // If user set a name in the previous screen, apply it to the local player entry once.
        let chosen_name = app_interface.state().name.clone();
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
                    ui.label(if *ready {
                        RichText::new("Ready").color(egui::Color32::GREEN)
                    } else {
                        RichText::new("Not Ready").color(egui::Color32::RED)
                    });
                });
            }
        });
        ui.add_space(12.0);
        {
            let is_ready = self
                .players
                .borrow()
                .iter()
                .find(|(name, _)| *name == chosen_name)
                .map(|(_, ready)| *ready)
                .unwrap_or(false);
            if ui
                .button(if is_ready { "Unready" } else { "Ready Up" })
                .clicked()
            {
                // Toggle ready state for the local player
                let mut players_b = self.players.borrow_mut();
                if let Some((_, ready)) =
                    players_b.iter_mut().find(|(name, _)| *name == chosen_name)
                {
                    *ready = !*ready;
                    // Send ready state to backend so we can tell the other players
                    let msg = Frontend2BackendMsg::ReadyUpdate(*ready);
                    app_interface.send_msg(msg);
                }
            }
        }
        if self.players.borrow().len() < 2 {
            // Not enough players to start
            ui.add_space(4.0);
            ui.label(RichText::new("Need at least 2 players to start!").color(egui::Color32::RED));
        } else if self.players.borrow().iter().any(|(_, ready)| !*ready) {
            // Not all players are ready
            ui.add_space(4.0);
            ui.label(
                RichText::new("Waiting for all players to be ready...")
                    .color(egui::Color32::YELLOW),
            );
        } else {
            // All players are ready, can start the game
            ui.add_space(12.0);
            if ui.button("Start Game").clicked() {
                //TODO
            }
            ui.add_space(4.0);
            ui.label(
                RichText::new("All players are ready! You can start the game.")
                    .color(egui::Color32::GREEN),
            );
        }
        ui.add_space(12.0);
        ui.horizontal(|ui| {
            if ui
                .button("Generate QR Code and let others scan it to join!")
                .clicked()
            {
                let msg = Frontend2BackendMsg::GetTicket;
                app_interface.send_msg(msg);
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
                let texture = ui
                    .ctx()
                    .load_texture("qr_code", texture, TextureOptions::default());
                ui.image(&texture);
            }
        }
    }

    fn on_exit(&mut self, app_interface: &mut FrontendInterface) {
        // Tell others we wish to disconnect
        let msg = Frontend2BackendMsg::Disconnect;
        app_interface.send_msg(msg);
    }

    fn on_message(&mut self, _app_interface: &mut FrontendInterface, message: Backend2FrontendMsg) {
        match message {
            Backend2FrontendMsg::TicketValue(ticket) => {
                sprintln!("Got a ticket value:\n\t- {:?}", ticket);
                *self.qr_payload.borrow_mut() = Some(ticket);
            }
            Backend2FrontendMsg::IPValue(ip) => {
                sprintln!("Got an IP value:\n\t- {:?}", ip);
                *self.qr_payload.borrow_mut() = Some(ip);
            }
            Backend2FrontendMsg::NewPlayer(name) => {
                if !self.players.borrow().iter().any(|(n, _)| n == &name) {
                    self.players.borrow_mut().push((name, false));
                    *self.ready_sync.borrow_mut() = true;
                }
            }
            Backend2FrontendMsg::RemovePlayer(name) => {
                sprintln!("Got a remove player message for: {}", name);
                self.players.borrow_mut().retain(|(n, _)| n != &name);
            }
            Backend2FrontendMsg::PlayerReady(name, ready) => {
                sprintln!("Got a ready update for player {}: {}", name, ready);
                if let Some((_, current)) = self
                    .players
                    .borrow_mut()
                    .iter_mut()
                    .find(|(n, _)| n == &name)
                {
                    *current = ready;
                }
            }
            Backend2FrontendMsg::OurName(name) => {
                sprintln!("Got our player name from backend: {}", name);
                if let Some((current, _)) = self.players.borrow_mut().first_mut() {
                    *current = name.clone();
                }
                *self.our_name_pending.borrow_mut() = Some(name);
            }
            other => sprintln!("Got an unhandled message:\n\t- {:?}", other),
        }
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
