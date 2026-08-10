//! Shared harness for the integration tests under `tests/`.
//!
//! Cargo compiles every top-level file in `tests/` as its own crate, so the
//! fixture loader and input helpers used by all of them live here. Files in
//! subdirectories of `tests/` are not auto-discovered as test targets — this
//! module is never run as a test itself.
//!
//! Each test crate includes a private copy of this module (`mod common;`),
//! and compiles only the items it actually uses; the `dead_code` lint would
//! fire on the unused remainder, so it is allowed at module scope.

#![allow(dead_code)]

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use cgdsl_engine::{
    GameData, Input, InputKind, InputSource, InputType, Location, TraceEntry, TraceEvent,
};
use front_end::ast::{ActionRule, GameRule};
use front_end::ir::{Ir, LoweredPayLoad};
use front_end::validation::parse_document;

/// Load and lower a `test_games/<name>` fixture.
pub fn load_game(name: &str) -> Ir<LoweredPayLoad> {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let path = manifest.join("test_games").join(name);
    let src =
        std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {}", path.display(), e));
    let game = parse_document(&src).unwrap_or_else(|e| panic!("parse {}: {}", path.display(), e));
    game.to_lowered_graph()
}

/// `InputSource::Player` that always answers `Choice { idx: 0 }` as P1.
pub fn default_input() -> InputSource {
    InputSource::Player(Box::new(|_it: InputType| Input {
        player_id: "P1".into(),
        kind: InputKind::Choice { idx: 0 },
    }))
}

/// `InputSource::TestFile` for a `test_games/<name>` replay file.
pub fn test_file(name: &str) -> InputSource {
    InputSource::TestFile(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("test_games")
            .join(name),
    )
}

/// Total cards across all locations (card-conservation helper).
pub fn total_cards(gd: &GameData) -> usize {
    gd.locations.iter().map(|l| l.cards.len()).sum()
}

/// Number of players still in the game.
pub fn players_in_game(gd: &GameData) -> usize {
    gd.players.iter().filter(|p| p.in_game).count()
}

/// The `Location` owned by `player_idx` with the given name, if present.
pub fn player_location<'a>(
    gd: &'a GameData,
    player_idx: usize,
    loc_name: &str,
) -> Option<&'a Location> {
    gd.players.get(player_idx).and_then(|p| {
        p.owner
            .locations
            .iter()
            .find_map(|&li| gd.locations.get(li).filter(|l| l.name == loc_name))
    })
}

/// The table-owned `Location` with the given name, if present.
pub fn table_location<'a>(gd: &'a GameData, loc_name: &str) -> Option<&'a Location> {
    gd.table
        .locations
        .iter()
        .find_map(|&li| gd.locations.get(li).filter(|l| l.name == loc_name))
}

/// Count `Action:Move` trace entries (one per dispatched move edge).
pub fn move_traces(trace: &[TraceEntry]) -> usize {
    trace
        .iter()
        .filter(|e| {
            matches!(
                e,
                TraceEntry::Step {
                    event: TraceEvent::Action { rule },
                    ..
                } if matches!(
                    rule,
                    GameRule::Action {
                        action: ActionRule::Move { .. }
                    }
                )
            )
        })
        .count()
}

/// Current-player tracker fed by an `event_sender`, so `InputSource::Player`
/// closures can stamp answers with the right `player_id` (I-23).
pub struct CurrentTracker(pub Arc<Mutex<Option<String>>>);

impl CurrentTracker {
    pub fn new() -> Self {
        Self(Arc::new(Mutex::new(None)))
    }

    pub fn sender(&self) -> Box<dyn Fn(&GameData) + Send> {
        let inner = self.0.clone();
        Box::new(move |gd: &GameData| {
            *inner.lock().unwrap() = gd.get_current_player().map(|p| p.name.clone());
        })
    }
}

/// A player that always accepts optionals, with a prompt counter. Answers
/// carry the current player's name (tracked via `event_sender`, I-23).
#[allow(clippy::type_complexity)]
pub fn accept_everything(
    prompts: Arc<Mutex<usize>>,
) -> (InputSource, Box<dyn Fn(&GameData) + Send>) {
    let current: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
    let sender_current = current.clone();
    let sender: Box<dyn Fn(&GameData) + Send> = Box::new(move |gd: &GameData| {
        *sender_current.lock().unwrap() = gd.get_current_player().map(|p| p.name.clone());
    });
    let source = InputSource::Player(Box::new(move |it: InputType| {
        let mut n = prompts.lock().unwrap();
        *n += 1;
        drop(n);
        let who = current
            .lock()
            .unwrap()
            .clone()
            .unwrap_or_else(|| "P1".to_string());
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
    }));
    (source, sender)
}
