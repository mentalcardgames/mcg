use crate::app::FrontendInterface;
use crate::widgets::screen::ScreenWidget;
use eframe::Frame;
use egui::{vec2, Color32, RichText, ScrollArea};
use js_sys::futures::spawn_local;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Post {
    #[serde(rename = "userId")]
    pub user_id: u32,
    pub id: u32,
    pub title: String,
    pub body: String,
}

pub async fn fetch_posts() -> Result<Vec<Post>, String> {
    let url = "https://jsonplaceholder.typicode.com/posts/";

    let response = reqwest::get(url)
        .await
        .map_err(|e| format!("Failed to fetch posts: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("HTTP error: {}", response.status()));
    }

    let posts: Vec<Post> = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse JSON: {}", e))?;

    Ok(posts)
}

#[derive(Clone, Debug, Default)]
pub enum ArticlesLoading {
    #[default]
    NotStarted,
    Loading,
    Loaded(Vec<Post>),
    Error(String),
}

#[derive(Default)]
pub struct ArticlesScreen {
    #[allow(clippy::type_complexity)]
    pending_result: Rc<RefCell<Option<Result<Vec<Post>, String>>>>,
    articles: ArticlesLoading,
}

impl ArticlesScreen {
    fn render_loading_ui(&self, ui: &mut egui::Ui) {
        ui.spinner();
        ui.label("Loading posts...");
        ui.add_space(10.0);
        ui.label(RichText::new("Fetching data from remote API...").color(Color32::GRAY));
    }

    fn render_error_ui(&self, ui: &mut egui::Ui, error: &str) {
        ui.label(RichText::new("❌ Error loading posts").color(Color32::RED));
        ui.label(RichText::new(error).color(Color32::GRAY));
        ui.add_space(20.0);
    }

    fn render_posts_list(
        &mut self,
        ui: &mut egui::Ui,
        posts: &[Post],
        _ctx: &egui::Context,
    ) {
        if ui
            .add_sized(vec2(150.0, 30.0), egui::Button::new("Refresh"))
            .clicked()
        {
            let pending_result = self.pending_result.clone();
            self.fetch_articles_effect(move |result| {
                *pending_result.borrow_mut() = Some(result);
            });
        }
        ui.add_space(10.0);
        ui.label(RichText::new(format!("Found {} posts", posts.len())).color(Color32::GREEN));
        ui.add_space(20.0);
        ScrollArea::vertical().max_height(500.0).show(ui, |ui| {
            for post in posts {
                ui.group(|ui| {
                    ui.set_min_width(ui.available_width());
                    ui.heading(&post.title);
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new(format!("Post ID: {}", post.id)).color(Color32::GRAY),
                        );
                        ui.separator();
                        ui.label(
                            RichText::new(format!("User ID: {}", post.user_id))
                                .color(Color32::GRAY),
                        );
                    });
                    ui.add_space(10.0);
                    ui.label(&post.body);
                });
                ui.add_space(15.0);
            }
        });
    }

    pub fn fetch_articles_effect(
        &mut self,
        on_done: impl FnOnce(Result<Vec<Post>, String>) + 'static,
    ) {
        self.articles = ArticlesLoading::Loading;

        spawn_local(async move {
            let result = fetch_posts().await;
            on_done(result);
        });
    }

}

impl ScreenWidget for ArticlesScreen {
    fn ui(&mut self, _app_interface: &mut FrontendInterface, ui: &mut egui::Ui, _frame: &mut Frame) {
        let ctx = ui.ctx().clone();
        if let Some(result) = self.pending_result.borrow_mut().take() {
            match result {
                Ok(posts) => self.articles = ArticlesLoading::Loaded(posts),
                Err(e) => {
                    self.articles = ArticlesLoading::Error(e);
                }
            }
        }

        ui.vertical_centered(|ui| {
            ui.add_space(20.0);
            ui.heading("Posts from JSONPlaceholder");
            ui.add_space(10.0);
            ui.label(
                RichText::new("Real REST API: https://jsonplaceholder.typicode.com/posts/")
                    .color(Color32::GRAY),
            );
            ui.add_space(20.0);

            match &self.articles {
                ArticlesLoading::NotStarted => {
                    if ui
                        .add_sized(vec2(150.0, 40.0), egui::Button::new("Fetch Posts"))
                        .clicked()
                    {
                        let pending_result = self.pending_result.clone();
                        self.fetch_articles_effect(move |result| {
                            *pending_result.borrow_mut() = Some(result);
                        });
                    }
                    ui.add_space(20.0);
                    ui.label("Click the button to fetch posts from JSONPlaceholder API.");
                    ui.add_space(10.0);
                    ui.label(
                        RichText::new("This demonstrates async HTTP requests with egui and WASM.")
                            .color(Color32::GRAY),
                    );
                }
                ArticlesLoading::Loading => {
                    self.render_loading_ui(ui);
                    ctx.request_repaint();
                }
                ArticlesLoading::Loaded(posts) => {
                    let posts = posts.clone();
                    self.render_posts_list(ui, &posts, &ctx);
                }
                ArticlesLoading::Error(err) => {
                    let err = err.clone();
                    self.render_error_ui(ui, &err);
                    if ui
                        .add_sized(vec2(150.0, 40.0), egui::Button::new("Retry"))
                        .clicked()
                    {
                        let pending_result = self.pending_result.clone();
                        self.fetch_articles_effect(move |result| {
                            *pending_result.borrow_mut() = Some(result);
                        });
                    }
                }
            }
            ui.add_space(50.0);
        });
    }
}

crate::impl_screen_def!(
    ArticlesScreen,
    "/articles",
    "Articles",
    "📰",
    "Fetch posts from a demo API",
    true
);
