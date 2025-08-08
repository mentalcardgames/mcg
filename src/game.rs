use egui::Context;
pub mod card;
pub mod field;
pub mod screen;
pub mod screens;
use screens::{MainMenu, ScreenType, ScreenWidget, AppInterface};

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
        crate::hardcoded_cards::set_deck_by_theme(
            &mut game_dnd_setup.card_config,
            "alt_cards",
        );
        
        Self {
            current_screen: ScreenType::Main,
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
                    self.current_screen = screen_type;
                }
                AppEvent::StartGame(config) => {
                    self.game.set_config(config);
                    self.current_screen = ScreenType::Game;
                }
                AppEvent::StartDndGame(config) => {
                    self.game_dnd.set_config(config);
                    self.current_screen = ScreenType::GameDnd;
                }
                AppEvent::ExitGame => {
                    self.current_screen = ScreenType::Main;
                }
            }
        }
    }

    /// Get the current screen type
    pub fn current_screen(&self) -> ScreenType {
        self.current_screen
    }

    /// Check if a specific screen is currently active
    pub fn is_current_screen(&self, screen_type: ScreenType) -> bool {
        self.current_screen == screen_type
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &Context, frame: &mut eframe::Frame) {
        // Set pixels_per_point based on screen resolution
        let pixels_per_point = crate::calculate_dpi_scale();
        ctx.set_pixels_per_point(pixels_per_point);

        // Process any pending events first
        self.process_events();

        // Prepare event queue for screens
        let mut events = Vec::new();
        let mut app_interface = AppInterface { events: &mut events };

        // Update the current screen
        match self.current_screen {
            ScreenType::Main => {
                self.main_menu.update(&mut app_interface, ctx, frame);
            }
            ScreenType::GameSetup => {
                self.game_setup.update(&mut app_interface, ctx, frame);
            }
            ScreenType::GameDndSetup => {
                self.game_dnd_setup.update(&mut app_interface, ctx, frame);
            }
            ScreenType::Game => {
                self.game.update(&mut app_interface, ctx, frame);
            }
            ScreenType::GameDnd => {
                self.game_dnd.update(&mut app_interface, ctx, frame);
            }
            ScreenType::Pairing => {
                self.pairing_screen.update(&mut app_interface, ctx, frame);
            }
            ScreenType::Articles => {
                self.articles_screen.update(&mut app_interface, ctx, frame);
            }
            ScreenType::QRScreen => {
                self.qr_screen.update(&mut app_interface, ctx, frame);
            }
            ScreenType::DndTest => {
                self.dnd_test.update(&mut app_interface, ctx, frame);
            }
            ScreenType::PokerOnline => {
                self.poker_online.update(&mut app_interface, ctx, frame);
            }
            ScreenType::Settings => {
                // Settings screen not implemented yet
                egui::CentralPanel::default().show(ctx, |ui| {
                    ui.label("Settings screen not implemented");
                    if ui.button("Back").clicked() {
                        app_interface.queue_event(AppEvent::ChangeScreen(ScreenType::Main));
                    }
                });
            }
        }

        // Process events from screens
        for event in events {
            self.queue_event(event);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_initialization() {
        let app = App::new();
        assert_eq!(app.current_screen(), ScreenType::Main);
        assert!(app.is_current_screen(ScreenType::Main));
        assert!(!app.is_current_screen(ScreenType::Game));
    }

    #[test]
    fn test_event_processing() {
        let mut app = App::new();
        
        // Test screen change event
        app.queue_event(AppEvent::ChangeScreen(ScreenType::Game));
        app.process_events();
        assert_eq!(app.current_screen(), ScreenType::Game);
        
        // Test exit game event
        app.queue_event(AppEvent::ExitGame);
        app.process_events();
        assert_eq!(app.current_screen(), ScreenType::Main);
    }
}
