use eframe::Frame;
use egui::{vec2, Align, Button, Color32, Grid, Layout, RichText, ScrollArea};

use super::{AppInterface, ScreenDef, ScreenMetadata, ScreenWidget};
use crate::sprintln;
use crate::store::{bootstrap_state, SharedState};
use crate::utils::emoji_hash;

/// Pairing screen refactored to use the central Store so UI remains thin.
/// State is stored in `store` (pairing_players + confirm fields). UI only renders snapshots
/// and issues intent via `store.update(...)`.
pub struct PairingScreen {
    state: SharedState,
}

impl PairingScreen {
    pub fn new() -> Self {
        let state = bootstrap_state();
        Self { state }
    }
}

impl Default for PairingScreen {
    fn default() -> Self {
        Self::new()
    }
}

impl ScreenWidget for PairingScreen {
    fn ui(&mut self, _app_interface: &mut AppInterface, ui: &mut egui::Ui, _frame: &mut Frame) {
        ui.heading("Player Pairing");
        ui.with_layout(Layout::right_to_left(Align::TOP), |ui| {
            // Global Back button is provided by the layout
            ui.add_space(0.0);
        });

        // Get a snapshot for rendering
        let snapshot = self.state.borrow().clone();
        let players = snapshot.pairing_players.clone();

        ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                Grid::new("player_grid")
                    .num_columns(3)
                    .spacing([20.0, 10.0])
                    .striped(true)
                    .show(ui, |ui| {
                        ui.strong("Player");
                        ui.strong("Status");
                        ui.strong("Action");
                        ui.end_row();
                        for player in players.iter() {
                            ui.horizontal(|ui| {
                                let icon = if player.paired { "👥" } else { "👤" };
                                ui.label(RichText::new(icon).size(24.0));
                                ui.label(&player.name);
                            });
                            let status_text = if player.paired {
                                RichText::new("Paired").color(Color32::GREEN)
                            } else {
                                RichText::new("Not Paired").color(Color32::RED)
                            };
                            ui.label(status_text);
                            let button_text = if player.paired { "Unpair" } else { "Pair" };
                            let button_color = if player.paired {
                                Color32::from_rgb(255, 180, 180)
                            } else {
                                Color32::from_rgb(180, 255, 180)
                            };
                            if ui
                                .add(
                                    Button::new(RichText::new(button_text).color(Color32::BLACK))
                                        .fill(button_color)
                                        .min_size(vec2(80.0, 24.0)),
                                )
                                .clicked()
                            {
                                let pname = player.name.clone();
                                let action = !player.paired;
                                // set confirm fields in the shared state
                                {
                                    let mut s = self.state.borrow_mut();
                                    s.pairing_confirm_player = Some(pname.clone());
                                    s.pairing_confirm_action = Some(action);
                                }
                            }
                            ui.end_row();
                        }
                    });
            });

        // Render confirmation window if requested in shared state
        let snapshot = self.state.borrow().clone();
        if let (Some(player_name), Some(pair_action)) = (
            snapshot.pairing_confirm_player.clone(),
            snapshot.pairing_confirm_action,
        ) {
            let action_text = if pair_action { "pair" } else { "unpair" };
            let player_name_clone = player_name.clone();
            let player_hash = emoji_hash(player_name_clone.as_bytes(), ui.ctx());
            egui::Window::new(format!("Confirm {}", action_text))
                .collapsible(false)
                .resizable(false)
                .min_width(300.0)
                .show(ui.ctx(), |ui| {
                    ui.vertical_centered(|ui| {
                        ui.add_space(8.0);
                        ui.label(RichText::new(&player_hash).size(48.0).strong());
                        ui.add_space(8.0);
                    });
                    ui.separator();
                    ui.label(format!(
                        "Are you sure you want to {} {}?",
                        action_text, player_name_clone
                    ));
                    ui.horizontal(|ui| {
                        if ui.button("Cancel").clicked() {
                            let mut s = self.state.borrow_mut();
                            s.pairing_confirm_player = None;
                            s.pairing_confirm_action = None;
                        }
                        let mut perform_action = false;
                        if ui.button("Confirm").clicked() {
                            perform_action = true;
                        }
                        if perform_action {
                            let target = player_name_clone.clone();
                            let pair_action = pair_action;
                            {
                                let mut s = self.state.borrow_mut();
                                for p in s.pairing_players.iter_mut() {
                                    if p.name == target {
                                        p.paired = pair_action;
                                        sprintln!(
                                            "Player {} ({}) is now {}",
                                            p.name,
                                            emoji_hash(p.name.as_bytes(), ui.ctx()),
                                            if p.paired { "paired" } else { "unpaired" }
                                        );
                                        break;
                                    }
                                }
                                s.pairing_confirm_player = None;
                                s.pairing_confirm_action = None;
                            }
                        }
                    });
                });
        }
    }
}

impl ScreenDef for PairingScreen {
    fn metadata() -> ScreenMetadata
    where
        Self: Sized,
    {
        ScreenMetadata {
            path: "/pairing",
            display_name: "Pairing",
            icon: "🔗",
            description: "Player pairing demo",
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
