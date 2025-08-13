//! Client-side routing for MCG WASM application

use crate::game::screens::{Routable, ScreenType};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{window, History, Location};

/// Router for managing client-side navigation and URL synchronization
pub struct Router {
    /// Current screen type parsed from URL
    current_screen: ScreenType,
    /// Browser history API
    history: History,
    /// Browser location API
    location: Location,
    /// Callback closure for popstate events
    _popstate_callback: Closure<dyn FnMut(web_sys::Event)>,
}

impl Router {
    /// Create a new router instance
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
    fn parse_current_screen(location: &Location) -> Result<ScreenType, JsValue> {
        let pathname = location.pathname()?;
        Ok(Routable::from_url_path(&pathname).unwrap_or(ScreenType::Main))
    }

    /// Get the current screen type
    pub fn current_screen_type(&self) -> ScreenType {
        self.current_screen
    }

    /// Navigate to a screen type
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
        // This should only be used in tests or fallback scenarios
        Self::new().unwrap_or_else(|_| Router {
            current_screen: ScreenType::Main,
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
