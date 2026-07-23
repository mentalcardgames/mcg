use crate::articles::{fetch_posts, Post};
use crate::store::{ArticlesLoading, ClientState};
use wasm_bindgen_futures::spawn_local;

pub fn fetch_articles_effect(
    state: &mut ClientState,
    on_done: impl FnOnce(Result<Vec<Post>, String>) + 'static,
) {
    state.ui.articles = ArticlesLoading::Loading;

    spawn_local(async move {
        let result = fetch_posts().await;
        on_done(result);
    });
}
