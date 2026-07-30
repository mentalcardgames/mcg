use mcg_shared::{Backend2FrontendMsg, Frontend2BackendMsg, PlayerConfig};
use std::sync::mpsc::Sender;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use web_sys::{CloseEvent, Event, MessageEvent, WebSocket};

/// Trait for sending messages to the server.
/// Allows UI components to use the application-owned WebSocket without owning it.
pub trait MessageSender {
    fn send(&self, msg: Frontend2BackendMsg);
}

pub struct WebSocketConnection {
    ws: Option<WebSocket>,
    message_sender: Sender<Backend2FrontendMsg>,
    error_sender: Sender<Event>,
    close_sender: Sender<CloseEvent>,
    _onopen: Option<Closure<dyn FnMut(Event)>>,
    _onmessage: Option<Closure<dyn FnMut(MessageEvent)>>,
    _onerror: Option<Closure<dyn FnMut(Event)>>,
    _onclose: Option<Closure<dyn FnMut(CloseEvent)>>,
}

impl WebSocketConnection {
    pub fn new(
        message_sender: Sender<Backend2FrontendMsg>,
        error_sender: Sender<Event>,
        close_sender: Sender<CloseEvent>,
    ) -> Self {
        Self {
            ws: None,
            message_sender,
            error_sender,
            close_sender,
            _onopen: None,
            _onmessage: None,
            _onerror: None,
            _onclose: None,
        }
    }

    /// Connect to a WebSocket server and install the application-level event handlers.
    pub fn connect(&mut self, server_address: &str) {
        self.close();

        let ws_url = format!("ws://{}/ws", server_address);
        match WebSocket::new(&ws_url) {
            Ok(ws) => {
                let subscribe_json = match serde_json::to_string(&Frontend2BackendMsg::Subscribe) {
                    Ok(payload) => payload,
                    Err(error) => {
                        web_sys::console::error_1(
                            &format!("Failed to serialize Subscribe message: {error:?}").into(),
                        );
                        return;
                    }
                };

                let ws_clone_for_open = ws.clone();
                let onopen = Closure::<dyn FnMut(Event)>::new(move |_event: Event| {
                    let _ = ws_clone_for_open.send_with_str(&subscribe_json);
                });
                ws.set_onopen(Some(onopen.as_ref().unchecked_ref()));

                let message_sender = self.message_sender.clone();
                let onmessage =
                    Closure::<dyn FnMut(MessageEvent)>::new(move |event: MessageEvent| {
                        if let Some(text) = event.data().as_string() {
                            match serde_json::from_str::<Backend2FrontendMsg>(&text) {
                                Ok(message) => {
                                    let _ = message_sender.send(message);
                                }
                                Err(error) => web_sys::console::error_1(
                                    &format!("Failed to deserialize server message: {error:?}")
                                        .into(),
                                ),
                            }
                        }
                    });
                ws.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));

                let error_sender = self.error_sender.clone();
                let onerror = Closure::<dyn FnMut(Event)>::new(move |event: Event| {
                    let _ = error_sender.send(event);
                });
                ws.set_onerror(Some(onerror.as_ref().unchecked_ref()));

                let close_sender = self.close_sender.clone();
                let onclose = Closure::<dyn FnMut(CloseEvent)>::new(move |event: CloseEvent| {
                    let _ = close_sender.send(event);
                });
                ws.set_onclose(Some(onclose.as_ref().unchecked_ref()));

                self._onopen = Some(onopen);
                self._onmessage = Some(onmessage);
                self._onerror = Some(onerror);
                self._onclose = Some(onclose);
                self.ws = Some(ws);
            }
            Err(error) => {
                web_sys::console::error_1(&format!("WebSocket connect error: {error:?}").into());
            }
        }
    }

    pub fn send_msg(&self, msg: Frontend2BackendMsg) {
        if let Some(ws) = &self.ws {
            if let Ok(text) = serde_json::to_string(&msg) {
                if let Err(error) = ws.send_with_str(&text) {
                    web_sys::console::error_1(&format!("Failed to send message: {error:?}").into());
                }
            }
        }
    }

    pub fn create_game(&self, players: Vec<PlayerConfig>) {
        self.send_msg(Frontend2BackendMsg::NewGame { players });
    }

    pub fn is_connected(&self) -> bool {
        self.ws
            .as_ref()
            .is_some_and(|ws| ws.ready_state() == WebSocket::OPEN)
    }

    pub fn close(&mut self) {
        if let Some(ws) = self.ws.take() {
            ws.set_onmessage(None);
            ws.set_onerror(None);
            ws.set_onclose(None);
            ws.set_onopen(None);
            let _ = ws.close();
        }

        self._onopen = None;
        self._onmessage = None;
        self._onerror = None;
        self._onclose = None;
    }
}

impl Drop for WebSocketConnection {
    fn drop(&mut self) {
        self.close();
    }
}

impl MessageSender for WebSocketConnection {
    fn send(&self, msg: Frontend2BackendMsg) {
        self.send_msg(msg);
    }
}
