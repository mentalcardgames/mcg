use crate::game::websocket::WebSocketConnection;
use crate::qr_scanner::QrScannerPopup;
use crate::store::{ClientState, ConnectionStatus};
use egui::{Color32, Context, RichText, Ui};
use mcg_shared::{PlayerConfig, ServerMsg};
use std::collections::VecDeque;

pub struct ConnectionManager {
    edit_server_address: String,
    qr_result_raw: Vec<u8>,
    scanner: QrScannerPopup,
    message_queue: Option<std::rc::Rc<std::cell::RefCell<VecDeque<ServerMsg>>>>,
    error_queue: Option<std::rc::Rc<std::cell::RefCell<VecDeque<String>>>>,
}

impl ConnectionManager {
    pub fn new(server_address: String) -> Self {
        Self {
            edit_server_address: server_address,
            qr_result_raw: Vec::new(),
            scanner: QrScannerPopup::new(),
            message_queue: None,
            error_queue: None,
        }
    }

    pub fn connect(
        &mut self,
        conn: &mut WebSocketConnection,
        app_state: &mut ClientState,
        ctx: &Context,
        players: Vec<PlayerConfig>,
    ) {
        app_state.connection.connection_status = ConnectionStatus::Connecting;
        app_state.ui.last_error = None;
        app_state.ui.last_info = Some(format!("Connecting to {}...", self.edit_server_address));
        app_state.settings.server_address = self.edit_server_address.clone();

        // Create a shared message queue using Rc<RefCell<VecDeque<ServerMsg>>>
        let message_queue =
            std::rc::Rc::new(std::cell::RefCell::new(std::collections::VecDeque::<
                mcg_shared::ServerMsg,
            >::new()));
        let error_queue = std::rc::Rc::new(std::cell::RefCell::new(std::collections::VecDeque::<
            String,
        >::new()));

        // Clone queues and context for each closure
        let msg_queue_for_msg = message_queue.clone();
        let error_queue_for_error = error_queue.clone();
        let error_queue_for_close = error_queue.clone();
        let ctx_for_msg = ctx.clone();
        let ctx_for_error = ctx.clone();
        let ctx_for_close = ctx.clone();

        conn.connect(
            &self.edit_server_address,
            players,
            move |msg: mcg_shared::ServerMsg| {
                // Queue the message safely
                if let Ok(mut queue) = msg_queue_for_msg.try_borrow_mut() {
                    queue.push_back(msg);
                    ctx_for_msg.request_repaint();
                }
            },
            move |error: String| {
                // Queue the error safely
                if let Ok(mut queue) = error_queue_for_error.try_borrow_mut() {
                    queue.push_back(error);
                    ctx_for_error.request_repaint();
                }
            },
            move |reason: String| {
                // Queue the close reason safely
                if let Ok(mut queue) = error_queue_for_close.try_borrow_mut() {
                    queue.push_back(reason);
                    ctx_for_close.request_repaint();
                }
            },
        );

        // Store the queues for processing in the update loop
        self.message_queue = Some(message_queue);
        self.error_queue = Some(error_queue);
    }

    /// Process any queued messages from WebSocket callbacks
    pub fn dispatch_queued_messages(&mut self, app_state: &mut ClientState) {
        if let Some(queue) = &self.message_queue {
            if let Ok(mut q) = queue.try_borrow_mut() {
                while let Some(msg) = q.pop_front() {
                    app_state.apply_server_msg(msg);
                }
            }
        }

        if let Some(queue) = &self.error_queue {
            if let Ok(mut q) = queue.try_borrow_mut() {
                while let Some(error) = q.pop_front() {
                    app_state.ui.last_error = Some(error);
                    app_state.connection.connection_status = ConnectionStatus::Disconnected;
                }
            }
        }
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
