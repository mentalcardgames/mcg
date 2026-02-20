use eframe::Frame;
use egui::{vec2, Color32, RichText, ScrollArea};

use super::{AppInterface, ScreenDef, ScreenMetadata, ScreenWidget};
use crate::articles::Post;
use crate::effects::fetch_articles_effect;
use crate::store::{ArticlesLoading, ClientState};
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Default)]
pub struct ArticlesScreen {
    #[allow(clippy::type_complexity)]
    pending_result: Rc<RefCell<Option<Result<Vec<Post>, String>>>>,
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
        app_state: &mut ClientState,
        ui: &mut egui::Ui,
        posts: &[Post],
        _ctx: &egui::Context,
    ) {
        if ui
            .add_sized(vec2(150.0, 30.0), egui::Button::new("Refresh"))
            .clicked()
        {
            let pending_result = self.pending_result.clone();
            fetch_articles_effect(app_state, move |result| {
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
}

impl ArticlesScreen {
    fn ui(&mut self, app_interface: &mut AppInterface, ui: &mut egui::Ui, _frame: &mut Frame) {
        let ctx = ui.ctx().clone();
        let app_state = &mut app_interface.app_state;

        if let Some(result) = self.pending_result.borrow_mut().take() {
            match result {
                Ok(posts) => app_state.ui.articles = ArticlesLoading::Loaded(posts),
                Err(e) => {
                    app_state.ui.articles = ArticlesLoading::Error(e);
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

            match &app_state.ui.articles {
                ArticlesLoading::NotStarted => {
                    if ui
                        .add_sized(vec2(150.0, 40.0), egui::Button::new("Fetch Posts"))
                        .clicked()
                    {
                        let pending_result = self.pending_result.clone();
                        fetch_articles_effect(app_state, move |result| {
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
                    self.render_posts_list(app_state, ui, &posts, &ctx);
                }
                ArticlesLoading::Error(err) => {
                    let err = err.clone();
                    self.render_error_ui(ui, &err);
                    if ui
                        .add_sized(vec2(150.0, 40.0), egui::Button::new("Retry"))
                        .clicked()
                    {
                        let pending_result = self.pending_result.clone();
                        fetch_articles_effect(app_state, move |result| {
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
