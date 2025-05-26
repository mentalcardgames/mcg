use std::cell::RefCell;
use std::rc::Rc;

use eframe::Frame;
use egui::Context;

pub mod main_menu;
pub mod pairing_screen;
pub mod game_setup_screen;
pub mod game;
pub mod dnd_test;
pub mod cards_test_dnd;
pub mod articles_screen;

pub use main_menu::MainMenu;
pub use pairing_screen::{PairingScreen, Player};
pub use game_setup_screen::GameSetupScreen;
pub use game::{Game, DNDSelector};
pub use dnd_test::DNDTest;
pub use cards_test_dnd::CardsTestDND;
pub use articles_screen::ArticlesScreen;

// Re-export GameConfig for use in other modules
pub use game::{GameConfig, DirectoryCardType};

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
    /// Convert a string to a ScreenType
    pub fn from_string(s: &str) -> Option<Self> {
        match s {
            "main" => Some(ScreenType::Main),
            "game_setup" => Some(ScreenType::GameSetup),
            "game" => Some(ScreenType::Game),
            "pairing" => Some(ScreenType::Pairing),
            "settings" => Some(ScreenType::Settings),
            "dnd_test" => Some(ScreenType::DndTest),
            "game_dnd_setup" => Some(ScreenType::GameDndSetup),
            "game_dnd" => Some(ScreenType::GameDnd),
            "articles" => Some(ScreenType::Articles),
            _ => None,
        }
    }

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