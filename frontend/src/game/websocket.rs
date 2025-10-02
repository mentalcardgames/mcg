use egui::Context;
use mcg_shared::{ClientMsg, PlayerConfig, ServerMsg};
use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use web_sys::{CloseEvent, Event, MessageEvent, WebSocket};

/// A simplified WebSocket connection service that queues incoming messages.
///
/// This service queues incoming `ServerMsg` instances into a shared `VecDeque`
/// and requests a repaint from the `egui::Context` to ensure the UI thread
/// processes the message promptly.
pub struct WebSocketConnection {
    ws: Option<WebSocket>,
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
            _onmessage: None,
            _onerror: None,
            _onclose: None,
        }
    }

    /// Connect to a WebSocket server and set up event handlers.
    ///
    /// Messages are queued into `pending_messages` and repaints are requested
    /// via the `egui::Context`.
    pub fn connect(
        &mut self,
        server_address: &str,
        players: Vec<PlayerConfig>,
        pending_messages: Rc<RefCell<VecDeque<ServerMsg>>>,
        ctx: Context,
    ) {
        // Close any existing connection before starting a new one
        self.close();

        let ws_url = format!("ws://{}/ws", server_address);
        match WebSocket::new(&ws_url) {
            Ok(ws) => {
                let newgame_msg = ClientMsg::NewGame {
                    players: players.clone(),
                };
                let newgame_json = match serde_json::to_string(&newgame_msg) {
                    Ok(s) => s,
                    Err(e) => {
                        let err_msg =
                            ServerMsg::Error(format!("Failed to serialize NewGame message: {:?}", e));
                        pending_messages.borrow_mut().push_back(err_msg);
                        ctx.request_repaint();
                        return;
                    }
                };

                // onmessage: Parse ServerMsg, queue it, and request a repaint.
                let ws_clone_for_msg = ws.clone();
                let pending_messages_clone = pending_messages.clone();
                let ctx_clone = ctx.clone();
                let onmessage = Closure::<dyn FnMut(MessageEvent)>::new(move |e: MessageEvent| {
                    if let Some(txt) = e.data().as_string() {
                        if let Ok(msg) = serde_json::from_str::<ServerMsg>(&txt) {
                            if let ServerMsg::Welcome = &msg {
                                if let Err(e) = ws_clone_for_msg.send_with_str(&newgame_json) {
                                    let err_msg =
                                        ServerMsg::Error(format!("Error sending NewGame: {:?}", e));
                                    pending_messages_clone.borrow_mut().push_back(err_msg);
                                }
                            }
                            pending_messages_clone.borrow_mut().push_back(msg);
                            ctx_clone.request_repaint();
                        }
                    }
                });
                ws.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));

                // onerror: Queue an error message.
                let server_address_err = server_address.to_string();
                let pending_messages_clone = pending_messages.clone();
                let ctx_clone = ctx.clone();
                let onerror = Closure::<dyn FnMut(Event)>::new(move |_e: Event| {
                    let err_msg =
                        ServerMsg::Error(format!("Failed to connect to {}.", server_address_err));
                    pending_messages_clone.borrow_mut().push_back(err_msg);
                    ctx_clone.request_repaint();
                });
                ws.set_onerror(Some(onerror.as_ref().unchecked_ref()));

                // onclose: Queue a close message as an error.
                let pending_messages_clone = pending_messages.clone();
                let onclose = Closure::<dyn FnMut(CloseEvent)>::new(move |e: CloseEvent| {
                    let reason = if e.reason().is_empty() {
                        format!("Connection closed (code {}).", e.code())
                    } else {
                        format!("Connection closed (code {}): {}", e.code(), e.reason())
                    };
                    pending_messages_clone
                        .borrow_mut()
                        .push_back(ServerMsg::Error(reason));
                    ctx.request_repaint();
                });
                ws.set_onclose(Some(onclose.as_ref().unchecked_ref()));

                // Store the closures to manage their lifetime properly
                self._onmessage = Some(onmessage);
                self._onerror = Some(onerror);
                self._onclose = Some(onclose);
                self.ws = Some(ws);
            }
            Err(err) => {
                let err_msg = ServerMsg::Error(format!("WebSocket connect error: {:?}", err));
                pending_messages.borrow_mut().push_back(err_msg);
                ctx.request_repaint();
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

            // The onclose event will handle state updates
            let _ = ws.close();
        }

        // Drop the closure handles to free memory
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
