pub mod websocket;
pub mod state;

use crate::app::websocket::{MessageSender, WebSocketConnection};
use crate::router::Router;
use crate::screens::game::GameState;
use crate::screens::{Game, LobbySelectionScreen, MainMenu};
use crate::widgets::card::DirectoryCardType;
use crate::widgets::screen::{ScreenDef, ScreenId, ScreenRegistry, ScreenWidget};
use crate::widgets::theme::*;
use crate::sprintln;
use egui::Context;
use mcg_shared::{Backend2FrontendMsg, Frontend2BackendMsg, PlayerConfig};
use std::sync::mpsc::{self, Receiver};
use crate::app::state::FrontendState;

/// Events that can be sent between screens
#[derive(Debug, Clone)]
pub enum FrontendEvent {
    ChangeScreen(ScreenId),
    StartGame(GameState<DirectoryCardType>),
    ExitGame,
}

pub struct FrontendInterface<'a> {
    events: &'a mut Vec<FrontendEvent>,
    app_state: &'a mut FrontendState,
    ws: &'a mut WebSocketConnection,
}
impl<'a> FrontendInterface<'a> {
    pub fn new(
        events: &'a mut Vec<FrontendEvent>,
        client_state: &'a mut FrontendState,
        websocket: &'a mut WebSocketConnection,
    ) -> Self {
        Self {
            events,
            app_state: client_state,
            ws: websocket,
        }
    }
    pub fn state(&mut self) -> &FrontendState {
        self.app_state
    }
    pub fn state_mut(&mut self) -> &mut FrontendState {
        self.app_state
    }
    pub fn change_screen<T: ScreenDef + 'static>(&mut self) {
        self.change_screen_id(ScreenId::of::<T>());
    }
    pub(crate) fn change_screen_id(&mut self, screen: ScreenId) {
        self.events
            .push(FrontendEvent::ChangeScreen(screen));
    }
    pub fn send_msg(&mut self, msg: Frontend2BackendMsg) {
        self.ws.send_msg(msg);
    }
    /// This starts a drag and drop app
    pub fn start_game(&mut self, config: GameState<DirectoryCardType>) {
        self.events.push(FrontendEvent::StartGame(config));
    }
    /// This starts the static poker implementation
    pub fn create_game(&mut self, config: Vec<PlayerConfig>) {
        self.ws.create_game(config)
    }
    pub fn exit_game(&mut self) {
        self.events.push(FrontendEvent::ExitGame);
    }
    pub fn is_connected(&self) -> bool {
        self.ws.is_connected()
    }
    pub fn connect(&mut self, address: &str) {
        self.ws.connect(address)
    }
    pub fn close_connection(&mut self) {
        self.ws.close();
    }
    pub fn message_sender(&mut self) -> &dyn MessageSender {
        &*self.ws
    }
}

/// Application UI/Screen manager
pub struct FrontendApp {
    // current screen type
    current_screen_id: ScreenId,
    // lazily-created screens by type
    screens: std::collections::HashMap<ScreenId, Box<dyn ScreenWidget>>,

    // Global settings UI state
    settings_open: bool,
    app_state: FrontendState,

    // Router for URL handling
    router: Option<Router>,

    ws_connection: websocket::WebSocketConnection,
    message_receiver: Receiver<Backend2FrontendMsg>,
    error_receiver: Receiver<web_sys::Event>,
    close_receiver: Receiver<web_sys::CloseEvent>,
}

impl FrontendApp {
    pub fn new(egui_ctx: Context) -> Self {
        let router = Router::new().ok();

        let current_path = router.as_ref().map(|r| r.current_path()).unwrap_or("/");
        let screen_registry = ScreenRegistry::new();
        let current_screen_id = screen_registry
            .id_by_path(current_path)
            .unwrap_or_else(ScreenId::of::<MainMenu>);

        let app_state = FrontendState::new();
        let (message_sender, message_receiver) = mpsc::channel();
        let (error_sender, error_receiver) = mpsc::channel();
        let (close_sender, close_receiver) = mpsc::channel();
        Self {
            current_screen_id,
            screens: std::collections::HashMap::new(),
            settings_open: false,
            app_state,
            router,
            ws_connection: WebSocketConnection::new(
                message_sender,
                error_sender,
                close_sender,
                egui_ctx,
            ),
            message_receiver,
            error_receiver,
            close_receiver,
        }
    }

    /// Change screen and update the URL with its registered path.
    fn change_screen(&mut self, screen_id: ScreenId) {
        let Some(meta) = self.app_state.screen_registry.meta_by_id(screen_id) else {
            return;
        };
        if self.current_screen_id != screen_id {
            self.current_screen_id = screen_id;
            if let Some(ref mut router) = self.router {
                let _ = router.navigate_to_path(meta.path);
            }
        }
    }

    /// Check for URL changes and update current path
    fn check_url_changes(&mut self) {
        if let Some(ref mut router) = self.router {
            if let Ok(changed) = router.check_for_url_changes() {
                if changed {
                    if let Some(screen_id) = self.app_state.screen_registry.id_by_path(router.current_path())
                    {
                        if screen_id != self.current_screen_id {
                            self.current_screen_id = screen_id;
                        }
                    }
                }
            }
        }
    }

    pub fn current_path(&self) -> &str {
        self.app_state
            .screen_registry
            .meta_by_id(self.current_screen_id)
            .map(|meta| meta.path)
            .unwrap_or("/")
    }

    fn ensure_current_screen(&mut self) {
        if self.screens.contains_key(&self.current_screen_id) {
            return;
        }

        if let Some(factory) = self.app_state.screen_registry.factory_by_id(self.current_screen_id) {
            self.screens.insert(self.current_screen_id, factory());
        }
    }

    fn dispatch_messages(&mut self, events: &mut Vec<FrontendEvent>) {
        let Some(screen) = self.screens.get_mut(&self.current_screen_id) else {
            // Leave messages in the channel until their destination screen exists.
            return;
        };
        let mut app_interface =
            FrontendInterface::new(events, &mut self.app_state, &mut self.ws_connection);
        while let Ok(msg) = self.message_receiver.try_recv() {
            // Keep application-owned state independent of screen lifetime.
            // Screens still receive every message for their screen-specific behavior.
            screen.on_message(&mut app_interface, msg);
        }
    }

    fn dispatch_error_events(&mut self) {
        while let Ok(event) = self.error_receiver.try_recv() {
            sprintln!("WebSocket error event occurred: {:?}", event);
        }
    }

    fn dispatch_close_events(&mut self) {
        while let Ok(event) = self.close_receiver.try_recv() {
            sprintln!(
                "Close event occurred:\n\tCode: {}\n\tReason: {}\n\tClean: {}",
                event.code(),
                event.reason(),
                event.was_clean()
            );
        }
    }
}

impl FrontendApp {
    fn render_top_bar(&mut self, ctx: &Context, events: &mut Vec<FrontendEvent>) {
        egui::TopBottomPanel::top("global_top_bar")
            .show_separator_line(false)
            .frame(
                egui::Frame::default()
                    .fill(ctx.style().visuals.window_fill())
                    .inner_margin(egui::Margin::symmetric(0, 8)),
            )
            .show(ctx, |ui| {
                egui::MenuBar::new().ui(ui, |ui| {
                    let avail = ui.available_width();
                    let left_w = NAVBAR_WIDTH_LEFT;
                    let right_w = NAVBAR_WIDTH_RIGHT;
                    let center_w = (avail - left_w - right_w).max(0.0);
                    let row_h = ui.spacing().interact_size.y + NAVBAR_ROW_HEIGHT_EXTRA;

                    ui.allocate_ui_with_layout(
                        egui::vec2(left_w, row_h),
                        egui::Layout::left_to_right(egui::Align::Min),
                        |ui| {
                            ui.add_space(MARGIN_SM);
                            if ui.button("⬅ Back").on_hover_text("Go back").clicked() {
                                if self.current_path().starts_with("/lobbyselect/") {
                                    let lobby_selection =
                                        ScreenId::of::<LobbySelectionScreen>();
                                    events.push(FrontendEvent::ChangeScreen(lobby_selection));
                                } else {
                                    events.push(FrontendEvent::ChangeScreen(ScreenId::of::<MainMenu>()));
                                }
                            }
                        },
                    );

                    ui.allocate_ui_with_layout(
                        egui::vec2(center_w, row_h),
                        egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
                        |ui| {
                            if let Some(meta) =
                                self.app_state.screen_registry.meta_by_id(self.current_screen_id)
                            {
                                ui.strong(meta.display_name);
                            }
                        },
                    );

                    ui.allocate_ui_with_layout(
                        egui::vec2(right_w, row_h),
                        egui::Layout::right_to_left(egui::Align::Center),
                        |ui| {
                            ui.add_space(MARGIN_SM);
                            if ui
                                .button("⚙ Settings")
                                .on_hover_text("Open global settings")
                                .clicked()
                            {
                                self.settings_open = true;
                            }
                        },
                    );
                });
            });

        if self.settings_open {
            let mut open = true;
            egui::Window::new("Settings")
                .open(&mut open)
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.heading("Global Settings");
                    ui.add_space(MARGIN_SM);
                    ui.label(format!("Version: {}", env!("CARGO_PKG_VERSION")));
                    ui.add_space(MARGIN_SM);
                    ui.add(
                        egui::Slider::new(&mut self.app_state.dpi, 0.75..=2.0)
                            .text("UI scale (DPI)"),
                    );
                    if ui.button("Reset to default").clicked() {
                        self.app_state.dpi = calculate_dpi_scale();
                    }
                    ui.checkbox(&mut self.app_state.dark_mode, "Dark mode");
                    ui.add_space(MARGIN_SM);
                    ui.horizontal(|ui| {
                        if ui.button("Apply").clicked() {
                            self.apply_ui_settings(ctx);
                        }
                        if ui.button("OK").clicked() {
                            self.apply_ui_settings(ctx);
                            self.settings_open = false;
                        }
                        if ui.button("Cancel").clicked() {
                            self.app_state.dpi = self.app_state.applied_dpi;
                            self.settings_open = false;
                        }
                    });
                });
            if !open {
                self.app_state.dpi = self.app_state.applied_dpi;
                self.settings_open = false;
            }
        }
    }

    fn apply_ui_settings(&mut self, ctx: &Context) {
        self.app_state.applied_dpi = self.app_state.dpi;
        ctx.set_pixels_per_point(self.app_state.applied_dpi);
        if self.app_state.dark_mode {
            ctx.set_visuals(egui::Visuals::dark());
        } else {
            ctx.set_visuals(egui::Visuals::light());
        }
    }
}

impl eframe::App for FrontendApp {
    fn update(&mut self, ctx: &Context, frame: &mut eframe::Frame) {
        self.check_url_changes();
        self.ensure_current_screen();

        // Process any pending messages from WebSocket callbacks
        let mut events = Vec::new();
        self.dispatch_messages(&mut events);
        self.dispatch_error_events();
        self.dispatch_close_events();

        // show top bar unless root
        if self.current_screen_id != ScreenId::of::<MainMenu>() {
            self.render_top_bar(ctx, &mut events);
        }
        let mut app_interface =
            FrontendInterface::new(&mut events, &mut self.app_state, &mut self.ws_connection);

        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(screen) = self.screens.get_mut(&self.current_screen_id) {
                screen.ui(&mut app_interface, ui, frame);
            } else {
                // fallback: main menu
                let mut mm = MainMenu;
                mm.ui(&mut app_interface, ui, frame);
            }
        });
        let events = std::mem::take(&mut events);
        for event in events {
            match event {
                FrontendEvent::ChangeScreen(screen_id) => {
                    if self.app_state.screen_registry.meta_by_id(screen_id).is_none() {
                        continue;
                    }
                    // Call on_exit for the current screen before changing routes
                    if let Some(mut screen) = self.screens.remove(&self.current_screen_id) {
                        let mut events = Vec::new();
                        let mut temp_interface = FrontendInterface::new(
                            &mut events,
                            &mut self.app_state,
                            &mut self.ws_connection,
                        );
                        screen.on_exit(&mut temp_interface);
                    }
                    self.change_screen(screen_id);
                }
                FrontendEvent::StartGame(config) => {
                    let game_id = ScreenId::of::<Game<DirectoryCardType>>();
                    if !self.screens.contains_key(&game_id) {
                        if let Some(factory) = self.app_state.screen_registry.factory_by_id(game_id) {
                            let boxed = factory();
                            self.screens.insert(game_id, boxed);
                        }
                    }
                    if let Some(screen) = self.screens.get_mut(&game_id) {
                        if let Some(game) = screen.downcast_mut::<Game<DirectoryCardType>>() {
                            game.set_state(config);
                            self.change_screen(game_id);
                        }
                    }
                }
                FrontendEvent::ExitGame => {
                    self.change_screen(ScreenId::of::<MainMenu>());
                }
            }
        }
    }
}
