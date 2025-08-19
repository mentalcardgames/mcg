use std::{cell::RefCell, rc::Rc};

use mcg_shared::{ClientMsg, GameStatePublic, ServerMsg};
use crate::articles::Post;

/// Small, UI-friendly application state and a basic store implementation.
///
/// Goals:
/// - Keep reducers pure-ish (no blocking I/O / side-effects here).
/// - Allow UI to get a cheap cloned snapshot each frame.
/// - Allow background effects (connection, persistence) to push messages back via `store.push_server_msg(...)`
/// - Notify subscribers after each dispatch so egui can request a repaint.
///
/// Notes:
/// - This is intentionally simple and targeted at egui (single-threaded). If you need multi-threading,
///   replace Rc<RefCell<...>> with Arc<Mutex<...>> and use thread-safe channels for cross-thread messages.
#[derive(Clone, Debug, Default)]
pub struct Settings {
    pub name: String,
    pub server_address: String,
    pub bots: usize,
    pub bots_auto: bool,
}

#[derive(Clone, Debug)]
pub enum ConnectionStatus {
    Disconnected,
    Connecting,
    Connected,
}
impl Default for ConnectionStatus {
    fn default() -> Self {
        ConnectionStatus::Disconnected
    }
}

#[derive(Clone, Debug)]
pub enum ArticlesLoading {
    NotStarted,
    Loading,
    Loaded(Vec<Post>),
    Error(String),
}
impl Default for ArticlesLoading {
    fn default() -> Self {
        ArticlesLoading::NotStarted
    }
}

#[derive(Clone, Debug, Default)]
pub struct AppState {
    pub game_state: Option<GameStatePublic>,
    pub last_error: Option<String>,
    pub last_info: Option<String>,
    pub connection_status: ConnectionStatus,
    pub settings: Settings,
    // articles-related state
    pub articles: ArticlesLoading,
}

/// Actions that mutate application state. Keep side-effectful semantics out of the reducer:
/// reducers only update the AppState. Effects should run *after* dispatch if needed.
#[derive(Clone, Debug)]
pub enum AppAction {
    Init,
    Connect { addr: String, name: String },
    Disconnect,
    SendClientMsg(ClientMsg),
    ServerMsgReceived(ServerMsg),
    LocalSettingsChanged(Settings),
    ResetGameRequested { bots: usize, bots_auto: bool },
    ShowError(String),
    ClearInfo,
    // Articles actions
    FetchPosts,
    PostsLoaded(Vec<Post>),
    PostsLoadError(String),
}

/// Simple subscriber type that's cloneable and can be called after state changes.
/// We use Rc<dyn Fn()> so subscribers can be cheaply cloned out of the RefCell for notification.
type Subscriber = Rc<dyn Fn()>;

struct Inner {
    state: AppState,
    subscribers: Vec<Subscriber>,
}

/// Store is a shared handle to the state + reducer + subscriber list.
/// UI frames should call `store.get_snapshot()` once per frame and render from that snapshot.
/// UI event handlers should call `store.dispatch(...)` to trigger state changes.
#[derive(Clone)]
pub struct Store {
    inner: Rc<RefCell<Inner>>,
}

impl Store {
    pub fn new() -> Self {
        let default_settings = Settings {
            name: "Player".to_string(),
            // Default server address — set to a sensible local default here.
            server_address: "127.0.0.1:3000".to_string(),
            bots: 1,
            bots_auto: true,
        };
        let inner = Inner {
            state: AppState {
                settings: default_settings,
                ..Default::default()
            },
            subscribers: Vec::new(),
        };
        Self {
            inner: Rc::new(RefCell::new(inner)),
        }
    }

    /// Return a cheap snapshot clone of the current AppState suitable for rendering.
    pub fn get_snapshot(&self) -> AppState {
        self.inner.borrow().state.clone()
    }

    /// Subscribe to state changes. Returns an Rc to the subscriber which the caller can keep alive.
    /// The closure should be lightweight (e.g., call ctx.request_repaint()).
    pub fn subscribe(&self, f: impl Fn() + 'static) -> Subscriber {
        let sub: Subscriber = Rc::new(f);
        self.inner.borrow_mut().subscribers.push(Rc::clone(&sub));
        sub
    }

    /// Dispatch an action: apply reducer, then notify subscribers.
    /// Side-effects should be implemented outside of this reducer (in effects modules).
    pub fn dispatch(&self, action: AppAction) {
        {
            // Apply reducer
            let mut inner = self.inner.borrow_mut();
            apply_action(&mut inner.state, &action);
            // drop inner before calling subscribers to avoid borrow conflicts
        }

        // Clone subscribers to call them without holding the RefCell borrow.
        let subs = {
            let inner = self.inner.borrow();
            inner.subscribers.clone()
        };
        for s in subs {
            (s)();
        }
    }

    /// Helper for effects/background code to push incoming server messages into the store.
    /// This simply dispatches the corresponding action so the reducer handles it.
    pub fn push_server_msg(&self, msg: ServerMsg) {
        self.dispatch(AppAction::ServerMsgReceived(msg));
    }
}

/// Reducer: pure-ish function that mutates AppState given an action.
/// Keep side effects out of this function (no network, no file/JS APIs).
fn apply_action(state: &mut AppState, action: &AppAction) {
    match action {
        AppAction::Init => {
            // Effects (like loading settings) should run separately after Init is dispatched.
        }
        AppAction::Connect { addr, name } => {
            state.connection_status = ConnectionStatus::Connecting;
            state.last_error = None;
            state.last_info = Some(format!("Connecting to {}", addr));
            // store settings locally so UI input and connection intent stay in sync
            state.settings.server_address = addr.clone();
            state.settings.name = name.clone();
        }
        AppAction::Disconnect => {
            state.connection_status = ConnectionStatus::Disconnected;
            state.last_info = Some("Disconnected".into());
        }
        AppAction::SendClientMsg(_msg) => {
            // Sending is a side-effect: the effect responsible for the connection should observe this
            // action and actually send the message. Optionally store info for last_info.
            state.last_info = Some("Outgoing message queued".into());
        }
        AppAction::ServerMsgReceived(msg) => match msg {
            ServerMsg::Welcome { .. } => {
                state.connection_status = ConnectionStatus::Connected;
                state.last_info = Some("Connected".into());
                state.last_error = None;
            }
            ServerMsg::State(gs) => {
                state.game_state = Some(gs.clone());
                state.last_info = None;
            }
            ServerMsg::Error(e) => {
                state.last_error = Some(e.clone());
                // Optionally reflect in connection status:
                // state.connection_status = ConnectionStatus::Disconnected;
            }
        },
        AppAction::LocalSettingsChanged(settings) => {
            state.settings = settings.clone();
            state.last_info = Some("Settings updated".into());
        }
        AppAction::ResetGameRequested { bots, bots_auto } => {
            state.last_info = Some(format!("Reset requested ({} bots)", bots));
            state.settings.bots = *bots;
            state.settings.bots_auto = *bots_auto;
        }
        AppAction::FetchPosts => {
            state.articles = ArticlesLoading::Loading;
            state.last_info = Some("Fetching posts...".into());
        }
        AppAction::PostsLoaded(posts) => {
            state.articles = ArticlesLoading::Loaded(posts.clone());
            state.last_info = Some("Posts loaded".into());
        }
        AppAction::PostsLoadError(e) => {
            state.articles = ArticlesLoading::Error(e.clone());
            state.last_error = Some(e.clone());
        }
        AppAction::ShowError(e) => {
            state.last_error = Some(e.clone());
        }
        AppAction::ClearInfo => {
            state.last_info = None;
        }
    }
}

// Small helper constructor to be used by UI when wiring up the store to the app.
pub fn bootstrap_store() -> Store {
    Store::new()
}
