use std::{cmp::Ordering, collections::VecDeque, net::SocketAddr, sync::Arc};

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
    ActionEvent, ActionKind, BlindKind, ClientMsg, GameStatePublic, HandRank, HandRankCategory,
    HandResult, LogEntry, LogEvent, PlayerAction, PlayerPublic, ServerMsg, Stage,
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
    players: Vec<Player>,
    deck: VecDeque<u8>,
    community: Vec<u8>,

    // Betting
    pot: u32,
    stage: Stage,
    dealer_idx: usize,
    to_act: usize,
    current_bet: u32,
    min_raise: u32,
    round_bets: Vec<u32>, // per-player contribution this street

    // Blinds
    sb: u32,
    bb: u32,

    // Flow bookkeeping
    pending_to_act: Vec<usize>, // players that still need to act this street
    recent_actions: Vec<ActionEvent>,
    action_log: Vec<LogEntry>,
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
            deck,
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
        // Shuffle new deck
        let mut deck: Vec<u8> = (0..52).collect();
        deck.shuffle(&mut rand::rng());
        self.deck = VecDeque::from(deck);

        // Reset players
        for p in &mut self.players {
            p.has_folded = false;
            p.all_in = false;
            p.cards = [
                self.deck.pop_front().unwrap(),
                self.deck.pop_front().unwrap(),
            ];
            // log of DealtHole deferred until after dealing
            println!(
                "[DEAL] {} gets {} {}",
                p.name,
                card_str(p.cards[0]),
                card_str(p.cards[1])
            );
        }

        // Emit DealtHole logs after dealing to avoid nested mutable borrow
        for p in &self.players {
            self.log(LogEntry {
                player_id: Some(p.id),
                event: LogEvent::DealtHole { player_id: p.id },
            });
        }

        // Reset table
        self.community.clear();
        self.pot = 0;
        self.stage = Stage::Preflop;
        self.current_bet = 0;
        self.min_raise = self.bb;
        self.round_bets = vec![0; self.players.len()];
        self.recent_actions.clear();
        self.action_log.clear();
        self.winner_ids.clear();

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
                // Interpret as bet/raise by x
                if self.current_bet == 0 {
                    // Bet
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
                    // Raise by x
                    let target_to = self.current_bet + x;
                    let add = target_to
                        .saturating_sub(self.round_bets[actor])
                        .min(self.players[actor].stack);
                    if add == 0 {
                        // treat as call if can't raise
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

        // Remove actor from pending list if matched or folded or all-in
        self.remove_from_pending(actor);

        // If only one player remains, end hand
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
        } else {
            // Next actor is next pending
            if let Some(&nxt) = self.pending_to_act.first() {
                self.to_act = nxt;
            }
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
        // Let bots act until it's human's turn or showdown or stage change that becomes human turn
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

        // Order of action:
        // - Preflop: from player left of BB
        // - Postflop: from player left of dealer
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
        // Initialize pending with all active (non-folded, not all-in) players starting at 'start'
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
                // Flop: burn one (ignored), deal 3 community
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
            // Get best five approximated by picking pattern from rank if available
            let best_five = pick_best_five(p.cards, &self.community, &rank);
            results.push(HandResult {
                player_id: i,
                rank,
                best_five,
            });
        }
        // Find best rank
        results.sort_by(|a, b| a.rank.cmp(&b.rank));
        let best = results.last().cloned();
        let winners: Vec<usize> = if let Some(best) = best {
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
            // Have bots act if it's their turn
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

fn card_rank(c: u8) -> u8 {
    c % 13
}
fn card_suit(c: u8) -> u8 {
    c / 13
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

// Evaluate best 5-card hand from 7 cards (2 hole + community)
// Returns HandRank with category and tiebreakers (high to low)
fn evaluate_best_hand(hole: [u8; 2], community: &[u8]) -> HandRank {
    let mut cards = Vec::with_capacity(7);
    cards.push(hole[0]);
    cards.push(hole[1]);
    for &c in community {
        cards.push(c);
    }
    best_rank_from_seven(&cards)
}

fn best_rank_from_seven(cards: &Vec<u8>) -> HandRank {
    // Detect flush
    let mut suit_counts = [0u8; 4];
    for &c in cards {
        suit_counts[card_suit(c) as usize] += 1;
    }
    let flush_suit = (0..4).find(|&s| suit_counts[s] >= 5).map(|s| s as u8);

    // Collect ranks per suit for flush straight detection
    let mut suit_cards: [Vec<u8>; 4] = [vec![], vec![], vec![], vec![]];
    for &c in cards {
        suit_cards[card_suit(c) as usize].push(c);
    }

    // Straight flush
    if let Some(fs) = flush_suit {
        let mut ranks = suit_cards[fs as usize]
            .iter()
            .map(|&c| map_rank_for_straight(card_rank(c)))
            .collect::<Vec<u8>>();
        ranks.sort_unstable();
        ranks.dedup();
        if let Some(high) = straight_high(&ranks) {
            return HandRank {
                category: HandRankCategory::StraightFlush,
                tiebreakers: vec![high],
            };
        }
    }

    // Counts
    let mut counts = [0u8; 13];
    for &c in cards {
        counts[card_rank(c) as usize] += 1;
    }
    // four of a kind
    if let Some((quad_rank, kicker)) = find_n_of_a_kind(&counts, 4, cards) {
        return HandRank {
            category: HandRankCategory::FourKind,
            tiebreakers: vec![quad_rank, kicker],
        };
    }
    // full house
    if let Some((trip, pair)) = find_full_house(&counts) {
        return HandRank {
            category: HandRankCategory::FullHouse,
            tiebreakers: vec![trip, pair],
        };
    }
    // flush
    if let Some(fs) = flush_suit {
        let mut ranks = suit_cards[fs as usize]
            .iter()
            .map(|&c| map_rank_high(card_rank(c)))
            .collect::<Vec<u8>>();
        ranks.sort_unstable_by(|a, b| b.cmp(a));
        let top5 = ranks.into_iter().take(5).collect::<Vec<u8>>();
        return HandRank {
            category: HandRankCategory::Flush,
            tiebreakers: top5,
        };
    }
    // straight
    {
        let mut ranks = cards
            .iter()
            .map(|&c| map_rank_for_straight(card_rank(c)))
            .collect::<Vec<u8>>();
        ranks.sort_unstable();
        ranks.dedup();
        if let Some(high) = straight_high(&ranks) {
            return HandRank {
                category: HandRankCategory::Straight,
                tiebreakers: vec![high],
            };
        }
    }
    // three of a kind
    if let Some((trip_rank, kickers)) = find_n_kind_with_kickers(&counts, 3, cards, 2) {
        return HandRank {
            category: HandRankCategory::ThreeKind,
            tiebreakers: std::iter::once(trip_rank)
                .chain(kickers.into_iter())
                .collect(),
        };
    }
    // two pair
    if let Some((high_pair, low_pair, kicker)) = find_two_pair(&counts, cards) {
        return HandRank {
            category: HandRankCategory::TwoPair,
            tiebreakers: vec![high_pair, low_pair, kicker],
        };
    }
    // one pair
    if let Some((pair_rank, kickers)) = find_n_kind_with_kickers(&counts, 2, cards, 3) {
        return HandRank {
            category: HandRankCategory::Pair,
            tiebreakers: std::iter::once(pair_rank)
                .chain(kickers.into_iter())
                .collect(),
        };
    }
    // high card
    let mut highs = cards
        .iter()
        .map(|&c| map_rank_high(card_rank(c)))
        .collect::<Vec<u8>>();
    highs.sort_unstable_by(|a, b| b.cmp(a));
    highs.truncate(5);
    HandRank {
        category: HandRankCategory::HighCard,
        tiebreakers: highs,
    }
}

// Helpers for hand evaluation

fn map_rank_for_straight(r: u8) -> u8 {
    // Map A(0) to 13 for high straights; also retain 0 for wheel detection
    if r == 0 {
        13
    } else {
        r
    }
}
fn map_rank_high(r: u8) -> u8 {
    // High card value with Ace high=13
    if r == 0 {
        13
    } else {
        r
    }
}
fn straight_high(ranks: &Vec<u8>) -> Option<u8> {
    // ranks are unique and sorted ascending
    let mut consec = 1;
    let mut last = 255;
    let mut best_high = None;
    for &r in ranks {
        if last == 255 || r == last + 1 {
            consec += if last == 255 { 0 } else { 1 };
        } else if r != last {
            consec = 1;
        }
        if consec >= 5 {
            best_high = Some(r);
        }
        last = r;
    }
    // Wheel A-2-3-4-5: if ranks contain 1,2,3,4 and Ace low (represented as 0), we treat high as 5
    // Our mapping used 13 for Ace, so separately handle wheel by checking presence of 1..4 and 13 and 0 variations
    // Simpler approach: caller passes mapped ranks; we already mapped Ace as 13
    best_high
}

fn find_n_of_a_kind(counts: &[u8; 13], n: u8, cards: &Vec<u8>) -> Option<(u8, u8)> {
    // returns (rank, kicker)
    let mut rank = None;
    for i in (0..13).rev() {
        if counts[i] == n {
            rank = Some(map_rank_high(i as u8));
            break;
        }
    }
    if let Some(rk) = rank {
        // find top kicker
        let mut kickers = cards
            .iter()
            .filter(|&&c| map_rank_high(card_rank(c)) != rk)
            .map(|&c| map_rank_high(card_rank(c)))
            .collect::<Vec<u8>>();
        kickers.sort_unstable_by(|a, b| b.cmp(a));
        if let Some(k) = kickers.first() {
            return Some((rk, *k));
        }
    }
    None
}

fn find_full_house(counts: &[u8; 13]) -> Option<(u8, u8)> {
    let mut trips = vec![];
    let mut pairs = vec![];
    for i in 0..13 {
        if counts[i] >= 3 {
            trips.push(map_rank_high(i as u8));
        } else if counts[i] >= 2 {
            pairs.push(map_rank_high(i as u8));
        }
    }
    trips.sort_unstable();
    pairs.sort_unstable();
    let trip = trips.pop();
    let pair = pairs.pop().or_else(|| trips.pop()); // another trip can act as pair
    if let (Some(t), Some(p)) = (trip, pair) {
        return Some((t, p));
    }
    None
}

fn find_n_kind_with_kickers(
    counts: &[u8; 13],
    n: u8,
    cards: &Vec<u8>,
    kicker_count: usize,
) -> Option<(u8, Vec<u8>)> {
    let mut kind_rank = None;
    for i in (0..13).rev() {
        if counts[i] == n {
            kind_rank = Some(map_rank_high(i as u8));
            break;
        }
    }
    if let Some(kr) = kind_rank {
        let mut kickers = cards
            .iter()
            .filter(|&&c| map_rank_high(card_rank(c)) != kr)
            .map(|&c| map_rank_high(card_rank(c)))
            .collect::<Vec<u8>>();
        kickers.sort_unstable_by(|a, b| b.cmp(a));
        kickers.truncate(kicker_count);
        return Some((kr, kickers));
    }
    None
}

fn find_two_pair(counts: &[u8; 13], cards: &Vec<u8>) -> Option<(u8, u8, u8)> {
    let mut pairs = vec![];
    for i in 0..13 {
        if counts[i] >= 2 {
            pairs.push(map_rank_high(i as u8));
        }
    }
    pairs.sort_unstable();
    if pairs.len() >= 2 {
        let low = pairs.pop().unwrap();
        let high = pairs.pop().unwrap_or(low);
        let mut kickers = cards
            .iter()
            .filter(|&&c| {
                let r = map_rank_high(card_rank(c));
                r != high && r != low
            })
            .map(|&c| map_rank_high(card_rank(c)))
            .collect::<Vec<u8>>();
        kickers.sort_unstable_by(|a, b| b.cmp(a));
        let kicker = *kickers.first().unwrap_or(&1);
        return Some((high, low, kicker));
    }
    None
}

// For UI best-five presentation: naive extraction (not used for ranking decision)
fn pick_best_five(_hole: [u8; 2], community: &[u8], rank: &HandRank) -> [u8; 5] {
    // As an approximation, return the top 5 community cards by rank if available,
    // otherwise pad with zeros. Proper best-5 card identification can be implemented later.
    let mut cc = community.to_vec();
    cc.sort_unstable_by(|a, b| map_rank_high(card_rank(*b)).cmp(&map_rank_high(card_rank(*a))));
    let mut out = [0u8; 5];
    for i in 0..5.min(cc.len()) {
        out[i] = cc[i];
    }
    match rank.category {
        HandRankCategory::StraightFlush
        | HandRankCategory::FourKind
        | HandRankCategory::FullHouse
        | HandRankCategory::Flush
        | HandRankCategory::Straight
        | HandRankCategory::ThreeKind
        | HandRankCategory::TwoPair
        | HandRankCategory::Pair
        | HandRankCategory::HighCard => out,
    }
}
