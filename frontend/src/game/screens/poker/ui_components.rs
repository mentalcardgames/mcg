use egui::{Color32, RichText, Ui, WidgetText};
use mcg_shared::{
    ActionEvent, ActionKind, BlindKind, Card, GameAction, GameStatePublic, HandRankCategory,
    HandResult, PlayerId, PlayerPublic, Stage,
};

pub fn card_chip(ui: &mut Ui, c: Card) {
    let (text, color) = card_text_and_color(c);
    let b = egui::widgets::Button::new(RichText::new(text).color(color).size(28.0))
        .min_size(egui::vec2(48.0, 40.0));
    ui.add(b);
}

pub fn card_text_and_color(c: Card) -> (String, Color32) {
    let text = c.to_string();
    let color = if c.is_red() {
        Color32::from_rgb(220, 50, 50)
    } else {
        Color32::WHITE
    };
    (text, color)
}

pub fn action_kind_text(kind: &ActionKind) -> (String, Color32) {
    match kind {
        ActionKind::Fold => ("🟥 folds".into(), Color32::from_rgb(220, 80, 80)),
        ActionKind::Check => ("⏭ checks".into(), Color32::from_rgb(120, 160, 220)),
        ActionKind::Call(n) => (format!("📞 calls {}", n), Color32::from_rgb(120, 160, 220)),
        ActionKind::Bet(n) => (format!("💰 bets {}", n), Color32::from_rgb(240, 200, 80)),
        ActionKind::Raise { to, by } => (
            format!("▲ raises to {} (+{})", to, by),
            Color32::from_rgb(250, 160, 60),
        ),
        ActionKind::PostBlind { kind, amount } => match kind {
            BlindKind::SmallBlind => (
                format!("● posts small blind {}", amount),
                Color32::from_rgb(170, 120, 60),
            ),
            BlindKind::BigBlind => (
                format!("⚫ posts big blind {}", amount),
                Color32::from_rgb(120, 120, 120),
            ),
        },
    }
}

pub fn category_text(cat: &HandRankCategory) -> &'static str {
    match cat {
        HandRankCategory::HighCard => "High Card",
        HandRankCategory::Pair => "Pair",
        HandRankCategory::TwoPair => "Two Pair",
        HandRankCategory::ThreeKind => "Three of a Kind",
        HandRankCategory::Straight => "Straight",
        HandRankCategory::Flush => "Flush",
        HandRankCategory::FullHouse => "Full House",
        HandRankCategory::FourKind => "Four of a Kind",
        HandRankCategory::StraightFlush => "Straight Flush",
    }
}

pub fn name_of(players: &[PlayerPublic], id: PlayerId) -> String {
    PlayerPublic::name_of(players, id)
}

pub fn card_text(c: Card) -> String {
    c.to_string()
}

pub fn stage_badge(stage: Stage) -> WidgetText {
    let (txt, color) = match stage {
        Stage::Preflop => ("Preflop", Color32::from_rgb(100, 150, 255)),
        Stage::Flop => ("Flop", Color32::from_rgb(100, 200, 120)),
        Stage::Turn => ("Turn", Color32::from_rgb(230, 180, 80)),
        Stage::River => ("River", Color32::from_rgb(220, 120, 120)),
        Stage::Showdown => ("Showdown", Color32::from_rgb(180, 100, 220)),
    };
    RichText::new(txt).color(color).strong().into()
}

pub fn stage_to_str(stage: Stage) -> &'static str {
    match stage {
        Stage::Preflop => "Preflop",
        Stage::Flop => "Flop",
        Stage::Turn => "Turn",
        Stage::River => "River",
        Stage::Showdown => "Showdown",
    }
}

pub fn format_game_for_clipboard(state: &GameStatePublic, you: PlayerId) -> String {
    let mut out = String::new();

    format_game_summary(&mut out, state, you);
    format_players_section(&mut out, state, you);
    format_board_section(&mut out, state);
    format_action_log(&mut out, state);

    out
}

fn format_game_summary(out: &mut String, state: &GameStatePublic, you: PlayerId) {
    out.push_str("Game summary\n");
    out.push_str(&format!("Stage: {}\n", stage_to_str(state.stage)));
    out.push_str(&format!("Pot: {}\n", state.pot));

    if let Some(p) = state.players.iter().find(|p| p.id == you) {
        if let Some(cards) = p.cards {
            out.push_str(&format!(
                "Your hole cards: {}, {}\n",
                card_text(cards[0]),
                card_text(cards[1])
            ));
        } else {
            out.push_str("Your hole cards: (hidden)\n");
        }
    }
    out.push('\n');
}

fn format_players_section(out: &mut String, state: &GameStatePublic, you: PlayerId) {
    out.push_str("Players\n");
    for p in state.players.iter() {
        format_player_entry(out, state, p, you);
    }
    out.push('\n');
}

fn format_player_entry(
    out: &mut String,
    state: &GameStatePublic,
    player: &PlayerPublic,
    you: PlayerId,
) {
    let you_str = if player.id == you { " (you)" } else { "" };
    let folded = if player.has_folded {
        ", folded:true"
    } else {
        ""
    };
    let to_act = if state.stage != Stage::Showdown && player.id == state.to_act {
        ", to_act:true"
    } else {
        ""
    };
    out.push_str(&format!(
        "- id:{}, name:{}, stack:{}{}{}{}\n",
        player.id, player.name, player.stack, you_str, folded, to_act
    ));

    if player.id == you {
        if let Some(cards) = player.cards {
            out.push_str(&format!(
                "  hole: {}, {}\n",
                card_text(cards[0]),
                card_text(cards[1])
            ));
        }
    }
}

fn format_board_section(out: &mut String, state: &GameStatePublic) {
    out.push_str("Board\n");
    if state.community.is_empty() {
        out.push_str("- (no community cards yet)\n");
    } else {
        let board = state
            .community
            .iter()
            .map(|&c| card_text(c))
            .collect::<Vec<_>>()
            .join(", ");
        out.push_str(&format!("- {}\n", board));
    }
    out.push('\n');
}

fn format_action_log(out: &mut String, state: &GameStatePublic) {
    out.push_str("Action log (chronological)\n");
    for entry in &state.action_log {
        format_action_log_entry(out, entry, state);
    }
}

fn format_action_log_entry(out: &mut String, entry: &ActionEvent, state: &GameStatePublic) {
    match entry {
        ActionEvent::PlayerAction { player_id, action } => {
            format_player_action_entry(out, *player_id, action, state);
        }
        ActionEvent::GameAction(game_action) => {
            format_game_action_entry(out, game_action, state);
        }
    }
}

fn format_player_action_entry(
    out: &mut String,
    player_id: PlayerId,
    action: &ActionKind,
    state: &GameStatePublic,
) {
    let who_name = name_of(&state.players, player_id);
    match action {
        ActionKind::Fold => out.push_str(&format!("- {} folds\n", who_name)),
        ActionKind::Check => out.push_str(&format!("- {} checks\n", who_name)),
        ActionKind::Call(n) => out.push_str(&format!("- {} calls {}\n", who_name, n)),
        ActionKind::Bet(n) => out.push_str(&format!("- {} bets {}\n", who_name, n)),
        ActionKind::Raise { to, by } => {
            out.push_str(&format!("- {} raises to {} (+{})\n", who_name, to, by))
        }
        ActionKind::PostBlind { kind, amount } => {
            format_blind_entry(out, &who_name, kind, amount);
        }
    }
}

fn format_blind_entry(out: &mut String, who_name: &str, kind: &BlindKind, amount: &u32) {
    match kind {
        BlindKind::SmallBlind => {
            out.push_str(&format!("- {} posts small blind {}\n", who_name, amount))
        }
        BlindKind::BigBlind => {
            out.push_str(&format!("- {} posts big blind {}\n", who_name, amount))
        }
    }
}

fn format_game_action_entry(out: &mut String, game_action: &GameAction, state: &GameStatePublic) {
    match game_action {
        GameAction::StageChanged(s) => {
            out.push_str(&format!("== Stage: {} ==\\n", stage_to_str(*s)));
        }
        GameAction::DealtHole { player_id } => {
            let who = name_of(&state.players, *player_id);
            out.push_str(&format!("- Dealt hole cards to {}\n", who));
        }
        GameAction::DealtCommunity { cards } => {
            format_community_cards_entry(out, cards);
        }
        GameAction::Showdown { hand_results } => {
            format_showdown_entry(out, hand_results, state);
        }
        GameAction::PotAwarded { winners, amount } => {
            format_pot_awarded_entry(out, winners, amount, state);
        }
    }
}

fn format_community_cards_entry(out: &mut String, cards: &[Card]) {
    match cards.len() {
        3 => out.push_str(&format!(
            "- Flop: {}, {}, {}\n",
            card_text(cards[0]),
            card_text(cards[1]),
            card_text(cards[2])
        )),
        4 => out.push_str(&format!("- Turn: {}\n", card_text(cards[3]))),
        5 => out.push_str(&format!("- River: {}\n", card_text(cards[4]))),
        _ => {
            let s = cards
                .iter()
                .map(|&c| card_text(c))
                .collect::<Vec<_>>()
                .join(", ");
            out.push_str(&format!("- Community: {}\n", s));
        }
    }
}

fn format_showdown_entry(out: &mut String, hand_results: &[HandResult], state: &GameStatePublic) {
    if hand_results.is_empty() {
        out.push_str("- Showdown\n");
    } else {
        for hr in hand_results {
            let who = name_of(&state.players, hr.player_id);
            let cat = category_text(&hr.rank.category);
            let best = hr
                .best_five
                .iter()
                .map(|&c| card_text(c))
                .collect::<Vec<_>>()
                .join(", ");
            out.push_str(&format!("- Showdown: {} -> {} [{}]\n", who, cat, best));
        }
    }
}

fn format_pot_awarded_entry(
    out: &mut String,
    winners: &[PlayerId],
    amount: &u32,
    state: &GameStatePublic,
) {
    let names = winners
        .iter()
        .map(|&id| name_of(&state.players, id))
        .collect::<Vec<_>>()
        .join(", ");
    out.push_str(&format!("- Pot {} awarded to {}\n", amount, names));
}

pub fn log_entry_row(ui: &mut Ui, entry: &ActionEvent, players: &[PlayerPublic], you_id: PlayerId) {
    match entry {
        ActionEvent::PlayerAction { player_id, action } => {
            let who_id = Some(*player_id);
            let who_name = name_of(players, *player_id);
            let (txt, color) = action_kind_text(action);
            let is_you = who_id == Some(you_id);
            let label = if is_you {
                RichText::new(format!("{} {}", who_name, txt))
                    .color(color)
                    .strong()
            } else {
                RichText::new(format!("{} {}", who_name, txt)).color(color)
            };
            ui.label(label);
        }
        ActionEvent::GameAction(GameAction::StageChanged(s)) => {
            ui.add_space(6.0);
            ui.separator();
            ui.horizontal(|ui| {
                ui.label(RichText::new("🕒").strong());
                ui.label(stage_badge(*s));
            });
            ui.separator();
            ui.add_space(6.0);
        }
        ActionEvent::GameAction(GameAction::DealtHole { player_id }) => {
            let who = name_of(players, *player_id);
            ui.colored_label(
                Color32::from_rgb(150, 150, 150),
                format!("🃏 Dealt hole cards to {}", who),
            );
        }
        ActionEvent::GameAction(GameAction::DealtCommunity { cards }) => match cards.len() {
            3 => {
                ui.colored_label(
                    Color32::from_rgb(100, 200, 120),
                    format!(
                        "🃏 Flop: {} {} {}",
                        card_text(cards[0]),
                        card_text(cards[1]),
                        card_text(cards[2])
                    ),
                );
            }
            4 => {
                ui.colored_label(
                    Color32::from_rgb(230, 180, 80),
                    format!("🃏 Turn: {}", card_text(cards[3])),
                );
            }
            5 => {
                ui.colored_label(
                    Color32::from_rgb(220, 120, 120),
                    format!("🃏 River: {}", card_text(cards[4])),
                );
            }
            _ => {
                ui.colored_label(
                    Color32::from_rgb(120, 120, 120),
                    format!(
                        "🃏 Community: {}",
                        cards
                            .iter()
                            .map(|&c| card_text(c))
                            .collect::<Vec<_>>()
                            .join(" ")
                    ),
                );
            }
        },
        ActionEvent::GameAction(GameAction::Showdown { hand_results }) => {
            let mut parts = Vec::new();
            for hr in hand_results {
                let who = name_of(players, hr.player_id);
                let cat = category_text(&hr.rank.category);
                parts.push(format!("{}: {}", who, cat));
            }
            let text = if parts.is_empty() {
                "🏁 Showdown".to_string()
            } else {
                format!("🏁 Showdown — {}", parts.join(", "))
            };
            ui.colored_label(Color32::from_rgb(180, 100, 220), text);
        }
        ActionEvent::GameAction(GameAction::PotAwarded { winners, amount }) => {
            let names = winners
                .iter()
                .map(|&id| name_of(players, id))
                .collect::<Vec<_>>()
                .join(", ");
            ui.colored_label(
                Color32::from_rgb(240, 200, 80),
                format!("🏆 Pot {} awarded to {}", amount, names),
            );
        }
    }
}
