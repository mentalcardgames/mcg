pub mod card;
pub mod field;
pub mod screens;
pub mod theme;
pub mod websocket;
use crate::router::Router;
use crate::{
    game::{card::DirectoryCardType, screens::Game},
    sprintln,
    store::ClientState,
};
use egui::Context;
use mcg_shared::Backend2FrontendMsg;
use screens::{AppInterface, MainMenu, ScreenId, ScreenWidget};
use std::sync::mpsc::{self, Receiver};
use theme::*;

/// Events that can be sent between screens
#[derive(Debug, Clone)]
pub enum AppEvent {
    ChangeScreen(ScreenId),
    StartGame(screens::GameState<screens::DirectoryCardType>),
    ExitGame,
}

/// Global settings for the application
#[derive(Clone)]
pub struct Settings {
    pub dpi: f32,
    pub applied_dpi: f32,
    pub dark_mode: bool,
}

#[derive(PartialEq, Debug, Clone, Copy, Default)]
pub enum GameType {
    #[default]
    Poker,
    Blackjack,
    // Add more game types here
}

/// Application UI/Screen manager
pub struct App {
    // current screen type
    current_screen_id: ScreenId,
    // lazily-created screens by type
    screens: std::collections::HashMap<ScreenId, Box<dyn ScreenWidget>>,
    // single shared screen registry
    screen_registry: screens::ScreenRegistry,

    // Global settings UI state
    settings_open: bool,
    pending_settings: Settings,
    app_state: ClientState,

    // Router for URL handling
    router: Option<Router>,

    ws_connection: websocket::WebSocketConnection,
    message_receiver: Receiver<Backend2FrontendMsg>,
    error_receiver: Receiver<web_sys::Event>,
    close_receiver: Receiver<web_sys::CloseEvent>,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    pub fn new() -> Self {
        // Initialize typed screens
        let mut game_setup = screens::GameSetupScreen::new();
        crate::hardcoded_cards::set_deck_by_theme(
            &mut game_setup.card_config,
            crate::hardcoded_cards::DEFAULT_THEME,
        );
        crate::hardcoded_cards::set_deck_by_theme(&mut game_setup.card_config, "alt_cards");

        let router = Router::new().ok();

        let current_path = router.as_ref().map(|r| r.current_path()).unwrap_or("/");
        let screen_registry = screens::ScreenRegistry::new();
        let current_screen_id = screen_registry
            .id_by_path(current_path)
            .unwrap_or_else(ScreenId::of::<MainMenu>);

        let app_state = ClientState::new();
        let (message_sender, message_receiver) = mpsc::channel();
        let (error_sender, error_receiver) = mpsc::channel();
        let (close_sender, close_receiver) = mpsc::channel();
        Self {
            current_screen_id,
            screens: std::collections::HashMap::new(),
            screen_registry,
            settings_open: false,
            pending_settings: Settings {
                dpi: crate::calculate_dpi_scale(),
                applied_dpi: crate::calculate_dpi_scale(),
                dark_mode: true,
            },
            app_state,
            router,
            ws_connection: websocket::WebSocketConnection::new(
                message_sender,
                error_sender,
                close_sender,
            ),
            message_receiver,
            error_receiver,
            close_receiver,
        }
    }

    /// Change screen and update the URL with its registered path.
    fn change_screen(&mut self, screen_id: ScreenId) {
        let Some(meta) = self.screen_registry.meta_by_id(screen_id) else {
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
                    if let Some(screen_id) = self.screen_registry.id_by_path(router.current_path())
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
        self.screen_registry
            .meta_by_id(self.current_screen_id)
            .map(|meta| meta.path)
            .unwrap_or("/")
    }

    fn ensure_current_screen(&mut self) {
        if self.screens.contains_key(&self.current_screen_id) {
            return;
        }

        if let Some(factory) = self.screen_registry.factory_by_id(self.current_screen_id) {
            self.screens.insert(self.current_screen_id, factory());
        }
    }

    fn dispatch_messages(&mut self, events: &mut Vec<AppEvent>) {
        let Some(screen) = self.screens.get_mut(&self.current_screen_id) else {
            // Leave messages in the channel until their destination screen exists.
            return;
        };
        let mut app_interface =
            AppInterface::new(events, &mut self.app_state, &mut self.ws_connection);
        while let Ok(msg) = self.message_receiver.try_recv() {
            // Keep application-owned state independent of screen lifetime.
            // Screens still receive every message for their screen-specific behavior.
            screen.on_message(&mut app_interface, msg);
        }
    }

    fn dispatch_error_events(&mut self) {
        while let Ok(event) = self.error_receiver.try_recv() {
            sprintln!("WebSocket error event occurred: {:?}", event);
            self.app_state.connection.connection_status =
                crate::store::ConnectionStatus::Disconnected;
            self.app_state.ui.last_error = Some("WebSocket connection error.".to_string());
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
            self.app_state.connection.connection_status =
                crate::store::ConnectionStatus::Disconnected;
        }
    }
}

impl App {
    fn render_top_bar(&mut self, ctx: &Context, events: &mut Vec<AppEvent>) {
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
                                        ScreenId::of::<screens::LobbySelectionScreen>();
                                    events.push(AppEvent::ChangeScreen(lobby_selection));
                                } else {
                                    events.push(AppEvent::ChangeScreen(ScreenId::of::<MainMenu>()));
                                }
                            }
                        },
                    );

                    ui.allocate_ui_with_layout(
                        egui::vec2(center_w, row_h),
                        egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
                        |ui| {
                            if let Some(meta) =
                                self.screen_registry.meta_by_id(self.current_screen_id)
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
                        egui::Slider::new(&mut self.pending_settings.dpi, 0.75..=2.0)
                            .text("UI scale (DPI)"),
                    );
                    if ui.button("Reset to default").clicked() {
                        self.pending_settings.dpi = crate::calculate_dpi_scale();
                    }
                    ui.checkbox(&mut self.pending_settings.dark_mode, "Dark mode");
                    ui.add_space(MARGIN_SM);
                    ui.horizontal(|ui| {
                        if ui.button("Apply").clicked() {
                            self.pending_settings.applied_dpi = self.pending_settings.dpi;
                            ctx.set_pixels_per_point(self.pending_settings.applied_dpi);
                            if self.pending_settings.dark_mode {
                                ctx.set_visuals(egui::Visuals::dark());
                            } else {
                                ctx.set_visuals(egui::Visuals::light());
                            }
                        }
                        if ui.button("OK").clicked() {
                            self.pending_settings.applied_dpi = self.pending_settings.dpi;
                            ctx.set_pixels_per_point(self.pending_settings.applied_dpi);
                            if self.pending_settings.dark_mode {
                                ctx.set_visuals(egui::Visuals::dark());
                            } else {
                                ctx.set_visuals(egui::Visuals::light());
                            }
                            self.settings_open = false;
                        }
                        if ui.button("Cancel").clicked() {
                            self.pending_settings.dpi = self.pending_settings.applied_dpi;
                            self.settings_open = false;
                        }
                    });
                });
            if !open {
                self.pending_settings.dpi = self.pending_settings.applied_dpi;
                self.settings_open = false;
            }
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &Context, frame: &mut eframe::Frame) {
        self.check_url_changes();
        self.ensure_current_screen();

        // Process any pending messages from WebSocket callbacks
        let mut events = Vec::new();
        self.dispatch_messages(&mut events);
        self.dispatch_error_events();
        self.dispatch_close_events();

        ctx.set_pixels_per_point(self.pending_settings.applied_dpi);
        if self.pending_settings.dark_mode {
            ctx.set_visuals(egui::Visuals::dark());
        } else {
            ctx.set_visuals(egui::Visuals::light());
        }

        // show top bar unless root
        if self.current_screen_id != ScreenId::of::<MainMenu>() {
            self.render_top_bar(ctx, &mut events);
        }
        let mut app_interface =
            AppInterface::new(&mut events, &mut self.app_state, &mut self.ws_connection);

        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(screen) = self.screens.get_mut(&self.current_screen_id) {
                screen.ui(&mut app_interface, ui, frame);
            } else {
                // fallback: main menu
                let mut mm = MainMenu::default();
                mm.ui(&mut app_interface, ui, frame);
            }
        });
        let events = std::mem::take(&mut events);
        for event in events {
            match event {
                AppEvent::ChangeScreen(screen_id) => {
                    if self.screen_registry.meta_by_id(screen_id).is_none() {
                        continue;
                    }
                    // Call on_exit for the current screen before changing routes
                    if let Some(mut screen) = self.screens.remove(&self.current_screen_id) {
                        let mut events = Vec::new();
                        let mut temp_interface = AppInterface::new(
                            &mut events,
                            &mut self.app_state,
                            &mut self.ws_connection,
                        );
                        screen.on_exit(&mut temp_interface);
                    }
                    self.change_screen(screen_id);
                }
                AppEvent::StartGame(config) => {
                    let game_id = ScreenId::of::<Game<DirectoryCardType>>();
                    if !self.screens.contains_key(&game_id) {
                        if let Some(factory) = self.screen_registry.factory_by_id(game_id) {
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
                AppEvent::ExitGame => {
                    self.change_screen(ScreenId::of::<MainMenu>());
                }
            }
        }

        // Request continuous repaints for real-time updates (WebSocket messages, animations, etc.)
        // This is the standard approach for egui applications that need real-time updates
        ctx.request_repaint();
    }
}
