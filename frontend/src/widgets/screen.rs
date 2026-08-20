use crate::screens::{ArticlesScreen, ExampleScreen, Game, GameSetupScreen, LobbyScreen, LobbySelectionScreen, MainMenu, PairingScreen, PokerOnlineScreen, QrScreen};
use crate::app::FrontendInterface;
use crate::screens::qr_test_receive::QrTestReceive;
use crate::screens::qr_test_transmit::QrTestTransmit;
use crate::widgets::card::DirectoryCardType;
use downcast_rs::{impl_downcast, Downcast};
use eframe::Frame;
use mcg_shared::Backend2FrontendMsg;
use std::any::TypeId;

/// Object-safe runtime trait for drawing a screen
pub trait ScreenWidget: Downcast {
    fn ui(&mut self, interface: &mut FrontendInterface, ui: &mut egui::Ui, frame: &mut Frame);
    /// Called when the screen is about to be exited. Implement to clean up resources.
    fn on_exit(&mut self, _interface: &mut FrontendInterface) {}
    fn on_message(&mut self, _interface: &mut FrontendInterface, _message: Backend2FrontendMsg) {}
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

/// Runtime identity for a registered screen type.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ScreenId(TypeId);

impl ScreenId {
    pub fn of<T: ScreenDef + 'static>() -> Self {
        Self(TypeId::of::<T>())
    }
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
    pub id: ScreenId,
    pub meta: ScreenMetadata,
    pub factory: fn() -> Box<dyn ScreenWidget>,
}

/// Screen registry for managing screen instances and metadata
pub struct ScreenRegistry {
    by_id: std::collections::HashMap<ScreenId, RegisteredScreen>,
    id_by_path: std::collections::HashMap<&'static str, ScreenId>,
}

impl ScreenRegistry {
    /// Ergonomic helper to register a screen type implementing ScreenDef
    pub fn register<T: ScreenDef + 'static>(&mut self) {
        let id = ScreenId::of::<T>();
        let meta = T::metadata();
        self.id_by_path.insert(meta.path, id);
        self.by_id.insert(
            id,
            RegisteredScreen {
                id,
                meta,
                factory: T::create,
            },
        );
    }

    pub fn new() -> Self {
        let mut reg = Self {
            by_id: std::collections::HashMap::new(),
            id_by_path: std::collections::HashMap::new(),
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

    /// Resolve a screen type from a URL path.
    pub fn id_by_path(&self, path: &str) -> Option<ScreenId> {
        let key = if path.is_empty() { "/" } else { path };
        self.id_by_path.get(key).copied()
    }

    /// Resolve metadata by screen type.
    pub fn meta_by_id(&self, id: ScreenId) -> Option<&ScreenMetadata> {
        self.by_id.get(&id).map(|r| &r.meta)
    }

    /// Get a screen factory by screen type.
    pub fn factory_by_id(&self, id: ScreenId) -> Option<fn() -> Box<dyn ScreenWidget>> {
        self.by_id.get(&id).map(|r| r.factory)
    }

    /// Iterate the menu screens.
    pub fn menu_entries(&self) -> Vec<(ScreenId, ScreenMetadata)> {
        let mut v: Vec<(ScreenId, ScreenMetadata)> = self
            .by_id
            .values()
            .filter(|r| r.meta.show_in_menu)
            .map(|r| (r.id, r.meta))
            .collect();
        // stable ordering by path for now
        v.sort_by_key(|(_, meta)| meta.path);
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
        impl $crate::widgets::screen::ScreenDef for $type {
            fn metadata() -> $crate::widgets::screen::ScreenMetadata
            where
                Self: Sized,
            {
                $crate::widgets::screen::ScreenMetadata {
                    path: $path,
                    display_name: $display_name,
                    icon: $icon,
                    description: $description,
                    show_in_menu: $show_in_menu,
                }
            }

            fn create() -> Box<dyn $crate::widgets::screen::ScreenWidget>
            where
                Self: Sized,
            {
                Box::new(Self::default())
            }
        }
    };
}
