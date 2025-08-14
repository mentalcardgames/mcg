//! Client-side (WASM) library for the MCG app.


pub mod articles;
pub mod game;
pub mod hardcoded_cards;
#[cfg(target_arch = "wasm32")]
pub mod router;
pub mod utils;
pub mod qr_scanner;

#[allow(unused_imports)]
use eframe::AppCreator;
#[cfg(target_arch = "wasm32")]
use eframe::{WebOptions, WebRunner};
#[cfg(target_arch = "wasm32")]
use egui_extras::install_image_loaders;
#[cfg(target_arch = "wasm32")]
use game::App;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::spawn_local;

#[cfg(target_arch = "wasm32")]
use web_sys::{window, HtmlCanvasElement};

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
extern "C" {
    /// JavaScript console.log binding for debug output
    #[wasm_bindgen(js_namespace = console)]
    pub fn log(s: &str);
}

/// Platform-agnostic println! alternative that works in both native and WASM targets
#[macro_export]
macro_rules! sprintln {
	($($arg:tt)*) => {{
		#[cfg(target_arch = "wasm32")]
		$crate::log(format!($($arg)*).as_str());
		#[cfg(not(target_arch = "wasm32"))]
		println!($($arg)*);
	}};
}

#[cfg(target_arch = "wasm32")]
pub fn start_game(
    canvas: web_sys::HtmlCanvasElement,
    init: AppCreator<'static>,
) -> Result<(), JsValue> {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
    let web_options = WebOptions::default();
    spawn_local(async move {
        WebRunner::new()
            .start(canvas, web_options, init)
            .await
            .expect("Failed to start eframe");
    });
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
pub fn start_game(_canvas: (), _init: AppCreator<'static>) -> Result<(), ()> { Ok(()) }

#[cfg(target_arch = "wasm32")]
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
    let scale = base_scale * (device_pixel_ratio / 2.0).max(0.75).min(1.5);
    scale
}

#[cfg(not(target_arch = "wasm32"))]
pub fn calculate_dpi_scale() -> f32 {
    1.5
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn start(canvas: HtmlCanvasElement) -> Result<(), JsValue> {
    let init = Box::new(|cc: &eframe::CreationContext| {
        install_image_loaders(&cc.egui_ctx);
        let app = App::new();
        let game: Box<dyn eframe::App> = Box::new(app);
        Ok(game)
    });
    start_game(canvas, init)
}
