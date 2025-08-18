use mcg_shared::{
    ActionKind as SharedActionKind, BlindKind, GameStatePublic, ActionEvent, GameAction,
    PlayerPublic, Stage,
};
use owo_colors::OwoColorize;

fn card_rank(c: u8) -> u8 {
    c % 13
}
fn card_suit(c: u8) -> u8 {
    c / 13
}
fn card_faces() -> [&'static str; 13] {
    [
        "A", "2", "3", "4", "5", "6", "7", "8", "9", "T", "J", "Q", "K",
    ]
}
fn suit_icon(s: u8) -> char {
    match s {
        0 => '♣',
        1 => '♦',
        2 => '♥',
        _ => '♠',
    }
}
fn suit_name(s: u8) -> &'static str {
    match s {
        0 => "Clubs",
        1 => "Diamonds",
        2 => "Hearts",
        _ => "Spades",
    }
}
fn rank_name(r: u8) -> &'static str {
    match r {
        0 => "Ace",
        1 => "Two",
        2 => "Three",
        3 => "Four",
        4 => "Five",
        5 => "Six",
        6 => "Seven",
        7 => "Eight",
        8 => "Nine",
        9 => "Ten",
        10 => "Jack",
        11 => "Queen",
        _ => "King",
    }
}

fn format_card(c: u8, color: bool) -> String {
    let r = card_rank(c) as usize;
    let s = card_suit(c);
    let face = card_faces()[r];
    let icon = suit_icon(s);
    let mut text = format!(
        "{}{} ({} of {})",
        face,
        icon,
        rank_name(r as u8),
        suit_name(s)
    );
    if color {
        text = match s {
            1 | 2 => text.red().to_string(),
            _ => text.to_string(),
        };
    }
    text
}

#[allow(dead_code)]
fn format_cards(cards: &[u8], color: bool) -> String {
    cards
        .iter()
        .map(|&c| format_card(c, color))
        .collect::<Vec<_>>()
        .join(", ")
}

fn player_name(players: &[PlayerPublic], id: usize, you_id: usize, color: bool) -> String {
    let base = players
        .iter()
        .find(|p| p.id == id)
        .map(|p| p.name.clone())
        .unwrap_or_else(|| format!("P{}", id));
    if id == you_id {
        if color {
            base.bold().to_string()
        } else {
            base
        }
    } else {
        base
    }
}

fn format_log_entry(
    entry: &ActionEvent,
    players: &[PlayerPublic],
    you_id: usize,
    color: bool,
) -> String {
    match entry {
        ActionEvent::PlayerAction { player_id, action } => {
            let who = player_name(players, *player_id, you_id, color);
            match action {
                SharedActionKind::Fold => format!(
                    "{} {} (fold)",
                    if color {
                        "↩".red().to_string()
                    } else {
                        "FOLD".into()
                    },
                    who
                ),
                SharedActionKind::Check => format!(
                    "{} {} (check)",
                    if color {
                        "✓".green().to_string()
                    } else {
                        "CHECK".into()
                    },
                    who
                ),
                SharedActionKind::Call(n) => format!(
                    "{} {} {} (call)",
                    if color {
                        "↪".cyan().to_string()
                    } else {
                        "CALL".into()
                    },
                    who,
                    n
                ),
                SharedActionKind::Bet(n) => format!(
                    "{} {} {} (bet)",
                    if color {
                        "●".yellow().to_string()
                    } else {
                        "BET".into()
                    },
                    who,
                    n
                ),
                SharedActionKind::Raise { to, by } => format!(
                    "{} {} to {} (+{}) (raise)",
                    if color {
                        "▲".magenta().to_string()
                    } else {
                        "RAISE".into()
                    },
                    who,
                    to,
                    by
                ),
                SharedActionKind::PostBlind { kind, amount } => {
                    let k = match *kind {
                        BlindKind::SmallBlind => "SB",
                        BlindKind::BigBlind => "BB",
                    };
                    format!("{} {} {}", k, who, amount)
                }
            }
        }
        ActionEvent::GameAction(GameAction::DealtCommunity { cards }) => {
            let list = cards
                .iter()
                .map(|c| format_card(*c, color))
                .collect::<Vec<_>>()
                .join(", ");
            format!("Board +[{}]", list)
        }
        ActionEvent::GameAction(GameAction::DealtHole { player_id }) => {
            let who = player_name(players, *player_id, you_id, color);
            format!("Dealt hole to {}", who)
        }
        ActionEvent::GameAction(GameAction::Showdown { .. }) => "Showdown".into(),
        ActionEvent::GameAction(GameAction::PotAwarded { winners, amount }) => {
            let names = winners
                .iter()
                .map(|id| player_name(players, *id, you_id, color))
                .collect::<Vec<_>>()
                .join(", ");
            format!("Pot awarded {} -> [{}]", amount, names)
        }
        ActionEvent::GameAction(GameAction::StageChanged(_)) => unreachable!(),
    }
}

pub fn format_event_human(
    entry: &ActionEvent,
    players: &[PlayerPublic],
    you_id: usize,
    color: bool,
) -> String {
    match entry {
        ActionEvent::GameAction(GameAction::StageChanged(s)) => {
            let sname = format!("== {:?} ==", s);
            if color {
                sname.bold().purple().to_string()
            } else {
                sname
            }
        }
        _ => format_log_entry(entry, players, you_id, color),
    }
}

pub fn format_table_header(gs: &GameStatePublic, sb: u32, bb: u32, color: bool) -> String {
    let mut out = String::new();
    let title = if color {
        "=== New Hand ===".bold().blue().to_string()
    } else {
        "=== New Hand ===".to_string()
    };
    let blinds = if color {
        format!("{} SB {} / BB {}", "Blinds:".bold().yellow(), sb, bb)
    } else {
        format!("Blinds: SB {} / BB {}", sb, bb)
    };
    out.push_str(&format!("{}\n{}\n", title, blinds));
    out.push_str("Players:\n");
    for p in &gs.players {
        let name = if p.id == gs.you_id {
            if color {
                format!("{}{}", p.name.bold(), " (You)".bold())
            } else {
                format!("{} (You)", p.name)
            }
        } else {
            p.name.clone()
        };
        let folded = if p.has_folded {
            if color {
                " [FOLDED]".red().to_string()
            } else {
                " [FOLDED]".to_string()
            }
        } else {
            String::new()
        };
        let to_act_icon = if p.id == gs.to_act {
            if color {
                " ●".green().to_string()
            } else {
                " *".to_string()
            }
        } else {
            String::new()
        };
        let to_act_text = if p.id == gs.to_act { " (to act)" } else { "" };
        out.push_str(&format!(
            "  #{} {}  stack={}{}{}{}\n",
            p.id, name, p.stack, folded, to_act_icon, to_act_text
        ));
    }
    out
}

#[allow(dead_code)]
pub fn format_state_human(gs: &GameStatePublic, color: bool) -> String {
    let mut out = String::new();

    // Header
    let stage = format!("{:?}", gs.stage);
    let stage_s = if color {
        stage.bold().blue().to_string()
    } else {
        stage
    };
    let pot_s = if color {
        format!("{} {}", "Pot:".bold().yellow(), gs.pot)
    } else {
        format!("Pot: {}", gs.pot)
    };
    out.push_str(&format!("{}  |  {}\n", stage_s, pot_s));

    // Board and hole
    if !gs.community.is_empty() {
        let board = format_cards(&gs.community, color);
        out.push_str(&format!("Board: [{}]\n", board));
    }
    if let Some(you) = gs.players.iter().find(|p| p.id == gs.you_id) {
        if let Some(cards) = you.cards {
            let you_cards = format_cards(&cards, color);
            out.push_str(&format!("Your cards: [{}]\n", you_cards));
        }
    }

    // Players
    out.push_str("Players:\n");
    for p in &gs.players {
        let name = if p.id == gs.you_id {
            format!(
                "{}{}",
                p.name,
                if color {
                    " (You)".bold().to_string()
                } else {
                    " (You)".to_string()
                }
            )
        } else {
            p.name.clone()
        };
        let folded = if p.has_folded {
            if color {
                " [FOLDED]".red().to_string()
            } else {
                " [FOLDED]".to_string()
            }
        } else {
            String::new()
        };
        let to_act_icon = if p.id == gs.to_act {
            if color {
                " ●".green().to_string()
            } else {
                " *".to_string()
            }
        } else {
            String::new()
        };
        let to_act_text = if p.id == gs.to_act { " (to act)" } else { "" };
        out.push_str(&format!(
            "  #{} {}  stack={}{}{}{}\n",
            p.id, name, p.stack, folded, to_act_icon, to_act_text
        ));
    }

    // Log
    if !gs.action_log.is_empty() {
        out.push_str("\nLog:\n");
        let mut last_stage: Option<Stage> = None;
        for e in &gs.action_log {
            if let ActionEvent::GameAction(GameAction::StageChanged(s)) = e {
                if last_stage != Some(*s) {
                    last_stage = Some(*s);
                    let sname = format!("== {:?} ==", s);
                    let sline = if color {
                        sname.bold().purple().to_string()
                    } else {
                        sname
                    };
                    out.push_str(&format!("{}\n", sline));
                }
                continue;
            }
            out.push_str(&format!(
                "  {}\n",
                format_log_entry(e, &gs.players, gs.you_id, color)
            ));
        }
    }

    out
}
