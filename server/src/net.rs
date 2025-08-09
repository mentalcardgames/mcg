use std::{collections::VecDeque, net::SocketAddr, sync::Arc};

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use futures::StreamExt;
use rand::seq::SliceRandom;
use tokio::sync::RwLock;

use crate::eval::{card_str, evaluate_best_hand, pick_best_five};
use mcg_shared::{
    ActionEvent, ActionKind, BlindKind, ClientMsg, GameStatePublic, HandResult, LogEntry, LogEvent,
    PlayerAction, PlayerPublic, ServerMsg, Stage,
};

#[derive(Clone, Debug)]
struct Player {
    id: usize,
    name: String,
    stack: u32,
    cards: [u8; 2],
    has_folded: bool,
    all_in: bool,
}

#[derive(Clone, Debug)]
struct Game {
    // Table
    players: Vec<Player>,
    deck: VecDeque<u8>,
    community: Vec<u8>,

    // Betting state
    pot: u32,
    stage: Stage,
    dealer_idx: usize,
    to_act: usize,
    current_bet: u32,
    min_raise: u32,
    round_bets: Vec<u32>, // contributions this street, indexed by player idx

    // Blinds
    sb: u32,
    bb: u32,

    // Flow bookkeeping
    pending_to_act: Vec<usize>, // players that still need to act this street (non-folded, non-all-in)
    recent_actions: Vec<ActionEvent>,
    action_log: Vec<LogEntry>,
    winner_ids: Vec<usize>,
}

impl Game {
    fn new(human_name: String, bot_count: usize) -> Self {
        let mut deck: Vec<u8> = (0..52).collect();
        deck.shuffle(&mut rand::rng());

        let mut players = Vec::with_capacity(1 + bot_count);
        players.push(Player {
            id: 0,
            name: human_name,
            stack: 1000,
            cards: [0, 0],
            has_folded: false,
            all_in: false,
        });
        for i in 0..bot_count {
            players.push(Player {
                id: i + 1,
                name: format!("Bot {}", i + 1),
                stack: 1000,
                cards: [0, 0],
                has_folded: false,
                all_in: false,
            });
        }

        let mut g = Self {
            players,
            deck: VecDeque::from(deck),
            community: vec![],

            pot: 0,
            stage: Stage::Preflop,
            dealer_idx: 0,
            to_act: 0,
            current_bet: 0,
            min_raise: 0,
            round_bets: vec![],

            sb: 5,
            bb: 10,

            pending_to_act: Vec::new(),
            recent_actions: Vec::new(),
            action_log: Vec::new(),
            winner_ids: Vec::new(),
        };
        g.start_new_hand();
        g
    }

    fn start_new_hand(&mut self) {
        // Shuffle fresh deck
        let mut deck: Vec<u8> = (0..52).collect();
        deck.shuffle(&mut rand::rng());
        self.deck = VecDeque::from(deck);

        // Deal hole cards
        let mut dealt_logs = Vec::with_capacity(self.players.len());
        for p in &mut self.players {
            p.has_folded = false;
            p.all_in = false;
            p.cards = [
                self.deck.pop_front().unwrap(),
                self.deck.pop_front().unwrap(),
            ];
            dealt_logs.push(LogEntry {
                player_id: Some(p.id),
                event: LogEvent::DealtHole { player_id: p.id },
            });
            println!(
                "[DEAL] {} gets {} {}",
                p.name,
                card_str(p.cards[0]),
                card_str(p.cards[1])
            );
        }

        // Reset table state
        self.community.clear();
        self.pot = 0;
        self.stage = Stage::Preflop;
        self.current_bet = 0;
        self.min_raise = self.bb;
        self.round_bets = vec![0; self.players.len()];
        self.recent_actions.clear();
        self.action_log.clear();
        self.winner_ids.clear();

        // Emit logs for dealing after the loop to avoid borrow conflicts
        self.action_log.extend(dealt_logs.into_iter());

        // Post blinds (SB at dealer+1, BB at dealer+2)
        let n = self.players.len();
        if n > 1 {
            let sb_idx = (self.dealer_idx + 1) % n;
            let bb_idx = (self.dealer_idx + 2) % n;
            self.post_blind(sb_idx, BlindKind::SmallBlind, self.sb);
            self.post_blind(bb_idx, BlindKind::BigBlind, self.bb);
            self.current_bet = self.bb;
            self.min_raise = self.bb;
            // Preflop first to act is left of BB
            self.to_act = (bb_idx + 1) % n;
        } else {
            self.to_act = self.dealer_idx;
        }

        self.init_round_for_stage();
        self.log(LogEntry {
            player_id: None,
            event: LogEvent::StageChanged(self.stage),
        });
    }

    fn post_blind(&mut self, idx: usize, kind: BlindKind, amount: u32) {
        let a = amount.min(self.players[idx].stack);
        self.players[idx].stack -= a;
        self.round_bets[idx] += a;
        self.pot += a;
        if a < amount {
            self.players[idx].all_in = true;
        }
        self.log(LogEntry {
            player_id: Some(idx),
            event: LogEvent::Action(ActionKind::PostBlind { kind, amount: a }),
        });
        println!(
            "[BLIND] {} posts {:?} {} -> stack {}",
            self.players[idx].name, kind, a, self.players[idx].stack
        );
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
            action_log: self.action_log.clone(),
        }
    }

    fn apply_player_action(&mut self, actor: usize, action: PlayerAction) {
        if actor != self.to_act || self.players[actor].has_folded || self.players[actor].all_in {
            return;
        }
        println!(
            "[ACTION] {}: {:?} (stage: {:?})",
            self.players[actor].name, action, self.stage
        );
        self.recent_actions.push(ActionEvent {
            player_id: actor,
            action: action.clone(),
        });

        match action {
            PlayerAction::Fold => {
                self.players[actor].has_folded = true;
                self.log(LogEntry {
                    player_id: Some(actor),
                    event: LogEvent::Action(ActionKind::Fold),
                });
            }
            PlayerAction::CheckCall => {
                // Check or call to match current bet
                let need = self.current_bet.saturating_sub(self.round_bets[actor]);
                if need == 0 {
                    self.log(LogEntry {
                        player_id: Some(actor),
                        event: LogEvent::Action(ActionKind::Check),
                    });
                } else {
                    let pay = need.min(self.players[actor].stack);
                    self.players[actor].stack -= pay;
                    self.round_bets[actor] += pay;
                    self.pot += pay;
                    if pay < need {
                        self.players[actor].all_in = true;
                    }
                    self.log(LogEntry {
                        player_id: Some(actor),
                        event: LogEvent::Action(ActionKind::Call(pay)),
                    });
                }
            }
            PlayerAction::Bet(x) => {
                if self.current_bet == 0 {
                    // Open bet
                    let bet_to = x
                        .max(self.bb)
                        .min(self.players[actor].stack + self.round_bets[actor]);
                    let add = bet_to
                        .saturating_sub(self.round_bets[actor])
                        .min(self.players[actor].stack);
                    self.players[actor].stack -= add;
                    self.round_bets[actor] += add;
                    self.pot += add;
                    self.current_bet = self.round_bets[actor];
                    self.min_raise = self.bb.max(add);
                    if self.players[actor].stack == 0 {
                        self.players[actor].all_in = true;
                    }
                    self.log(LogEntry {
                        player_id: Some(actor),
                        event: LogEvent::Action(ActionKind::Bet(add)),
                    });
                } else {
                    // Raise
                    let target_to = self.current_bet + x;
                    let add = target_to
                        .saturating_sub(self.round_bets[actor])
                        .min(self.players[actor].stack);
                    if add == 0 {
                        // treat as call if cannot raise
                        let need = self.current_bet.saturating_sub(self.round_bets[actor]);
                        let pay = need.min(self.players[actor].stack);
                        self.players[actor].stack -= pay;
                        self.round_bets[actor] += pay;
                        self.pot += pay;
                        if pay < need {
                            self.players[actor].all_in = true;
                        }
                        self.log(LogEntry {
                            player_id: Some(actor),
                            event: LogEvent::Action(ActionKind::Call(pay)),
                        });
                    } else {
                        self.players[actor].stack -= add;
                        self.round_bets[actor] += add;
                        self.pot += add;
                        let by = (self.round_bets[actor] - self.current_bet).max(add);
                        self.current_bet = self.round_bets[actor];
                        self.min_raise = self.min_raise.max(by);
                        if self.players[actor].stack == 0 {
                            self.players[actor].all_in = true;
                        }
                        self.log(LogEntry {
                            player_id: Some(actor),
                            event: LogEvent::Action(ActionKind::Raise {
                                to: self.current_bet,
                                by,
                            }),
                        });
                    }
                }
            }
        }

        // Remove actor from pending list if matched requirement, folded, or all-in
        self.remove_from_pending(actor);

        // If only one player remains, end the hand
        if self.active_players().len() <= 1 {
            self.stage = Stage::Showdown;
            self.finish_showdown();
            return;
        }

        // If betting round complete, advance stage
        if self.is_betting_round_complete() {
            self.advance_stage();
            if self.stage == Stage::Showdown {
                self.finish_showdown();
                return;
            }
            self.init_round_for_stage();
        } else if let Some(&nxt) = self.pending_to_act.first() {
            self.to_act = nxt;
        }
    }

    fn active_players(&self) -> Vec<usize> {
        self.players
            .iter()
            .enumerate()
            .filter_map(|(i, p)| (!p.has_folded).then_some(i))
            .collect()
    }

    fn remove_from_pending(&mut self, actor: usize) {
        if let Some(pos) = self.pending_to_act.iter().position(|&i| i == actor) {
            let need = self.current_bet.saturating_sub(self.round_bets[actor]);
            if self.players[actor].has_folded || self.players[actor].all_in || need == 0 {
                self.pending_to_act.remove(pos);
            }
        }
    }

    fn is_betting_round_complete(&self) -> bool {
        // Complete if every non-folded, non-all-in player has matched current_bet
        for (i, p) in self.players.iter().enumerate() {
            if p.has_folded || p.all_in {
                continue;
            }
            if self.round_bets[i] != self.current_bet {
                return false;
            }
        }
        true
    }

    fn random_bot_action(&self, bot_index: usize) -> PlayerAction {
        use rand::Rng;
        let mut rng = rand::rng();
        let need = self.current_bet.saturating_sub(self.round_bets[bot_index]);
        if need == 0 {
            let r: u8 = rng.random_range(0..100);
            if r < 70 {
                PlayerAction::CheckCall
            } else {
                PlayerAction::Bet(self.bb)
            }
        } else {
            let r: u8 = rng.random_range(0..100);
            if r < 60 {
                PlayerAction::CheckCall
            } else if r < 80 {
                PlayerAction::Fold
            } else {
                PlayerAction::Bet(self.bb)
            }
        }
    }

    fn play_out_bots(&mut self) {
        // Let bots act until it's human's turn or showdown
        while self.stage != Stage::Showdown && self.to_act != 0 {
            let actor = self.to_act;
            if actor == 0
                || actor >= self.players.len()
                || self.players[actor].has_folded
                || self.players[actor].all_in
            {
                break;
            }
            let action = self.random_bot_action(actor);
            println!("[BOT] {}: {:?}", self.players[actor].name, action);
            self.apply_player_action(actor, action);
        }
    }

    fn init_round_for_stage(&mut self) {
        self.round_bets.fill(0);
        self.current_bet = 0;
        self.min_raise = self.bb;

        let n = self.players.len();
        let start = match self.stage {
            Stage::Preflop => {
                if n > 1 {
                    (self.dealer_idx + 3) % n // left of BB
                } else {
                    self.dealer_idx
                }
            }
            Stage::Flop | Stage::Turn | Stage::River => (self.dealer_idx + 1) % n,
            Stage::Showdown => self.dealer_idx,
        };
        self.pending_to_act.clear();
        for i in 0..n {
            let idx = (start + i) % n;
            if !self.players[idx].has_folded && !self.players[idx].all_in {
                self.pending_to_act.push(idx);
            }
        }
        self.to_act = *self.pending_to_act.first().unwrap_or(&self.dealer_idx);
    }

    fn advance_stage(&mut self) {
        match self.stage {
            Stage::Preflop => {
                // Flop (burn ignored for simplicity)
                self.community.push(self.deck.pop_front().unwrap());
                self.community.push(self.deck.pop_front().unwrap());
                self.community.push(self.deck.pop_front().unwrap());
                self.stage = Stage::Flop;
                self.log(LogEntry {
                    player_id: None,
                    event: LogEvent::DealtCommunity {
                        cards: self.community.clone(),
                    },
                });
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
                self.log(LogEntry {
                    player_id: None,
                    event: LogEvent::DealtCommunity {
                        cards: self.community.clone(),
                    },
                });
                println!("[STAGE] Turn: {}", card_str(self.community[3]));
            }
            Stage::Turn => {
                self.community.push(self.deck.pop_front().unwrap());
                self.stage = Stage::River;
                self.log(LogEntry {
                    player_id: None,
                    event: LogEvent::DealtCommunity {
                        cards: self.community.clone(),
                    },
                });
                println!("[STAGE] River: {}", card_str(self.community[4]));
            }
            Stage::River => {
                self.stage = Stage::Showdown;
            }
            Stage::Showdown => {}
        }
        self.log(LogEntry {
            player_id: None,
            event: LogEvent::StageChanged(self.stage),
        });
    }

    fn finish_showdown(&mut self) {
        // Evaluate all non-folded players
        let mut results: Vec<HandResult> = Vec::new();
        for (i, p) in self.players.iter().enumerate() {
            if p.has_folded {
                continue;
            }
            let rank = evaluate_best_hand(p.cards, &self.community);
            let best_five = pick_best_five(p.cards, &self.community, &rank);
            results.push(HandResult {
                player_id: i,
                rank,
                best_five,
            });
        }
        // Determine winners (top rank; split on ties)
        results.sort_by(|a, b| a.rank.cmp(&b.rank));
        let winners: Vec<usize> = if let Some(best) = results.last().cloned() {
            results
                .iter()
                .rev()
                .take_while(|r| r.rank == best.rank)
                .map(|r| r.player_id)
                .collect()
        } else {
            vec![]
        };
        self.winner_ids = winners.clone();

        self.log(LogEntry {
            player_id: None,
            event: LogEvent::Showdown {
                hand_results: results.clone(),
            },
        });

        if !winners.is_empty() && self.pot > 0 {
            let share = self.pot / winners.len() as u32;
            let mut remainder = self.pot % winners.len() as u32;
            for &w in &winners {
                let mut win = share;
                if remainder > 0 {
                    win += 1;
                    remainder -= 1;
                }
                self.players[w].stack += win;
            }
            self.log(LogEntry {
                player_id: None,
                event: LogEvent::PotAwarded {
                    winners: winners.clone(),
                    amount: self.pot,
                },
            });
            println!("[SHOWDOWN] Pot {} awarded to {:?}", self.pot, winners);
            self.pot = 0;
        }
    }

    fn log(&mut self, entry: LogEntry) {
        self.action_log.push(entry);
    }
}

#[derive(Clone, Default)]
struct Lobby {
    game: Option<Game>,
}

#[derive(Clone, Default)]
pub struct AppState {
    lobby: Arc<RwLock<Lobby>>,
    pub bot_count: usize,
}

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route(
            "/health",
            get(|| async { Json(serde_json::json!({ "ok": true })) }),
        )
        .route("/ws", get(ws_handler))
        .with_state(state)
}

pub async fn run_server(addr: SocketAddr, state: AppState) {
    let app = build_router(state.clone());
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
            // Let bots act if it's their turn
            if let Some(game) = &mut lobby.game {
                if game.players.len() > 1 && game.to_act != 0 {
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
                        ClientMsg::NextHand => {
                            println!("[WS] NextHand requested by {}", name);
                            {
                                let mut lobby = state.lobby.write().await;
                                if let Some(game) = &mut lobby.game {
                                    let n = game.players.len();
                                    if n > 0 {
                                        game.dealer_idx = (game.dealer_idx + 1) % n;
                                    }
                                    game.start_new_hand();
                                    if game.to_act != 0 {
                                        game.play_out_bots();
                                    }
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
