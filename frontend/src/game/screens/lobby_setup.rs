use eframe::Frame;
use egui::{vec2, Align, UiBuilder, Layout, ComboBox, RichText};
use std::rc::Rc;

use super::{AppInterface, ScreenDef, ScreenMetadata, ScreenWidget};
use crate::game::GameType;
use crate::game::websocket::WebSocketConnection;
use mcg_shared::{Frontend2BackendMsg, PlayerConfig, PlayerId, Backend2FrontendMsg};
use crate::sprintln;
use std::cell::RefCell;
use crate::qr_scanner::QrScannerPopup;


pub struct LobbySelectionScreen {
    pub players: usize,
    pub game_type: GameType,
    input: String,
    scanner: QrScannerPopup,
    web_socket_connection: WebSocketConnection,
    qr_payload: Rc<RefCell<Option<String>>>,
    raw: Vec<u8>,
    player_name: String,
    initialized: bool,
}

impl Default for LobbySelectionScreen {
    fn default() -> Self {
        Self {
            players: 2,
            game_type: GameType::default(),
            input: String::new(),
            scanner: QrScannerPopup::default(),
            web_socket_connection: WebSocketConnection::default(),
            qr_payload: Rc::new(RefCell::new(None)),
            raw: Vec::new(),
            player_name: String::new(),
            initialized: false,
        }
    }
}

impl ScreenWidget for LobbySelectionScreen {
    fn ui(
        &mut self,
        app_interface: &mut AppInterface,
        ui: &mut egui::Ui,
        _frame: &mut eframe::Frame,
    ) {
        let before = self.game_type;
        // If user set a name in the previous screen, apply it to the local player entry once.
        if !self.initialized {
            let global = app_interface.state().settings.name.clone();
            if !global.trim().is_empty() {
                self.player_name = global;
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
            ui.label(RichText::new("Select Your Name (This is used for both hosting and joining):").strong());
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
            self.web_socket_connection.send_msg(&msg);
            let msg = Frontend2BackendMsg::PlayerName(self.player_name.clone());
            self.web_socket_connection.send_msg(&msg);
            let msg = Frontend2BackendMsg::LobbyOpen;
            self.web_socket_connection.send_msg(&msg);
            // Persist chosen name into global client state prior to join
            app_interface.state().settings.name = self.player_name.clone();
            match self.game_type {
                GameType::Poker => {
                    // Transition to poker lobby setup
                    eprintln!("Hosting Poker game with max {} players", self.players);
                    //TODO: Make a new poker screen that takes players as a param and transitions to it here
                    //based on the PokerOnlineScreen with the QR code generation thing? To use later when I
                    //add the actual online multiplayer functionality
                    app_interface.queue_event(crate::game::AppEvent::ChangeRoute("/lobbyselect/pokerlobby".to_string()));
                }
                GameType::Blackjack => {
                    // Transition to blackjack lobby setup
                    eprintln!("Hosting Blackjack game with max {} players", self.players);
                    // We dont have blackjack implemented, so this is just a dummy for testing's sake.
                    // In the future, this would change to a screen like the Poker one.
                }
            }
        }
        ui.add_space(8.0);
        ui.label("Click 'Host Game' to open your own lobby!");

        ui.add_space(12.0);
        let ctx = ui.ctx().clone();
        self.scanner.button_and_popup(ui, &ctx, &mut self.input, &mut self.raw);

        ui.add_space(8.0);
        ui.label("Click 'Scan QR' to connect to another player's lobby by scanning a QR code!");
        // If our input is an endpoint, send it to get a connection
        if self.input.starts_with("endpoint"){
            tracing::info!("Sending endpoint ticket to server: {}", self.input);
            // Persist chosen name into global client state prior to join
            app_interface.state().settings.name = self.player_name.clone();
            let ticket = self.input.clone();
            let msg = Frontend2BackendMsg::PlayerName(self.player_name.clone());
            self.web_socket_connection.send_msg(&msg);
            let msg = Frontend2BackendMsg::QrValue(ticket);
            self.web_socket_connection.send_msg(&msg);
            self.input.clear();

        
        }
    }
    fn on_exit(&mut self, app_interface: &mut AppInterface) {
        // Disconnect when leaving this screen
        app_interface.state().settings.name = self.player_name.clone();
        self.web_socket_connection.close();
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
            description: "Host your own lobby, or join another player's lobby by scanning a QR code.",
            show_in_menu: true,
        }
    }

    fn create() -> Box<dyn ScreenWidget>
    where
        Self: Sized,
    {
        let mut me = Self::default();
        let payload = me.qr_payload.clone();
        let on_msg = move |x| match x {
            Backend2FrontendMsg::State(s) => {
                sprintln!("Got a message state:\n\t- {:?}", s);
            }
            Backend2FrontendMsg::Error(e) => {
                sprintln!("Got a message error:\n\t- {:?}", e);
            }
            Backend2FrontendMsg::Pong => {
                sprintln!("Got a pong message");
            }
            Backend2FrontendMsg::TicketValue(ticket) => {
                sprintln!("Got a ticket value:\n\t- {:?}", ticket);
                *payload.borrow_mut() = Some(ticket);
            }
            Backend2FrontendMsg::IPValue(ip) => {
                sprintln!("Got an IP value:\n\t- {:?}", ip);
                *payload.borrow_mut() = Some(ip);
            }
            Backend2FrontendMsg::QrRes(_) => {
                todo!("Handle QR result from server");
            }
            Backend2FrontendMsg::NewPlayer(_name) => {
                sprintln!("Got a new player message");
            }
            Backend2FrontendMsg::OurName(name) => {
                sprintln!("Got our name message:\n\t- {:?}", name);
            }
            Backend2FrontendMsg::RemovePlayer(_name) => {
                sprintln!("Got a remove player message");
            }
        };
        let on_err = |e| {
            sprintln!("Got an error:\n\t- {:?}", e);
        };
        let on_cls = |c| {
            sprintln!("Got a close:\n\t- {:?}", c);
        };
        let mut players = Vec::new();
        let p = PlayerConfig {
            id: PlayerId::from(1337),
            name: "Select_Lobby".to_string(),
            is_bot: false,
        };
        players.push(p);
        me.web_socket_connection
            .connect("127.0.0.1:3000", players, on_msg, on_err, on_cls);
        Box::new(me)
    }
}