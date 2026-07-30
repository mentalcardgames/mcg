use crate::app::AppInterface;
use crate::store::ClientState;
use crate::widgets::qr_scanner::QrScannerPopup;
use egui::{Context, Ui};

pub struct ConnectionManager {
    edit_server_address: String,
    qr_result_raw: Vec<u8>,
    scanner: QrScannerPopup,
    pub(crate) last_error: Option<String>,
    pub(crate) last_info: Option<String>,
}

impl ConnectionManager {
    pub fn new(server_address: String) -> Self {
        Self {
            edit_server_address: server_address,
            qr_result_raw: Vec::new(),
            scanner: QrScannerPopup::default(),
            last_error: None,
            last_info: None,
        }
    }

    pub fn connect(&mut self, app_interface: &mut AppInterface) {
        {
            let app_state = app_interface.state_mut();
            self.last_error = None;
            self.last_info = Some(format!("Connecting to {}...", self.edit_server_address));
            app_state.settings.server_address = self.edit_server_address.clone();
        }
        app_interface.connect(&self.edit_server_address);
    }

    pub fn render_connection_controls(
        &mut self,
        _app_state: &mut ClientState,
        ui: &mut Ui,
        ctx: &Context,
        connect_clicked: &mut bool,
        disconnect_clicked: &mut bool,
    ) {
        let narrow = ui.available_width() < 900.0;
        if narrow {
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    if ui.button("Connect").clicked() {
                        *connect_clicked = true;
                    }
                    if ui.button("Disconnect").clicked() {
                        *disconnect_clicked = true;
                    }
                });
                ui.horizontal(|ui| {
                    ui.label("Server:");
                    ui.text_edit_singleline(&mut self.edit_server_address)
                        .on_hover_text("Server address (IP:PORT)");
                    self.scanner.button_and_popup(
                        ui,
                        ctx,
                        &mut self.edit_server_address,
                        &mut self.qr_result_raw,
                    );
                });
            });
        } else {
            ui.horizontal(|ui| {
                ui.label("Server:");
                ui.text_edit_singleline(&mut self.edit_server_address)
                    .on_hover_text("Server address (IP:PORT)");
                self.scanner.button_and_popup(
                    ui,
                    ctx,
                    &mut self.edit_server_address,
                    &mut self.qr_result_raw,
                );
                ui.add_space(12.0);
                if ui.button("Connect").clicked() {
                    *connect_clicked = true;
                }
                if ui.button("Disconnect").clicked() {
                    *disconnect_clicked = true;
                }
            });
        }
    }
}
