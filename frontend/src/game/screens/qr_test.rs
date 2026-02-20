use super::{AppInterface, ScreenDef, ScreenMetadata, ScreenWidget};
use crate::qr_scanner::QrScannerPopup;
use egui::{vec2, ColorImage, Context, Image, TextureHandle, TextureOptions};
use image::{ImageBuffer, Luma};
use crate::game::websocket::WebSocketConnection;
use mcg_shared::{ClientMsg, PlayerConfig, PlayerId, ServerMsg};
use crate::sprintln;
use qrcode::QrCode;
use std::cell::RefCell;
use std::rc::Rc;


pub struct QrScreen {
    input: String,
    scanner: QrScannerPopup,
    web_socket_connection: WebSocketConnection,
    qr_payload: Rc<RefCell<Option<String>>>,
}

impl QrScreen {
    pub fn new() -> Self {
        Self {
            input: String::new(),
            scanner: QrScannerPopup::new(),
            web_socket_connection: WebSocketConnection::new(),
            qr_payload: Rc::new(RefCell::new(None)),
        }
    }
}

impl Default for QrScreen {
    fn default() -> Self {
        Self::new()
    }
}

impl ScreenWidget for QrScreen {
    fn ui(
        &mut self,
        _app_interface: &mut AppInterface,
        ui: &mut egui::Ui,
        _frame: &mut eframe::Frame,
    ) {
        let ctx = ui.ctx().clone();
        ui.heading("QR Scanner Demo");
        ui.add_space(12.0);
        ui.horizontal(|ui| {
            self.scanner.button_and_popup(ui, &ctx, &mut self.input);
            if ui.button("Generate Endpoint Ticket QR Code").clicked() {
                let msg = ClientMsg::GetTicket;
                self.web_socket_connection.send_msg(&msg);
            }
            if ui.button("Generate Local IP QR Code").clicked() {
                let msg = ClientMsg::GetIP;
                self.web_socket_connection.send_msg(&msg);
            }
        });
        ui.add_space(8.0);
        ui.label("Tip: Click 'Scan QR' to fill this field from a QR code.");
        // If our input is an endpoint, send it to get a connection
        if self.input.starts_with("endpoint"){
            let ticket = self.input.clone();
            let msg = ClientMsg::QrValue(ticket);
            self.web_socket_connection.send_msg(&msg);
            self.input.clear();
        }
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

impl ScreenDef for QrScreen {
    fn metadata() -> ScreenMetadata
    where
        Self: Sized,
    {
        ScreenMetadata {
            path: "/qr",
            display_name: "QR Demo",
            icon: "🔍",
            description: "Scan QR codes into an input",
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
            ServerMsg::State(s) => {
                sprintln!("Got a message state:\n\t- {:?}", s);
            }
            ServerMsg::Error(e) => {
                sprintln!("Got a message error:\n\t- {:?}", e);
            }
            ServerMsg::Pong => {
                sprintln!("Got a pong message");
            }
            ServerMsg::TicketValue(ticket) => {
                sprintln!("Got a ticket value:\n\t- {:?}", ticket);
                *payload.borrow_mut() = Some(ticket);
            }
            ServerMsg::IPValue(ip) => {
                sprintln!("Got an IP value:\n\t- {:?}", ip);
                *payload.borrow_mut() = Some(ip);
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
            name: "QR_Lobby".to_string(),
            is_bot: false,
        };
        players.push(p);
        me.web_socket_connection
            .connect("127.0.0.1:3000", players, on_msg, on_err, on_cls);
        Box::new(me)
    }
}