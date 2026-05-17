use crate::game::{AppInterface, ScreenWidget};
use super::{ScreenDef, ScreenMetadata};
use egui::{vec2, ColorImage, Context, Image, TextureHandle, TextureOptions, RichText};
use image::{ImageBuffer, Luma};
use crate::game::websocket::WebSocketConnection;
use mcg_shared::{Frontend2BackendMsg, PlayerConfig, Backend2FrontendMsg};
use crate::sprintln;
use qrcode::QrCode;
use std::cell::RefCell;
use std::rc::Rc;

pub struct LobbyScreen {
    web_socket_connection: WebSocketConnection,
    qr_payload: Rc<RefCell<Option<String>>>,
    player_names: Rc<RefCell<Vec<String>>>,
}

impl Default for LobbyScreen {
    fn default() -> Self {
        let initial_names: Vec<String> = Vec::new(); // start empty, will push local name once
        Self {
            web_socket_connection: WebSocketConnection::default(),
            qr_payload: Rc::new(RefCell::new(None)),
            player_names: Rc::new(RefCell::new(initial_names)),
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
        // If user set a name in the previous screen, apply it to the local player entry once.
        let chosen_name = app_interface.state().settings.name.clone();
        if !chosen_name.is_empty() {
            // Only add the chosen name once at the start if the list is currently empty
            let mut names_b = self.player_names.borrow_mut();
            if names_b.is_empty(){
                names_b.insert(0, chosen_name.clone());
            }
        }

        ui.heading("Card Game Lobby");
        ui.add_space(12.0);
        ui.group(|ui| {
            ui.label(RichText::new("Current Players:").strong());
            for name in self.player_names.borrow().iter() {
                ui.label(name);
            }
        });
        ui.add_space(12.0);
        if ui.button("Start Game").clicked() {
            //TODO
        }
        ui.add_space(12.0);
        ui.horizontal(|ui| {
            if ui.button("Generate QR Code and let others scan it to join!").clicked() {
                let msg = Frontend2BackendMsg::GetTicket;
                self.web_socket_connection.send_msg(&msg);
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

    fn on_exit(&mut self, _app_interface: &mut AppInterface) {
        // Disconnect when leaving this screen
        let msg = Frontend2BackendMsg::Disconnect;
        self.web_socket_connection.send_msg(&msg);
        self.web_socket_connection.close();
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
        let mut me = Self::default();
        let payload = me.qr_payload.clone();
        let names = me.player_names.clone();

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
            Backend2FrontendMsg::NewPlayer(name) => {
                names.borrow_mut().push(name);
            }
            Backend2FrontendMsg::OurName(name) => {
                tracing::info!("Player name received: {}", name);
            }
            Backend2FrontendMsg::RemovePlayer(name) => {
                sprintln!("Got a remove player message for: {}", name);
                names.borrow_mut().retain(|n| n != &name);
            }
        };
        let on_err = |e| {
            sprintln!("Got an error:\n\t- {:?}", e);
        };
        let on_cls = |c| {
            sprintln!("Got a close:\n\t- {:?}", c);
        };

        // initial connection data still uses PlayerConfig for the websocket API,
        // but internal state keeps only names.
        let mut players = Vec::new();
        let p = PlayerConfig {
            id: mcg_shared::PlayerId::from(1337),
            name: "Lobby".to_string(),
            is_bot: false,
        };
        players.push(p);
        me.web_socket_connection
            .connect("127.0.0.1:3000", players, on_msg, on_err, on_cls);
        Box::new(me)
    }
}