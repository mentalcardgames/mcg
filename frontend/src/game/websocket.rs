use mcg_shared::{ClientMsg, PlayerConfig, ServerMsg};
use std::rc::Rc;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use web_sys::{CloseEvent, Event, MessageEvent, WebSocket};

/// A simplified WebSocket connection service with immediate message processing.
///
/// This service processes incoming messages immediately without queuing and triggers
/// immediate UI repaints via callback functions.
pub struct WebSocketConnection {
    ws: Option<WebSocket>,
    _onopen: Option<Closure<dyn FnMut(Event)>>,
    // Store closure handles to prevent memory leaks
    _onmessage: Option<Closure<dyn FnMut(MessageEvent)>>,
    _onerror: Option<Closure<dyn FnMut(Event)>>,
    _onclose: Option<Closure<dyn FnMut(CloseEvent)>>,
}

impl Default for WebSocketConnection {
    fn default() -> Self {
        Self::new()
    }
}

impl WebSocketConnection {
    pub fn new() -> Self {
        Self {
            ws: None,
            _onopen: None,
            _onmessage: None,
            _onerror: None,
            _onclose: None,
        }
    }

    /// Connect to a WebSocket server with immediate message processing.
    ///
    /// Establishes a connection and sets up event handlers that immediately
    /// process incoming messages and trigger UI updates via callbacks.
    pub fn connect(
        &mut self,
        server_address: &str,
        players: Vec<PlayerConfig>,
        on_message: impl Fn(ServerMsg) + 'static,
        on_error: impl Fn(String) + 'static,
        on_close: impl Fn(String) + 'static,
    ) {
        // Close any existing connection before starting a new one
        self.close();

        // Wrap callbacks in Rc to share with closures
        let on_message = Rc::new(on_message);
        let on_error = Rc::new(on_error);
        let on_close = Rc::new(on_close);

        let ws_url = format!("ws://{}/ws", server_address);
        match WebSocket::new(&ws_url) {
            Ok(ws) => {
                // Prepare the Subscribe and initial NewGame messages
                let subscribe_json = match serde_json::to_string(&ClientMsg::Subscribe) {
                    Ok(s) => s,
                    Err(e) => {
                        on_error(format!("Failed to serialize Subscribe message: {:?}", e));
                        return;
                    }
                };
                let newgame_msg = ClientMsg::NewGame {
                    players: players.clone(),
                };
                let newgame_json = match serde_json::to_string(&newgame_msg) {
                    Ok(s) => s,
                    Err(e) => {
                        on_error(format!("Failed to serialize NewGame message: {:?}", e));
                        return;
                    }
                };

                let ws_clone_for_open = ws.clone();
                let subscribe_payload = subscribe_json;
                let newgame_payload = newgame_json;
                let on_error_clone = on_error.clone();
                let onopen = Closure::<dyn FnMut(Event)>::new(move |_e: Event| {
                    if let Err(e) = ws_clone_for_open.send_with_str(&subscribe_payload) {
                        on_error_clone(format!("Error sending Subscribe: {:?}", e));
                        return;
                    }
                    if let Err(e) = ws_clone_for_open.send_with_str(&newgame_payload) {
                        on_error_clone(format!("Error sending NewGame: {:?}", e));
                    }
                });
                ws.set_onopen(Some(onopen.as_ref().unchecked_ref()));

                // onmessage: Parse ServerMsg and process immediately
                let on_message_clone = on_message.clone();
                let onmessage = Closure::<dyn FnMut(MessageEvent)>::new(move |e: MessageEvent| {
                    if let Some(txt) = e.data().as_string() {
                        if let Ok(msg) = serde_json::from_str::<ServerMsg>(&txt) {
                            // Process the message immediately via callback
                            on_message_clone(msg);
                        }
                    }
                });
                ws.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));

                // onerror: Process error immediately
                let server_address_err = server_address.to_string();
                let on_error_clone = on_error.clone();
                let onerror = Closure::<dyn FnMut(Event)>::new(move |_e: Event| {
                    on_error_clone(format!("Failed to connect to {}.", server_address_err));
                });
                ws.set_onerror(Some(onerror.as_ref().unchecked_ref()));

                // onclose: Process close immediately
                let on_close_clone = on_close.clone();
                let onclose = Closure::<dyn FnMut(CloseEvent)>::new(move |e: CloseEvent| {
                    let reason = if e.reason().is_empty() {
                        format!("Connection closed (code {}).", e.code())
                    } else {
                        format!("Connection closed (code {}): {}", e.code(), e.reason())
                    };
                    on_close_clone(reason);
                });
                ws.set_onclose(Some(onclose.as_ref().unchecked_ref()));

                // Store the closures to manage their lifetime properly
                self._onmessage = Some(onmessage);
                self._onerror = Some(onerror);
                self._onclose = Some(onclose);
                self._onopen = Some(onopen);
                self.ws = Some(ws);
            }
            Err(err) => {
                // Handle initial WebSocket creation error
                on_error(format!("WebSocket connect error: {:?}", err));
            }
        }
    }

    /// Send a `ClientMsg` to the server if connected.
    pub fn send_msg(&self, msg: &ClientMsg) {
        if let Some(ws) = &self.ws {
            if let Ok(txt) = serde_json::to_string(msg) {
                if let Err(e) = ws.send_with_str(&txt) {
                    web_sys::console::log_1(&format!("Failed to send message: {:?}", e).into());
                }
            }
        }
    }

    /// Close the WebSocket connection.
    pub fn close(&mut self) {
        if let Some(ws) = self.ws.take() {
            // Clear event handlers before closing to prevent leaks
            ws.set_onmessage(None);
            ws.set_onerror(None);
            ws.set_onclose(None);
            ws.set_onopen(None);

            // The onclose event will handle state updates
            let _ = ws.close();
        }

        // Drop the closure handles to free memory
        self._onopen = None;
        self._onmessage = None;
        self._onerror = None;
        self._onclose = None;
    }
}

/// Implement Drop to ensure proper cleanup even if close() isn't called explicitly
impl Drop for WebSocketConnection {
    fn drop(&mut self) {
        // Calling close() here handles all cleanup and is idempotent
        self.close();
    }
}
