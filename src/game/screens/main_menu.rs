use std::cell::RefCell;
use std::rc::Rc;

use eframe::Frame;
use egui::{Context, vec2};

use crate::sprintln;
use super::{ScreenWidget, ScreenType};

/// Main menu screen
pub struct MainMenu {}

impl MainMenu {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for MainMenu {
    fn default() -> Self {
        Self::new()
    }
}

impl ScreenWidget for MainMenu {
    fn update(&mut self, next_screen: Rc<RefCell<String>>, ctx: &Context, _frame: &mut Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(50.0); // Add top spacing

                let button_size = vec2(200.0, 40.0); // Consistent button size

                if ui
                    .add_sized(button_size, egui::Button::new("Start"))
                    .clicked()
                {
                    sprintln!("setup started");
                    *next_screen.borrow_mut() = ScreenType::GameSetup.to_string();
                };

                ui.add_space(20.0); // Add spacing between buttons

                if ui
                    .add_sized(button_size, egui::Button::new("Drag & Drop Game"))
                    .clicked()
                {
                    sprintln!("game_dnd opened");
                    *next_screen.borrow_mut() = ScreenType::GameDndSetup.to_string();
                };

                ui.add_space(20.0); // Add spacing between buttons

                if ui
                    .add_sized(button_size, egui::Button::new("Pairing"))
                    .clicked()
                {
                    sprintln!("pairing opened");
                    *next_screen.borrow_mut() = ScreenType::Pairing.to_string();
                };

                ui.add_space(20.0); // Add spacing between buttons

                if ui
                    .add_sized(button_size, egui::Button::new("Settings"))
                    .clicked()
                {
                    sprintln!("settings opened");
                    *next_screen.borrow_mut() = ScreenType::Settings.to_string();
                };

                ui.add_space(20.0); // Add spacing between buttons

                if ui
                    .add_sized(button_size, egui::Button::new("Drag & Drop Test"))
                    .clicked()
                {
                    sprintln!("dnd_test opened");
                    *next_screen.borrow_mut() = ScreenType::DndTest.to_string();
                };

                ui.add_space(20.0); // Add spacing between buttons

                if ui
                    .add_sized(button_size, egui::Button::new("Print Screen"))
                    .clicked()
                {
                    sprintln!("{}", next_screen.borrow());
                };

                ui.add_space(50.0); // Add bottom spacing
            });
        });
    }
}