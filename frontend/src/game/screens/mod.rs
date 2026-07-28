use eframe::Frame;
use crate::game::websocket::WebSocketConnection;

pub mod articles_screen;
pub mod example_screen;
pub mod game;
pub mod game_setup_screen;
pub mod main_menu;
pub mod pairing_screen;
pub mod lobby_setup;
pub mod mcg_lobby;

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
pub use lobby_setup::LobbySelectionScreen;
pub use mcg_lobby::LobbyScreen;
use mcg_shared::{Backend2FrontendMsg, Frontend2BackendMsg, PlayerConfig};

pub struct AppInterface<'a> {
    events: &'a mut Vec<crate::game::AppEvent>,
    app_state: &'a mut crate::store::ClientState,
    ws: &'a mut WebSocketConnection,
}
impl<'a> AppInterface<'a> {
    pub fn new(events: &'a mut Vec<crate::game::AppEvent>,
               client_state: &'a mut crate::store::ClientState,
               websocket: &'a mut WebSocketConnection) -> Self {
        Self {
            events,
            app_state: client_state,
            ws: websocket,
        }
    }
    pub fn state(&mut self) -> &crate::store::ClientState {
        self.app_state
    }
    pub fn state_mut(&mut self) -> &mut crate::store::ClientState {
        self.app_state
    }
    pub fn change_screen(&mut self, screen: String) {
        self.events.push(crate::game::AppEvent::ChangeRoute(screen));
    }
    pub fn send_msg(&mut self, msg: Frontend2BackendMsg) {
        self.ws.send_msg(msg);
    }
    /// This starts a drag and drop game
    pub fn start_game(&mut self, config: GameState<DirectoryCardType>) {
        self.events.push(crate::game::AppEvent::StartGame(config));
    }
    /// This starts the static poker implementation
    pub fn create_game(&mut self, config: Vec<PlayerConfig>) {
        self.ws.create_game(config)
    }
    pub fn exit_game(&mut self) {
        self.events.push(crate::game::AppEvent::ExitGame);
    }
    pub fn is_connected(&self) -> bool {
        self.ws.is_connected()
    }
    pub fn connect(&mut self, address: &str) {
        self.ws.connect(address)
    }
    pub fn set_active_listener(&mut self, screen: Option<&str>) {
        self.ws.set_active_listener(screen)
    }
    pub fn register_listener_once(
        &mut self,
        screen: &str,
        on_message: impl Fn(Backend2FrontendMsg) + 'static,
        on_error: impl Fn(String) + 'static,
        on_close: impl Fn(String) + 'static) {
        self.ws.register_listener_once(screen, on_message, on_error, on_close)
    }
}

/// Object-safe runtime trait for drawing a screen
pub trait ScreenWidget: Downcast {
    fn ui(&mut self, app_interface: &mut AppInterface, ui: &mut egui::Ui, frame: &mut Frame);
    /// Called when the screen is about to be exited. Implement to clean up resources.
    fn on_exit(&mut self, _app_interface: &mut AppInterface) {}
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
        reg.register::<LobbySelectionScreen>();
        reg.register::<LobbyScreen>();
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
                Box::new(Self::default())
            }
        }
    };
}
