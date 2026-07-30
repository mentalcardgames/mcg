//! Client-side (WASM) library for the MCG app.

pub mod app;
pub mod widgets;
pub mod screens;
pub mod router;
pub mod utils;

use eframe::AppCreator;
use app::FrontendApp;
use eframe::{WebOptions, WebRunner};
use egui_extras::install_image_loaders;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlCanvasElement;

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
        $crate::log(format!($($arg)*).as_str());
    }};
}

pub fn start_game(
    canvas: HtmlCanvasElement,
    init: AppCreator<'static>,
) -> Result<(), JsValue> {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();

    // Initialize a wasm-friendly tracing subscriber so tracing::info!/warn!/error!
    // are forwarded to the browser console. tracing-wasm provides such a subscriber.
    tracing_wasm::set_as_global_default();

    let web_options = WebOptions::default();
    spawn_local(async move {
        if let Err(e) = WebRunner::new().start(canvas, web_options, init).await {
            // Avoid panicking inside wasm task; log instead
            sprintln!("Failed to start eframe: {:?}", e);
        }
    });
    Ok(())
}

#[wasm_bindgen]
pub fn start(canvas: HtmlCanvasElement) -> Result<(), JsValue> {
    let init = Box::new(|cc: &eframe::CreationContext| {
        install_image_loaders(&cc.egui_ctx);
        let app = FrontendApp::new(cc.egui_ctx.clone());
        let game: Box<dyn eframe::App> = Box::new(app);
        Ok(game)
    });
    start_game(canvas, init)
}
