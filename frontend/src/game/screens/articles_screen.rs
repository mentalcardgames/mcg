use eframe::Frame;
use egui::{vec2, Color32, RichText, ScrollArea};
use std::rc::Rc;

use super::{AppInterface, ScreenDef, ScreenMetadata, ScreenWidget};
use crate::articles::Post;
use crate::effects::ArticlesEffect;
use crate::store::bootstrap_store;
use crate::store::AppState;

/// Thin UI for articles; application state and fetching is handled by the shared Store + ArticlesEffect.
pub struct ArticlesScreen {
    store: crate::store::Store,
    articles_eff: crate::effects::ArticlesEffect,
    // subscriber kept alive so we can request_repaint on updates
    subscriber: Option<Rc<dyn Fn()>>,
}

impl ArticlesScreen {
    pub fn new() -> Self {
        let store = bootstrap_store();
        let articles_eff = ArticlesEffect::new(store.clone());
        Self {
            store,
            articles_eff,
            subscriber: None,
        }
    }

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

    fn render_posts_list(&self, ui: &mut egui::Ui, posts: &[Post]) {
        if ui
            .add_sized(vec2(150.0, 30.0), egui::Button::new("Refresh"))
            .clicked()
        {
            self.articles_eff.fetch_posts();
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

impl Default for ArticlesScreen {
    fn default() -> Self {
        Self::new()
    }
}

impl ScreenWidget for ArticlesScreen {
    fn ui(&mut self, _app_interface: &mut AppInterface, ui: &mut egui::Ui, _frame: &mut Frame) {
        let ctx = ui.ctx().clone();

        // Ensure we have a subscriber that requests repaint on store updates.
        if self.subscriber.is_none() {
            let ctx_clone = ctx.clone();
            let sub = self.store.subscribe(move || {
                ctx_clone.request_repaint();
            });
            self.subscriber = Some(sub);
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

            // Render based on store snapshot
            let snapshot: AppState = self.store.get_snapshot();
            match snapshot.articles {
                crate::store::ArticlesLoading::NotStarted => {
                    if ui
                        .add_sized(vec2(150.0, 40.0), egui::Button::new("Fetch Posts"))
                        .clicked()
                    {
                        self.articles_eff.fetch_posts();
                    }
                    ui.add_space(20.0);
                    ui.label("Click the button to fetch posts from JSONPlaceholder API.");
                    ui.add_space(10.0);
                    ui.label(
                        RichText::new("This demonstrates async HTTP requests with egui and WASM.")
                            .color(Color32::GRAY),
                    );
                }
                crate::store::ArticlesLoading::Loading => {
                    self.render_loading_ui(ui);
                    ctx.request_repaint();
                }
                crate::store::ArticlesLoading::Loaded(posts) => {
                    let posts = posts.clone();
                    self.render_posts_list(ui, &posts);
                }
                crate::store::ArticlesLoading::Error(err) => {
                    let err = err.clone();
                    self.render_error_ui(ui, &err);
                    if ui
                        .add_sized(vec2(150.0, 40.0), egui::Button::new("Retry"))
                        .clicked()
                    {
                        self.articles_eff.fetch_posts();
                    }
                }
            }

            ui.add_space(50.0);
        });
    }
}

impl ScreenDef for ArticlesScreen {
    fn metadata() -> ScreenMetadata
    where
        Self: Sized,
    {
        ScreenMetadata {
            path: "/articles",
            display_name: "Articles",
            icon: "📰",
            description: "Fetch posts from a demo API",
            show_in_menu: true,
        }
    }

    fn create() -> Box<dyn ScreenWidget>
    where
        Self: Sized,
    {
        Box::new(Self::new())
    }
}
