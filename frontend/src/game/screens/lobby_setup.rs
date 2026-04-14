use eframe::Frame;
use egui::{vec2, Align, UiBuilder, Layout, ComboBox};
use std::rc::Rc;

use super::{AppInterface, ScreenDef, ScreenMetadata, ScreenWidget, ScreenRegistry};
use crate::game::GameType;

pub struct LobbySelectionScreen {
    pub players: usize,
    pub game_type: GameType,
    screen_registry: ScreenRegistry,
}

impl LobbySelectionScreen {
    pub fn new() -> Self {
        Self {
            players: 2,
            game_type: GameType::Poker,
            screen_registry: ScreenRegistry::new(),
        }
    }
}

impl Default for LobbySelectionScreen {
    fn default() -> Self {
        Self::new()
    }
}

impl ScreenWidget for LobbySelectionScreen {
    fn ui(
        &mut self,
        _app_interface: &mut AppInterface,
        ui: &mut egui::Ui,
        _frame: &mut eframe::Frame,
    ) {
        // --- First dropdown: Game ---
        let before = self.game_type;

        ComboBox::from_label("Select Game")
            .selected_text(format!("{:?}", self.game_type))
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut self.game_type, GameType::Poker, "Poker");
                ui.selectable_value(&mut self.game_type, GameType::Blackjack, "Blackjack");
            });

        // Reset player count if game changes
        if self.game_type != before {
            self.players = match self.game_type {
                GameType::Poker => 2,
                GameType::Blackjack => 1,
            };
        }
        // Define valid player counts based on selected game
        let valid_counts: &[usize] = match self.game_type {
            GameType::Poker => &[2, 4, 8],
            GameType::Blackjack => &[1, 2, 3, 4],
        };
        // --- Second dropdown: Players ---
        ComboBox::from_label("Select Player Count")
            .selected_text(self.players.to_string())
            .show_ui(ui, |ui| {
                for &count in valid_counts {
                    ui.selectable_value(&mut self.players, count, count.to_string());
                }
            });
        // Open lobby button
        ui.add_space(5.0);
        if ui.button("Host Game").clicked() {
            match self.game_type {
                GameType::Poker => {
                    // Transition to poker lobby setup
                    eprintln!("Hosting Poker game with max {} players", self.players);
                    //TODO: Make a new poker screen that takes players as a param and transitions to it here
                    //based on the PokerOnlineScreen with the QR code generation thing? To use later when I
                    //add the actual online multiplayer functionality
                }
                GameType::Blackjack => {
                    // Transition to blackjack lobby setup
                    eprintln!("Hosting Blackjack game with max {} players", self.players);
                    // We dont have blackjack implemented, so this is just a dummy for testing's sake.
                    // In the future, this would change to a screen like the Poker one.
                }
            }
        }
    }
}

impl ScreenDef for LobbySelectionScreen {
    fn metadata() -> ScreenMetadata
    where
        Self: Sized,
    {
        ScreenMetadata {
            path: "/lobbyselect",
            display_name: "Game Lobby Selection",
            icon: "⚙",
            description: "Choose which game to play.",
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