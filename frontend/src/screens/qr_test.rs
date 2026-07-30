use crate::app::AppInterface;
use crate::sprintln;
use crate::widgets::qr_scanner::QrScannerPopup;
use crate::widgets::screen::{ScreenDef, ScreenMetadata, ScreenWidget};
use egui::TextureOptions;
use mcg_shared::{Backend2FrontendMsg, Frontend2BackendMsg};
use qrcode::QrCode;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Default)]
pub struct QrScreen {
    input: String,
    scanner: QrScannerPopup,
    qr_payload: Rc<RefCell<Option<String>>>,
    raw: Vec<u8>,
    initialized: bool,
}

impl ScreenWidget for QrScreen {
    fn ui(
        &mut self,
        app_interface: &mut AppInterface,
        ui: &mut egui::Ui,
        _frame: &mut eframe::Frame,
    ) {
        let ctx = ui.ctx().clone();

        // Lazy connect using central WebSocket
        if !self.initialized {
            let server = app_interface.state().settings.server_address.clone();
            if !app_interface.is_connected() {
                app_interface.connect(&server);
            }

            self.initialized = true;
        }

        ui.heading("QR Scanner Demo");
        ui.add_space(12.0);
        ui.horizontal(|ui| {
            self.scanner
                .button_and_popup(ui, &ctx, &mut self.input, &mut self.raw);
            if ui.button("Generate Endpoint Ticket QR Code").clicked() {
                let msg = Frontend2BackendMsg::GetTicket;
                app_interface.send_msg(msg);
            }
            if ui.button("Generate Local IP QR Code").clicked() {
                let msg = Frontend2BackendMsg::GetIP;
                app_interface.send_msg(msg);
            }
        });
        ui.add_space(8.0);
        ui.label("Tip: Click 'Scan QR' to fill this field from a QR code.");
        // If our input is an endpoint, send it to get a connection
        if self.input.starts_with("endpoint") {
            tracing::info!("Sending endpoint ticket to server: {}", self.input);
            let ticket = self.input.clone();
            let msg = Frontend2BackendMsg::QrValue(ticket);
            app_interface.send_msg(msg);
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
                let texture = ui
                    .ctx()
                    .load_texture("qr_code", texture, TextureOptions::default());
                ui.image(&texture);
            }
        }
    }

    fn on_message(&mut self, _app_interface: &mut AppInterface, message: Backend2FrontendMsg) {
        match message {
            Backend2FrontendMsg::TicketValue(ticket) => {
                sprintln!("Got a ticket value:\n\t- {:?}", ticket);
                *self.qr_payload.borrow_mut() = Some(ticket);
            }
            Backend2FrontendMsg::IPValue(ip) => {
                sprintln!("Got an IP value:\n\t- {:?}", ip);
                *self.qr_payload.borrow_mut() = Some(ip);
            }
            other => sprintln!("Got an unhandled message:\n\t- {:?}", other),
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
        Box::new(Self::default())
    }
}
