use web_sys::window;

pub const MARGIN_SM: f32 = 8.0;
pub const MARGIN_MD: f32 = 12.0;
pub const MARGIN_LG: f32 = 16.0;
pub const MARGIN_XL: f32 = 32.0;

pub const NAVBAR_WIDTH_LEFT: f32 = 120.0;
pub const NAVBAR_WIDTH_RIGHT: f32 = 140.0;
pub const NAVBAR_ROW_HEIGHT_EXTRA: f32 = 12.0;

pub const FONT_SIZE_XS: f32 = 14.0;
pub const FONT_SIZE_SM: f32 = 16.0;
pub const FONT_SIZE_MD: f32 = 24.0;
pub const FONT_SIZE_LG: f32 = 48.0;

pub const BUTTON_MIN_HEIGHT: f32 = 24.0;
pub const BUTTON_MIN_WIDTH: f32 = 80.0;

pub fn calculate_dpi_scale() -> f32 {
    let window = window().expect("no global window exists");
    let device_pixel_ratio = window.device_pixel_ratio() as f32;
    let screen = window.screen().expect("unable to get screen object");
    let width = screen.width().unwrap_or(1920) as f32;
    let height = screen.height().unwrap_or(1080) as f32;
    let diagonal = (width * width + height * height).sqrt();
    let base_scale = if diagonal > 3000.0 {
        1.8
    } else if diagonal > 2000.0 {
        1.4
    } else if diagonal > 1500.0 {
        1.2
    } else {
        1.0
    };
    base_scale * (device_pixel_ratio / 2.0).clamp(0.75, 1.5)
}
