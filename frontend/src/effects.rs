use crate::articles::{fetch_posts, Post};
use crate::store::{AppState, ArticlesLoading};
use wasm_bindgen_futures::spawn_local;

pub fn fetch_articles_effect(
    state: &mut AppState,
    on_done: impl FnOnce(Result<Vec<Post>, String>) + 'static,
) {
    state.articles = ArticlesLoading::Loading;

    spawn_local(async move {
        let result = fetch_posts().await;
        on_done(result);
    });
}
