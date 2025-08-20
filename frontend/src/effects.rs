use eframe::egui;
use mcg_shared::ClientMsg;

use crate::game::connection::ConnectionService;
use crate::store::{apply_server_msg, SharedState};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::spawn_local;

/// ConnectionEffect owns the ConnectionService and mediates side-effects between the network
/// and the Store. It intentionally does not spawn threads — instead a short `poll()` method
/// should be called from the UI main loop (egui frame) to drain incoming messages and errors.
///
/// This design works well with egui's immediate-mode model and avoids cross-thread RefCell/borrow issues.
/// If you later prefer background threads, replace the polling with a cross-thread channel + thread.
pub struct ConnectionEffect {
    conn: Option<ConnectionService>,
    state: SharedState,
    ctx: Option<egui::Context>,
}

impl ConnectionEffect {
    /// Create a new ConnectionEffect bound to `state`.
    pub fn new(state: SharedState) -> Self {
        Self {
            conn: None,
            state,
            ctx: None,
        }
    }

    /// Start/establish a connection using ConnectionService.
    /// `ctx` is required by ConnectionService.connect for wasm/native UI integration.
    /// This will create a new ConnectionService, call connect, and mutate the shared state.
    pub fn start_connect(&mut self, ctx: &egui::Context, addr: &str, name: &str) {
        let mut conn = ConnectionService::new();
        conn.connect(addr, name, ctx);
        self.conn = Some(conn);
        self.ctx = Some(ctx.clone());
        // update shared state directly
        {
            let mut s = self.state.borrow_mut();
            s.connection_status = crate::store::ConnectionStatus::Connecting;
            s.last_error = None;
            s.last_info = Some(format!("Connecting to {}", addr));
            s.settings.server_address = addr.to_string();
            s.settings.name = name.to_string();
        }
        if let Some(ctx) = &self.ctx {
            ctx.request_repaint();
        }
    }

    /// Close and drop the active connection, if any.
    pub fn close(&mut self) {
        if let Some(mut c) = self.conn.take() {
            c.close();
        }
        {
            let mut s = self.state.borrow_mut();
            s.connection_status = crate::store::ConnectionStatus::Disconnected;
            s.last_info = Some("Disconnected".into());
        }
        if let Some(ctx) = &self.ctx {
            ctx.request_repaint();
        }
    }

    /// Send a client message through the active ConnectionService.
    pub fn send(&self, msg: &ClientMsg) {
        if let Some(c) = &self.conn {
            c.send(msg);
            // update state with outgoing info
            let mut s = self.state.borrow_mut();
            s.last_info = Some("Outgoing message queued".into());
        } else {
            let mut s = self.state.borrow_mut();
            s.last_error = Some("Not connected".into());
        }
        if let Some(ctx) = &self.ctx {
            ctx.request_repaint();
        }
    }

    /// Poll the ConnectionService once: drain messages and errors, forward them into the shared state.
    /// This should be called each frame (or on a timer) from the UI thread so the UI observes mutated state.
    pub fn poll(&mut self) {
        if let Some(c) = &mut self.conn {
            for msg in c.drain_messages() {
                // apply incoming server messages into the shared AppState
                apply_server_msg(&self.state, msg);
            }

            for err in c.drain_errors() {
                let mut s = self.state.borrow_mut();
                s.last_error = Some(err);
            }

            // request repaint so the UI reflects new state quickly
            if let Some(ctx) = &self.ctx {
                ctx.request_repaint();
            }
        }
    }
}

/* Small example of how a UI screen can use the effect:

   // setup (once)
   let store = crate::store::bootstrap_store();
   let mut conn_eff = crate::effects::ConnectionEffect::new(store.clone());

   // in ui() per-frame:
   conn_eff.poll(); // drain incoming messages into store
   let state = store.get_snapshot();
   // render based on state
   // on connect button:
   conn_eff.start_connect(&ui.ctx(), &state.settings.server_address, &state.settings.name);
   // on send:
   conn_eff.send(&ClientMsg::Action { ... });
*/

/* ArticlesEffect: fetch posts and dispatch results into the store.
   - On WASM uses spawn_local + async fetch_posts()
   - On native spawns a blocking thread using reqwest::blocking
*/
pub struct ArticlesEffect {
    state: SharedState,
}

impl ArticlesEffect {
    pub fn new(state: SharedState) -> Self {
        Self { state }
    }

    /// Trigger fetching posts. Mutates shared state: Loading -> Loaded/Error.
    pub fn fetch_posts(&self, ctx: Option<&egui::Context>) {
        // mark loading in the shared state
        {
            let mut s = self.state.borrow_mut();
            s.articles = crate::store::ArticlesLoading::Loading;
            s.last_info = Some("Fetching posts...".into());
        }
        if let Some(ctx) = ctx {
            ctx.request_repaint();
        }

        #[cfg(target_arch = "wasm32")]
        {
            let state = self.state.clone();
            let ctx = ctx.cloned();
            spawn_local(async move {
                match crate::articles::fetch_posts().await {
                    Ok(posts) => {
                        {
                            let mut s = state.borrow_mut();
                            s.articles = crate::store::ArticlesLoading::Loaded(posts.clone());
                            s.last_info = Some("Posts loaded".into());
                        }
                        if let Some(c) = &ctx {
                            c.request_repaint();
                        }
                    }
                    Err(e) => {
                        {
                            let mut s = state.borrow_mut();
                            s.articles = crate::store::ArticlesLoading::Error(e.clone());
                            s.last_error = Some(e.clone());
                        }
                        if let Some(c) = &ctx {
                            c.request_repaint();
                        }
                    }
                }
            });
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            // Perform blocking fetch on the current thread. This avoids moving the
            // non-Send `Rc<RefCell<_>>` into a spawned thread. It will block the UI
            // briefly while fetching; consider refactoring SharedState to a
            // thread-safe wrapper (Arc<Mutex<...>>) if background threads are needed.
            match crate::articles::fetch_posts_blocking() {
                Ok(posts) => {
                    let mut s = self.state.borrow_mut();
                    s.articles = crate::store::ArticlesLoading::Loaded(posts.clone());
                    s.last_info = Some("Posts loaded".into());
                }
                Err(e) => {
                    let mut s = self.state.borrow_mut();
                    s.articles = crate::store::ArticlesLoading::Error(e.clone());
                    s.last_error = Some(e.clone());
                }
            }
        }
    }
}
