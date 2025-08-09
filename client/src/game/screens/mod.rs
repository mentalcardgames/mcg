use eframe::Frame;

pub mod articles_screen;
pub mod cards_test_dnd;
pub mod dnd_test;
pub mod game;
pub mod game_setup_screen;
pub mod main_menu;
pub mod pairing_screen;
pub mod poker_online;
pub mod qr_test;

pub use articles_screen::ArticlesScreen;
pub use cards_test_dnd::CardsTestDND;
pub use dnd_test::DNDTest;
pub use game::{DNDSelector, Game};
pub use game_setup_screen::GameSetupScreen;
pub use main_menu::MainMenu;
pub use pairing_screen::{PairingScreen, Player};
pub use poker_online::PokerOnlineScreen;
pub use qr_test::QrScreen;

pub use game::{DirectoryCardType, GameConfig};

pub struct AppInterface<'a> {
    pub events: &'a mut Vec<crate::game::AppEvent>,
}
impl<'a> AppInterface<'a> {
    pub fn queue_event(&mut self, event: crate::game::AppEvent) {
        self.events.push(event);
    }
}

pub trait ScreenWidget {
    // Render the screen into the given Ui. Root controls panels/layout.
    fn ui(&mut self, app_interface: &mut AppInterface, ui: &mut egui::Ui, frame: &mut Frame);
}

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
    pub fn to_string(&self) -> String {
        match self {
            ScreenType::Main => "main".into(),
            ScreenType::GameSetup => "game_setup".into(),
            ScreenType::Game => "game".into(),
            ScreenType::Pairing => "pairing".into(),
            ScreenType::Settings => "settings".into(),
            ScreenType::DndTest => "dnd_test".into(),
            ScreenType::GameDndSetup => "game_dnd_setup".into(),
            ScreenType::GameDnd => "game_dnd".into(),
            ScreenType::Articles => "articles".into(),
            ScreenType::QRScreen => "qr".into(),
            ScreenType::PokerOnline => "poker_online".into(),
        }
    }
}
