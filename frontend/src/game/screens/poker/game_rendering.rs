use egui::{Color32, Ui};
use mcg_shared::{GameStatePublic, PlayerId, PlayerPublic};

pub fn render_showdown_banner(ui: &mut Ui, state: &GameStatePublic, preferred_player: PlayerId) {
    if state.stage == mcg_shared::Stage::Showdown {
        let you_won = state.winner_ids.contains(&preferred_player);
        if you_won {
            ui.colored_label(Color32::LIGHT_GREEN, "You won!");
        } else {
            ui.colored_label(Color32::LIGHT_RED, "You lost.");
        }
        let winners: Vec<String> = state
            .players
            .iter()
            .filter(|p| state.winner_ids.contains(&p.id))
            .map(|p| p.name.clone())
            .collect();
        if !winners.is_empty() {
            ui.label(format!("Winners: {}", winners.join(", ")));
        }
        ui.add_space(8.0);
    }
}

pub fn render_table_panel(ui: &mut Ui, state: &GameStatePublic, preferred_player: PlayerId) {
    ui.group(|ui| {
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Pot:").strong());
            ui.monospace(format!(" {}", state.pot));
        });
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Board:").strong());
            if state.community.is_empty() {
                ui.label("—");
            }
            for &c in &state.community {
                super::ui_components::card_chip(ui, c);
            }
        });
        ui.add_space(8.0);
        ui.separator();
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Action log:").strong());
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui
                    .add(egui::Button::new("Copy to clipboard"))
                    .on_hover_text(
                        "Copy a structured summary of the current game and full action log",
                    )
                    .clicked()
                {
                    let clip =
                        super::ui_components::format_game_for_clipboard(state, preferred_player);
                    ui.ctx().copy_text(clip);
                }
            });
        });
        egui::ScrollArea::vertical()
            .id_salt("action_log_scroll")
            .max_height(200.0)
            .show(ui, |ui| {
                for entry in state.action_log.iter().rev().take(100) {
                    super::ui_components::log_entry_row(
                        ui,
                        entry,
                        &state.players,
                        preferred_player,
                    );
                }
            });
    });
}

pub fn render_player_status_and_bet(
    ui: &mut Ui,
    state: &GameStatePublic,
    p: &PlayerPublic,
    preferred_player: PlayerId,
) {
    if p.id == state.to_act && state.stage != mcg_shared::Stage::Showdown {
        ui.colored_label(Color32::from_rgb(255, 215, 0), "●");
    } else {
        ui.label("  ");
    }

    if p.id == preferred_player {
        ui.colored_label(Color32::LIGHT_GREEN, "You");
    }
    ui.label(egui::RichText::new(&p.name).strong());

    if p.bet_this_round > 0 {
        ui.label(format!("Bet: {}", p.bet_this_round));
    }

    if p.has_folded {
        ui.colored_label(Color32::LIGHT_RED, "(folded)");
    }

    if state.stage == mcg_shared::Stage::Showdown && state.winner_ids.contains(&p.id) {
        ui.colored_label(Color32::YELLOW, "WINNER");
    }

    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
        ui.monospace(format!("stack: {}", p.stack));
    });
}

pub fn render_my_cards_and_actions(
    ui: &mut Ui,
    state: &GameStatePublic,
    p: &PlayerPublic,
    preferred_player: PlayerId,
    poker_screen: &mut dyn PokerScreenActions,
) {
    ui.vertical(|ui| {
        if let Some(cards) = p.cards {
            ui.horizontal(|ui| {
                ui.add_space(12.0);
                super::ui_components::card_chip(ui, cards[0]);
                super::ui_components::card_chip(ui, cards[1]);
            });
            ui.add_space(6.0);
            ui.separator();
            ui.add_space(6.0);
        } else {
            ui.add_space(6.0);
            ui.separator();
            ui.add_space(6.0);
        }

        if p.id == state.to_act && state.stage != mcg_shared::Stage::Showdown {
            poker_screen.render_action_row(ui, state, p.id, true, false);
            ui.add_space(6.0);
            ui.separator();
        } else if p.id == preferred_player
            && (state.stage == mcg_shared::Stage::Showdown || p.cards.is_none())
        {
            poker_screen.render_action_row(ui, state, p.id, false, true);
            ui.add_space(6.0);
            ui.separator();
        } else {
            ui.add_space(8.0);
        }
    });
}

pub fn render_player(
    ui: &mut Ui,
    state: &GameStatePublic,
    p: &PlayerPublic,
    preferred_player: PlayerId,
    poker_screen: &mut dyn PokerScreenActions,
) {
    ui.horizontal(|ui| {
        render_player_status_and_bet(ui, state, p, preferred_player);
    });

    if p.id == preferred_player {
        render_my_cards_and_actions(ui, state, p, preferred_player, poker_screen);
    } else if state.stage == mcg_shared::Stage::Showdown {
        if let Some(cards) = p.cards {
            ui.horizontal(|ui| {
                ui.add_space(12.0);
                super::ui_components::card_chip(ui, cards[0]);
                super::ui_components::card_chip(ui, cards[1]);
            });
        }
    }
    ui.add_space(8.0);
}

pub fn render_players_panel(
    ui: &mut Ui,
    state: &GameStatePublic,
    preferred_player: PlayerId,
    poker_screen: &mut dyn PokerScreenActions,
) {
    ui.group(|ui| {
        for p in state.players.iter() {
            render_player(ui, state, p, preferred_player, poker_screen);
        }
    });
}

pub fn render_panels(
    ui: &mut Ui,
    state: &GameStatePublic,
    preferred_player: PlayerId,
    poker_screen: &mut dyn PokerScreenActions,
) {
    let narrow = ui.available_width() < 900.0;
    if narrow {
        render_players_panel(ui, state, preferred_player, poker_screen);
        ui.add_space(8.0);
        render_table_panel(ui, state, preferred_player);
    } else {
        ui.columns(2, |cols| {
            render_table_panel(&mut cols[0], state, preferred_player);
            render_players_panel(&mut cols[1], state, preferred_player, poker_screen);
        });
    }
}

// Trait to define poker screen actions that need to be implemented by the screen
pub trait PokerScreenActions {
    fn render_action_buttons(
        &mut self,
        ui: &mut Ui,
        state: &GameStatePublic,
        player_id: mcg_shared::PlayerId,
        enabled: bool,
    );
    fn render_action_row(
        &mut self,
        ui: &mut Ui,
        state: &GameStatePublic,
        player_id: mcg_shared::PlayerId,
        enabled: bool,
        show_next: bool,
    );
    fn send(&self, msg: &mcg_shared::ClientMsg);
}
