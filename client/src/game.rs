use egui::Context;
pub mod card;
pub mod field;
pub mod screen;
pub mod screens;
#[cfg(target_arch = "wasm32")]
use crate::router::Router;
use screens::{AppInterface, MainMenu, ScreenWidget};

/// Events that can be sent between screens
#[derive(Debug, Clone)]
pub enum AppEvent {
    ChangeRoute(String),
    StartGame(screens::GameConfig<screens::DirectoryCardType>),
    StartDndGame(screens::GameConfig<screens::DirectoryCardType>),
    ExitGame,
}

/// Global settings for the application
#[derive(Clone)]
pub struct Settings {
    pub dpi: f32,
    pub dark_mode: bool,
}

/// Application state that owns non-UI state
#[derive(Clone)]
pub struct AppState {
    pub settings: Settings,
}
impl AppState {
    pub fn new() -> Self {
        Self {
            settings: Settings {
                dpi: crate::calculate_dpi_scale(),
                dark_mode: true,
            },
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

/// Application UI/Screen manager
pub struct App {
    // current route path ("/", "/game-setup", etc.)
    current_screen_path: String,
    // lazily-created screens by path
    screens: std::collections::HashMap<String, Box<dyn ScreenWidget>>,

    // typed screens for special access
    game: screens::Game<screens::DirectoryCardType>,
    game_dnd: screens::CardsTestDND,

    // Global settings UI state
    settings_open: bool,
    pending_settings: Settings,
    app_state: AppState,

    // Router for URL handling
    #[allow(dead_code)]
    #[cfg(target_arch = "wasm32")]
    router: Option<Router>,
    #[allow(dead_code)]
    #[cfg(not(target_arch = "wasm32"))]
    router: Option<()>,


    // Event queue for handling screen transitions
    pending_events: Vec<AppEvent>,
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
        let mut game_dnd_setup = screens::GameSetupScreen::new_dnd();
        crate::hardcoded_cards::set_deck_by_theme(
            &mut game_setup.card_config,
            crate::hardcoded_cards::DEFAULT_THEME,
        );
        crate::hardcoded_cards::set_deck_by_theme(&mut game_dnd_setup.card_config, "alt_cards");

        #[cfg(target_arch = "wasm32")]
        let router = Router::new().ok();

        #[cfg(target_arch = "wasm32")]
        let current_path = router
            .as_ref()
            .map(|r| r.current_path().to_string())
            .unwrap_or_else(|| "/".to_string());

        #[cfg(not(target_arch = "wasm32"))]
        let router: Option<()> = None;
        #[cfg(not(target_arch = "wasm32"))]
        let current_path = "/".to_string();

        Self {
            current_screen_path: current_path,
            screens: std::collections::HashMap::new(),
            game: screens::Game::new(),
            game_dnd: screens::CardsTestDND::new(),
            settings_open: false,
            pending_settings: Settings {
                dpi: crate::calculate_dpi_scale(),
                dark_mode: true,
            },
            app_state: AppState::new(),
            #[cfg(target_arch = "wasm32")]
            router,
            #[cfg(not(target_arch = "wasm32"))]
            router,
            pending_events: Vec::new(),
        }
    }

    /// Queue an event to be processed
    pub fn queue_event(&mut self, event: AppEvent) {
        self.pending_events.push(event);
    }

    /// Process all pending events
    fn process_events(&mut self) {
        let events = std::mem::take(&mut self.pending_events);
        for event in events {
            match event {
                AppEvent::ChangeRoute(path) => {
                    self.change_route(&path);
                }
                AppEvent::StartGame(config) => {
                    self.game.set_config(config);
                    self.change_route("/game");
                }
                AppEvent::StartDndGame(config) => {
                    self.game_dnd.set_config(config);
                    self.change_route("/game-dnd");
                }
                AppEvent::ExitGame => {
                    self.change_route("/");
                }
            }
        }
    }

    /// Change route by path and update URL
    fn change_route(&mut self, path: &str) {
        let reg = screens::ScreenRegistry::new();
        let new_path = reg.path_from_path(path).unwrap_or("/");
        if self.current_screen_path != new_path {
            self.current_screen_path = new_path.to_string();
            #[cfg(target_arch = "wasm32")]
            if let Some(ref mut router) = self.router {
                let _ = router.navigate_to_path(new_path);
            }
        }
    }

    /// Check for URL changes and update current path
    fn check_url_changes(&mut self) {
        #[cfg(target_arch = "wasm32")]
        if let Some(ref mut router) = self.router {
            if let Ok(changed) = router.check_for_url_changes() {
                if changed {
                    let reg = screens::ScreenRegistry::new();
                    if let Some(new_path) = reg.path_from_path(router.current_path()) {
                        if new_path != self.current_screen_path {
                            self.current_screen_path = new_path.to_string();
                        }
                    }
                }
            }
        }
    }

    pub fn current_path(&self) -> &str {
        &self.current_screen_path
    }
}

impl App {
    fn render_top_bar(&mut self, ctx: &Context, events: &mut Vec<AppEvent>) {
        egui::TopBottomPanel::top("global_top_bar")
            .show_separator_line(false)
            .frame(egui::Frame::default().fill(ctx.style().visuals.window_fill()).inner_margin(egui::Margin::symmetric(0, 8)))
            .show(ctx, |ui| {
                egui::menu::bar(ui, |ui| {
                    let avail = ui.available_width();
                    let left_w = 120.0;
                    let right_w = 140.0;
                    let center_w = (avail - left_w - right_w).max(0.0);
                    let row_h = ui.spacing().interact_size.y + 12.0;

                    ui.allocate_ui_with_layout(
                        egui::vec2(left_w, row_h),
                        egui::Layout::left_to_right(egui::Align::Min),
                        |ui| {
                            ui.add_space(8.0);
                            if ui.button("⬅ Back").on_hover_text("Go back").clicked() {
                                events.push(AppEvent::ChangeRoute("/".to_string()));
                            }
                        },
                    );

                    ui.allocate_ui_with_layout(
                        egui::vec2(center_w, row_h),
                        egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
                        |ui| {
                            let reg = screens::ScreenRegistry::new();
                            if let Some(meta) = reg.meta_by_path(&self.current_screen_path) {
                                ui.strong(meta.display_name);
                            }
                        },
                    );

                    ui.allocate_ui_with_layout(
                        egui::vec2(right_w, row_h),
                        egui::Layout::right_to_left(egui::Align::Center),
                        |ui| {
                            ui.add_space(8.0);
                            if ui
                                .button("⚙ Settings")
                                .on_hover_text("Open global settings")
                                .clicked()
                            {
                                self.pending_settings = self.app_state.settings.clone();
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
                    ui.add_space(8.0);
                    ui.add(
                        egui::Slider::new(&mut self.pending_settings.dpi, 0.75..=2.0)
                            .text("UI scale (DPI)"),
                    );
                    if ui.button("Reset to default").clicked() {
                        self.pending_settings.dpi = crate::calculate_dpi_scale();
                    }
                    ui.checkbox(&mut self.pending_settings.dark_mode, "Dark mode");
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        if ui.button("Apply").clicked() {
                            self.app_state.settings = self.pending_settings.clone();
                            ctx.set_pixels_per_point(self.app_state.settings.dpi);
                            if self.app_state.settings.dark_mode {
                                ctx.set_visuals(egui::Visuals::dark());
                            } else {
                                ctx.set_visuals(egui::Visuals::light());
                            }
                        }
                        if ui.button("OK").clicked() {
                            self.app_state.settings = self.pending_settings.clone();
                            ctx.set_pixels_per_point(self.app_state.settings.dpi);
                            if self.app_state.settings.dark_mode {
                                ctx.set_visuals(egui::Visuals::dark());
                            } else {
                                ctx.set_visuals(egui::Visuals::light());
                            }
                            self.settings_open = false;
                        }
                        if ui.button("Cancel").clicked() {
                            self.settings_open = false;
                        }
                    });
                });
            if !open {
                self.settings_open = false;
            }
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &Context, frame: &mut eframe::Frame) {
        ctx.set_pixels_per_point(self.app_state.settings.dpi);
        if self.app_state.settings.dark_mode {
            ctx.set_visuals(egui::Visuals::dark());
        } else {
            ctx.set_visuals(egui::Visuals::light());
        }
        self.check_url_changes();
        self.process_events();

        let mut events = Vec::new();

        // show top bar unless root
        if self.current_screen_path != "/" {
            self.render_top_bar(ctx, &mut events);
        }

        let mut app_interface = AppInterface {
            events: &mut events,
            app_state: &mut self.app_state,
        };

        egui::CentralPanel::default().show(ctx, |ui| {
            // Special-case typed screens that are owned directly on the App
            if self.current_screen_path == "/game" {
                self.game.ui(&mut app_interface, ui, frame);
                return;
            } else if self.current_screen_path == "/game-dnd" {
                self.game_dnd.ui(&mut app_interface, ui, frame);
                return;
            }

            // Ensure screen exists
            if !self.screens.contains_key(&self.current_screen_path) {
                if let Some(factory) =
                    screens::ScreenRegistry::new().factory_by_path(&self.current_screen_path)
                {
                    let boxed = factory();
                    self.screens.insert(self.current_screen_path.clone(), boxed);
                }
            }
            if let Some(screen) = self.screens.get_mut(&self.current_screen_path) {
                screen.ui(&mut app_interface, ui, frame);
            } else {
                // fallback: main menu
                let mut mm = MainMenu::new();
                mm.ui(&mut app_interface, ui, frame);
            }
        });

        for event in events {
            self.queue_event(event);
        }

        // Process newly-queued events immediately so UI transitions (like StartGame)
        // take effect within the same frame instead of waiting one frame.
        self.process_events();
    }
}
