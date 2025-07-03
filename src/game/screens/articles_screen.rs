use std::cell::RefCell;
use std::rc::Rc;

use eframe::Frame;
use egui::{vec2, Color32, Context, RichText, ScrollArea};

use super::{ScreenType, ScreenWidget, AppInterface};
use crate::articles::{fetch_posts, Post};
use crate::sprintln;

#[derive(Debug, Clone)]
enum LoadingState {
    NotStarted,
    Loading,
    Loaded(Vec<Post>),
    Error(String),
}

/// Posts screen that fetches and displays posts from JSONPlaceholder API
pub struct ArticlesScreen {
    loading_state: Rc<RefCell<LoadingState>>,
}

impl ArticlesScreen {
    pub fn new() -> Self {
        Self {
            loading_state: Rc::new(RefCell::new(LoadingState::NotStarted)),
        }
    }

    fn start_fetch(&mut self, ctx: &Context) {
        if matches!(*self.loading_state.borrow(), LoadingState::Loading) {
            return;
        }

        *self.loading_state.borrow_mut() = LoadingState::Loading;

        let ctx_clone = ctx.clone();
        let loading_state = Rc::clone(&self.loading_state);

        wasm_bindgen_futures::spawn_local(async move {
            match fetch_posts().await {
                Ok(posts) => {
                    sprintln!("Successfully fetched {} posts", posts.len());
                    *loading_state.borrow_mut() = LoadingState::Loaded(posts);
                    ctx_clone.request_repaint();
                }
                Err(e) => {
                    sprintln!("Failed to fetch posts: {}", e);
                    *loading_state.borrow_mut() = LoadingState::Error(e);
                    ctx_clone.request_repaint();
                }
            }
        });
    }

    fn render_post(&self, ui: &mut egui::Ui, post: &Post) {
        ui.group(|ui| {
            ui.set_min_width(ui.available_width());

            // Post title
            ui.heading(&post.title);

            // Post metadata
            ui.horizontal(|ui| {
                ui.label(RichText::new(format!("Post ID: {}", post.id)).color(Color32::GRAY));
                ui.separator();
                ui.label(RichText::new(format!("User ID: {}", post.user_id)).color(Color32::GRAY));
            });

            ui.add_space(10.0);

            // Post body
            ui.label(&post.body);
        });
    }
}

impl Default for ArticlesScreen {
    fn default() -> Self {
        Self::new()
    }
}

impl ScreenWidget for ArticlesScreen {
    fn update(&mut self, app_interface: &mut AppInterface, ctx: &Context, _frame: &mut Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(20.0);

                ui.heading("Posts from JSONPlaceholder");
                ui.add_space(10.0);
                ui.label(
                    RichText::new("Real REST API: https://jsonplaceholder.typicode.com/posts/")
                        .color(Color32::GRAY),
                );
                ui.add_space(20.0);

                // Back button
                if ui
                    .add_sized(vec2(100.0, 30.0), egui::Button::new("Back"))
                    .clicked()
                {
                    app_interface.queue_event(crate::game::AppEvent::ChangeScreen(ScreenType::Main));
                }

                ui.add_space(20.0);

                let current_state = self.loading_state.borrow().clone();
                match &current_state {
                    LoadingState::NotStarted => {
                        if ui
                            .add_sized(vec2(150.0, 40.0), egui::Button::new("Fetch Posts"))
                            .clicked()
                        {
                            self.start_fetch(ctx);
                        }
                        ui.add_space(20.0);
                        ui.label("Click the button to fetch posts from JSONPlaceholder API.");
                        ui.add_space(10.0);
                        ui.label(
                            RichText::new(
                                "This demonstrates async HTTP requests with egui and WASM.",
                            )
                            .color(Color32::GRAY),
                        );
                    }
                    LoadingState::Loading => {
                        ui.spinner();
                        ui.label("Loading posts...");
                        ui.add_space(10.0);
                        ui.label(
                            RichText::new("Fetching data from remote API...").color(Color32::GRAY),
                        );
                    }
                    LoadingState::Loaded(posts) => {
                        if ui
                            .add_sized(vec2(150.0, 30.0), egui::Button::new("Refresh"))
                            .clicked()
                        {
                            *self.loading_state.borrow_mut() = LoadingState::NotStarted;
                        }

                        ui.add_space(10.0);

                        ui.label(
                            RichText::new(format!("Found {} posts", posts.len()))
                                .color(Color32::GREEN),
                        );

                        ui.add_space(20.0);

                        // Scrollable area for posts
                        ScrollArea::vertical().max_height(500.0).show(ui, |ui| {
                            for post in posts {
                                self.render_post(ui, post);
                                ui.add_space(15.0);
                            }
                        });
                    }
                    LoadingState::Error(error) => {
                        ui.label(RichText::new("❌ Error loading posts").color(Color32::RED));
                        ui.label(RichText::new(error).color(Color32::GRAY));

                        ui.add_space(20.0);

                        if ui
                            .add_sized(vec2(100.0, 30.0), egui::Button::new("Retry"))
                            .clicked()
                        {
                            *self.loading_state.borrow_mut() = LoadingState::NotStarted;
                        }

                        ui.add_space(10.0);
                        ui.label(
                            RichText::new("Check your internet connection and try again.")
                                .color(Color32::GRAY),
                        );
                    }
                }

                ui.add_space(50.0);
            });
        });
    }
}
