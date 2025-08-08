use eframe::Frame;
use egui::Context;

pub mod articles_screen;
pub mod cards_test_dnd;
pub mod dnd_test;
pub mod game;
pub mod game_setup_screen;
pub mod main_menu;
pub mod pairing_screen;
pub mod qr_test;
pub mod poker_online;

pub use articles_screen::ArticlesScreen;
pub use cards_test_dnd::CardsTestDND;
pub use dnd_test::DNDTest;
pub use game::{DNDSelector, Game};
pub use game_setup_screen::GameSetupScreen;
pub use main_menu::MainMenu;
pub use pairing_screen::{PairingScreen, Player};
pub use qr_test::QrScreen;
pub use poker_online::PokerOnlineScreen;

// Re-export GameConfig for use in other modules
pub use game::{DirectoryCardType, GameConfig};

/// Interface for screens to interact with the app
pub struct AppInterface<'a> {
    pub events: &'a mut Vec<crate::game::AppEvent>,
}

impl<'a> AppInterface<'a> {
    pub fn queue_event(&mut self, event: crate::game::AppEvent) {
        self.events.push(event);
    }
}

/// Common trait for all screen widgets
pub trait ScreenWidget {
    fn update(&mut self, app_interface: &mut AppInterface, ctx: &Context, frame: &mut Frame);
}

/// Enum representing all available screen types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ScreenType {
    Main,
    GameSetup,
    Game,
    Pairing,
    Settings,
    DndTest,
    GameDndSetup,
    GameDnd,
    Articles,
    QRScreen,
    PokerOnline,
}

impl ScreenType {
    /// Convert a ScreenType to a string
    pub fn to_string(&self) -> String {
        match self {
            ScreenType::Main => "main".to_string(),
            ScreenType::GameSetup => "game_setup".to_string(),
            ScreenType::Game => "game".to_string(),
            ScreenType::Pairing => "pairing".to_string(),
            ScreenType::Settings => "settings".to_string(),
            ScreenType::DndTest => "dnd_test".to_string(),
            ScreenType::GameDndSetup => "game_dnd_setup".to_string(),
            ScreenType::GameDnd => "game_dnd".to_string(),
            ScreenType::Articles => "articles".to_string(),
            ScreenType::QRScreen => "qr".to_string(),
            ScreenType::PokerOnline => "poker_online".to_string(),
        }
    }
}

/// Reusable "Back" button helper.
///
/// Renders a button with the given label. When clicked, it enqueues a ChangeScreen event
/// to switch to the provided target screen. Returns true if clicked.
pub fn back_button(
    ui: &mut egui::Ui,
    app_interface: &mut AppInterface,
    to: ScreenType,
    label: &str,
) -> bool {
    let clicked = ui.button(label).clicked();
    if clicked {
        app_interface.queue_event(crate::game::AppEvent::ChangeScreen(to));
    }
    clicked
}
