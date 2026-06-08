use mcg_shared::{Frontend2BackendMsg, PlayerConfig, Backend2FrontendMsg};
use std::rc::Rc;
use std::collections::HashMap;
use std::cell::RefCell;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use web_sys::{CloseEvent, Event, MessageEvent, WebSocket};

/// Trait for sending messages to the server.
/// Allows decoupling UI components from the concrete WebSocket implementation.
pub trait MessageSender {
    fn send(&self, msg: &Frontend2BackendMsg);
}

/// A simplified WebSocket connection service with immediate message processing.
///
/// This service processes incoming messages immediately without queuing and triggers
/// immediate UI repaints via callback functions.
pub struct WebSocketConnection {
    ws: Option<WebSocket>,
    _onopen: Option<Closure<dyn FnMut(Event)>>,
    _onmessage: Option<Closure<dyn FnMut(MessageEvent)>>,
    _onerror: Option<Closure<dyn FnMut(Event)>>,
    _onclose: Option<Closure<dyn FnMut(CloseEvent)>>,

    /// Persistent listener maps keyed by name (e.g. screen path)
    message_listeners: Rc<RefCell<HashMap<String, Rc<dyn Fn(Backend2FrontendMsg)>>>>,
    error_listeners: Rc<RefCell<HashMap<String, Rc<dyn Fn(String)>>>>,
    close_listeners: Rc<RefCell<HashMap<String, Rc<dyn Fn(String)>>>>,

    /// The key of the currently active listener; only this listener receives messages.
    active_listener: Rc<RefCell<Option<String>>>,
// new fields active_error_listener and active_close_listener
    active_error_listener: Rc<RefCell<Option<String>>>,
    active_close_listener: Rc<RefCell<Option<String>>>,
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
            message_listeners: Rc::new(RefCell::new(HashMap::new())),
            error_listeners: Rc::new(RefCell::new(HashMap::new())),
            close_listeners: Rc::new(RefCell::new(HashMap::new())),
            active_listener: Rc::new(RefCell::new(None)),
            active_error_listener: Rc::new(RefCell::new(None)),
            active_close_listener: Rc::new(RefCell::new(None)),
        }
    }

    /// Connect to a WebSocket server. This opens a new connection and installs event handlers.
    /// It does not manage which listener is active — that is done via `set_active_listener`.
    pub fn connect(
        &mut self,
        server_address: &str,
        players: Vec<PlayerConfig>,
    ) {
        // Close any existing connection first (prevents leaking handlers)
        self.close();

        let ws_url = format!("ws://{}/ws", server_address);
        match WebSocket::new(&ws_url) {
            Ok(ws) => {
                let subscribe_json = match serde_json::to_string(&Frontend2BackendMsg::Subscribe) {
                    Ok(s) => s,
                    Err(e) => {
                        self.route_error(&format!("Failed to serialize Subscribe message: {:?}", e));
                        return;
                    }
                };
                let newgame_msg = Frontend2BackendMsg::NewGame { players: players.clone() };
                let newgame_json = match serde_json::to_string(&newgame_msg) {
                    Ok(s) => s,
                    Err(e) => {
                        self.route_error(&format!("Failed to serialize NewGame message: {:?}", e));
                        return;
                    }
                };

                let msg_map = self.message_listeners.clone();
                let err_map = self.error_listeners.clone();
                let cls_map = self.close_listeners.clone();
                let active_key = self.active_listener.clone();
                let active_err = self.active_error_listener.clone();
                let active_cls = self.active_close_listener.clone();

                // onopen: send Subscribe and NewGame
                let ws_clone_for_open = ws.clone();
                let subscribe_payload = subscribe_json;
                let newgame_payload = newgame_json;
                let onopen = Closure::<dyn FnMut(Event)>::new(move |_e: Event| {
                    let _ = ws_clone_for_open.send_with_str(&subscribe_payload);
                    let _ = ws_clone_for_open.send_with_str(&newgame_payload);
                });
                ws.set_onopen(Some(onopen.as_ref().unchecked_ref()));

                // onmessage: parse Backend2FrontendMsg and route to active listener only
                let onmessage = Closure::<dyn FnMut(MessageEvent)>::new(move |e: MessageEvent| {
                    if let Some(txt) = e.data().as_string() {
                        if let Ok(msg) = serde_json::from_str::<Backend2FrontendMsg>(&txt) {
                            if let Some(active) = active_key.borrow().as_ref() {
                                if let Some(cb) = msg_map.borrow().get(active) {
                                    cb(msg);
                                }
                            }
                        }
                    }
                });
                ws.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));

                // onerror: route to active error listener only
                let server_address_err = server_address.to_string();
                let onerror = Closure::<dyn FnMut(Event)>::new(move |_e: Event| {
                    let reason = format!("Failed to connect to {}.", server_address_err);
                    if let Some(active) = active_err.borrow().as_ref() {
                        if let Some(cb) = err_map.borrow().get(active) {
                            cb(reason);
                            return;
                        }
                    }
                    // fallback to all error listeners
                    for cb in err_map.borrow().values() {
                        cb(reason.clone());
                    }
                });
                ws.set_onerror(Some(onerror.as_ref().unchecked_ref()));

                // onclose: route to active close listener only (fallback to all)
                let onclose = Closure::<dyn FnMut(CloseEvent)>::new(move |e: CloseEvent| {
                    let reason = if e.reason().is_empty() {
                        format!("Connection closed (code {}).", e.code())
                    } else {
                        format!("Connection closed (code {}): {}", e.code(), e.reason())
                    };
                    if let Some(active) = active_cls.borrow().as_ref() {
                        if let Some(cb) = cls_map.borrow().get(active) {
                            cb(reason);
                            return;
                        }
                    }
                    for cb in cls_map.borrow().values() {
                        cb(reason.clone());
                    }
                });
                ws.set_onclose(Some(onclose.as_ref().unchecked_ref()));

                self._onopen = Some(onopen);
                self._onmessage = Some(onmessage);
                self._onerror = Some(onerror);
                self._onclose = Some(onclose);
                self.ws = Some(ws);
            }
            Err(err) => {
                self.route_error(&format!("WebSocket connect error: {:?}", err));
            }
        }
    }

    /// Register a named listener set once. If `key` already exists, just updates the callbacks without adding a new entry.
    /// Use a unique key per screen (e.g. screen path).
    pub fn register_listener_once(
        &mut self,
        key: &str,
        on_message: impl Fn(Backend2FrontendMsg) + 'static,
        on_error: impl Fn(String) + 'static,
        on_close: impl Fn(String) + 'static,
    ) {
        let key_str = key.to_string();
        {
            let mut msgs = self.message_listeners.borrow_mut();
            msgs.insert(key_str.clone(), Rc::new(on_message));
        }
        {
            let mut errs = self.error_listeners.borrow_mut();
            errs.insert(key_str.clone(), Rc::new(on_error));
        }
        {
            let mut cls = self.close_listeners.borrow_mut();
            cls.insert(key_str, Rc::new(on_close));
        }
    }

    /// Set which listener key is active. Only that listener receives events.
    pub fn set_active_listener(&mut self, key: Option<&str>) {
        *self.active_listener.borrow_mut() = key.map(|s| s.to_string());
        *self.active_error_listener.borrow_mut() = key.map(|s| s.to_string());
        *self.active_close_listener.borrow_mut() = key.map(|s| s.to_string());
    }

    /// Remove previously registered listeners if needed.
    pub fn remove_listeners(&mut self, key: &str) {
        self.message_listeners.borrow_mut().remove(key);
        self.error_listeners.borrow_mut().remove(key);
        self.close_listeners.borrow_mut().remove(key);
    }

    fn route_error(&self, err: &str) {
        // If an active error listener exists, route to it; otherwise route to all error listeners.
        if let Some(active) = self.active_error_listener.borrow().as_ref() {
            if let Some(cb) = self.error_listeners.borrow().get(active) {
                cb(err.to_string());
                return;
            }
        }
        for cb in self.error_listeners.borrow().values() {
            cb(err.to_string());
        }
    }

    /// Send a `Frontend2BackendMsg` to the server if connected.
    pub fn send_msg(&self, msg: &Frontend2BackendMsg) {
        if let Some(ws) = &self.ws {
            if let Ok(txt) = serde_json::to_string(msg) {
                if let Err(e) = ws.send_with_str(&txt) {
                    web_sys::console::log_1(&format!("Failed to send message: {:?}", e).into());
                }
            }
        }
    }

    /// Check if the WebSocket connection is open.
    pub fn is_connected(&self) -> bool {
        if let Some(ws) = &self.ws {
            ws.ready_state() == WebSocket::OPEN
        } else {
            false
        }
    }

    /// Close the WebSocket connection and drop handlers.
    pub fn close(&mut self) {
        if let Some(ws) = self.ws.take() {
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

impl MessageSender for WebSocketConnection {
    fn send(&self, msg: &Frontend2BackendMsg) {
        self.send_msg(msg);
    }
}
