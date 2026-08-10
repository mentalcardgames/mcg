//! Random-input ("monkey") tests: drive the demo games with fully random
//! player decisions across many seeds.
//!
//! The property under test: a well-formed game must **never crash or hang,
//! no matter what the player does**. Every run must either complete (with all
//! 52 cards conserved) — the demo games are designed to terminate for any
//! input sequence — or fail in a way the test reports.
//!
//! Two failure modes are guarded explicitly:
//! - **panics** propagate out of `run_game` and fail the test (the engine
//!   only catches panics for trace logging);
//! - **infinite re-prompt loops** (I-15) are caught by an input-call cap — a
//!   buggy validation path would call the input closure forever.
//!
//! Note on reproducibility: the engine shuffles with `rand::thread_rng()`
//! (not injectable), so two runs with the same seed still differ in the
//! shuffle. Per-run seeds are printed in failure messages so the *input
//! sequence* is reproducible; a seeded engine RNG would be needed for full
//! replay determinism (see NEXT_STEPS.md).

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use cgdsl_engine::{run_game, EngineError, GameData, Input, InputKind, InputSource, InputType};
use front_end::ir::{Ir, LoweredPayLoad};
use front_end::validation::parse_document;
use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use rand::{Rng, SeedableRng};

/// Number of random games played per demo game, per test invocation.
const RUNS_PER_GAME: usize = 40;
/// Cap on input-closure invocations before the run is declared an
/// infinite re-prompt loop. Legitimate runs stay far below this.
const INPUT_CALL_CAP: usize = 2000;
/// Probability (0..1) that an answer is deliberately out of range, to
/// exercise the controller's re-prompt validation loop (I-15).
const INVALID_ANSWER_CHANCE: f64 = 0.1;

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

/// A seeded random player. Answers every prompt with a uniformly random
/// *valid* answer, except with `INVALID_ANSWER_CHANCE` probability, where the
/// answer is deliberately out of range (the controller rejects it and
/// re-prompts — exercising the I-15 validation loop).
///
/// The current player is tracked via an `event_sender` so every answer
/// carries the right `player_id` (I-23).
struct RandomPlayer {
    rng: Arc<Mutex<StdRng>>,
    current: Arc<Mutex<Option<String>>>,
    calls: Arc<Mutex<usize>>,
}

impl RandomPlayer {
    fn new(seed: u64) -> Self {
        Self {
            rng: Arc::new(Mutex::new(StdRng::seed_from_u64(seed))),
            current: Arc::new(Mutex::new(None)),
            calls: Arc::new(Mutex::new(0)),
        }
    }

    fn event_sender(&self) -> Box<dyn Fn(&GameData) + Send> {
        let current = self.current.clone();
        Box::new(move |gd: &GameData| {
            *current.lock().unwrap() = gd.get_current_player().map(|p| p.name.clone());
        })
    }

    fn input_source(&self) -> InputSource {
        let rng = self.rng.clone();
        let current = self.current.clone();
        let calls = self.calls.clone();
        InputSource::Player(Box::new(move |it: InputType| {
            let mut n = calls.lock().unwrap();
            *n += 1;
            if *n > INPUT_CALL_CAP {
                panic!("input call cap exceeded (possible infinite re-prompt loop)");
            }
            drop(n);

            let who = current
                .lock()
                .unwrap()
                .clone()
                .unwrap_or_else(|| "P1".to_string());
            let mut r = rng.lock().unwrap();
            let invalid: bool = r.gen_bool(INVALID_ANSWER_CHANCE);

            match it {
                InputType::Choice { max_index, .. } => {
                    let idx = if invalid {
                        max_index + 1 + r.gen_range(0..3)
                    } else {
                        r.gen_range(0..=max_index)
                    };
                    Input {
                        player_id: who,
                        kind: InputKind::Choice { idx },
                    }
                }
                InputType::Optional { .. } => Input {
                    player_id: who,
                    kind: if r.gen_bool(0.5) {
                        InputKind::OptionalAccept
                    } else {
                        InputKind::OptionalDecline
                    },
                },
                InputType::ChoosePlayer { candidates, .. } => {
                    let idx = if invalid {
                        candidates.len() + r.gen_range(0..3)
                    } else if candidates.is_empty() {
                        0
                    } else {
                        r.gen_range(0..candidates.len())
                    };
                    Input {
                        player_id: who,
                        kind: InputKind::ChoosePlayer { idx },
                    }
                }
                InputType::ChooseCards {
                    display, min, max, ..
                } => {
                    if invalid {
                        Input {
                            player_id: who,
                            kind: InputKind::ChooseCards {
                                selected: vec![display.len() + 5],
                            },
                        }
                    } else if r.gen_bool(0.5) {
                        // Half the time submit nothing: valid as a "skip" for
                        // combo prompts (min = 0); rejected + re-prompted for
                        // prompts with min >= 1 (the next roll may succeed).
                        Input {
                            player_id: who,
                            kind: InputKind::ChooseCards { selected: vec![] },
                        }
                    } else {
                        let count = if max > min {
                            r.gen_range(min..=max)
                        } else {
                            min
                        };
                        let mut idxs: Vec<usize> = (0..display.len()).collect();
                        idxs.shuffle(&mut *r);
                        Input {
                            player_id: who,
                            kind: InputKind::ChooseCards {
                                selected: idxs.into_iter().take(count).collect(),
                            },
                        }
                    }
                }
            }
        }))
    }
}

/// Play one full game with random inputs derived from `seed`. Returns the
/// terminal state, or the engine's recoverable error.
fn play_random_game(name: &str, seed: u64) -> Result<GameData, EngineError> {
    let ir = load_game(name);
    let player = RandomPlayer::new(seed);
    run_game(
        ir,
        GameData::new(),
        player.input_source(),
        Some(player.event_sender()),
        None,
    )
}

/// The core property: for `RUNS_PER_GAME` seeds, the game completes with no
/// panic and all 52 cards conserved. Failure messages include the seed.
fn assert_random_plays_complete(name: &str) {
    let base: u64 = rand::random();
    for i in 0..RUNS_PER_GAME {
        let seed = base.wrapping_add((i as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15));
        match play_random_game(name, seed) {
            Ok(gd) => {
                assert_eq!(
                    total_cards(&gd),
                    52,
                    "seed {seed:#x}: card conservation violated"
                );
            }
            Err(e) => panic!("seed {seed:#x}: game returned a recoverable error: {e}"),
        }
    }
}

#[test]
fn random_war_completes() {
    // War takes no input; this exercises the automatic game across 40
    // random shuffles (ties, deck splits, winner paths).
    assert_random_plays_complete("war.cgdsl");
}

#[test]
fn random_blackjack_completes() {
    // Random hit/stand: busts, all-bust games, dealer draws, guarded cycles.
    assert_random_plays_complete("blackjack.cgdsl");
}

#[test]
fn random_crazy_eights_completes() {
    // Random plays, random card picks, random draw recipients, random
    // accept/decline — deck drains, 30-turn cap terminates.
    assert_random_plays_complete("crazy_eights.cgdsl");
}

#[test]
fn random_five_card_draw_completes() {
    // Random discards (any subset) and draw decisions.
    assert_random_plays_complete("five_card_draw.cgdsl");
}

#[test]
fn random_go_fish_completes() {
    // Random rank asks across all 13 options.
    assert_random_plays_complete("go_fish.cgdsl");
}
