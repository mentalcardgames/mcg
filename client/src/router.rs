//! Client-side routing for MCG WASM application

use crate::game::screens::{Routable, ScreenType};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;
#[cfg(target_arch = "wasm32")]
use web_sys::{window, History, Location};

/// Router for managing client-side navigation and URL synchronization
pub struct Router {
    /// Current screen type parsed from URL
    current_screen: ScreenType,
    /// Browser history API
    #[cfg(target_arch = "wasm32")]
    history: History,
    /// Browser location API
    #[cfg(target_arch = "wasm32")]
    location: Location,
    /// Callback closure for popstate events
    #[cfg(target_arch = "wasm32")]
    _popstate_callback: Closure<dyn FnMut(web_sys::Event)>,
}

impl Router {
    /// Create a new router instance
    #[cfg(target_arch = "wasm32")]
    pub fn new() -> Result<Self, JsValue> {
        let window = window().ok_or("No window object")?;
        let history = window.history()?;
        let location = window.location();

        // Parse initial screen from current URL
        let current_screen = Self::parse_current_screen(&location)?;

        // Create popstate event listener
        let popstate_callback = Closure::wrap(Box::new(move |_event: web_sys::Event| {
            // The popstate event will be handled by checking the URL in update()
        }) as Box<dyn FnMut(web_sys::Event)>);

        // Add event listener for browser back/forward buttons
        window.add_event_listener_with_callback(
            "popstate",
            popstate_callback.as_ref().unchecked_ref(),
        )?;

        Ok(Router {
            current_screen,
            history,
            location,
            _popstate_callback: popstate_callback,
        })
    }

    /// Parse the current screen from browser location
    #[cfg(target_arch = "wasm32")]
    fn parse_current_screen(location: &Location) -> Result<ScreenType, JsValue> {
        let pathname = location.pathname()?;
        Ok(Routable::from_url_path(&pathname).unwrap_or(ScreenType::Main))
    }

    /// Get the current screen type
    pub fn current_screen_type(&self) -> ScreenType {
        self.current_screen
    }

    /// Navigate to a screen type
    #[cfg(target_arch = "wasm32")]
    pub fn navigate_to_screen(&mut self, screen_type: ScreenType) -> Result<(), JsValue> {
        if screen_type != self.current_screen {
            let path = screen_type.url_path();

            // Update browser history
            self.history
                .push_state_with_url(&JsValue::NULL, "", Some(path))?;

            self.current_screen = screen_type;
        }
        Ok(())
    }

    /// Check if the URL has changed and update current screen
    /// Returns true if the screen changed
    #[cfg(target_arch = "wasm32")]
    pub fn check_for_url_changes(&mut self) -> Result<bool, JsValue> {
        let new_screen = Self::parse_current_screen(&self.location)?;
        if new_screen != self.current_screen {
            self.current_screen = new_screen;
            return Ok(true);
        }
        Ok(false)
    }
}

#[cfg(target_arch = "wasm32")]
impl Default for Router {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| {
            panic!("Failed to create router: This should only be used in tests")
        })
    }
}

