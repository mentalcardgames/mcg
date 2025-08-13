use egui::Context;
pub mod card;
pub mod field;
pub mod screen;
pub mod screens;
#[cfg(target_arch = "wasm32")]
use crate::router::Router;
use screens::{AppInterface, MainMenu, Routable, ScreenType, ScreenWidget};

/// Events that can be sent between screens
#[derive(Debug, Clone)]
pub enum AppEvent {
    ChangeScreen(ScreenType),
    StartGame(screens::GameConfig<screens::DirectoryCardType>),
    StartDndGame(screens::GameConfig<screens::DirectoryCardType>),
    ExitGame,
}

/// Application state that owns all screen data
pub struct App {
    // Screen management
    current_screen: ScreenType,
    main_menu: MainMenu,
    game_setup: screens::GameSetupScreen,
    game_dnd_setup: screens::GameSetupScreen,
    game: screens::Game<screens::DirectoryCardType>,
    game_dnd: screens::CardsTestDND,
    pairing_screen: screens::PairingScreen,
    articles_screen: screens::ArticlesScreen,
    qr_screen: screens::QrScreen,
    dnd_test: screens::DNDTest,
    poker_online: screens::PokerOnlineScreen,
    example_screen: screens::ExampleScreen,

    // Router for URL handling
    #[cfg(target_arch = "wasm32")]
    router: Option<Router>,

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
        // Initialize game components
        let mut game_setup = screens::GameSetupScreen::new();
        let mut game_dnd_setup = screens::GameSetupScreen::new_dnd();

        // Set default theme for the main game
        crate::hardcoded_cards::set_deck_by_theme(
            &mut game_setup.card_config,
            crate::hardcoded_cards::DEFAULT_THEME,
        );

        // Set alternative theme for drag and drop game
        crate::hardcoded_cards::set_deck_by_theme(&mut game_dnd_setup.card_config, "alt_cards");

        // Initialize router for WASM targets
        #[cfg(target_arch = "wasm32")]
        let router = Router::new().ok();

        #[cfg(target_arch = "wasm32")]
        let current_screen = router
            .as_ref()
            .map(|r| r.current_screen_type())
            .unwrap_or(ScreenType::Main);

        #[cfg(not(target_arch = "wasm32"))]
        let current_screen = ScreenType::Main;

        Self {
            current_screen,
            main_menu: MainMenu::new(),
            game_setup,
            game_dnd_setup,
            game: screens::Game::new(),
            game_dnd: screens::CardsTestDND::new(),
            pairing_screen: screens::PairingScreen::new(),
            articles_screen: screens::ArticlesScreen::new(),
            qr_screen: screens::QrScreen::new(),
            dnd_test: screens::DNDTest::new(),
            poker_online: screens::PokerOnlineScreen::new(),
            example_screen: screens::ExampleScreen::new(),
            #[cfg(target_arch = "wasm32")]
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
                AppEvent::ChangeScreen(screen_type) => {
                    self.change_screen(screen_type);
                }
                AppEvent::StartGame(config) => {
                    self.game.set_config(config);
                    self.change_screen(ScreenType::Game);
                }
                AppEvent::StartDndGame(config) => {
                    self.game_dnd.set_config(config);
                    self.change_screen(ScreenType::GameDnd);
                }
                AppEvent::ExitGame => {
                    self.change_screen(ScreenType::Main);
                }
            }
        }
    }

    /// Change screen and update URL
    fn change_screen(&mut self, screen_type: ScreenType) {
        if self.current_screen != screen_type {
            self.current_screen = screen_type;

            // Update URL if router is available
            #[cfg(target_arch = "wasm32")]
            if let Some(ref mut router) = self.router {
                if let Err(e) = router.navigate_to_screen(screen_type) {
                    crate::sprintln!("Failed to navigate to screen: {:?}", e);
                }
            }
        }
    }

    /// Check for URL changes and update screen accordingly
    fn check_url_changes(&mut self) {
        #[cfg(target_arch = "wasm32")]
        if let Some(ref mut router) = self.router {
            if let Ok(changed) = router.check_for_url_changes() {
                if changed {
                    let new_screen = router.current_screen_type();
                    if new_screen != self.current_screen {
                        self.current_screen = new_screen;
                    }
                }
            }
        }
    }

    /// Get the current screen type
    pub fn current_screen(&self) -> ScreenType {
        self.current_screen
    }
    pub fn is_current_screen(&self, screen_type: ScreenType) -> bool {
        self.current_screen == screen_type
    }
}

impl App {
    fn render_top_bar(&mut self, ctx: &Context, app_interface: &mut AppInterface) {
        egui::TopBottomPanel::top("global_top_bar")
            .exact_height(40.0)
            .show_separator_line(false)
            .frame(egui::Frame::default())
            .show(ctx, |ui| {
                ui.horizontal_centered(|ui| {
                    ui.add_space(16.0);
                    let back_button = egui::Button::new("⬅ Back").min_size(egui::vec2(110.0, 32.0));
                    if ui.add(back_button).clicked() {
                        app_interface.queue_event(AppEvent::ChangeScreen(ScreenType::Main));
                    }
                    ui.add_space(16.0);
                    ui.with_layout(
                        egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
                        |ui| {
                            ui.strong(self.current_screen.display_name());
                        },
                    );
                });
            });
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &Context, frame: &mut eframe::Frame) {
        // Set pixels_per_point based on screen resolution
        let pixels_per_point = crate::calculate_dpi_scale();
        ctx.set_pixels_per_point(pixels_per_point);

        // Check for URL changes first
        self.check_url_changes();

        // Process any pending events
        self.process_events();

        // Prepare event queue for screens
        let mut events = Vec::new();
        let mut app_interface = AppInterface {
            events: &mut events,
        };

        // Root layout: fixed top bar + central panel content
        // Always show top bar except for Main screen
        if self.current_screen != ScreenType::Main {
            self.render_top_bar(ctx, &mut app_interface);
        }
        egui::CentralPanel::default().show(ctx, |ui| match self.current_screen {
            ScreenType::Main => self.main_menu.ui(&mut app_interface, ui, frame),
            ScreenType::GameSetup => self.game_setup.ui(&mut app_interface, ui, frame),
            ScreenType::GameDndSetup => self.game_dnd_setup.ui(&mut app_interface, ui, frame),
            ScreenType::Game => self.game.ui(&mut app_interface, ui, frame),
            ScreenType::GameDnd => self.game_dnd.ui(&mut app_interface, ui, frame),
            ScreenType::Pairing => self.pairing_screen.ui(&mut app_interface, ui, frame),
            ScreenType::Articles => self.articles_screen.ui(&mut app_interface, ui, frame),
            ScreenType::QRScreen => self.qr_screen.ui(&mut app_interface, ui, frame),
            ScreenType::DndTest => self.dnd_test.ui(&mut app_interface, ui, frame),
            ScreenType::PokerOnline => self.poker_online.ui(&mut app_interface, ui, frame),
            ScreenType::Example => self.example_screen.ui(&mut app_interface, ui, frame),
        });

        for event in events {
            self.queue_event(event);
        }
    }
}
