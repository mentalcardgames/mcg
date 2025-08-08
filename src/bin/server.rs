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
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
enum ClientMsg {
    Join { name: String },
    Action(PlayerAction),
    RequestState,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
enum ServerMsg {
    Welcome { you: usize },
    State(GameStatePublic),
    Error(String),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
enum PlayerAction {
    Fold,
    CheckCall,
    Bet(u32),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct GameStatePublic {
    players: Vec<PlayerPublic>,
    community: Vec<u8>,
    pot: u32,
    to_act: usize,
    stage: Stage,
    you_id: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct PlayerPublic {
    id: usize,
    name: String,
    stack: u32,
    cards: Option<[u8; 2]>, // only set for the viewer
    has_folded: bool,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
enum Stage {
    Preflop,
    Flop,
    Turn,
    River,
    Showdown,
}

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
    players: Vec<Player>, // 0: human, 1: bot
    deck: VecDeque<u8>,   // 0..52
    community: Vec<u8>,
    pot: u32,
    to_act: usize,
    stage: Stage,
    sb: u32,
    bb: u32,
}

impl Game {
    fn new(human_name: String) -> Self {
        let mut deck: Vec<u8> = (0..52).collect();
    deck.shuffle(&mut rand::rng());
    let deck = VecDeque::from(deck);
        let players = vec![
            Player { id: 0, name: human_name, stack: 1000, cards: [0, 0], has_folded: false },
            Player { id: 1, name: "Bot".into(), stack: 1000, cards: [0, 0], has_folded: false },
        ];
        let mut g = Self {
            players,
            deck,
            community: vec![],
            pot: 0,
            to_act: 0,
            stage: Stage::Preflop,
            sb: 5,
            bb: 10,
        };
        g.deal();
        g
    }

    fn deal(&mut self) {
        println!("[DEAL] Shuffling and dealing new hand");
        for p in &mut self.players {
            p.cards = [self.deck.pop_front().unwrap(), self.deck.pop_front().unwrap()];
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
                cards: if p.id == viewer_id { Some(p.cards) } else { None },
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
        match action {
            PlayerAction::Fold => {
                self.players[actor].has_folded = true;
                self.stage = Stage::Showdown;
                println!("[STATE] {} folds. Moving to Showdown", self.players[actor].name);
            }
            PlayerAction::CheckCall => {
                // Simplified: just advance stage and bot mirrors
                self.advance_stage();
            }
            PlayerAction::Bet(amount) => {
                let a = amount.min(self.players[actor].stack);
                self.players[actor].stack -= a;
                self.pot += a;
                self.advance_stage();
                println!(
                    "[BET] {} bets {} -> pot {}",
                    self.players[actor].name, a, self.pot
                );
            }
        }
        // Bot acts automatically after player in this demo
        if self.stage != Stage::Showdown {
            self.bot_act();
        }
    }

    fn bot_act(&mut self) {
        // Toy bot: random choice between check/call and small bet
    use rand::Rng;
    let mut rng = rand::rng();
    let r: u8 = rng.random_range(0..100);
        if r < 70 {
            // check/call
            println!("[BOT] {} chooses Check/Call", self.players[1].name);
            self.advance_stage();
        } else {
            let a = (self.bb).min(self.players[1].stack);
            self.players[1].stack -= a;
            self.pot += a;
            self.advance_stage();
            println!("[BOT] {} bets {} -> pot {}", self.players[1].name, a, self.pot);
        }
    }

    fn advance_stage(&mut self) {
        match self.stage {
            Stage::Preflop => {
                // Burn one (ignored) + 3 community cards
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
        self.to_act = 0; // always return to player for simplicity
    }
}

#[derive(Clone, Default)]
struct Lobby {
    game: Option<Game>,
}

#[derive(Clone, Default)]
struct AppState {
    lobby: Arc<RwLock<Lobby>>,
}

#[tokio::main]
async fn main() {
    let state = AppState::default();
    let app = Router::new()
        .route("/health", get(|| async { Json(serde_json::json!({"ok": true})) }))
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
    // Expect a Join message first
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
            lobby.game = Some(Game::new(name.clone()));
            println!("[GAME] Created new game for {} vs Bot", name);
        }
    }

    let you_id = 0usize;
    let _ = socket
        .send(Message::Text(
            serde_json::to_string(&ServerMsg::Welcome { you: you_id }).unwrap(),
        ))
        .await;

    // After join, push current state
    if let Some(gs) = current_state_public(&state, you_id).await {
        let _ = socket
            .send(Message::Text(serde_json::to_string(&ServerMsg::State(gs)).unwrap()))
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
                                    game.apply_player_action(0, a);
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

    // Pretty format a 0..51 card as rank+symbol, e.g. A♣, T♦, Q♥, 9♠
    fn card_str(c: u8) -> String {
        let rank_idx = (c % 13) as usize;
        let suit_idx = (c / 13) as usize;
        let ranks = ["A", "2", "3", "4", "5", "6", "7", "8", "9", "T", "J", "Q", "K"];    
        let suits = ['♣', '♦', '♥', '♠'];
        format!("{}{}", ranks[rank_idx], suits[suit_idx])
    }
