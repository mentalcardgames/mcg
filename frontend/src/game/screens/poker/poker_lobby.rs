use crate::game::{AppInterface, ScreenWidget};
use crate::game::screens::{ScreenDef, ScreenMetadata};
use egui::{vec2, ColorImage, Context, Image, TextureHandle, TextureOptions, RichText};
use image::{ImageBuffer, Luma};
use crate::game::websocket::WebSocketConnection;
use mcg_shared::{Frontend2BackendMsg, PlayerConfig, PlayerId, Backend2FrontendMsg};
use crate::sprintln;
use qrcode::QrCode;
use std::cell::RefCell;
use std::rc::Rc;
use super::player_manager::PlayerManager;

#[derive(Default)]
pub struct PokerLobbyScreen {
    input: String,
    web_socket_connection: WebSocketConnection,
    qr_payload: Rc<RefCell<Option<String>>>,
    raw: Vec<u8>,
    player_manager: Rc<RefCell<PlayerManager>>,
}

impl ScreenWidget for PokerLobbyScreen {
    fn ui(
        &mut self,
        _app_interface: &mut AppInterface,
        ui: &mut egui::Ui,
        _frame: &mut eframe::Frame,
    ) {
        ui.heading("Poker Lobby");
        ui.add_space(12.0);
        ui.group(|ui| {
            ui.label(RichText::new("Current Players:").strong());
            for player in self.player_manager.borrow().get_players() {
                ui.label(&player.name);
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
}

impl ScreenDef for PokerLobbyScreen {
    fn metadata() -> ScreenMetadata
    where
        Self: Sized,
    {
        ScreenMetadata {
            path: "/lobbyselect/pokerlobby",
            display_name: "Poker Lobby",
            icon: "🂱",
            description: "Lobby for online Poker",
            show_in_menu: false,
        }
    }

    fn create() -> Box<dyn ScreenWidget>
    where
        Self: Sized,
    {
        let mut me = Self::default();
        let payload = me.qr_payload.clone();
        let pm = me.player_manager.clone();
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
                pm.borrow_mut().handle_named_player(name);
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
            name: "Poker_Lobby".to_string(),
            is_bot: false,
        };
        players.push(p);
        me.web_socket_connection
            .connect("127.0.0.1:3000", players, on_msg, on_err, on_cls);
        Box::new(me)
    }
}