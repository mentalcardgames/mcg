//! Client-side routing for MCG WASM application

use crate::game::screens::ScreenType;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{window, History, Location, PopStateEvent};

/// Router for managing client-side navigation and URL synchronization
pub struct Router {
    /// Current route parsed from URL
    current_route: Route,
    /// Browser history API
    history: History,
    /// Browser location API
    location: Location,
    /// Callback closure for popstate events
    _popstate_callback: Closure<dyn FnMut(web_sys::Event)>,
}

/// Represents different routes in the application
#[derive(Debug, Clone, PartialEq)]
pub enum Route {
    Home,
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

impl Route {
    /// Convert route to URL path
    pub fn to_path(&self) -> &'static str {
        match self {
            Route::Home => "/",
            Route::GameSetup => "/game-setup",
            Route::Game => "/game",
            Route::Pairing => "/pairing",
            Route::Settings => "/settings",
            Route::DndTest => "/dnd-test",
            Route::GameDndSetup => "/game-dnd-setup",
            Route::GameDnd => "/game-dnd",
            Route::Articles => "/articles",
            Route::QRScreen => "/qr",
            Route::PokerOnline => "/poker-online",
        }
    }

    /// Parse URL path to route
    pub fn from_path(path: &str) -> Route {
        match path {
            "/" | "" => Route::Home,
            "/game-setup" => Route::GameSetup,
            "/game" => Route::Game,
            "/pairing" => Route::Pairing,
            "/settings" => Route::Settings,
            "/dnd-test" => Route::DndTest,
            "/game-dnd-setup" => Route::GameDndSetup,
            "/game-dnd" => Route::GameDnd,
            "/articles" => Route::Articles,
            "/qr" => Route::QRScreen,
            "/poker-online" => Route::PokerOnline,
            _ => Route::Home, // Default fallback
        }
    }

    /// Convert route to screen type
    pub fn to_screen_type(&self) -> ScreenType {
        match self {
            Route::Home => ScreenType::Main,
            Route::GameSetup => ScreenType::GameSetup,
            Route::Game => ScreenType::Game,
            Route::Pairing => ScreenType::Pairing,
            Route::Settings => ScreenType::Settings,
            Route::DndTest => ScreenType::DndTest,
            Route::GameDndSetup => ScreenType::GameDndSetup,
            Route::GameDnd => ScreenType::GameDnd,
            Route::Articles => ScreenType::Articles,
            Route::QRScreen => ScreenType::QRScreen,
            Route::PokerOnline => ScreenType::PokerOnline,
        }
    }

    /// Convert screen type to route
    pub fn from_screen_type(screen_type: ScreenType) -> Route {
        match screen_type {
            ScreenType::Main => Route::Home,
            ScreenType::GameSetup => Route::GameSetup,
            ScreenType::Game => Route::Game,
            ScreenType::Pairing => Route::Pairing,
            ScreenType::Settings => Route::Settings,
            ScreenType::DndTest => Route::DndTest,
            ScreenType::GameDndSetup => Route::GameDndSetup,
            ScreenType::GameDnd => Route::GameDnd,
            ScreenType::Articles => Route::Articles,
            ScreenType::QRScreen => Route::QRScreen,
            ScreenType::PokerOnline => Route::PokerOnline,
        }
    }
}

impl Router {
    /// Create a new router instance
    pub fn new() -> Result<Self, JsValue> {
        let window = window().ok_or("No window object")?;
        let history = window.history()?;
        let location = window.location();

        // Parse initial route from current URL
        let current_route = Self::parse_current_route(&location)?;

        // Create popstate event listener
        let popstate_callback = Closure::wrap(Box::new(move |_event: web_sys::Event| {
            // The popstate event will be handled by checking the URL in update()
            crate::sprintln!("Popstate event received");
        }) as Box<dyn FnMut(web_sys::Event)>);

        // Add event listener for browser back/forward buttons
        window.add_event_listener_with_callback(
            "popstate",
            popstate_callback.as_ref().unchecked_ref(),
        )?;

        Ok(Router {
            current_route,
            history,
            location,
            _popstate_callback: popstate_callback,
        })
    }

    /// Parse the current route from browser location
    fn parse_current_route(location: &Location) -> Result<Route, JsValue> {
        let pathname = location.pathname()?;
        Ok(Route::from_path(&pathname))
    }

    /// Get the current route
    pub fn current_route(&self) -> &Route {
        &self.current_route
    }

    /// Navigate to a new route
    pub fn navigate_to(&mut self, route: Route) -> Result<(), JsValue> {
        if route != self.current_route {
            let path = route.to_path();

            // Update browser history
            self.history
                .push_state_with_url(&JsValue::NULL, "", Some(path))?;

            self.current_route = route;
            crate::sprintln!("Navigated to: {}", path);
        }
        Ok(())
    }

    /// Navigate to a route by screen type
    pub fn navigate_to_screen(&mut self, screen_type: ScreenType) -> Result<(), JsValue> {
        let route = Route::from_screen_type(screen_type);
        self.navigate_to(route)
    }

    /// Check if the URL has changed and update current route
    /// Returns true if the route changed
    pub fn check_for_url_changes(&mut self) -> Result<bool, JsValue> {
        let new_route = Self::parse_current_route(&self.location)?;
        if new_route != self.current_route {
            crate::sprintln!(
                "Route changed from {:?} to {:?}",
                self.current_route,
                new_route
            );
            self.current_route = new_route;
            return Ok(true);
        }
        Ok(false)
    }

    /// Get the screen type for the current route
    pub fn current_screen_type(&self) -> ScreenType {
        self.current_route.to_screen_type()
    }
}

#[cfg(target_arch = "wasm32")]
impl Default for Router {
    fn default() -> Self {
        // This should only be used in tests or fallback scenarios
        Self::new().unwrap_or_else(|_| Router {
            current_route: Route::Home,
            history: unsafe { std::mem::zeroed() },
            location: unsafe { std::mem::zeroed() },
            _popstate_callback: Closure::wrap(Box::new(|_| {}) as Box<dyn FnMut(web_sys::Event)>),
        })
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl Default for Router {
    fn default() -> Self {
        panic!("Router is only available on WASM targets");
    }
}
