use crate::widgets::screen::ScreenRegistry;
use crate::widgets::theme::calculate_dpi_scale;

/// Global state/settings for the application
pub struct FrontendState {
    pub name: String,
    pub server_address: String,
    pub dpi: f32,
    pub applied_dpi: f32,
    pub dark_mode: bool,
    pub screen_registry: ScreenRegistry,
}

impl Default for FrontendState {
    fn default() -> Self {
        Self::new()
    }
}

impl FrontendState {
    pub fn new() -> Self {
        let dpi = calculate_dpi_scale();
        FrontendState {
            name: "Player".to_string(),
            server_address: "127.0.0.1:3000".to_string(),
            dpi,
            applied_dpi: dpi,
            dark_mode: true,
            screen_registry: ScreenRegistry::new(),
        }
    }
}
