use eframe::Frame;

pub mod articles_screen;
pub mod example_screen;
pub mod game;
pub mod game_setup_screen;
pub mod main_menu;
pub mod pairing_screen;

pub mod poker;
pub mod qr_test;
pub mod qr_test_receive;
pub mod qr_test_transmit;

use crate::game::screens::qr_test_receive::QrTestReceive;
use crate::game::screens::qr_test_transmit::QrTestTransmit;
pub use articles_screen::ArticlesScreen;
use downcast_rs::{impl_downcast, Downcast};
pub use example_screen::ExampleScreen;
pub use game::{DNDSelector, DirectoryCardType, Game, GameState};
pub use game_setup_screen::GameSetupScreen;
pub use main_menu::MainMenu;
pub use pairing_screen::PairingScreen;
pub use poker::PokerOnlineScreen;
pub use qr_test::QrScreen;

pub struct AppInterface<'a> {
    pub events: &'a mut Vec<crate::game::AppEvent>,
    pub app_state: &'a mut crate::store::ClientState,
}
impl<'a> AppInterface<'a> {
    pub fn queue_event(&mut self, event: crate::game::AppEvent) {
        self.events.push(event);
    }
    pub fn state(&mut self) -> &mut crate::store::ClientState {
        self.app_state
    }
}

/// Object-safe runtime trait for drawing a screen
pub trait ScreenWidget: Downcast {
    fn ui(&mut self, app_interface: &mut AppInterface, ui: &mut egui::Ui, frame: &mut Frame);
}
impl_downcast!(ScreenWidget);

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
    /// Ergonomic helper to register a screen type implementing ScreenDef
    pub fn register<T: ScreenDef + 'static>(&mut self) {
        let meta = T::metadata();
        self.by_path.insert(
            meta.path,
            RegisteredScreen {
                meta,
                factory: T::create,
            },
        );
    }

    pub fn new() -> Self {
        let mut reg = Self {
            by_path: std::collections::HashMap::new(),
        };

        // Register all screens by calling their ScreenDef implementations
        reg.register::<MainMenu>();
        reg.register::<GameSetupScreen>();
        reg.register::<Game<DirectoryCardType>>();
        reg.register::<PairingScreen>();
        reg.register::<ArticlesScreen>();
        reg.register::<QrScreen>();
        reg.register::<QrTestTransmit>();
        reg.register::<QrTestReceive>();
        reg.register::<PokerOnlineScreen>();
        reg.register::<ExampleScreen>();

        reg
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

#[macro_export]
macro_rules! impl_screen_def {
    ($type:ty, $path:literal, $display_name:literal, $icon:literal, $description:literal, $show_in_menu:expr) => {
        impl ScreenDef for $type {
            fn metadata() -> ScreenMetadata
            where
                Self: Sized,
            {
                ScreenMetadata {
                    path: $path,
                    display_name: $display_name,
                    icon: $icon,
                    description: $description,
                    show_in_menu: $show_in_menu,
                }
            }

            fn create() -> Box<dyn ScreenWidget>
            where
                Self: Sized,
            {
                Box::new(Self::new())
            }
        }
    };
}
