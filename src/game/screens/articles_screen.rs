use eframe::Frame;
use egui::{vec2, Color32, Context, RichText, ScrollArea};
use std::sync::mpsc::{self, Sender, Receiver};

use super::{ScreenType, ScreenWidget, AppInterface, back_button};
use crate::articles::{fetch_posts, Post};

#[derive(Debug)]
enum Message {
    PostsLoaded(Vec<Post>),
    PostsLoadError(String),
}

#[derive(Debug, Clone)]
enum LoadingState {
    NotStarted,
    Loading,
    Loaded(Vec<Post>),
    Error(String),
}

/// Holds the application state for the Articles screen
struct ArticlesState {
    loading_state: LoadingState,
}

impl Default for ArticlesState {
    fn default() -> Self {
        Self {
            loading_state: LoadingState::NotStarted,
        }
    }
}

impl ArticlesState {
    fn start_fetch(&mut self) -> bool {
        if matches!(self.loading_state, LoadingState::Loading) {
            return false;
        }
        self.loading_state = LoadingState::Loading;
        true
    }
}

/// Posts screen that fetches and displays posts from JSONPlaceholder API
pub struct ArticlesScreen {
    state: ArticlesState,
    message_sender: Sender<Message>,
    message_receiver: Receiver<Message>,
}

impl ArticlesScreen {
    pub fn new() -> Self {
        let (message_sender, message_receiver) = mpsc::channel();
        Self {
            state: ArticlesState::default(),
            message_sender,
            message_receiver,
        }
    }

    fn render_loading_ui(&self, ui: &mut egui::Ui) {
        ui.spinner();
        ui.label("Loading posts...");
        ui.add_space(10.0);
        ui.label(RichText::new("Fetching data from remote API...").color(Color32::GRAY));
    }

    fn render_error_ui(&mut self, ui: &mut egui::Ui, error: &str) {
        ui.label(RichText::new("❌ Error loading posts").color(Color32::RED));
        ui.label(RichText::new(error).color(Color32::GRAY));
        ui.add_space(20.0);

        if ui.add_sized(vec2(100.0, 30.0), egui::Button::new("Retry")).clicked() {
            self.state.loading_state = LoadingState::NotStarted;
        }

        ui.add_space(10.0);
        ui.label(
            RichText::new("Check your internet connection and try again.")
                .color(Color32::GRAY),
        );
    }

    fn render_posts_list(&mut self, ui: &mut egui::Ui, posts: &[Post]) {
        if ui
            .add_sized(vec2(150.0, 30.0), egui::Button::new("Refresh"))
            .clicked()
        {
            self.state.loading_state = LoadingState::NotStarted;
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

    fn render_initial_ui(&mut self, ui: &mut egui::Ui, ctx: &Context) {
        if ui
            .add_sized(vec2(150.0, 40.0), egui::Button::new("Fetch Posts"))
            .clicked()
        {
            if self.state.start_fetch() {
                let sender = self.message_sender.clone();
                
                // Spawn the async task
                wasm_bindgen_futures::spawn_local(async move {
                    match fetch_posts().await {
                        Ok(posts) => {
                            let _ = sender.send(Message::PostsLoaded(posts));
                        }
                        Err(e) => {
                            let _ = sender.send(Message::PostsLoadError(e));
                        }
                    }
                });
                
                // Request repaint to process messages
                ctx.request_repaint();
            }
        }
        ui.add_space(20.0);
        ui.label("Click the button to fetch posts from JSONPlaceholder API.");
        ui.add_space(10.0);
        ui.label(
            RichText::new("This demonstrates async HTTP requests with egui and WASM.")
                .color(Color32::GRAY),
        );
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
                back_button(ui, app_interface, ScreenType::Main, "Back");

                ui.add_space(20.0);

                // Check for pending messages first
                while let Ok(message) = self.message_receiver.try_recv() {
                    match message {
                        Message::PostsLoaded(posts) => {
                            self.state.loading_state = LoadingState::Loaded(posts);
                        }
                        Message::PostsLoadError(e) => {
                            self.state.loading_state = LoadingState::Error(e);
                        }
                    }
                    // Request another repaint to update the UI
                    ctx.request_repaint();
                }

                // Then render based on current state
                match &self.state.loading_state {
                    LoadingState::NotStarted => {
                        self.render_initial_ui(ui, ctx);
                    }
                    LoadingState::Loading => {
                        self.render_loading_ui(ui);
                        // Request another repaint to check for updates
                        ctx.request_repaint();
                    }
                    LoadingState::Loaded(posts) => {
                        let posts = posts.clone();
                        self.render_posts_list(ui, &posts);
                    }
                    LoadingState::Error(error) => {
                        let error = error.clone();
                        self.render_error_ui(ui, &error);
                    }
                }

                ui.add_space(50.0);
            });
        });
    }
}
