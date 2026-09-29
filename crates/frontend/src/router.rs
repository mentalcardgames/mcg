//! Client-side routing for MCG WASM application

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{window, History, Location};

/// Router for managing client-side navigation and URL synchronization
pub struct Router {
    /// Current path (pathname) observed in the browser
    current_path: String,
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

        // Parse initial path from current URL
        let current_path = Self::parse_current_path(&location)?;

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
            current_path,
            history,
            location,
            _popstate_callback: popstate_callback,
        })
    }

    /// Parse the current path from browser location
    fn parse_current_path(location: &Location) -> Result<String, JsValue> {
        let pathname = location.pathname()?;
        // Ensure non-empty and always start with '/'
        let p = if pathname.is_empty() {
            "/".to_string()
        } else {
            pathname
        };
        Ok(p)
    }

    /// Get the current path string
    pub fn current_path(&self) -> &str {
        &self.current_path
    }

    /// Navigate to a path
    pub fn navigate_to_path(&mut self, path: &str) -> Result<(), JsValue> {
        let path = if path.is_empty() { "/" } else { path };
        if path != self.current_path {
            // Update browser history
            self.history
                .push_state_with_url(&JsValue::NULL, "", Some(path))?;
            self.current_path = path.to_string();
        }
        Ok(())
    }

    /// Check if the URL has changed and update current path
    /// Returns true if the path changed
    pub fn check_for_url_changes(&mut self) -> Result<bool, JsValue> {
        let new_path = Self::parse_current_path(&self.location)?;
        if new_path != self.current_path {
            self.current_path = new_path;
            return Ok(true);
        }
        Ok(false)
    }
}

impl Default for Router {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| {
            panic!("Failed to create router: This should only be used in tests")
        })
    }
}
