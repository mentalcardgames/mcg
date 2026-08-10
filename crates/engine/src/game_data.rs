/*
The purpose of game_data.rs is to define the structs, enums and traits that represent the game data.
This includes:
 - Players
 - Teams
 - Turn Order
 - Locations
 - Cards
 - (Tokens)
 - Combos
 - (Memories)
 - Precedences
 - Point Maps
 - current player
 - stage counters
*/

use front_end::ast::{FilterExpr, MemoryType};
use std::collections::HashMap;

// while we don't need any auxiliary functions on Cards, we can just use a type rather than a struct.
pub type Card = HashMap<String, String>;

/// Per-card visibility state, stored parallel to `GameData::cards` (indexed by
/// card id). Reserved for the card-encryption work: flipping a card is
/// (de)encrypting its face, so `FlipAction` will map onto this slot once
/// cryptography lands. Currently every card starts `FaceUp` and nothing reads
/// or writes the field (see `action.rs` `FlipAction`).
#[derive(Clone, Debug, PartialEq, Copy)]
pub enum CardStatus {
    FaceUp,
    FaceDown,
    Private,
}

#[derive(Clone)]
pub struct GameData {
    pub table: OwnerData,
    pub players: Vec<Player>,
    pub teams: Vec<Team>,
    pub turn_order: Vec<usize>,
    pub locations: Vec<Location>,
    pub cards: Vec<Card>,
    pub card_statuses: Vec<CardStatus>,
    pub combos: Vec<Combo>,
    pub precedences: Vec<Precedence>,
    pub point_maps: Vec<PointMap>,
    pub current_player: Option<usize>,
    pub stage_counters: HashMap<String, u32>,
    pub stage_stack: Vec<String>,
    pub memories: HashMap<String, MemoryValue>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum MemoryValue {
    Int(i32),
    String(String),
    CardSet(Vec<usize>),
    PlayerCollection(Vec<usize>),
    Team(String),
    TeamCollection(Vec<String>),
    IntCollection(Vec<i32>),
    StringCollection(Vec<String>),
    LocationCollection(Vec<usize>),
}

#[derive(Clone)]
pub struct OwnerData {
    pub locations: Vec<usize>,
    // later: memories
}

#[derive(Clone)]
pub struct Player {
    pub name: String,
    pub score: i32,
    pub owner: OwnerData,
    pub in_game: bool,
    pub in_stage: HashMap<String, bool>,
}

#[derive(Clone)]
pub struct Location {
    pub name: String,
    pub cards: Vec<usize>,
}

#[derive(Clone)]
pub struct Team {
    pub name: String,
    pub players: Vec<usize>,
}

#[derive(Clone)]
pub struct Combo {
    pub name: String,
    pub filter: FilterExpr,
}

// Precedences are an ordered list of values on a single key, defining a strict ordering low -> high
#[derive(Clone)]
pub struct Precedence {
    pub name: String,
    pub key: String,
    pub values: Vec<String>,
}

// point maps maps a card to an integer value. These cannot be references in the card list, since point maps can technically map points of cards that aren't in play.
#[derive(Clone)]
pub struct PointMap {
    pub name: String,
    pub map: HashMap<String, i32>,
}

impl GameData {
    // No `Default` impl: `new()` seeds `current_player = Some(0)` as a
    // sentinel (I-2), which a derived `Default` would not.
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        GameData {
            table: OwnerData { locations: vec![] },
            players: vec![],
            teams: vec![],
            turn_order: vec![],
            locations: vec![],
            cards: vec![],
            card_statuses: vec![],
            combos: vec![],
            precedences: vec![],
            point_maps: vec![],
            current_player: Some(0),
            stage_counters: HashMap::new(),
            stage_stack: vec![],
            memories: HashMap::new(),
        }
    }

    // setup stuff

    pub fn add_location(&mut self, owner_name: String, location: Location) -> usize {
        self.locations.push(location);
        let location_id = self.locations.len() - 1;

        // find owner and push location
        if owner_name == "Table" {
            self.table.locations.push(location_id);
        } else {
            let player_id = self
                .players
                .iter()
                .position(|p| p.name == owner_name)
                .unwrap_or_else(|| {
                    panic!("add_location: owner {} not found in players", owner_name)
                });
            self.players[player_id].owner.locations.push(location_id);
        }

        location_id
    }

    pub fn add_player(&mut self, name: String) -> usize {
        let player = Player {
            name,
            score: 0,
            owner: OwnerData { locations: vec![] },
            in_game: true,
            in_stage: HashMap::new(),
        };
        self.players.push(player);
        self.players.len() - 1
    }

    // card stuff
    pub fn add_card(&mut self, _location_id: usize, card: Card) -> usize {
        self.cards.push(card);
        self.card_statuses.push(CardStatus::FaceUp);
        self.cards.len() - 1
    }

    pub fn get_card(&self, card_id: usize) -> Option<&Card> {
        self.cards.get(card_id)
    }

    /// Current visibility state of a card (unused by the engine until card
    /// encryption lands; see [`CardStatus`]).
    pub fn card_status(&self, card_id: usize) -> Option<CardStatus> {
        self.card_statuses.get(card_id).copied()
    }

    /// Reserve a card's status slot for future `FlipAction`/encryption use.
    pub fn set_card_status(&mut self, card_id: usize, status: CardStatus) {
        if let Some(slot) = self.card_statuses.get_mut(card_id) {
            *slot = status;
        }
    }

    /// Returns the index of the (first) location whose `cards` vec contains
    /// `card_id`, or `None` if no location holds it. Linear scan; see invariant
    /// I-6. Callers that need a `0` fallback sentinel (e.g. invariant I-14)
    /// apply `.unwrap_or(0)` at the call site.
    pub fn find_location_of_card(&self, card_id: usize) -> Option<usize> {
        self.locations
            .iter()
            .position(|l| l.cards.contains(&card_id))
    }

    // stage counter stuff
    pub fn increment_stage_counter(&mut self, stage: String) {
        let counter = self.stage_counters.entry(stage).or_insert(0);
        *counter += 1;
    }

    pub fn reset_stage_counter(&mut self, stage: String) {
        self.stage_counters.insert(stage, 0);
    }

    pub fn get_stage_counter(&self, stage: String) -> u32 {
        *self.stage_counters.get(&stage).unwrap_or(&0)
    }

    // turn order stuff
    pub fn get_current_player(&self) -> Option<&Player> {
        self.current_player.and_then(|idx| {
            let player_idx = *self.turn_order.get(idx)?;
            self.players.get(player_idx)
        })
    }

    pub fn next_player(&mut self) {
        // resolve the next player
        if let Some(next_player) = self.resolve_turn() {
            self.current_player = Some(
                self.turn_order
                    .iter()
                    .position(|&idx| idx == next_player)
                    .unwrap_or_else(|| {
                        panic!(
                            "next_player: next_player {} not found in turn_order {:?}",
                            next_player, self.turn_order
                        )
                    }),
            );
        } else {
            self.current_player = None;
        }
    }

    // stage and game flags
    pub fn set_player_out(&mut self, player_id: usize) {
        if let Some(player) = self.players.get_mut(player_id) {
            player.in_game = false;
        }
    }

    pub fn set_player_stage_flag(&mut self, player_id: usize, stage: String, flag: bool) {
        if let Some(player) = self.players.get_mut(player_id) {
            player.in_stage.insert(stage, flag);
        }
    }

    pub fn get_current_stage(&self) -> Option<String> {
        self.stage_stack.last().cloned()
    }

    /// The winner set at game end: every player still `in_game`, in
    /// declaration order. Both explicit winner declarations (`winner is X`,
    /// `end game with winner X` — which eliminate everyone else) and
    /// elimination-only endings reduce to this rule: the survivors ARE the
    /// winners. An empty result means nobody won (2026-08-10).
    pub fn winner_names(&self) -> Vec<String> {
        self.players
            .iter()
            .filter(|p| p.in_game)
            .map(|p| p.name.clone())
            .collect()
    }

    pub fn enter_stage(&mut self, stage: String, players_in: Vec<String>) {
        self.stage_stack.push(stage.clone());

        // for each player in players_in, set their stage flag to true. For each player not in players_in, set their stage flag to false.
        for player in self.players.iter_mut() {
            player
                .in_stage
                .insert(stage.clone(), players_in.contains(&player.name));
        }
    }

    pub fn ensure_stage_entered(&mut self, stage: &str) {
        if self.stage_stack.iter().any(|s| s == stage) {
            return;
        }
        let players_in: Vec<String> = self.players.iter().map(|p| p.name.clone()).collect();
        self.enter_stage(stage.to_string(), players_in);
    }

    pub fn leave_stage(&mut self, stage: String) {
        // pop until we pop the stage we're leaving - this allows for stages to be left out of order in the case of end conditions that jump multiple stages at once.
        while let Some(current_stage) = self.stage_stack.pop() {
            if current_stage == stage {
                break;
            }
        }
    }

    /// The next *eligible* player index after `from` in `turn_order`,
    /// wrapping. Eligible = `in_game && in_stage[stage]`. If no *other*
    /// player is eligible but `from` itself still is, `from` is returned
    /// (the turn wraps onto the same player instead of erroring — I-13,
    /// relaxed 2026-08-10 so elimination games no longer need a
    /// `size(playersin) >= 2` guard on `cycle to next`).
    pub(crate) fn next_eligible_player(&self, from: usize, stage: &str) -> Option<usize> {
        let turn_len = self.turn_order.len();
        if turn_len == 0 {
            return None;
        }
        for i in 1..turn_len {
            let player_idx = self.turn_order[(from + i) % turn_len];
            if let Some(player) = self.players.get(player_idx) {
                if player.in_game && *player.in_stage.get(stage).unwrap_or(&false) {
                    return Some(player_idx);
                }
            }
        }
        self.eligible_self(from, stage)
    }

    /// The previous *eligible* player index before `from` in `turn_order`,
    /// wrapping (mirror of [`Self::next_eligible_player`]; D-12, fixed
    /// 2026-08-10 — `previous` now skips ineligible players like `next`
    /// does, and wraps onto `from` itself when it is the only eligible one).
    pub(crate) fn previous_eligible_player(&self, from: usize, stage: &str) -> Option<usize> {
        let turn_len = self.turn_order.len();
        if turn_len == 0 {
            return None;
        }
        for i in 1..turn_len {
            let player_idx = self.turn_order[(from + turn_len - i) % turn_len];
            if let Some(player) = self.players.get(player_idx) {
                if player.in_game && *player.in_stage.get(stage).unwrap_or(&false) {
                    return Some(player_idx);
                }
            }
        }
        self.eligible_self(from, stage)
    }

    fn eligible_self(&self, from: usize, stage: &str) -> Option<usize> {
        let self_idx = *self.turn_order.get(from)?;
        let player = self.players.get(self_idx)?;
        if player.in_game && *player.in_stage.get(stage).unwrap_or(&false) {
            Some(self_idx)
        } else {
            None
        }
    }

    pub fn resolve_turn(&mut self) -> Option<usize> {
        let current_idx = self.current_player?;
        let current_stage = self.get_current_stage()?;
        self.next_eligible_player(current_idx, &current_stage)
    }

    /// Adds a memory slot under `key`. `initial` is used verbatim when
    /// provided (setup-time evaluation happens in `action.rs`); otherwise a
    /// per-type default is inserted. Since 2026-08-10 the defaults for
    /// `MemoryType::Player` and `MemoryType::TeamCollection` are *typed*
    /// (I-10): a Player memory initialises to the owner's own name (as a
    /// `String`, matching `SetMemory`'s storage convention) instead of
    /// `Int(0)`, and a TeamCollection to an empty team list.
    pub fn add_memory(
        &mut self,
        key: String,
        owner_name: &str,
        memory_type: Option<MemoryType>,
        initial: Option<MemoryValue>,
    ) {
        let value = match initial {
            Some(v) => v,
            None => default_memory_value(memory_type, owner_name),
        };
        self.memories.insert(key, value);
    }

    /// Resolves the owner prefix a bare memory write/read (`M is 5`,
    /// `reset M`, `&I:M` without `of <owner>`) should target. Since
    /// 2026-08-10: if exactly one existing slot ends in `_{memory}`
    /// (i.e. the memory was declared at setup, e.g. `memory pot on table`
    /// → `Table_pot`), that owner wins — otherwise the current player's
    /// name is returned as the bridge fallback (D-14).
    pub fn memory_write_owner(&self, memory: &str, current_player: Option<&str>) -> Option<String> {
        if memory == crate::quantifier::SYNTH_MEMORY_KEY {
            return current_player.map(|p| p.to_string());
        }
        let suffix = format!("_{}", memory);
        let mut owners: Vec<&str> = self
            .memories
            .keys()
            .filter_map(|k| k.strip_suffix(&suffix))
            .filter(|prefix| !prefix.is_empty())
            .collect();
        owners.sort_unstable();
        owners.dedup();
        match owners.as_slice() {
            [single] => Some((*single).to_string()),
            _ => current_player.map(|p| p.to_string()),
        }
    }

    pub fn get_memory(&self, key: &str) -> Option<&MemoryValue> {
        self.memories.get(key)
    }

    /// Stores `value` at `key`, overwriting any prior value. This is the
    /// write-side primitive used by `ActionRule::SetMemory` after the
    /// `MemoryType` expression has been evaluated by `action.rs`. See
    /// invariant I-9.
    pub fn set_memory(&mut self, key: String, value: MemoryValue) {
        self.memories.insert(key, value);
    }

    /// Resets the memory at `key` to its per-type zero value. Since
    /// 2026-08-10 this works for every variant (previously only `Int`
    /// memories were reset; everything else silently no-oped).
    pub fn reset_memory(&mut self, key: &str) {
        let zero = match self.memories.get(key) {
            Some(MemoryValue::Int(_)) => MemoryValue::Int(0),
            Some(MemoryValue::String(_)) => MemoryValue::String(String::new()),
            Some(MemoryValue::CardSet(_)) => MemoryValue::CardSet(vec![]),
            Some(MemoryValue::PlayerCollection(_)) => MemoryValue::PlayerCollection(vec![]),
            Some(MemoryValue::Team(_)) => MemoryValue::Team(String::new()),
            Some(MemoryValue::TeamCollection(_)) => MemoryValue::TeamCollection(vec![]),
            Some(MemoryValue::IntCollection(_)) => MemoryValue::IntCollection(vec![]),
            Some(MemoryValue::StringCollection(_)) => MemoryValue::StringCollection(vec![]),
            Some(MemoryValue::LocationCollection(_)) => MemoryValue::LocationCollection(vec![]),
            None => return,
        };
        self.memories.insert(key.to_string(), zero);
    }
}

/// Per-type default `MemoryValue` for a fresh memory slot.
fn default_memory_value(memory_type: Option<MemoryType>, owner_name: &str) -> MemoryValue {
    match memory_type {
        Some(MemoryType::Int { .. }) => MemoryValue::Int(0),
        Some(MemoryType::String { .. }) => MemoryValue::String(String::new()),
        // I-10 fix: Player memories are stored as player *names* (String);
        // a player-owned slot initialises to its own owner.
        Some(MemoryType::Player { .. }) => MemoryValue::String(owner_name.to_string()),
        Some(MemoryType::PlayerCollection { .. }) => MemoryValue::PlayerCollection(vec![]),
        Some(MemoryType::CardSet { .. }) => MemoryValue::CardSet(vec![]),
        Some(MemoryType::Team { .. }) => MemoryValue::Team(String::new()),
        Some(MemoryType::TeamCollection { .. }) => MemoryValue::TeamCollection(vec![]),
        Some(MemoryType::IntCollection { .. }) => MemoryValue::IntCollection(vec![]),
        Some(MemoryType::StringCollection { .. }) => MemoryValue::StringCollection(vec![]),
        Some(MemoryType::LocationCollection { .. }) => MemoryValue::LocationCollection(vec![]),
        None => MemoryValue::Int(0),
    }
}

#[cfg(test)]
#[path = "game_data_tests.rs"]
mod tests;
