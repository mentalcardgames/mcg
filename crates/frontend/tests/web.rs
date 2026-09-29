use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn test_browser_environment() {
    let window = web_sys::window();
    assert!(window.is_some(), "browser window should be available");
}
