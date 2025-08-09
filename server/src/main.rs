use std::{collections::VecDeque, net::SocketAddr, sync::Arc};

use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    extract::State,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use futures::StreamExt;
use rand::seq::SliceRandom;
use tokio::sync::RwLock;

use mcg_shared::{
    ActionEvent, ClientMsg, GameStatePublic, PlayerAction, PlayerPublic, ServerMsg, Stage,
};

#[derive(Clone, Debug)]
struct Player {
    id: usize,
    name: String,
    stack: u32,
    cards: [u8; 2],
    has_folded: bool,
}

#[derive(Clone, Debug)]
struct Game {
    players: Vec<Player>,
    deck: VecDeque<u8>,
    community: Vec<u8>,
    pot: u32,
    to_act: usize,
    stage: Stage,
    sb: u32,
    bb: u32,
    pending_to_act: Vec<usize>,
    recent_actions: Vec<ActionEvent>,
    winner_ids: Vec<usize>,
}

impl Game {
    fn new(human_name: String, bot_count: usize) -> Self {
        let mut deck: Vec<u8> = (0..52).collect();
        deck.shuffle(&mut rand::rng());
        let deck = VecDeque::from(deck);
        let mut players = Vec::with_capacity(1 + bot_count);
        players.push(Player {
            id: 0,
            name: human_name,
            stack: 1000,
            cards: [0, 0],
            has_folded: false,
        });
        for i in 0..bot_count {
            players.push(Player {
                id: i + 1,
                name: format!("Bot {}", i + 1),
                stack: 1000,
                cards: [0, 0],
                has_folded: false,
            });
        }
        let mut g = Self {
            players,
            deck,
            community: vec![],
            pot: 0,
            to_act: 0,
            stage: Stage::Preflop,
            sb: 5,
            bb: 10,
            pending_to_act: Vec::new(),
            recent_actions: Vec::new(),
            winner_ids: Vec::new(),
        };
        g.deal();
        g
    }

    fn deal(&mut self) {
        println!("[DEAL] Shuffling and dealing new hand");
        for p in &mut self.players {
            p.cards = [
                self.deck.pop_front().unwrap(),
                self.deck.pop_front().unwrap(),
            ];
            p.has_folded = false;
            println!(
                "[DEAL] {} gets {} {}",
                p.name,
                card_str(p.cards[0]),
                card_str(p.cards[1])
            );
        }
        self.community.clear();
        self.pot = self.sb + self.bb; // blinds auto-posted for simplicity
        self.to_act = 0; // hero acts first for demo simplicity
        self.stage = Stage::Preflop;
        self.recent_actions.clear();
        self.winner_ids.clear();
        self.init_round();
        println!("[BLINDS] SB {} BB {} -> pot {}", self.sb, self.bb, self.pot);
    }

    fn public_for(&self, viewer_id: usize) -> GameStatePublic {
        let players = self
            .players
            .iter()
            .map(|p| PlayerPublic {
                id: p.id,
                name: p.name.clone(),
                stack: p.stack,
                cards: if p.id == viewer_id {
                    Some(p.cards)
                } else {
                    None
                },
                has_folded: p.has_folded,
            })
            .collect();
        GameStatePublic {
            players,
            community: self.community.clone(),
            pot: self.pot,
            to_act: self.to_act,
            stage: self.stage,
            you_id: viewer_id,
            bot_count: self.players.len().saturating_sub(1),
            recent_actions: self.recent_actions.clone(),
            winner_ids: self.winner_ids.clone(),
        }
    }

    fn apply_player_action(&mut self, actor: usize, action: PlayerAction) {
        if actor != self.to_act || self.players[actor].has_folded {
            return;
        }
        println!(
            "[ACTION] {}: {:?} (stage: {:?})",
            self.players[actor].name, action, self.stage
        );
        // record action for clients
        self.recent_actions.push(ActionEvent {
            player_id: actor,
            action: action.clone(),
        });
        match action {
            PlayerAction::Fold => {
                self.players[actor].has_folded = true;
                println!("[STATE] {} folds.", self.players[actor].name);
            }
            PlayerAction::CheckCall => {
                println!("[ACTION] {} chooses Check/Call", self.players[actor].name);
            }
            PlayerAction::Bet(amount) => {
                let a = amount.min(self.players[actor].stack);
                self.players[actor].stack -= a;
                self.pot += a;
                println!(
                    "[BET] {} bets {} -> pot {}",
                    self.players[actor].name, a, self.pot
                );
            }
        }
        // remove actor from pending list for this round
        if let Some(pos) = self.pending_to_act.iter().position(|&i| i == actor) {
            self.pending_to_act.remove(pos);
        }
        // check if only one player remains -> showdown
        let active_left: Vec<usize> = self
            .players
            .iter()
            .enumerate()
            .filter_map(|(i, p)| (!p.has_folded).then_some(i))
            .collect();
        if active_left.len() <= 1 {
            self.stage = Stage::Showdown;
            self.determine_winners();
            return;
        }
        // choose next actor or advance stage if round complete
        if let Some(&next) = self.pending_to_act.first() {
            self.to_act = next;
        } else {
            self.advance_stage();
            if self.stage == Stage::Showdown {
                self.determine_winners();
            } else {
                self.init_round();
            }
        }
    }

    fn random_bot_action(&self, bot_index: usize) -> PlayerAction {
        use rand::Rng;
        let mut rng = rand::rng();
        let r: u8 = rng.random_range(0..100);
        if r < 70 {
            PlayerAction::CheckCall
        } else {
            let a = self.bb.min(self.players[bot_index].stack);
            if a == 0 {
                PlayerAction::CheckCall
            } else {
                PlayerAction::Bet(a)
            }
        }
    }
    fn play_out_bots(&mut self) {
        // Let all bots act in order until it is the human's turn again or showdown
        while self.stage != Stage::Showdown && self.to_act != 0 {
            let actor = self.to_act;
            // Safety check
            if actor == 0 || actor >= self.players.len() || self.players[actor].has_folded {
                break;
            }
            let action = self.random_bot_action(actor);
            println!("[BOT] {}: {:?}", self.players[actor].name, action);
            self.apply_player_action(actor, action);
        }
    }
    fn init_round(&mut self) {
        self.pending_to_act.clear();
        for i in 0..self.players.len() {
            if !self.players[i].has_folded {
                self.pending_to_act.push(i);
            }
        }
        if let Some(&first) = self.pending_to_act.first() {
            self.to_act = first;
        }
    }
    fn determine_winners(&mut self) {
        // Simplified: all players who haven't folded share the pot
        self.winner_ids = self
            .players
            .iter()
            .enumerate()
            .filter_map(|(i, p)| (!p.has_folded).then_some(i))
            .collect();
        println!(
            "[SHOWDOWN] Winners: {:?} (pot: {})",
            self.winner_ids, self.pot
        );
    }

    fn advance_stage(&mut self) {
        match self.stage {
            Stage::Preflop => {
                self.community.push(self.deck.pop_front().unwrap());
                self.community.push(self.deck.pop_front().unwrap());
                self.community.push(self.deck.pop_front().unwrap());
                self.stage = Stage::Flop;
                println!(
                    "[STAGE] Flop: {} {} {}",
                    card_str(self.community[0]),
                    card_str(self.community[1]),
                    card_str(self.community[2])
                );
            }
            Stage::Flop => {
                self.community.push(self.deck.pop_front().unwrap());
                self.stage = Stage::Turn;
                println!("[STAGE] Turn: {}", card_str(self.community[3]));
            }
            Stage::Turn => {
                self.community.push(self.deck.pop_front().unwrap());
                self.stage = Stage::River;
                println!("[STAGE] River: {}", card_str(self.community[4]));
            }
            Stage::River => {
                self.stage = Stage::Showdown;
                println!("[STAGE] Showdown. Pot: {}", self.pot);
            }
            Stage::Showdown => {}
        }
        self.to_act = 0;
    }
}

#[derive(Clone, Default)]
struct Lobby {
    game: Option<Game>,
}

#[derive(Clone, Default)]
struct AppState {
    lobby: Arc<RwLock<Lobby>>,
    bot_count: usize,
}

#[tokio::main]
async fn main() {
    // Parse simple CLI argument: --bots <N>
    let mut bots: usize = 1;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        if arg == "--bots" {
            if let Some(n) = args.next() {
                if let Ok(v) = n.parse::<usize>() {
                    bots = v;
                }
            }
        }
    }
    let mut state = AppState::default();
    state.bot_count = bots;
    let app = Router::new()
        .route(
            "/health",
            get(|| async { Json(serde_json::json!({"ok": true})) }),
        )
        .route("/ws", get(ws_handler))
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("[START] Server running at http://{addr}");
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn ws_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: AppState) {
    let name = match socket.next().await {
        Some(Ok(Message::Text(t))) => match serde_json::from_str::<ClientMsg>(&t) {
            Ok(ClientMsg::Join { name }) => name,
            _ => {
                let _ = socket
                    .send(Message::Text(
                        serde_json::to_string(&ServerMsg::Error("Expected Join".into())).unwrap(),
                    ))
                    .await;
                return;
            }
        },
        _ => return,
    };
    println!("[CONNECT] New player: {}", name);

    {
        let mut lobby = state.lobby.write().await;
        if lobby.game.is_none() {
            lobby.game = Some(Game::new(name.clone(), state.bot_count));
            println!(
                "[GAME] Created new game for {} with {} bot(s)",
                name, state.bot_count
            );
            // Kick off initial bot actions so the client sees immediate activity
            if let Some(game) = &mut lobby.game {
                if game.players.len() > 1 {
                    game.to_act = 1;
                    game.play_out_bots();
                }
            }
        }
    }

    let you_id = 0usize;
    let _ = socket
        .send(Message::Text(
            serde_json::to_string(&ServerMsg::Welcome { you: you_id }).unwrap(),
        ))
        .await;

    if let Some(gs) = current_state_public(&state, you_id).await {
        let _ = socket
            .send(Message::Text(
                serde_json::to_string(&ServerMsg::State(gs)).unwrap(),
            ))
            .await;
    }

    while let Some(msg) = socket.next().await {
        match msg {
            Ok(Message::Text(txt)) => {
                if let Ok(cm) = serde_json::from_str::<ClientMsg>(&txt) {
                    match cm {
                        ClientMsg::Action(a) => {
                            println!("[WS] Action from {}: {:?}", name, a);
                            {
                                let mut lobby = state.lobby.write().await;
                                if let Some(game) = &mut lobby.game {
                                    game.apply_player_action(0, a.clone());
                                    // Let all bots act until it's the human's turn or showdown
                                    game.play_out_bots();
                                }
                            }
                            if let Some(gs) = current_state_public(&state, you_id).await {
                                let _ = socket
                                    .send(Message::Text(
                                        serde_json::to_string(&ServerMsg::State(gs)).unwrap(),
                                    ))
                                    .await;
                            }
                        }
                        ClientMsg::RequestState => {
                            println!("[WS] State requested by {}", name);
                            {
                                // Let bots play until it's the human's turn (or showdown)
                                let mut lobby = state.lobby.write().await;
                                if let Some(game) = &mut lobby.game {
                                    game.play_out_bots();
                                }
                            }
                            if let Some(gs) = current_state_public(&state, you_id).await {
                                let _ = socket
                                    .send(Message::Text(
                                        serde_json::to_string(&ServerMsg::State(gs)).unwrap(),
                                    ))
                                    .await;
                            }
                        }
                        ClientMsg::Join { .. } => {}
                    }
                }
            }
            Ok(Message::Close(_)) | Err(_) => break,
            _ => {}
        }
    }
    println!("[DISCONNECT] {} disconnected", name);
}

async fn current_state_public(state: &AppState, you_id: usize) -> Option<GameStatePublic> {
    let lobby = state.lobby.read().await;
    lobby.game.as_ref().map(|g| g.public_for(you_id))
}

fn card_str(c: u8) -> String {
    let rank_idx = (c % 13) as usize;
    let suit_idx = (c / 13) as usize;
    let ranks = [
        "A", "2", "3", "4", "5", "6", "7", "8", "9", "T", "J", "Q", "K",
    ];
    let suits = ['♣', '♦', '♥', '♠'];
    format!("{}{}", ranks[rank_idx], suits[suit_idx])
}
