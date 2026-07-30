use egui::{ComboBox, RichText};
use std::rc::Rc;

use crate::app::FrontendInterface;
use crate::sprintln;
use crate::widgets::qr_scanner::QrScannerPopup;
use crate::widgets::screen::{ScreenDef, ScreenMetadata, ScreenWidget};
use mcg_shared::{Backend2FrontendMsg, Frontend2BackendMsg};
use std::cell::RefCell;
use crate::screens::LobbyScreen;

#[derive(PartialEq, Debug, Clone, Copy, Default)]
pub enum GameType {
    #[default]
    Poker,
    Blackjack,
    // Add more game types here
}

pub struct LobbySelectionScreen {
    pub players: usize,
    pub game_type: GameType,
    input: String,
    scanner: QrScannerPopup,
    name_storage: Rc<RefCell<Option<String>>>,
    raw: Vec<u8>,
    player_name: String,
    initialized: bool,
    switch: Rc<RefCell<bool>>,
    manual_ticket: String,
}

impl Default for LobbySelectionScreen {
    fn default() -> Self {
        Self {
            players: 2,
            game_type: GameType::default(),
            input: String::new(),
            scanner: QrScannerPopup::default(),
            name_storage: Rc::new(RefCell::new(None)),
            raw: Vec::new(),
            player_name: String::new(),
            initialized: false,
            switch: Rc::new(RefCell::new(false)),
            manual_ticket: String::new(),
        }
    }
}

impl ScreenWidget for LobbySelectionScreen {
    fn ui(
        &mut self,
        app_interface: &mut FrontendInterface,
        ui: &mut egui::Ui,
        _frame: &mut eframe::Frame,
    ) {
        let before = self.game_type;
        // If user set a name in the previous screen, apply it to the local player entry once.
        if !self.initialized {
            let global = app_interface.state().name.clone();
            if !global.trim().is_empty() {
                self.player_name = global;
            }

            let server = app_interface.state().server_address.clone();
            if !app_interface.is_connected() {
                app_interface.connect(&server);
            }

            self.initialized = true;
        }

        ui.heading("Host or Join Game");
        ui.group(|ui| {
            // --- First dropdown: Game ---
            ComboBox::from_label("Select Game")
                .selected_text(format!("{:?}", self.game_type))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.game_type, GameType::Poker, "Poker");
                    ui.selectable_value(&mut self.game_type, GameType::Blackjack, "Blackjack");
                });

            // Reset player count if game changes
            if self.game_type != before {
                self.players = match self.game_type {
                    GameType::Poker => 2,
                    GameType::Blackjack => 2,
                };
            }
            // Define valid player counts based on selected game
            let valid_counts: &[usize] = match self.game_type {
                GameType::Poker => &[2, 4, 8],
                GameType::Blackjack => &[2, 3, 4],
            };
            // --- Second dropdown: Players ---
            ComboBox::from_label("Select Player Count")
                .selected_text(self.players.to_string())
                .show_ui(ui, |ui| {
                    for &count in valid_counts {
                        ui.selectable_value(&mut self.players, count, count.to_string());
                    }
                });
            ui.add_space(8.0);
            ui.label(
                RichText::new("Select Your Name (This is used for both hosting and joining):")
                    .strong(),
            );
            ui.add_space(4.0);

            ui.horizontal(|ui| {
                ui.label("Name:");
                ui.text_edit_singleline(&mut self.player_name);
            });
        });
        // Open lobby button
        ui.add_space(8.0);
        if ui.button("Host Game").clicked() {
            // Set max players and player name on the server, then open the lobby
            let msg = Frontend2BackendMsg::PlayerCount(self.players);
            app_interface.send_msg(msg);
            let msg = Frontend2BackendMsg::PlayerName(self.player_name.clone());
            app_interface.send_msg(msg);
            // Persist chosen name into global client state prior to join
            app_interface.state_mut().name = self.player_name.clone();
            match self.game_type {
                GameType::Poker => {
                    // Transition to poker lobby setup
                    eprintln!("Hosting Poker game with max {} players", self.players);
                    let msg = Frontend2BackendMsg::LobbyOpen("Poker".to_string());
                    app_interface.send_msg(msg);
                    app_interface.change_screen::<LobbyScreen>();
                }
                GameType::Blackjack => {
                    // Transition to blackjack lobby setup
                    eprintln!("Hosting Blackjack game with max {} players", self.players);
                    let msg = Frontend2BackendMsg::LobbyOpen("Blackjack".to_string());
                    app_interface.send_msg(msg);
                    app_interface.change_screen::<LobbyScreen>();
                }
            }
        }
        ui.add_space(8.0);
        ui.label("Click 'Host Game' to open your own lobby!");

        ui.add_space(12.0);
        let ctx = ui.ctx().clone();
        self.scanner
            .button_and_popup(ui, &ctx, &mut self.input, &mut self.raw);

        ui.add_space(8.0);
        ui.label("Click 'Scan QR' to connect to another player's lobby by scanning a QR code!");

        // Manually enter a ticket (in case scanner doesn't work).
        ui.add_space(6.0);
        ui.horizontal(|ui| {
            ui.label("Manually enter ticket in case the scanner doesn't work:");
            // keep the text edit compact; let it expand horizontally
            ui.add(egui::TextEdit::singleline(&mut self.manual_ticket).desired_width(360.0));
            if ui.button("Connect").clicked() {
                self.input = self.manual_ticket.trim().to_string();
            }
        });

        // If our input is an endpoint, send it to get a connection
        if self.input.starts_with("endpoint") {
            tracing::info!("Sending endpoint ticket to server: {}", self.input);
            // Persist chosen name into global client state prior to join
            app_interface.state_mut().name = self.player_name.clone();
            let ticket = self.input.clone();
            let msg = Frontend2BackendMsg::PlayerName(self.player_name.clone());
            app_interface.send_msg(msg);
            let msg = Frontend2BackendMsg::QrValue(ticket);
            app_interface.send_msg(msg);
            self.input.clear();
        }

        // Only switch screens if we got accepted into the lobby
        if *self.switch.borrow() {
            app_interface.change_screen::<LobbyScreen>();
        }
        // If we received a new name, update our name both here and
        // in the global state so it persists across screens
        let name_opt = {
            if let Ok(name_ref) = self.name_storage.try_borrow() {
                name_ref.as_ref().cloned()
            } else {
                None
            }
        };

        if let Some(name) = name_opt {
            app_interface.state_mut().name = name.clone();
            self.player_name = name.clone();

            if let Ok(mut storage) = self.name_storage.try_borrow_mut() {
                storage.take();
            }
        }
    }
    fn on_exit(&mut self, app_interface: &mut FrontendInterface) {
        // Persist name when leaving this screen
        app_interface.state_mut().name = self.player_name.clone();
    }
    fn on_message(&mut self, _app_interface: &mut FrontendInterface, message: Backend2FrontendMsg) {
        match message {
            Backend2FrontendMsg::OurName(name) => {
                sprintln!("Got our name from the server:\n\t- {:?}", name);
                *self.name_storage.borrow_mut() = Some(name);
            }
            Backend2FrontendMsg::Pong => {
                *self.switch.borrow_mut() = true;
            }
            _ => {
                sprintln!("Got an unhandled message:\n\t- {:?}", message);
            }
        }
    }
}

impl ScreenDef for LobbySelectionScreen {
    fn metadata() -> ScreenMetadata
    where
        Self: Sized,
    {
        ScreenMetadata {
            path: "/lobbyselect",
            display_name: "Host or Join Game",
            icon: "⚙",
            description:
                "Host your own lobby, or join another player's lobby by scanning a QR code.",
            show_in_menu: true,
        }
    }

    fn create() -> Box<dyn ScreenWidget>
    where
        Self: Sized,
    {
        let me = Self::default();
        // do not connect here — connect in ui() where AppInterface.ws is available
        Box::new(me)
    }
}
