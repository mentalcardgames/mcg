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

/// Trait for screen routing capabilities following SOLID principles
pub trait Routable {
    /// Get the URL path for this screen
    fn url_path(&self) -> &'static str;

    /// Get display name for this screen
    fn display_name(&self) -> &'static str;

    /// Parse a URL path to a screen type (if it matches)
    fn from_url_path(path: &str) -> Option<Self>
    where
        Self: Sized;
}

/// Trait that combines screen widgets with their metadata
pub trait Screen {
    /// Get the screen type
    fn screen_type(&self) -> ScreenType;

    /// Get screen metadata
    fn metadata(&self) -> ScreenMetadata {
        self.screen_type().metadata()
    }

    /// Render the screen
    fn ui(&mut self, app_interface: &mut AppInterface, ui: &mut egui::Ui, frame: &mut Frame);
}

/// Blanket implementation for all ScreenWidget types
impl<T: ScreenWidget> Screen for T {
    fn screen_type(&self) -> ScreenType {
        // This will need to be overridden by each screen implementation
        ScreenType::Main
    }

    fn ui(&mut self, app_interface: &mut AppInterface, ui: &mut egui::Ui, frame: &mut Frame) {
        ScreenWidget::ui(self, app_interface, ui, frame);
    }
}

/// Metadata for screen configuration and display
pub struct ScreenMetadata {
    /// Display name for the screen
    pub display_name: &'static str,
    /// Icon/emoji for the screen
    pub icon: &'static str,
    /// URL path for routing
    pub url_path: &'static str,
    /// Description for main menu
    pub description: &'static str,
    /// Whether this screen should appear in the main menu
    pub show_in_menu: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ScreenType {
    Main,
    GameSetup,
    Game,
    Pairing,

    DndTest,
    GameDndSetup,
    GameDnd,
    Articles,
    QRScreen,
    PokerOnline,
    Example,
}

impl Routable for ScreenType {
    fn url_path(&self) -> &'static str {
        self.metadata().url_path
    }

    fn display_name(&self) -> &'static str {
        self.metadata().display_name
    }

    fn from_url_path(path: &str) -> Option<Self> {
        Self::all_screens().into_iter().find(|screen_type| {
            screen_type.url_path() == path || (path.is_empty() && screen_type.url_path() == "/")
        })
    }
}

impl ScreenType {
    /// Get metadata for this screen
    pub fn metadata(&self) -> ScreenMetadata {
        match self {
            ScreenType::Main => ScreenMetadata {
                display_name: "Main Menu",
                icon: "🎮",
                url_path: "/",
                description: "Main menu",
                show_in_menu: false,
            },
            ScreenType::GameSetup => ScreenMetadata {
                display_name: "Game Setup",
                icon: "🎮",
                url_path: "/game-setup",
                description: "Start a new game",
                show_in_menu: true,
            },
            ScreenType::Game => ScreenMetadata {
                display_name: "Game",
                icon: "🎯",
                url_path: "/game",
                description: "Active game",
                show_in_menu: false,
            },
            ScreenType::Pairing => ScreenMetadata {
                display_name: "Pairing",
                icon: "📱",
                url_path: "/pairing",
                description: "Device pairing",
                show_in_menu: true,
            },

            ScreenType::DndTest => ScreenMetadata {
                display_name: "DND Test",
                icon: "✋",
                url_path: "/dnd-test",
                description: "Drag and drop test",
                show_in_menu: true,
            },
            ScreenType::GameDndSetup => ScreenMetadata {
                display_name: "Drag & Drop Setup",
                icon: "🎯",
                url_path: "/game-dnd-setup",
                description: "Drag and drop game setup",
                show_in_menu: true,
            },
            ScreenType::GameDnd => ScreenMetadata {
                display_name: "Drag & Drop Game",
                icon: "🎯",
                url_path: "/game-dnd",
                description: "Drag and drop game",
                show_in_menu: false,
            },
            ScreenType::Articles => ScreenMetadata {
                display_name: "Articles",
                icon: "📚",
                url_path: "/articles",
                description: "Read articles",
                show_in_menu: true,
            },
            ScreenType::QRScreen => ScreenMetadata {
                display_name: "QR Test",
                icon: "🔎",
                url_path: "/qr",
                description: "QR code test",
                show_in_menu: true,
            },
            ScreenType::PokerOnline => ScreenMetadata {
                display_name: "Poker Online",
                icon: "🃏",
                url_path: "/poker-online",
                description: "Online poker game",
                show_in_menu: true,
            },
            ScreenType::Example => ScreenMetadata {
                display_name: "Example Screen",
                icon: "🧪",
                url_path: "/example",
                description: "Example screen to demonstrate the new system",
                show_in_menu: true,
            },
        }
    }

    /// Get all screen types that should appear in the main menu
    pub fn menu_screens() -> Vec<ScreenType> {
        vec![
            ScreenType::GameSetup,
            ScreenType::GameDndSetup,
            ScreenType::Pairing,
            ScreenType::DndTest,
            ScreenType::Articles,
            ScreenType::QRScreen,
            ScreenType::PokerOnline,
            ScreenType::Example,
        ]
        .into_iter()
        .collect()
    }

    /// Get all available screen types
    pub fn all_screens() -> Vec<ScreenType> {
        vec![
            ScreenType::Main,
            ScreenType::GameSetup,
            ScreenType::Game,
            ScreenType::Pairing,
            ScreenType::DndTest,
            ScreenType::GameDndSetup,
            ScreenType::GameDnd,
            ScreenType::Articles,
            ScreenType::QRScreen,
            ScreenType::PokerOnline,
            ScreenType::Example,
        ]
    }
}

/// Screen registry for managing screen instances and metadata
pub struct ScreenRegistry {
    /// Cache of screen metadata for quick access
    metadata_cache: std::collections::HashMap<ScreenType, ScreenMetadata>,
}

impl ScreenRegistry {
    pub fn new() -> Self {
        let mut metadata_cache = std::collections::HashMap::new();

        // Pre-populate metadata cache
        for screen_type in ScreenType::all_screens() {
            metadata_cache.insert(screen_type, screen_type.metadata());
        }

        Self { metadata_cache }
    }

    /// Get metadata for a screen type
    pub fn get_metadata(&self, screen_type: ScreenType) -> &ScreenMetadata {
        self.metadata_cache
            .get(&screen_type)
            .unwrap_or_else(|| panic!("No metadata found for screen type: {:?}", screen_type))
    }

    /// Get all screens that should appear in the main menu
    pub fn get_menu_screens(&self) -> Vec<ScreenType> {
        ScreenType::menu_screens()
    }
}

impl Default for ScreenRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Trait for screen factory following SOLID principles
pub trait ScreenFactory {
    /// Create a new screen instance
    fn create() -> Box<dyn ScreenWidget>
    where
        Self: Sized;
}

impl ScreenType {
    /// Create a screen instance for this screen type
    pub fn create_screen(self) -> Box<dyn ScreenWidget> {
        match self {
            ScreenType::Main => MainMenu::create(),
            ScreenType::GameSetup => GameSetupScreen::create(),
            ScreenType::Game => Game::create(),
            ScreenType::Pairing => PairingScreen::create(),
            ScreenType::DndTest => DNDTest::create(),
            ScreenType::GameDndSetup => GameSetupScreen::create_dnd(),
            ScreenType::GameDnd => CardsTestDND::create(),
            ScreenType::Articles => ArticlesScreen::create(),
            ScreenType::QRScreen => QrScreen::create(),
            ScreenType::PokerOnline => PokerOnlineScreen::create(),
            ScreenType::Example => ExampleScreen::create(),
        }
    }
}

// Implement ScreenFactory for all existing screens
impl ScreenFactory for MainMenu {
    fn create() -> Box<dyn ScreenWidget> {
        Box::new(MainMenu::new())
    }
}

impl ScreenFactory for GameSetupScreen {
    fn create() -> Box<dyn ScreenWidget> {
        Box::new(GameSetupScreen::new())
    }
}

impl GameSetupScreen {
    /// Special factory method for DND variant
    pub fn create_dnd() -> Box<dyn ScreenWidget> {
        Box::new(GameSetupScreen::new_dnd())
    }
}

impl ScreenFactory for Game<crate::game::screens::DirectoryCardType> {
    fn create() -> Box<dyn ScreenWidget> {
        Box::new(Game::new())
    }
}

impl ScreenFactory for PairingScreen {
    fn create() -> Box<dyn ScreenWidget> {
        Box::new(PairingScreen::new())
    }
}

impl ScreenFactory for DNDTest {
    fn create() -> Box<dyn ScreenWidget> {
        Box::new(DNDTest::new())
    }
}

impl ScreenFactory for CardsTestDND {
    fn create() -> Box<dyn ScreenWidget> {
        Box::new(CardsTestDND::new())
    }
}

impl ScreenFactory for ArticlesScreen {
    fn create() -> Box<dyn ScreenWidget> {
        Box::new(ArticlesScreen::new())
    }
}

impl ScreenFactory for QrScreen {
    fn create() -> Box<dyn ScreenWidget> {
        Box::new(QrScreen::new())
    }
}

impl ScreenFactory for PokerOnlineScreen {
    fn create() -> Box<dyn ScreenWidget> {
        Box::new(PokerOnlineScreen::new())
    }
}

impl ScreenFactory for ExampleScreen {
    fn create() -> Box<dyn ScreenWidget> {
        Box::new(ExampleScreen::new())
    }
}
