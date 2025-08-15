use eframe::Frame;

pub mod articles_screen;
pub mod cards_test_dnd;
pub mod dnd_test;
pub mod example_screen;
pub mod game;
pub mod game_setup_screen;
pub mod main_menu;
pub mod pairing_screen;
pub mod poker_online;
pub mod qr_test;

pub use articles_screen::ArticlesScreen;
pub use cards_test_dnd::CardsTestDND;
pub use dnd_test::DNDTest;
pub use example_screen::ExampleScreen;
pub use game::{DNDSelector, DirectoryCardType, Game, GameConfig};
pub use game_setup_screen::GameSetupScreen;
pub use main_menu::MainMenu;
pub use pairing_screen::{PairingScreen, Player};
pub use poker_online::PokerOnlineScreen;
pub use qr_test::QrScreen;

pub struct AppInterface<'a> {
    pub events: &'a mut Vec<crate::game::AppEvent>,
    pub app_state: &'a mut crate::game::AppState,
}
impl<'a> AppInterface<'a> {
    pub fn queue_event(&mut self, event: crate::game::AppEvent) {
        self.events.push(event);
    }
    pub fn state(&mut self) -> &mut crate::game::AppState {
        self.app_state
    }
}

/// Object-safe runtime trait for drawing a screen
pub trait ScreenWidget {
    fn ui(&mut self, app_interface: &mut AppInterface, ui: &mut egui::Ui, frame: &mut Frame);
}

/// Compile-time definition trait: metadata + factory
pub trait ScreenDef {
    fn metadata() -> ScreenMetadata
    where
        Self: Sized;
    fn create() -> Box<dyn ScreenWidget>
    where
        Self: Sized;
}

/// Metadata for screen configuration and display
#[derive(Clone, Copy)]
pub struct ScreenMetadata {
    /// URL path for routing and stable id (must be URL-safe)
    pub path: &'static str,
    /// Display name for the screen
    pub display_name: &'static str,
    /// Icon/emoji for the screen
    pub icon: &'static str,
    /// Description for main menu
    pub description: &'static str,
    /// Whether this screen should appear in the main menu
    pub show_in_menu: bool,
}

/// A registered screen entry holding metadata and a factory
pub struct RegisteredScreen {
    pub meta: ScreenMetadata,
    pub factory: fn() -> Box<dyn ScreenWidget>,
}

/// Screen registry for managing screen instances and metadata
pub struct ScreenRegistry {
    by_path: std::collections::HashMap<&'static str, RegisteredScreen>,
}

impl ScreenRegistry {
    pub fn new() -> Self {
        let mut by_path = std::collections::HashMap::new();

        // Register all screens by calling their ScreenDef implementations
        let regs: &[RegisteredScreen] = &[
            RegisteredScreen {
                meta: MainMenu::metadata(),
                factory: MainMenu::create,
            },
            RegisteredScreen {
                meta: GameSetupScreen::metadata(),
                factory: GameSetupScreen::create,
            },
            RegisteredScreen {
                meta: Game::<DirectoryCardType>::metadata(),
                factory: Game::<DirectoryCardType>::create,
            },
            RegisteredScreen {
                meta: PairingScreen::metadata(),
                factory: PairingScreen::create,
            },
            RegisteredScreen {
                meta: DNDTest::metadata(),
                factory: DNDTest::create,
            },
            RegisteredScreen {
                meta: CardsTestDND::metadata(),
                factory: CardsTestDND::create,
            },
            RegisteredScreen {
                meta: ArticlesScreen::metadata(),
                factory: ArticlesScreen::create,
            },
            RegisteredScreen {
                meta: QrScreen::metadata(),
                factory: QrScreen::create,
            },
            RegisteredScreen {
                meta: PokerOnlineScreen::metadata(),
                factory: PokerOnlineScreen::create,
            },
            RegisteredScreen {
                meta: ExampleScreen::metadata(),
                factory: ExampleScreen::create,
            },
        ];

        for r in regs {
            by_path.insert(
                r.meta.path,
                RegisteredScreen {
                    meta: r.meta,
                    factory: r.factory,
                },
            );
        }

        Self { by_path }
    }

    /// Resolve metadata by path
    pub fn meta_by_path(&self, path: &str) -> Option<&ScreenMetadata> {
        let key = if path.is_empty() { "/" } else { path };
        self.by_path.get(key).map(|r| &r.meta)
    }

    /// Resolve path from a URL path (identity), for symmetry
    pub fn path_from_path(&self, path: &str) -> Option<&'static str> {
        self.meta_by_path(path).map(|m| m.path)
    }

    /// Get a screen factory by path
    pub fn factory_by_path(&self, path: &str) -> Option<fn() -> Box<dyn ScreenWidget>> {
        let key = if path.is_empty() { "/" } else { path };
        self.by_path.get(key).map(|r| r.factory)
    }

    /// Iterate the menu screens: return metadata with show_in_menu
    pub fn menu_metas(&self) -> Vec<&ScreenMetadata> {
        let mut v: Vec<&ScreenMetadata> = self
            .by_path
            .values()
            .filter(|r| r.meta.show_in_menu)
            .map(|r| &r.meta)
            .collect();
        // stable ordering by path for now
        v.sort_by_key(|m| m.path);
        v
    }
}

impl Default for ScreenRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// Implement ScreenDef for simple screens that already exist
// Individual screen modules will provide their own impls when needed
