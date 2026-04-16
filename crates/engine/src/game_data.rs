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

use front_end::ast::FilterExpr;
use std::collections::HashMap;

// while we don't need any auxiliary functions on Cards, we can just use a type rather than a struct.
pub type Card = HashMap<String, String>;

#[derive(Clone)]
pub struct GameData {
    pub table: OwnerData,
    pub players: Vec<Player>,
    pub teams: Vec<Team>,
    /// A list of indices into self::players, representing the turn order of the game.
    pub turn_order: Vec<usize>,
    pub locations: Vec<Location>,
    pub cards: Vec<Card>,
    pub combos: Vec<Combo>,
    pub precedences: Vec<Precedence>,
    pub point_maps: Vec<PointMap>,

    /// An index into self::turn_order, representing the current player.
    pub current_player: Option<usize>,
    /// A dictionary
    pub stage_counters: HashMap<String, u32>,

    /// A stack of stage names - this is used to keep track of which stages we're currently in, for the purposes of stage-specific flags and turn resolution. The top of the stack is the current stage.
    pub stage_stack: Vec<String>,
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
    pub map: HashMap<Card, i32>,
}

impl GameData {
    pub fn new() -> Self {
        GameData {
            table: OwnerData { locations: vec![] },
            players: vec![],
            teams: vec![],
            turn_order: vec![],
            locations: vec![],
            cards: vec![],
            combos: vec![],
            precedences: vec![],
            point_maps: vec![],
            current_player: Some(0),
            stage_counters: HashMap::new(),
            stage_stack: vec![],
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
                .expect("Owner not found");
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
        self.cards.len() - 1
    }

    pub fn get_card(&self, card_id: usize) -> Option<&Card> {
        self.cards.get(card_id)
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
                    .unwrap(),
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

    pub fn enter_stage(&mut self, stage: String, players_in: Vec<String>) {
        self.stage_stack.push(stage.clone());

        // for each player in players_in, set their stage flag to true. For each player not in players_in, set their stage flag to false.
        for player in self.players.iter_mut() {
            player
                .in_stage
                .insert(stage.clone(), players_in.contains(&player.name));
        }
    }

    pub fn leave_stage(&mut self, stage: String) {
        // pop until we pop the stage we're leaving - this allows for stages to be left out of order in the case of end conditions that jump multiple stages at once.
        while let Some(current_stage) = self.stage_stack.pop() {
            if current_stage == stage {
                break;
            }
        }
    }

    pub fn resolve_turn(&mut self) -> Option<usize> {
        // find the next player in turn order who is still in both the game and the current stage
        if let Some(current_idx) = self.current_player {
            let current_stage = self.get_current_stage()?;
            for i in 1..self.turn_order.len() {
                let player_idx = self.turn_order[(current_idx + i) % self.turn_order.len()];
                if let Some(player) = self.players.get(player_idx) {
                    if player.in_game && *player.in_stage.get(&current_stage).unwrap_or(&false) {
                        return Some(player_idx);
                    }
                }
            }
        }
        None
    }
}
