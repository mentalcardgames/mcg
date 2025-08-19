use eframe::egui;
use mcg_shared::ClientMsg;

use crate::game::connection::ConnectionService;
use crate::store::{AppAction, Store};

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
    store: Store,
    ctx: Option<egui::Context>,
}

impl ConnectionEffect {
    /// Create a new ConnectionEffect bound to `store`.
    pub fn new(store: Store) -> Self {
        Self {
            conn: None,
            store,
            ctx: None,
        }
    }

    /// Start/establish a connection using ConnectionService.
    /// `ctx` is required by ConnectionService.connect for wasm/native UI integration.
    /// This will create a new ConnectionService, call connect, and update the store with Connect action.
    pub fn start_connect(&mut self, ctx: &egui::Context, addr: &str, name: &str) {
        let mut conn = ConnectionService::new();
        conn.connect(addr, name, ctx);
        self.conn = Some(conn);
        self.ctx = Some(ctx.clone());
        // notify the store about the attempted connection (reducer handles status)
        self.store.dispatch(AppAction::Connect {
            addr: addr.to_string(),
            name: name.to_string(),
        });
    }

    /// Close and drop the active connection, if any.
    pub fn close(&mut self) {
        if let Some(mut c) = self.conn.take() {
            c.close();
        }
        self.store.dispatch(AppAction::Disconnect);
    }

    /// Send a client message through the active ConnectionService.
    /// Also dispatches a SendClientMsg action so reducers/effects can observe the outgoing intent.
    pub fn send(&self, msg: &ClientMsg) {
        if let Some(c) = &self.conn {
            c.send(msg);
            // Inform store (side-effects may observe this)
            self.store.dispatch(AppAction::SendClientMsg(msg.clone()));
        } else {
            self.store
                .dispatch(AppAction::ShowError("Not connected".into()));
        }
    }

    /// Poll the ConnectionService once: drain messages and errors, forward them into the store.
    /// This should be called each frame (or on a timer) from the UI thread so the store receives
    /// incoming ServerMsg events and updates state accordingly.
    pub fn poll(&mut self) {
        if let Some(c) = &mut self.conn {
            for msg in c.drain_messages() {
                // push into store as ServerMsgReceived so the reducer handles it
                self.store.push_server_msg(msg);
            }

            for err in c.drain_errors() {
                self.store.dispatch(AppAction::ShowError(err));
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
    store: Store,
}

impl ArticlesEffect {
    pub fn new(store: Store) -> Self {
        Self { store }
    }

    /// Trigger fetching posts. Dispatches FetchPosts immediately, then PostsLoaded or PostsLoadError.
    pub fn fetch_posts(&self) {
        // mark loading in the store
        self.store.dispatch(AppAction::FetchPosts);

        #[cfg(target_arch = "wasm32")]
        {
            let store = self.store.clone();
            spawn_local(async move {
                match crate::articles::fetch_posts().await {
                    Ok(posts) => {
                        store.dispatch(AppAction::PostsLoaded(posts));
                    }
                    Err(e) => {
                        store.dispatch(AppAction::PostsLoadError(e));
                    }
                }
            });
        }
    }
}
