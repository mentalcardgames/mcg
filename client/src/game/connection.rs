use egui::Context;
use mcg_shared::{ClientMsg, ServerMsg};
use std::cell::RefCell;
use std::rc::Rc;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::closure::Closure;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;
#[cfg(target_arch = "wasm32")]
use web_sys::{CloseEvent, Event, MessageEvent, WebSocket};

/// Small connection service that encapsulates WebSocket logic for the UI.
/// - Provides connect/send/drain APIs
/// - Keeps inbox/error buffers in Rc<RefCell<..>> so UI can consume them safely
/// - On non-wasm targets this is a stub that records a helpful error
pub struct ConnectionService {
    #[cfg(target_arch = "wasm32")]
    ws: Option<WebSocket>,

    inbox: Rc<RefCell<Vec<ServerMsg>>>,
    error_inbox: Rc<RefCell<Vec<String>>>,
}

impl ConnectionService {
    pub fn new() -> Self {
        Self {
            #[cfg(target_arch = "wasm32")]
            ws: None,
            inbox: Rc::new(RefCell::new(Vec::new())),
            error_inbox: Rc::new(RefCell::new(Vec::new())),
        }
    }

    /// Connect to a server (wasm) or record an error on native builds.
    /// `name` is sent as a Join message once the socket is open.
    pub fn connect(&mut self, server_address: &str, name: &str, ctx: &Context) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.error_inbox
                .borrow_mut()
                .push("Online mode is unavailable on native builds".into());
            ctx.request_repaint();
        }

        #[cfg(target_arch = "wasm32")]
        {
            let ws_url = format!("ws://{}/ws", server_address);
            match WebSocket::new(&ws_url) {
                Ok(ws) => {
                    // prepare join payload
                    let join = serde_json::to_string(&ClientMsg::Join {
                        name: name.to_string(),
                    })
                    .unwrap();

                    // onopen -> send join
                    let ws_clone = ws.clone();
                    let join_clone = join.clone();
                    let onopen = Closure::<dyn FnMut()>::new(move || {
                        let _ = ws_clone.send_with_str(&join_clone);
                    });
                    ws.set_onopen(Some(onopen.as_ref().unchecked_ref()));
                    onopen.forget();

                    // onmessage -> push into inbox
                    let inbox = Rc::clone(&self.inbox);
                    let ctx_for_msg = ctx.clone();
                    let onmessage = Closure::<dyn FnMut(MessageEvent)>::new(move |e: MessageEvent| {
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
                    let server_address_err = server_address.to_string();
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
                    let server_address_close = server_address.to_string();
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
