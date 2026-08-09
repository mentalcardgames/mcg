// Integration tests for the handoff demo games (well-known card games
// implemented in .cgdsl). The games use a shuffled deck, so assertions are
// structural (card counts, completion, winner existence) rather than
// value-based.

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use cgdsl_engine::{run_game, GameData, Input, InputKind, InputSource, InputType};
use front_end::ir::{Ir, LoweredPayLoad};
use front_end::validation::parse_document;

fn load_game(name: &str) -> Ir<LoweredPayLoad> {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let path = manifest.join("test_games").join(name);
    let src =
        std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {}", path.display(), e));
    let game = parse_document(&src).unwrap_or_else(|e| panic!("parse {}: {}", path.display(), e));
    game.to_lowered_graph()
}

/// Current-player tracker fed by an event_sender; answers carry the right
/// player_id so the controller's player validation (I-23) never re-prompts.
struct CurrentTracker(Arc<Mutex<Option<String>>>);

impl CurrentTracker {
    fn new() -> Self {
        Self(Arc::new(Mutex::new(None)))
    }
    fn sender(&self) -> Box<dyn Fn(&GameData) + Send> {
        let inner = self.0.clone();
        Box::new(move |gd: &GameData| {
            *inner.lock().unwrap() = gd.get_current_player().map(|p| p.name.clone());
        })
    }
}

fn total_cards(gd: &GameData) -> usize {
    gd.locations.iter().map(|l| l.cards.len()).sum()
}

fn players_in_game(gd: &GameData) -> usize {
    gd.players.iter().filter(|p| p.in_game).count()
}

#[test]
fn war_runs_to_completion() {
    let ir = load_game("war.cgdsl");
    let gd = run_game(
        ir,
        GameData::new(),
        InputSource::Player(Box::new(|_| Input {
            player_id: "P1".into(),
            kind: InputKind::Choice { idx: 0 },
        })),
        None,
        None,
    )
    .expect("war should complete");

    assert_eq!(total_cards(&gd), 52, "no cards may be lost or duplicated");
    assert!(
        players_in_game(&gd) >= 1,
        "at least one player must survive (tie => both)"
    );
    // The battle loop ends when either pile is empty; the other pile may
    // still hold cards.
    let remaining: usize = gd
        .locations
        .iter()
        .filter(|l| l.name.starts_with("P") && l.name.ends_with("Pile"))
        .map(|l| l.cards.len())
        .sum();
    assert!(
        remaining < 26,
        "battle must have consumed most of the piles"
    );
    // Scores are winnings counts; the winner is the higher score.
    let max_score = gd.players.iter().map(|p| p.score).max().unwrap_or(0);
    assert!(max_score > 0, "the winner must have captured cards");
}

#[test]
fn blackjack_runs_to_completion() {
    let ir = load_game("blackjack.cgdsl");
    // Script: P1 hits, P2 stands, P3 hits; everyone declines afterwards.
    let calls = Arc::new(Mutex::new(0usize));
    let calls_clone = calls.clone();
    let tracker = CurrentTracker::new();
    let current_for_closure = tracker.0.clone();
    let gd = run_game(
        ir,
        GameData::new(),
        InputSource::Player(Box::new(move |it: InputType| {
            let mut count = calls_clone.lock().unwrap();
            *count += 1;
            let n = *count;
            drop(count);
            let who = current_for_closure.lock().unwrap().clone();
            match it {
                InputType::Optional { .. } => {
                    let accept = n == 1 || n == 3;
                    Input {
                        player_id: who.clone().unwrap_or_else(|| "P1".into()),
                        kind: if accept {
                            InputKind::OptionalAccept
                        } else {
                            InputKind::OptionalDecline
                        },
                    }
                }
                _ => Input {
                    player_id: who.unwrap_or_else(|| "P1".into()),
                    kind: InputKind::Choice { idx: 0 },
                },
            }
        })),
        Some(tracker.sender()),
        None,
    )
    .expect("blackjack should complete");

    assert_eq!(total_cards(&gd), 52, "no cards may be lost or duplicated");
    for p in &gd.players {
        assert!(p.score >= 0, "scores must not go negative");
    }
}

#[test]
fn crazy_eights_runs_to_completion() {
    let ir = load_game("crazy_eights.cgdsl");
    // Every turn: play one card (accept the play-optional, pick the first
    // card), decline the draw-optional. No player is ever chosen for a gift.
    let tracker = CurrentTracker::new();
    let current_for_closure = tracker.0.clone();
    let gd = run_game(
        ir,
        GameData::new(),
        InputSource::Player(Box::new(move |it: InputType| {
            let who = current_for_closure.lock().unwrap().clone();
            let who = who.unwrap_or_else(|| "P1".into());
            match it {
                InputType::Optional(prompt) => {
                    let is_play = prompt.contains("deal any");
                    Input {
                        player_id: who,
                        kind: if is_play {
                            InputKind::OptionalAccept
                        } else {
                            InputKind::OptionalDecline
                        },
                    }
                }
                InputType::ChooseCards { .. } => Input {
                    player_id: who,
                    kind: InputKind::ChooseCards { selected: vec![0] },
                },
                InputType::ChoosePlayer { .. } => Input {
                    player_id: who,
                    kind: InputKind::ChoosePlayer { idx: 0 },
                },
                InputType::Choice { .. } => Input {
                    player_id: who,
                    kind: InputKind::Choice { idx: 0 },
                },
            }
        })),
        Some(tracker.sender()),
        None,
    )
    .expect("crazy eights should complete");

    assert_eq!(total_cards(&gd), 52, "no cards may be lost or duplicated");
    let discard = gd
        .locations
        .iter()
        .find(|l| l.name == "Discard")
        .map(|l| l.cards.len())
        .unwrap_or(0);
    assert!(
        discard >= 6,
        "starter + at least one play per first round: got {}",
        discard
    );
    // Someone must have won (possibly several on a tie).
    assert!(players_in_game(&gd) >= 1, "a winner must be declared");
}

#[test]
fn five_card_draw_runs_to_completion() {
    let ir = load_game("five_card_draw.cgdsl");
    // Every draw round: discard exactly one card (pick index 0), draw one.
    let tracker = CurrentTracker::new();
    let current_for_closure = tracker.0.clone();
    let gd = run_game(
        ir,
        GameData::new(),
        InputSource::Player(Box::new(move |it: InputType| {
            let who = current_for_closure.lock().unwrap().clone();
            let who = who.unwrap_or_else(|| "P1".into());
            match it {
                InputType::Optional { .. } => Input {
                    player_id: who,
                    kind: InputKind::OptionalAccept,
                },
                InputType::ChooseCards { .. } => Input {
                    player_id: who,
                    kind: InputKind::ChooseCards { selected: vec![0] },
                },
                _ => Input {
                    player_id: who,
                    kind: InputKind::Choice { idx: 0 },
                },
            }
        })),
        Some(tracker.sender()),
        None,
    )
    .expect("five card draw should complete");

    assert_eq!(total_cards(&gd), 52, "no cards may be lost or duplicated");
    for p in &gd.players {
        assert!(p.score > 0, "{} must have a positive hand value", p.name);
    }
    // 15 cards dealt, 9 discarded (one per player per round), 9 drawn back;
    // hands hold 5 each.
    let deck = gd
        .locations
        .iter()
        .find(|l| l.name == "Deck")
        .map(|l| l.cards.len())
        .unwrap_or(0);
    assert_eq!(deck, 52 - 15 - 9, "three rounds of draw-1 keep hands at 5");
    let discard = gd
        .locations
        .iter()
        .find(|l| l.name == "Discard")
        .map(|l| l.cards.len())
        .unwrap_or(0);
    assert_eq!(discard, 9, "one discard per player per round");
}

#[test]
fn blackjack_all_hit_until_bust() {
    // Adversarial path: every player accepts every hit until the round cap.
    // Previously crashed with "CycleAction: ... No next player available"
    // once everyone busted (guards now prevent that).
    let ir = load_game("blackjack.cgdsl");
    let tracker = CurrentTracker::new();
    let current_for_closure = tracker.0.clone();
    let gd = run_game(
        ir,
        GameData::new(),
        InputSource::Player(Box::new(move |it: InputType| {
            let who = current_for_closure.lock().unwrap().clone();
            let who = who.unwrap_or_else(|| "P1".into());
            match it {
                InputType::Optional { .. } => Input {
                    player_id: who,
                    kind: InputKind::OptionalAccept,
                },
                _ => Input {
                    player_id: who,
                    kind: InputKind::Choice { idx: 0 },
                },
            }
        })),
        Some(tracker.sender()),
        None,
    )
    .expect("blackjack must not crash even when everyone busts");

    assert_eq!(total_cards(&gd), 52, "no cards may be lost or duplicated");
}

#[test]
fn crazy_eights_draw_every_turn() {
    // Adversarial path: play a card, then accept the draw each turn and
    // gift it to P1 (ChoosePlayer index 0). The deck must drain and the
    // game must still terminate at the round cap.
    let ir = load_game("crazy_eights.cgdsl");
    let tracker = CurrentTracker::new();
    let current_for_closure = tracker.0.clone();
    let gd = run_game(
        ir,
        GameData::new(),
        InputSource::Player(Box::new(move |it: InputType| {
            let who = current_for_closure.lock().unwrap().clone();
            let who = who.unwrap_or_else(|| "P1".into());
            match it {
                InputType::Optional(prompt) => {
                    // Accept both the play and the draw optionals.
                    let _ = prompt;
                    Input {
                        player_id: who,
                        kind: InputKind::OptionalAccept,
                    }
                }
                InputType::ChooseCards { .. } => Input {
                    player_id: who,
                    kind: InputKind::ChooseCards { selected: vec![0] },
                },
                InputType::ChoosePlayer { .. } => Input {
                    player_id: who,
                    kind: InputKind::ChoosePlayer { idx: 0 },
                },
                InputType::Choice { .. } => Input {
                    player_id: who,
                    kind: InputKind::Choice { idx: 0 },
                },
            }
        })),
        Some(tracker.sender()),
        None,
    )
    .expect("crazy eights must terminate with the draw path");

    assert_eq!(total_cards(&gd), 52, "no cards may be lost or duplicated");
}

#[test]
fn go_fish_rotating_asks() {
    // Adversarial path: ask for a different rank every turn (0..12), so
    // every choose-option is exercised.
    let ir = load_game("go_fish.cgdsl");
    let tracker = CurrentTracker::new();
    let current_for_closure = tracker.0.clone();
    let asks = Arc::new(Mutex::new(0usize));
    let asks_clone = asks.clone();
    let gd = run_game(
        ir,
        GameData::new(),
        InputSource::Player(Box::new(move |it: InputType| {
            let who = current_for_closure.lock().unwrap().clone();
            let who = who.unwrap_or_else(|| "P1".into());
            match it {
                InputType::Choice { .. } => {
                    let mut n = asks_clone.lock().unwrap();
                    let idx = *n % 13;
                    *n += 1;
                    drop(n);
                    Input {
                        player_id: who,
                        kind: InputKind::Choice { idx },
                    }
                }
                InputType::Optional(prompt) => {
                    // Decline the book-laydown optional ("Book" in the label).
                    let accept = !prompt.contains("Book");
                    Input {
                        player_id: who,
                        kind: if accept {
                            InputKind::OptionalAccept
                        } else {
                            InputKind::OptionalDecline
                        },
                    }
                }
                InputType::ChooseCards { .. } => Input {
                    player_id: who,
                    kind: InputKind::ChooseCards { selected: vec![] }, // skip book
                },
                _ => Input {
                    player_id: who,
                    kind: InputKind::Choice { idx: 0 },
                },
            }
        })),
        Some(tracker.sender()),
        None,
    )
    .expect("go fish must complete with rotating asks");

    assert_eq!(total_cards(&gd), 52, "no cards may be lost or duplicated");
}

#[test]
fn go_fish_runs_to_completion() {
    let ir = load_game("go_fish.cgdsl");
    // Always ask for "Ace" (option 0).
    let tracker = CurrentTracker::new();
    let current_for_closure = tracker.0.clone();
    let gd = run_game(
        ir,
        GameData::new(),
        InputSource::Player(Box::new(move |it: InputType| {
            let who = current_for_closure.lock().unwrap().clone();
            let who = who.unwrap_or_else(|| "P1".into());
            match it {
                InputType::Choice { .. } => Input {
                    player_id: who,
                    kind: InputKind::Choice { idx: 0 },
                },
                InputType::Optional(prompt) => {
                    // Decline the book-laydown optional ("Book" in the label).
                    let accept = !prompt.contains("Book");
                    Input {
                        player_id: who,
                        kind: if accept {
                            InputKind::OptionalAccept
                        } else {
                            InputKind::OptionalDecline
                        },
                    }
                }
                InputType::ChooseCards { .. } => Input {
                    player_id: who,
                    kind: InputKind::ChooseCards { selected: vec![] }, // skip book
                },
                _ => Input {
                    player_id: who,
                    kind: InputKind::Choice { idx: 0 },
                },
            }
        })),
        Some(tracker.sender()),
        None,
    )
    .expect("go fish should complete");

    assert_eq!(total_cards(&gd), 52, "no cards may be lost or duplicated");
    assert!(players_in_game(&gd) >= 1, "a winner must be declared");
}
