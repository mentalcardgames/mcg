use eframe::egui;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::spawn_local;

use crate::store::SharedState;

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
