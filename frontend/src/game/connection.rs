use egui::Context;
use mcg_shared::{ClientMsg, ServerMsg};
use std::cell::RefCell;
use std::rc::Rc;

use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use web_sys::{CloseEvent, Event, MessageEvent, WebSocket};

/// Small connection service that encapsulates WebSocket logic for the UI.
/// - Provides connect/send/drain APIs
/// - Keeps inbox/error buffers in Rc<RefCell<..>> so UI can consume them safely
/// - On non-wasm targets this is a stub that records a helpful error
pub struct ConnectionService {
    ws: Option<WebSocket>,

    inbox: Rc<RefCell<Vec<ServerMsg>>>,
    error_inbox: Rc<RefCell<Vec<String>>>,
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
            inbox: Rc::new(RefCell::new(Vec::new())),
            error_inbox: Rc::new(RefCell::new(Vec::new())),
        }
    }

    /// Connect to a server (wasm) or record an error on native builds.
    /// `players` is sent as a NewGame message once the socket is open.
    pub fn connect_with_players(&mut self, _server_address: &str, _players: Vec<mcg_shared::PlayerConfig>, ctx: &Context) {
        {
            let ws_url = format!("ws://{}/ws", _server_address);
            match WebSocket::new(&ws_url) {
                Ok(ws) => {
                    // prepare newgame payload
                    let newgame = serde_json::to_string(&ClientMsg::NewGame {
                        players: _players.clone(),
                    })
                    .unwrap();

                    // onopen -> send newgame
                    let ws_clone = ws.clone();
                    let newgame_clone = newgame.clone();
                    let onopen = Closure::<dyn FnMut()>::new(move || {
                        let _ = ws_clone.send_with_str(&newgame_clone);
                    });
                    ws.set_onopen(Some(onopen.as_ref().unchecked_ref()));
                    onopen.forget();

                    // onmessage -> push into inbox
                    let inbox = Rc::clone(&self.inbox);
                    let ctx_for_msg = ctx.clone();
                    let onmessage =
                        Closure::<dyn FnMut(MessageEvent)>::new(move |e: MessageEvent| {
                            if let Some(txt) = e.data().as_string() {
                                if let Ok(msg) = serde_json::from_str::<ServerMsg>(&txt) {
                                    inbox.borrow_mut().push(msg);
                                    ctx_for_msg.request_repaint();
                                }
                            }
                        });
                    ws.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));
                    onmessage.forget();

                    // onerror -> push into error_inbox
                    let err_inbox = Rc::clone(&self.error_inbox);
                    let ctx_for_err = ctx.clone();
                    let server_address_err = _server_address.to_string();
                    let onerror = Closure::<dyn FnMut(Event)>::new(move |_e: Event| {
                        err_inbox.borrow_mut().push(format!(
                            "Failed to connect to server at {}.",
                            server_address_err
                        ));
                        ctx_for_err.request_repaint();
                    });
                    ws.set_onerror(Some(onerror.as_ref().unchecked_ref()));
                    onerror.forget();

                    // onclose -> push into error_inbox
                    let err_inbox = Rc::clone(&self.error_inbox);
                    let ctx_for_close = ctx.clone();
                    let server_address_close = _server_address.to_string();
                    let onclose = Closure::<dyn FnMut(CloseEvent)>::new(move |e: CloseEvent| {
                        let code = e.code();
                        let reason = e.reason();
                        let msg = if reason.is_empty() {
                            format!(
                                "Connection closed (code {}). Is the server running at {}?",
                                code, server_address_close
                            )
                        } else {
                            format!("Connection closed (code {}): {}", code, reason)
                        };
                        err_inbox.borrow_mut().push(msg);
                        ctx_for_close.request_repaint();
                    });
                    ws.set_onclose(Some(onclose.as_ref().unchecked_ref()));
                    onclose.forget();

                    self.ws = Some(ws);
                }
                Err(err) => {
                    self.error_inbox
                        .borrow_mut()
                        .push(format!("WS connect error: {err:?}"));
                    ctx.request_repaint();
                }
            }
        }
    }

    /// Send a ClientMsg if connected (no-op otherwise)
    pub fn send(&self, msg: &ClientMsg) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            // no-op on native
            let _ = msg;
        }
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(ws) = &self.ws {
                if let Ok(txt) = serde_json::to_string(msg) {
                    let _ = ws.send_with_str(&txt);
                }
            }
        }
    }

    /// Drain and return all queued ServerMsg messages
    pub fn drain_messages(&self) -> Vec<ServerMsg> {
        let mut v = self.inbox.borrow_mut();
        v.drain(..).collect()
    }

    /// Drain and return all queued error strings
    pub fn drain_errors(&self) -> Vec<String> {
        let mut v = self.error_inbox.borrow_mut();
        v.drain(..).collect()
    }

    /// Close the connection if any (best-effort; wasm only)
    pub fn close(&mut self) {
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(ws) = self.ws.take() {
                let _ = ws.close();
            }
        }
    }
}
