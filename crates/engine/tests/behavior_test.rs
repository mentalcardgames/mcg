//! Behavioral fixtures: deterministic (non-shuffled) games where the exact
//! outcome is known from the game rules, so the assertions verify that the
//! engine *plays the game correctly* — not merely that it terminates.
//!
//! Determinism trick: without `shuffle Deck`, card creation order defines the
//! deck order (`expand_types` cartesian product), and `deal N` takes the top
//! N cards. Every hand, draw, and score below is therefore fully predictable.

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

fn total_cards(gd: &GameData) -> usize {
    gd.locations.iter().map(|l| l.cards.len()).sum()
}

/// The `Hand` location owned by `player_name`.
fn hand_location<'a>(gd: &'a GameData, player_name: &str) -> &'a cgdsl_engine::Location {
    let player = gd
        .players
        .iter()
        .find(|p| p.name == player_name)
        .unwrap_or_else(|| panic!("player {player_name} not found"));
    let idx = *player
        .owner
        .locations
        .iter()
        .find(|&&i| gd.locations[i].name == "Hand")
        .expect("player has a Hand location");
    &gd.locations[idx]
}

fn count_rank(gd: &GameData, player_name: &str, rank: &str) -> usize {
    hand_location(gd, player_name)
        .cards
        .iter()
        .filter(|&&id| gd.cards[id].get("Rank").map(|r| r == rank).unwrap_or(false))
        .count()
}

fn players_in_game(gd: &GameData) -> Vec<&str> {
    gd.players
        .iter()
        .filter(|p| p.in_game)
        .map(|p| p.name.as_str())
        .collect()
}

/// Current-player tracker fed by an event_sender (I-23).
struct Tracker(Arc<Mutex<Option<String>>>);

impl Tracker {
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

// ---------------------------------------------------------------------------
// Go Fish — ask semantics
// ---------------------------------------------------------------------------

fn play_go_fish_ask(option_idx: usize) -> GameData {
    let ir = load_game("behavior_go_fish_ask.cgdsl");
    let tracker = Tracker::new();
    let who_for_closure = tracker.0.clone();
    run_game(
        ir,
        GameData::new(),
        InputSource::Player(Box::new(move |it: InputType| {
            let who = who_for_closure
                .lock()
                .unwrap()
                .clone()
                .unwrap_or_else(|| "P1".into());
            match it {
                InputType::Choice { .. } => Input {
                    player_id: who,
                    kind: InputKind::Choice { idx: option_idx },
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
    .expect("go fish ask must complete")
}

#[test]
fn go_fish_ask_held_rank_transfers_no_draw() {
    // P2 holds two Threes (Three-C, Three-H); asking "Three" must transfer
    // both, not draw.
    let gd = play_go_fish_ask(2); // option 2 = "Three"
    assert_eq!(
        hand_location(&gd, "P1").cards.len(),
        9,
        "asker gains both cards"
    );
    assert_eq!(
        hand_location(&gd, "P2").cards.len(),
        5,
        "asked player loses them"
    );
    assert_eq!(hand_location(&gd, "P3").cards.len(), 7, "P3 untouched");
    assert_eq!(
        count_rank(&gd, "P1", "Three"),
        3,
        "Three-D joined by Three-C/H"
    );
    assert_eq!(
        count_rank(&gd, "P2", "Three"),
        0,
        "P2 no longer holds a Three"
    );
    let deck = gd.locations.iter().find(|l| l.name == "Deck").unwrap();
    assert_eq!(deck.cards.len(), 3, "no draw on a hit");
    assert_eq!(total_cards(&gd), 24);
}

#[test]
fn go_fish_ask_missing_rank_draws_one() {
    // P2 holds no Eight; asking "Eight" must draw exactly one card.
    let gd = play_go_fish_ask(7); // option 7 = "Eight"
    assert_eq!(hand_location(&gd, "P1").cards.len(), 8, "asker draws one");
    assert_eq!(hand_location(&gd, "P2").cards.len(), 7, "P2 untouched");
    assert_eq!(count_rank(&gd, "P1", "Eight"), 1, "drew Eight-D");
    let deck = gd.locations.iter().find(|l| l.name == "Deck").unwrap();
    assert_eq!(deck.cards.len(), 2, "exactly one draw on a miss");
    assert_eq!(total_cards(&gd), 24);
}

// ---------------------------------------------------------------------------
// War — battle and winner
// ---------------------------------------------------------------------------

#[test]
fn war_battle_captures_and_declares_winner() {
    let ir = load_game("behavior_war.cgdsl");
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
    .expect("war must complete");

    // Rounds: Ace-D beats Two-C (P1), Two-D loses to Three-C (P2),
    // Three-D loses to Ace-C (P2). P2 captures 4, P1 captures 2.
    assert_eq!(players_in_game(&gd), vec!["P2"], "P2 wins");
    let p1 = &gd.players[0];
    let p2 = &gd.players[1];
    assert_eq!(p1.score, 2);
    assert_eq!(p2.score, 4);
    let winnings: Vec<&cgdsl_engine::Location> = gd
        .locations
        .iter()
        .filter(|l| l.name.contains("Winnings"))
        .collect();
    assert_eq!(winnings.iter().map(|l| l.cards.len()).sum::<usize>(), 6);
    let discard = gd.locations.iter().find(|l| l.name == "Discard").unwrap();
    assert_eq!(discard.cards.len(), 0, "no ties in this deck");
    assert_eq!(total_cards(&gd), 6);
}

// ---------------------------------------------------------------------------
// Blackjack — dealing, dealer draw, scoring, winner
// ---------------------------------------------------------------------------

#[test]
fn blackjack_stands_dealer_draws_and_best_hand_wins() {
    let ir = load_game("behavior_blackjack.cgdsl");
    let tracker = Tracker::new();
    let who_for_closure = tracker.0.clone();
    let gd = run_game(
        ir,
        GameData::new(),
        InputSource::Player(Box::new(move |it: InputType| {
            let who = who_for_closure
                .lock()
                .unwrap()
                .clone()
                .unwrap_or_else(|| "P1".into());
            match it {
                InputType::Optional { .. } => Input {
                    player_id: who,
                    kind: InputKind::OptionalDecline, // everyone stands
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
    .expect("blackjack must complete");

    assert_eq!(
        players_in_game(&gd),
        vec!["P1"],
        "P1's 21 beats the dealer's 18"
    );
    assert_eq!(gd.players[0].score, 21);
    assert_eq!(gd.players[1].score, 0);
    assert_eq!(gd.players[2].score, 0);
    let dealer = gd
        .locations
        .iter()
        .find(|l| l.name == "DealerHand")
        .unwrap();
    assert_eq!(
        dealer.cards.len(),
        3,
        "Six + Five + Seven = 18, then stands"
    );
    let deck = gd.locations.iter().find(|l| l.name == "Deck").unwrap();
    assert_eq!(deck.cards.len(), 0, "all 9 cards dealt or drawn");
    assert_eq!(total_cards(&gd), 9);
}

// ---------------------------------------------------------------------------
// Five-Card Draw — sum + pair bonus + flush bonus
// ---------------------------------------------------------------------------

#[test]
fn five_card_draw_scores_hand_bonuses() {
    let ir = load_game("behavior_five_card_draw.cgdsl");
    let tracker = Tracker::new();
    let who_for_closure = tracker.0.clone();
    let gd = run_game(
        ir,
        GameData::new(),
        InputSource::Player(Box::new(move |it: InputType| {
            let who = who_for_closure
                .lock()
                .unwrap()
                .clone()
                .unwrap_or_else(|| "P1".into());
            match it {
                InputType::Optional { .. } => Input {
                    player_id: who,
                    kind: InputKind::OptionalDecline, // keep the dealt hands
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
    .expect("five card draw must complete");

    // P1: 29 + pair(10) + flush(20) = 59; P2: 20 + flush(20) = 40;
    // P3: 48 + flush(20) = 68 -> winner.
    let scores: Vec<i32> = gd.players.iter().map(|p| p.score).collect();
    assert_eq!(scores, vec![59, 40, 68], "sum + pair + flush bonuses");
    assert_eq!(players_in_game(&gd), vec!["P3"]);
    assert_eq!(total_cards(&gd), 15);
}

// ---------------------------------------------------------------------------
// Combos — laying down sets that match a combo definition
// ---------------------------------------------------------------------------

#[test]
fn combo_laydown_prompts_and_validates() {
    let ir = load_game("behavior_combo_laydown.cgdsl");
    let tracker = Tracker::new();
    let who_for_closure = tracker.0.clone();
    let asks = Arc::new(Mutex::new(0usize));
    let asks_clone = asks.clone();
    let gd = run_game(
        ir,
        GameData::new(),
        InputSource::Player(Box::new(move |it: InputType| {
            let who = who_for_closure
                .lock()
                .unwrap()
                .clone()
                .unwrap_or_else(|| "P1".into());
            match it {
                InputType::ChooseCards { .. } => {
                    let mut n = asks_clone.lock().unwrap();
                    *n += 1;
                    let i = *n;
                    drop(n);
                    // LaySet: first an INVALID set (2 Aces + a Two) to
                    // exercise the re-prompt, then the valid three Aces.
                    // LayRun: Three..Seven of the remaining 7 cards.
                    let selected = match i {
                        1 => vec![0, 1, 3],
                        2 => vec![0, 1, 2],
                        _ => vec![2, 3, 4, 5, 6],
                    };
                    Input {
                        player_id: who,
                        kind: InputKind::ChooseCards { selected },
                    }
                }
                _ => Input {
                    player_id: who,
                    kind: InputKind::Choice { idx: 0 },
                },
            }
        })),
        Some(tracker.sender()),
        None,
    )
    .expect("combo laydown must complete");

    let hand = hand_location(&gd, "P1");
    let set_table = gd.locations.iter().find(|l| l.name == "SetTable").unwrap();
    let run_table = gd.locations.iter().find(|l| l.name == "RunTable").unwrap();

    // Set: exactly the three Aces — the invalid 2-Ace+Two choice was rejected.
    assert_eq!(set_table.cards.len(), 3, "validated laydown of three Aces");
    for &id in &set_table.cards {
        assert_eq!(
            gd.cards[id].get("Rank").map(|r| r.as_str()),
            Some("Ace"),
            "only Aces on SetTable"
        );
    }

    // Run: the chosen Three..Seven.
    assert_eq!(run_table.cards.len(), 5, "validated laydown of the run");
    assert_eq!(hand.cards.len(), 2, "the pair of Twos stays in hand");
    assert_eq!(total_cards(&gd), 10);
    assert_eq!(
        *asks.lock().unwrap(),
        3,
        "one re-prompt + two valid selections"
    );
}

#[test]
fn combo_book_lays_down_four_of_a_kind() {
    let ir = load_game("behavior_combo_book.cgdsl");
    let tracker = Tracker::new();
    let who_for_closure = tracker.0.clone();
    let gd = run_game(
        ir,
        GameData::new(),
        InputSource::Player(Box::new(move |it: InputType| {
            let who = who_for_closure
                .lock()
                .unwrap()
                .clone()
                .unwrap_or_else(|| "P1".into());
            match it {
                InputType::ChooseCards { .. } => Input {
                    player_id: who,
                    kind: InputKind::ChooseCards {
                        selected: vec![0, 1, 2, 3],
                    }, // the Aces
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
    .expect("combo book must complete");

    let books = gd.locations.iter().find(|l| l.name == "Books").unwrap();
    let hand = hand_location(&gd, "P1");
    assert_eq!(books.cards.len(), 4, "the four Aces form a book");
    for &id in &books.cards {
        assert_eq!(
            gd.cards[id].get("Rank").map(|r| r.as_str()),
            Some("Ace"),
            "only Aces on the book pile"
        );
    }
    assert_eq!(hand.cards.len(), 6, "Kings and Threes stay in hand");
    assert_eq!(total_cards(&gd), 10);
}

#[test]
fn combo_until_stage_loops_until_hand_cleared() {
    // Proposal B: `until Set in Hand empty` drives repeated laydown prompts
    // until no combo-matching cards remain.
    let ir = load_game("behavior_combo_until.cgdsl");
    let tracker = Tracker::new();
    let who_for_closure = tracker.0.clone();
    let gd = run_game(
        ir,
        GameData::new(),
        InputSource::Player(Box::new(move |it: InputType| {
            let who = who_for_closure
                .lock()
                .unwrap()
                .clone()
                .unwrap_or_else(|| "P1".into());
            match it {
                InputType::ChooseCards { .. } => Input {
                    player_id: who,
                    kind: InputKind::ChooseCards {
                        selected: vec![0, 1, 2],
                    }, // the Aces
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
    .expect("combo-until stage must complete");

    let table = gd.locations.iter().find(|l| l.name == "Table").unwrap();
    let hand = hand_location(&gd, "P1");
    assert_eq!(table.cards.len(), 3, "the three Aces were laid down");
    assert_eq!(
        hand.cards.len(),
        2,
        "the pair of Twos no longer matches the combo"
    );
    assert_eq!(total_cards(&gd), 5);
}

// ---------------------------------------------------------------------------
// Crazy Eights — empty-hand win
// ---------------------------------------------------------------------------

#[test]
fn crazy_eights_empty_hand_wins() {
    let ir = load_game("behavior_crazy_eights.cgdsl");
    let tracker = Tracker::new();
    let who_for_closure = tracker.0.clone();
    let gd = run_game(
        ir,
        GameData::new(),
        InputSource::Player(Box::new(move |it: InputType| {
            let who = who_for_closure
                .lock()
                .unwrap()
                .clone()
                .unwrap_or_else(|| "P1".into());
            match it {
                InputType::Optional(prompt) => {
                    let is_play = prompt.contains("deal any");
                    // P1 plays every turn; P2/P3 always decline.
                    let accept = is_play && who == "P1";
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
    .expect("crazy eights must complete");

    // P1 sheds all 5 cards; P2/P3 keep theirs. Lowest hand wins.
    assert_eq!(players_in_game(&gd), vec!["P1"], "empty hand wins");
    assert_eq!(gd.players[0].score, 0);
    assert_eq!(gd.players[1].score, 5);
    assert_eq!(gd.players[2].score, 5);
    let discard = gd.locations.iter().find(|l| l.name == "Discard").unwrap();
    assert_eq!(discard.cards.len(), 6, "starter + 5 plays");
    let deck = gd.locations.iter().find(|l| l.name == "Deck").unwrap();
    assert_eq!(deck.cards.len(), 4, "undrawn cards stay in the deck");
    assert_eq!(total_cards(&gd), 20);
}
