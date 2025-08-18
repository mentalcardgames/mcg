use eframe::Frame;
use egui::{vec2, Color32, RichText, ScrollArea};
use std::sync::mpsc::{self, Receiver, Sender};

use super::{AppInterface, ScreenDef, ScreenMetadata, ScreenWidget};
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
        if ui
            .add_sized(vec2(100.0, 30.0), egui::Button::new("Retry"))
            .clicked()
        {
            self.state.loading_state = LoadingState::NotStarted;
        }
        ui.add_space(10.0);
        ui.label(
            RichText::new("Check your internet connection and try again.").color(Color32::GRAY),
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
        ui.label(RichText::new(format!("Found {} posts", posts.len())).color(Color32::GREEN));
        ui.add_space(20.0);
        ScrollArea::vertical().max_height(500.0).show(ui, |ui| {
            for post in posts {
                self.render_post(ui, post);
                ui.add_space(15.0);
            }
        });
    }
    fn render_initial_ui(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        if ui
            .add_sized(vec2(150.0, 40.0), egui::Button::new("Fetch Posts"))
            .clicked()
            && self.state.start_fetch()
        {
            let sender = self.message_sender.clone();
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
            ctx.request_repaint();
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
            ui.heading(&post.title);
            ui.horizontal(|ui| {
                ui.label(RichText::new(format!("Post ID: {}", post.id)).color(Color32::GRAY));
                ui.separator();
                ui.label(RichText::new(format!("User ID: {}", post.user_id)).color(Color32::GRAY));
            });
            ui.add_space(10.0);
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
    fn ui(&mut self, _app_interface: &mut AppInterface, ui: &mut egui::Ui, _frame: &mut Frame) {
        let ctx = ui.ctx().clone();
        ui.vertical_centered(|ui| {
            ui.add_space(20.0);
            ui.heading("Posts from JSONPlaceholder");
            ui.add_space(10.0);
            ui.label(
                RichText::new("Real REST API: https://jsonplaceholder.typicode.com/posts/")
                    .color(Color32::GRAY),
            );
            ui.add_space(20.0);
            // Global Back button is provided by the layout
            ui.add_space(20.0);
            while let Ok(message) = self.message_receiver.try_recv() {
                match message {
                    Message::PostsLoaded(posts) => {
                        self.state.loading_state = LoadingState::Loaded(posts);
                    }
                    Message::PostsLoadError(e) => {
                        self.state.loading_state = LoadingState::Error(e);
                    }
                }
                ctx.request_repaint();
            }
            match &self.state.loading_state {
                LoadingState::NotStarted => {
                    self.render_initial_ui(ui, &ctx);
                }
                LoadingState::Loading => {
                    self.render_loading_ui(ui);
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
