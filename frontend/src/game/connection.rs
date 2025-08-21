use mcg_shared::{ClientMsg, PlayerConfig, ServerMsg};
use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use web_sys::{CloseEvent, Event, MessageEvent, WebSocket};

/// An event-driven WebSocket connection service.
///
/// This service encapsulates WebSocket logic. Instead of buffering messages
/// for polling, it directly applies incoming messages and errors to a shared
/// state object (`SharedState`) and requests a repaint from the `egui` context.
/// This makes the architecture event-driven and removes the need for polling.
pub struct ConnectionService {
    ws: Option<WebSocket>,
    event_queue: Rc<RefCell<VecDeque<ServerMsg>>>,
}

impl Default for ConnectionService {
    fn default() -> Self {
        Self::new()
    }
}

impl ConnectionService {
    pub fn new() -> Self {
        Self {
            ws: None,
            event_queue: Rc::new(RefCell::new(VecDeque::new())),
        }
    }

    pub fn poll_messages(&mut self) -> impl Iterator<Item = ServerMsg> {
        let mut queue = self.event_queue.borrow_mut();
        std::mem::take(&mut *queue).into_iter()
    }

    /// Connect to a WebSocket server.
    ///
    /// Establishes a connection and sets up event handlers (`onmessage`, `onerror`, `onclose`)
    /// that directly mutate the provided `SharedState` and request UI repaints.
    pub fn connect(&mut self, server_address: &str, players: Vec<PlayerConfig>) {
        // Close any existing connection before starting a new one.
        self.close();
        // The event queue is now part of `self`, so we just clone it for the closures.
        let event_queue = self.event_queue.clone();

        let ws_url = format!("ws://{}/ws", server_address);
        match WebSocket::new(&ws_url) {
            Ok(ws) => {
                // Prepare the initial NewGame message payload.
                let newgame_msg = ClientMsg::NewGame {
                    players: players.clone(),
                };
                let newgame_json = serde_json::to_string(&newgame_msg).unwrap();

                // Clone shared resources for the event handlers.
                let event_queue_for_msg = event_queue.clone();
                let ws_clone_for_msg = ws.clone();

                // onmessage: Parse ServerMsg, apply to state, and request repaint.
                let onmessage = Closure::<dyn FnMut(MessageEvent)>::new(move |e: MessageEvent| {
                    if let Some(txt) = e.data().as_string() {
                        if let Ok(msg) = serde_json::from_str::<ServerMsg>(&txt) {
                            // If the server sends Welcome, respond with the NewGame message.
                            if let ServerMsg::Welcome = &msg {
                                if let Err(e) = ws_clone_for_msg.send_with_str(&newgame_json) {
                                    // Handle potential send error
                                    event_queue_for_msg.borrow_mut().push_back(ServerMsg::Error(
                                        format!("Error sending NewGame: {:?}", e),
                                    ));
                                }
                            }
                            // Apply the message to the shared state.
                            event_queue_for_msg.borrow_mut().push_back(msg);
                        }
                    }
                });
                ws.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));
                onmessage.forget(); // Leaks the closure to keep it alive.

                // onerror: Update state with the error and request repaint.
                let event_queue_for_err = event_queue.clone();
                let server_address_err = server_address.to_string();
                let onerror = Closure::<dyn FnMut(Event)>::new(move |_e: Event| {
                    event_queue_for_err.borrow_mut().push_back(ServerMsg::Error(
                        format!("Failed to connect to {}.", server_address_err),
                    ));
                });
                ws.set_onerror(Some(onerror.as_ref().unchecked_ref()));
                onerror.forget();

                // onclose: Update state with close info and request repaint.
                let event_queue_for_close = event_queue.clone();
                let onclose = Closure::<dyn FnMut(CloseEvent)>::new(move |e: CloseEvent| {
                    let reason = if e.reason().is_empty() {
                        format!("Connection closed (code {}).", e.code())
                    } else {
                        format!("Connection closed (code {}): {}", e.code(), e.reason())
                    };
                    event_queue_for_close
                        .borrow_mut()
                        .push_back(ServerMsg::Error(reason));
                });
                ws.set_onclose(Some(onclose.as_ref().unchecked_ref()));
                onclose.forget();

                self.ws = Some(ws);
            }
            Err(err) => {
                // Handle initial WebSocket creation error.
                event_queue.borrow_mut().push_back(ServerMsg::Error(
                    format!("WebSocket connect error: {:?}", err),
                ));
            }
        }
        // The Rc is now held by self, so we don't need to unwrap it.
        // This was the source of the panic.
    }

    /// Send a `ClientMsg` to the server if connected.
    pub fn send(&self, msg: &ClientMsg) {
        if let Some(ws) = &self.ws {
            if let Ok(txt) = serde_json::to_string(msg) {
                if let Err(e) = ws.send_with_str(&txt) {
                    // Optionally handle send errors, e.g., by updating the state.
                    // For now, we log to the console.
                    crate::sprintln!("Failed to send message: {:?}", e);
                }
            }
        }
    }

    /// Close the WebSocket connection.
    pub fn close(&mut self) {
        if let Some(ws) = self.ws.take() {
            // The onclose event will handle state updates.
            let _ = ws.close();
        }
    }
}
