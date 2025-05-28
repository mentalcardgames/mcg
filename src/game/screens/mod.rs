use std::cell::RefCell;
use std::rc::Rc;

use eframe::Frame;
use egui::Context;

pub mod articles_screen;
pub mod cards_test_dnd;
pub mod dnd_test;
pub mod game;
pub mod game_setup_screen;
pub mod main_menu;
pub mod pairing_screen;

pub use articles_screen::ArticlesScreen;
pub use cards_test_dnd::CardsTestDND;
pub use dnd_test::DNDTest;
pub use game::{DNDSelector, Game};
pub use game_setup_screen::GameSetupScreen;
pub use main_menu::MainMenu;
pub use pairing_screen::{PairingScreen, Player};

// Re-export GameConfig for use in other modules
pub use game::{DirectoryCardType, GameConfig};

/// Common trait for all screen widgets
pub trait ScreenWidget {
    fn update(&mut self, next_screen: Rc<RefCell<ScreenType>>, ctx: &Context, frame: &mut Frame);
}

/// Enum representing all available screen types
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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
        }
    }
}
