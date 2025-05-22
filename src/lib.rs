//! MCG - A mental  card game implementation for the browser
//!
//! This provides a MCG implemetation in wasm with an egui frontend.

pub mod example;
pub mod game;
pub mod hardcoded_cards;
mod utils;

#[allow(unused_imports)]
use eframe::AppCreator;
#[cfg(target_arch = "wasm32")]
use eframe::{WebOptions, WebRunner};
#[cfg(target_arch = "wasm32")]
use egui_extras::install_image_loaders;
#[cfg(target_arch = "wasm32")]
use game::screen::{CardsTestDND, DNDTest, Game, GameSetupScreen};
#[cfg(target_arch = "wasm32")]
use game::App;
#[cfg(target_arch = "wasm32")]
use std::cell::RefCell;
#[cfg(target_arch = "wasm32")]
use std::rc::Rc;
#[allow(unused_imports)]
use wasm_bindgen::prelude::*;
#[allow(unused_imports)]
use wasm_bindgen_futures::spawn_local;
#[allow(unused_imports)]
use web_sys::js_sys::Promise;
#[cfg(target_arch = "wasm32")]
use web_sys::HtmlCanvasElement;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
extern "C" {
    /// JavaScript console.log binding for debug output
    #[wasm_bindgen(js_namespace = console)]
    pub fn log(s: &str);
}

/// Helper function to start the eframe app with a canvas element
///
/// This function is used internally by the `start` function, but can also
/// be used directly for more control over the initialization process.
///
/// # Arguments
///
/// * `canvas` - HTML canvas element to render to
/// * `init` - App creator function
///
/// # Returns
///
/// Result indicating success or failure
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

/* TODO implement right-click with popup when this
    https://github.com/emilk/egui/blob/master/crates/egui/src/containers/popup.rs
    gets into a proper egui release
*/

/// Platform-agnostic println! alternative that works in both native and WASM targets
///
/// This macro will use console.log() in WASM targets and regular println! in native targets,
/// allowing for consistent debug output across platforms.
///
/// # Examples
///
/// ```
/// use mcg::sprintln;
///
/// // Works the same as println!
/// sprintln!("Hello, world!");
/// sprintln!("Value: {}", 42);
/// ```
#[macro_export]
macro_rules! sprintln {
    ($($arg:tt)*) => {{
        #[cfg(target_arch = "wasm32")]
        $crate::log(format!($($arg)*).as_str());
        #[cfg(not(target_arch = "wasm32"))]
        println!($($arg)*);
    }};
}

/// Main entry point for starting the WASM application in a browser
///
/// This function should be called from JavaScript to initialize and run the application
/// on the provided canvas element.
///
/// # Arguments
///
/// * `canvas` - The HTML canvas element to render the application to
///
/// # Returns
///
/// Result indicating success or failure
///
/// # Example (JavaScript)
///
/// ```javascript
/// import init, {start} from './pkg/mcg.js';
///
/// async function run() {
///     await init();
///     start(document.getElementById("mcg_canvas"));
/// }
///
/// run();
/// ```
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn start(canvas: HtmlCanvasElement) -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    let init = Box::new(|cc: &eframe::CreationContext| {
        install_image_loaders(&cc.egui_ctx);
        let mut app = App::default();

        // Create game components
        let game_widget = Rc::new(RefCell::new(Game::new()));
        let game_conf = Rc::new(RefCell::new(GameSetupScreen::new(Rc::downgrade(
            &game_widget,
        ))));

        // Set hardcoded standard deck (img_cards) for the main game
        hardcoded_cards::set_hardcoded_deck(&game_conf.borrow().directory, false);

        let dnd_test = Rc::new(RefCell::new(DNDTest::new()));

        // Register main game screens
        app.register_screen(String::from("game"), game_widget)
            .unwrap();
        app.register_screen(String::from("game_setup"), game_conf)
            .unwrap();
        app.register_screen(String::from("dnd_test"), dnd_test)
            .unwrap();

        // Register drag and drop game screens
        let game_dnd_widget = Rc::new(RefCell::new(CardsTestDND::new()));
        let game_dnd_conf = Rc::new(RefCell::new(GameSetupScreen::new(Rc::downgrade(
            &game_dnd_widget,
        ))));

        // Set hardcoded alternative deck (alt_cards) for drag and drop game
        hardcoded_cards::set_hardcoded_deck(&game_dnd_conf.borrow().directory, true);
        app.register_screen(String::from("game_dnd_setup"), game_dnd_conf)
            .unwrap();
        app.register_screen(String::from("game_dnd"), game_dnd_widget)
            .unwrap();

        let game: Box<dyn eframe::App> = Box::new(app);
        Ok(game)
    });
    start_game(canvas, init)
}
