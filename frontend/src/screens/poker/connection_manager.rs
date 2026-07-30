use crate::app::AppInterface;
use crate::store::{ClientState, ConnectionStatus};
use crate::widgets::qr_scanner::QrScannerPopup;
use egui::{Color32, Context, RichText, Ui};

pub struct ConnectionManager {
    edit_server_address: String,
    qr_result_raw: Vec<u8>,
    scanner: QrScannerPopup,
}

impl ConnectionManager {
    pub fn new(server_address: String) -> Self {
        Self {
            edit_server_address: server_address,
            qr_result_raw: Vec::new(),
            scanner: QrScannerPopup::default(),
        }
    }

    pub fn connect(&mut self, app_interface: &mut AppInterface) {
        {
            let app_state = app_interface.state_mut();
            app_state.connection.connection_status = ConnectionStatus::Connecting;
            app_state.ui.last_error = None;
            app_state.ui.last_info = Some(format!("Connecting to {}...", self.edit_server_address));
            app_state.settings.server_address = self.edit_server_address.clone();
        }
        app_interface.connect(&self.edit_server_address);
    }

    pub fn render_header(&mut self, app_state: &mut ClientState, ui: &mut Ui, ctx: &Context) {
        ui.horizontal(|ui| {
            ui.heading("Poker Online");
            ui.add_space(16.0);
            if let Some(s) = &app_state.session.game_state {
                ui.label(super::ui_components::stage_badge(s.stage));
                ui.add_space(8.0);
            }
        });

        let default_open = app_state.session.game_state.is_none();
        egui::CollapsingHeader::new("Connection & session")
            .default_open(default_open)
            .show(ui, |ui| {
                let mut connect_clicked = false;
                let mut disconnect_clicked = false;
                self.render_connection_controls(
                    app_state,
                    ui,
                    ctx,
                    &mut connect_clicked,
                    &mut disconnect_clicked,
                );
            });

        egui::CollapsingHeader::new("Player Setup")
            .default_open(false)
            .show(ui, |ui| {
                super::player_manager::render_player_setup(ui, ctx);
            });

        if let Some(err) = &app_state.ui.last_error {
            ui.colored_label(Color32::RED, err);
        }
        if let Some(info) = &app_state.ui.last_info {
            ui.label(RichText::new(info));
        }
        ui.separator();
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
