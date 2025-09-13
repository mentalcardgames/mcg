use crate::game::websocket::WebSocketConnection;
use crate::qr_scanner::QrScannerPopup;
use crate::store::{AppState, ConnectionStatus};
use egui::{Color32, Context, RichText, Ui};
use mcg_shared::PlayerConfig;

pub struct ConnectionManager {
    edit_server_address: String,
    scanner: QrScannerPopup,
}

impl ConnectionManager {
    pub fn new(server_address: String) -> Self {
        Self {
            edit_server_address: server_address,
            scanner: QrScannerPopup::new(),
        }
    }

    pub fn connect(
        &mut self,
        conn: &mut WebSocketConnection,
        app_state: &mut AppState,
        ctx: &Context,
        players: Vec<PlayerConfig>,
    ) {
        app_state.connection_status = ConnectionStatus::Connecting;
        app_state.last_error = None;
        app_state.last_info = Some(format!("Connecting to {}...", self.edit_server_address));
        app_state.settings.server_address = self.edit_server_address.clone();

        let ctx_for_callback = ctx.clone();
        let app_state_ptr = std::rc::Rc::new(std::cell::RefCell::new(app_state as *mut AppState));

        // Clone for each closure to avoid move issues
        let ctx_for_msg = ctx_for_callback.clone();
        let app_state_ptr_for_msg = app_state_ptr.clone();
        let ctx_for_error = ctx_for_callback.clone();
        let app_state_ptr_for_error = app_state_ptr.clone();
        let ctx_for_close = ctx_for_callback.clone();
        let app_state_ptr_for_close = app_state_ptr.clone();

        conn.connect(
            &self.edit_server_address,
            players,
            move |msg: mcg_shared::ServerMsg| {
                // handle_msg - immediate processing
                unsafe {
                    if let Some(app_state) = app_state_ptr_for_msg.borrow_mut().as_mut() {
                        app_state.apply_server_msg(msg);
                        ctx_for_msg.request_repaint(); // immediate UI update
                    }
                }
            },
            move |error: String| {
                // handle error
                unsafe {
                    if let Some(app_state) = app_state_ptr_for_error.borrow_mut().as_mut() {
                        app_state.last_error = Some(error);
                        app_state.connection_status = ConnectionStatus::Disconnected;
                        ctx_for_error.request_repaint();
                    }
                }
            },
            move |reason: String| {
                // handle close
                unsafe {
                    if let Some(app_state) = app_state_ptr_for_close.borrow_mut().as_mut() {
                        app_state.connection_status = ConnectionStatus::Disconnected;
                        app_state.last_error = Some(reason);
                        ctx_for_close.request_repaint();
                    }
                }
            },
        );
    }

    pub fn render_header(&mut self, app_state: &mut AppState, ui: &mut Ui, ctx: &Context) {
        // Title row with current stage badge
        ui.horizontal(|ui| {
            ui.heading("Poker Online");
            ui.add_space(16.0);
            if let Some(s) = &app_state.game_state {
                ui.label(super::ui_components::stage_badge(s.stage));
                ui.add_space(8.0);
            }
        });

        // Collapsible connection & session controls
        let default_open = app_state.game_state.is_none();
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

        // Collapsible player setup section
        egui::CollapsingHeader::new("Player Setup")
            .default_open(false)
            .show(ui, |ui| {
                super::player_manager::render_player_setup(ui, ctx);
            });

        if let Some(err) = &app_state.last_error {
            ui.colored_label(Color32::RED, err);
        }
        if let Some(info) = &app_state.last_info {
            ui.label(RichText::new(info));
        }
        ui.separator();
    }

    pub fn render_connection_controls(
        &mut self,
        _app_state: &mut AppState,
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
                    self.scanner
                        .button_and_popup(ui, ctx, &mut self.edit_server_address);
                });
            });
        } else {
            ui.horizontal(|ui| {
                ui.label("Server:");
                ui.text_edit_singleline(&mut self.edit_server_address)
                    .on_hover_text("Server address (IP:PORT)");
                self.scanner
                    .button_and_popup(ui, ctx, &mut self.edit_server_address);
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
