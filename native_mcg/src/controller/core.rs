use std::collections::HashMap;
use std::fs::File;
use std::io::{IsTerminal, Read};
use std::path::PathBuf;

use anyhow::Result;
use mcg_shared::{
    Backend2FrontendMsg, Card, CardRank, CardSuit, Frontend2BackendMsg, Peer2PeerMsg, PlayerAction,
    PlayerConfig, PlayerId, PokerStatePublic, Stage,
};
use tokio::sync::watch;

use crate::bot::BotManager;
use crate::config::{path_for_config, Config, PublicInfo};
use crate::game::{Game, Player};
use crate::network::{ConnectionId, NetworkEvent, NetworkHandle, PeerId};
use mcg_shared::pretty;

use super::types::ControllerEvent;

/// Peer information held directly by the Controller.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct PeerInfo {
    pub name: String,
    pub ticket: String,
}

/// Lobby and game session state owned exclusively by the Controller.
#[derive(Clone, Debug)]
pub struct Lobby {
    pub game: Option<Game>,
    pub last_printed_log_len: usize,
    pub bots: Vec<PlayerId>,
    pub bot_manager: BotManager,
    pub max_players: usize,
    pub lobby_open: bool,
    pub our_name: String,
    pub ready: bool,
    pub game_type: String,
    pub game_running: bool,
}

impl Default for Lobby {
    fn default() -> Self {
        Self {
            game: None,
            last_printed_log_len: 0,
            bots: Vec::new(),
            bot_manager: BotManager::default(),
            max_players: 2,
            lobby_open: false,
            our_name: String::new(),
            ready: false,
            game_type: String::new(),
            game_running: false,
        }
    }
}

/// The synchronous Controller.
///
/// Owns core domain models (Lobby, Game, Configuration, Identity) and processes
/// all game and network events sequentially on a single dedicated OS thread.
pub struct Controller {
    lobby: Lobby,
    #[allow(dead_code)]
    config: Config,
    #[allow(dead_code)]
    config_path: Option<PathBuf>,
    ticket: Option<String>,
    peers: HashMap<iroh::EndpointId, PeerInfo>,
    peer_connections: HashMap<PeerId, ConnectionId>,
    connection_peers: HashMap<ConnectionId, PeerId>,
    state_watch_tx: Option<watch::Sender<Option<PokerStatePublic>>>,
}

impl Controller {
    /// Creates a new Controller instance with the specified configuration.
    pub fn new(config: Config, config_path: Option<PathBuf>) -> Self {
        Self {
            lobby: Lobby::default(),
            config,
            config_path,
            ticket: None,
            peers: HashMap::new(),
            peer_connections: HashMap::new(),
            connection_peers: HashMap::new(),
            state_watch_tx: None,
        }
    }

    /// Attaches a public state watch sender for bot observation.
    pub fn with_state_watch(
        mut self,
        state_watch_tx: watch::Sender<Option<PokerStatePublic>>,
    ) -> Self {
        self.state_watch_tx = Some(state_watch_tx);
        self
    }

    /// Sets or updates the public state watch sender for bot observation.
    pub fn set_state_watch_tx(&mut self, state_watch_tx: watch::Sender<Option<PokerStatePublic>>) {
        self.state_watch_tx = Some(state_watch_tx);
    }

    /// Sets the local ticket and registers the local peer identity in the peer table.
    pub fn set_local_ticket(&mut self, endpoint_id: iroh::EndpointId, ticket: String) {
        self.ticket = Some(ticket.clone());
        self.peers.insert(
            endpoint_id,
            PeerInfo {
                name: self.lobby.our_name.clone(),
                ticket,
            },
        );

        let public_path = path_for_config(self.config_path.as_deref());
        if let Err(error) = PublicInfo::write_iroh_node_id(&public_path, endpoint_id.to_string()) {
            tracing::warn!(%error, path = %public_path.display(), "failed to persist iroh node id");
        }
    }

    /// Main dispatch entry point for processing incoming `ControllerEvent`s.
    pub fn handle_event(&mut self, event: ControllerEvent, network: &NetworkHandle) {
        match event {
            ControllerEvent::Network(network_event) => {
                self.handle_network_event(network_event, network);
            }
            ControllerEvent::BotAction { player_id, action } => {
                match self.validate_and_apply_action(player_id, action.clone()) {
                    Ok(()) => {
                        self.print_latest_changes();
                        self.broadcast_state(network);
                    }
                    Err(error) => {
                        tracing::warn!(%player_id, ?action, %error, "failed to execute bot action");
                    }
                }
            }
            ControllerEvent::Shutdown => {
                tracing::info!("Controller received shutdown event");
            }
        }
    }

    /// Handles network-level events from the network supervisor.
    fn handle_network_event(&mut self, event: NetworkEvent, network: &NetworkHandle) {
        match event {
            NetworkEvent::FrontendConnected {
                connection_id,
                transport,
            } => {
                tracing::debug!(%connection_id, ?transport, "frontend connection registered in controller");
                // Immediately synchronize the current poker state with the new frontend if active.
                if let Some(state) = self.current_state_public() {
                    if let Err(error) = network.blocking_unicast_frontend(
                        connection_id,
                        Backend2FrontendMsg::UpdatePokerState(state),
                    ) {
                        tracing::warn!(%connection_id, %error, "failed to send frontend message from controller");
                    }
                }
            }
            NetworkEvent::PeerConnected {
                connection_id,
                peer_id,
                transport,
                direction,
            } => {
                tracing::debug!(%connection_id, %peer_id, ?transport, ?direction, "peer connection registered in controller");
                self.peer_connections.insert(peer_id.clone(), connection_id);
                self.connection_peers.insert(connection_id, peer_id);
            }
            NetworkEvent::ConnectionClosed {
                connection_id,
                reason,
            } => {
                tracing::debug!(%connection_id, ?reason, "connection closed");
                self.remove_connection(connection_id, network);
            }
            NetworkEvent::FrontendMessage {
                connection_id,
                message,
            } => {
                if let Some(response) =
                    self.handle_frontend_message(Some(connection_id), message, network)
                {
                    if let Err(error) = network.blocking_unicast_frontend(connection_id, response) {
                        tracing::warn!(%connection_id, %error, "failed to send frontend message from controller");
                    }
                }
            }
            NetworkEvent::PeerMessage {
                connection_id,
                message,
            } => {
                let Some(peer_id) = self.connection_peers.get(&connection_id).cloned() else {
                    tracing::error!(%connection_id, "peer message from unregistered peer connection");
                    if let Err(error) =
                        network.blocking_close_connection(connection_id, "missing peer identity")
                    {
                        tracing::warn!(%connection_id, %error, "failed to close connection from controller");
                    }
                    return;
                };
                self.handle_peer_message(connection_id, peer_id, message, network);
            }
        }
    }

    /// Handles a typed frontend message, mutating domain state and broadcasting updates.
    pub fn handle_frontend_message(
        &mut self,
        _source: Option<ConnectionId>,
        msg: Frontend2BackendMsg,
        network: &NetworkHandle,
    ) -> Option<Backend2FrontendMsg> {
        match msg {
            Frontend2BackendMsg::Action { player_id, action } => {
                match self.validate_and_apply_action(player_id, action) {
                    Ok(()) => {
                        self.print_latest_changes();
                        self.broadcast_state(network);
                        None
                    }
                    Err(e) => Some(Backend2FrontendMsg::Error(e)),
                }
            }
            Frontend2BackendMsg::RequestState => {
                if let Some(gs) = self.current_state_public() {
                    Some(Backend2FrontendMsg::UpdatePokerState(gs))
                } else {
                    Some(Backend2FrontendMsg::Error(
                        "No active game. Please start a new game first.".into(),
                    ))
                }
            }
            Frontend2BackendMsg::Ping => {
                tracing::info!("received ping from client");
                Some(Backend2FrontendMsg::Pong)
            }
            Frontend2BackendMsg::NextHand => match self.advance_to_next_hand() {
                Ok(()) => {
                    self.print_latest_changes();
                    self.broadcast_state(network);
                    None
                }
                Err(e) => Some(Backend2FrontendMsg::Error(e)),
            },
            Frontend2BackendMsg::NewGame { players } => match self.create_game_session(players) {
                Ok(()) => {
                    self.print_latest_changes();
                    self.broadcast_state(network);
                    None
                }
                Err(e) => Some(Backend2FrontendMsg::Error(e)),
            },
            Frontend2BackendMsg::PushState { state } => match self.import_game_state(state) {
                Ok(()) => {
                    self.print_latest_changes();
                    self.broadcast_state(network);
                    None
                }
                Err(e) => Some(Backend2FrontendMsg::Error(e)),
            },
            Frontend2BackendMsg::QrValue(ticket) => {
                if let Err(error) = network.blocking_establish_iroh_peer_connection(ticket) {
                    tracing::warn!(%error, "failed to connect to iroh peer from controller");
                }
                Some(Backend2FrontendMsg::Pong)
            }
            Frontend2BackendMsg::GetTicket => match &self.ticket {
                Some(ticket) => Some(Backend2FrontendMsg::TicketValue(ticket.clone())),
                None => Some(Backend2FrontendMsg::Error("Iroh not initialized".into())),
            },
            Frontend2BackendMsg::GetIP => {
                let ip = match local_ipaddress::get() {
                    Some(ip_addr) => ip_addr,
                    None => {
                        return Some(Backend2FrontendMsg::Error(
                            "Unable to determine local IP".into(),
                        ))
                    }
                };
                Some(Backend2FrontendMsg::IPValue(ip))
            }
            Frontend2BackendMsg::QrReq(file) => {
                let path = format!("media/qr_test/{}", file);
                match File::open(&path) {
                    Ok(mut file) => {
                        let mut buf = Vec::new();
                        match file.read_to_end(&mut buf) {
                            Ok(_) => {
                                let content: Box<[u8]> = buf.into();
                                Some(Backend2FrontendMsg::QrRes(content))
                            }
                            Err(e) => Some(Backend2FrontendMsg::Error(e.to_string())),
                        }
                    }
                    Err(e) => Some(Backend2FrontendMsg::Error(e.to_string())),
                }
            }
            Frontend2BackendMsg::PlayerCount(count) => {
                self.lobby.max_players = count;
                tracing::info!("Max player count set to {}", count);
                Some(Backend2FrontendMsg::Error(format!(
                    "Max player count set to {}",
                    count
                )))
            }
            Frontend2BackendMsg::LobbyOpen(game_type) => {
                self.lobby.lobby_open = true;
                self.lobby.game_type = game_type.clone();
                tracing::info!("Lobby opened for game type: {}", game_type);
                Some(Backend2FrontendMsg::Error("Lobby is now open".to_string()))
            }
            Frontend2BackendMsg::PlayerName(name) => {
                for peer in self.peers.values_mut() {
                    if peer.name == self.lobby.our_name {
                        peer.name = name.clone();
                        break;
                    }
                }
                self.lobby.our_name = name.clone();
                tracing::info!("Player name set to {}", name);
                Some(Backend2FrontendMsg::Error(format!(
                    "Player name set to {}",
                    name
                )))
            }
            Frontend2BackendMsg::GetOurName => {
                Some(Backend2FrontendMsg::OurName(self.lobby.our_name.clone()))
            }
            Frontend2BackendMsg::Disconnect => {
                tracing::info!("Received disconnect message from client");
                if let Err(error) = network
                    .blocking_broadcast_peer(Peer2PeerMsg::Disconnect(self.lobby.our_name.clone()))
                {
                    tracing::warn!(%error, "failed to broadcast peer message from controller");
                }

                self.lobby.lobby_open = false;
                self.lobby.game_running = false;
                self.lobby.game_type = String::new();
                let our_name = self.lobby.our_name.clone();
                self.peers.retain(|_, p| p.name == our_name);
                tracing::info!("Lobby closed.");

                Some(Backend2FrontendMsg::Error("Goodbye".into()))
            }
            Frontend2BackendMsg::ReadyUpdate(ready) => {
                self.lobby.ready = ready;
                if let Err(error) = network.blocking_broadcast_peer(Peer2PeerMsg::PeerReady(
                    self.lobby.our_name.clone(),
                    ready,
                )) {
                    tracing::warn!(%error, "failed to broadcast peer message from controller");
                }
                Some(Backend2FrontendMsg::Error(format!(
                    "Ready status updated: {}",
                    ready
                )))
            }
            Frontend2BackendMsg::GetPlayers => {
                for peer in self.peers.values() {
                    if let Err(error) = network.blocking_broadcast_frontend(
                        Backend2FrontendMsg::NewPlayer(peer.name.clone()),
                    ) {
                        tracing::warn!(%error, "failed to broadcast frontend message from controller");
                    }
                }
                if let Err(error) = network.blocking_broadcast_peer(Peer2PeerMsg::RequestReady) {
                    tracing::warn!(%error, "failed to broadcast peer message from controller");
                }
                Some(Backend2FrontendMsg::Error("Player list sent".into()))
            }
        }
    }

    /// Handles a typed peer message received from a remote peer node.
    pub fn handle_peer_message(
        &mut self,
        connection_id: ConnectionId,
        peer_id: PeerId,
        msg: Peer2PeerMsg,
        network: &NetworkHandle,
    ) {
        match msg {
            Peer2PeerMsg::Connect(name, ticket) => {
                let rejection = if !self.lobby.lobby_open {
                    Some("Lobby is closed")
                } else if self.lobby.game_running {
                    Some("Game is already running, wait until it finishes and try again")
                } else if self.peers.len() >= self.lobby.max_players {
                    Some("Lobby is full")
                } else {
                    None
                };

                if let Some(reason) = rejection {
                    if let Err(error) = network
                        .blocking_unicast_peer(connection_id, Peer2PeerMsg::Reject(reason.into()))
                    {
                        tracing::warn!(%connection_id, %error, "failed to send peer message from controller");
                    }
                    if let Err(error) =
                        network.blocking_close_connection(connection_id, "peer connection rejected")
                    {
                        tracing::warn!(%connection_id, %error, "failed to close connection from controller");
                    }
                    return;
                }

                let Some(endpoint_id) = Self::parse_peer_endpoint_id(&peer_id) else {
                    if let Err(error) =
                        network.blocking_close_connection(connection_id, "invalid peer identity")
                    {
                        tracing::warn!(%connection_id, %error, "failed to close connection from controller");
                    }
                    return;
                };

                let max_players = self.lobby.max_players;
                let game_type = self.lobby.game_type.clone();
                if let Err(error) = network.blocking_unicast_peer(
                    connection_id,
                    Peer2PeerMsg::LobbyAccept(max_players, game_type),
                ) {
                    tracing::warn!(%connection_id, %error, "failed to send peer message from controller");
                }

                let assigned_name = if self.peers.values().any(|peer| peer.name == name) {
                    let mut counter = 2;
                    loop {
                        let candidate = format!("{name} {counter}");
                        if !self.peers.values().any(|peer| peer.name == candidate) {
                            break candidate;
                        }
                        counter += 1;
                    }
                } else {
                    name.clone()
                };

                if assigned_name != name {
                    if let Err(error) = network.blocking_unicast_peer(
                        connection_id,
                        Peer2PeerMsg::NewName(assigned_name.clone()),
                    ) {
                        tracing::warn!(%connection_id, %error, "failed to send peer message from controller");
                    }
                }

                let known_peers: HashMap<String, (String, String)> = self
                    .peers
                    .iter()
                    .map(|(id, info)| (id.to_string(), (info.name.clone(), info.ticket.clone())))
                    .collect();
                if let Err(error) =
                    network.blocking_unicast_peer(connection_id, Peer2PeerMsg::Peers(known_peers))
                {
                    tracing::warn!(%connection_id, %error, "failed to send peer message from controller");
                }

                self.peers.insert(
                    endpoint_id,
                    PeerInfo {
                        name: assigned_name.clone(),
                        ticket: ticket.unwrap_or_default(),
                    },
                );
                if let Err(error) = network
                    .blocking_broadcast_frontend(Backend2FrontendMsg::NewPlayer(assigned_name))
                {
                    tracing::warn!(%error, "failed to broadcast frontend message from controller");
                }
            }
            Peer2PeerMsg::Disconnect(name) => {
                self.remove_peer_from_table(&peer_id);
                if let Err(error) =
                    network.blocking_broadcast_frontend(Backend2FrontendMsg::RemovePlayer(name))
                {
                    tracing::warn!(%error, "failed to broadcast frontend message from controller");
                }
                if let Err(error) =
                    network.blocking_close_connection(connection_id, "peer requested disconnect")
                {
                    tracing::warn!(%connection_id, %error, "failed to close connection from controller");
                }
            }
            Peer2PeerMsg::Peers(advertised_peers) => {
                let mut discovered_tickets = Vec::new();
                for (id, (name, ticket)) in advertised_peers {
                    let Ok(endpoint_id) = id.parse::<iroh::EndpointId>() else {
                        tracing::warn!(peer_id = %id, "received invalid advertised peer identity");
                        continue;
                    };
                    if self.peers.contains_key(&endpoint_id) {
                        continue;
                    }

                    self.peers.insert(
                        endpoint_id,
                        PeerInfo {
                            name: name.clone(),
                            ticket: ticket.clone(),
                        },
                    );
                    if let Err(error) =
                        network.blocking_broadcast_frontend(Backend2FrontendMsg::NewPlayer(name))
                    {
                        tracing::warn!(%error, "failed to broadcast frontend message from controller");
                    }
                    if id != peer_id.as_str() {
                        discovered_tickets.push(ticket);
                    }
                }

                for ticket in discovered_tickets {
                    if let Err(error) = network.blocking_establish_iroh_peer_connection(ticket) {
                        tracing::warn!(%error, "failed to connect to iroh peer from controller");
                    }
                }
            }
            Peer2PeerMsg::NewName(name) => {
                self.lobby.our_name = name.clone();
                let own_ticket = self.ticket.clone().unwrap_or_default();
                if let Some(own_peer) = self.peers.values_mut().find(|p| p.ticket == own_ticket) {
                    own_peer.name = name.clone();
                }
                if let Err(error) =
                    network.blocking_broadcast_frontend(Backend2FrontendMsg::OurName(name))
                {
                    tracing::warn!(%error, "failed to broadcast frontend message from controller");
                }
            }
            Peer2PeerMsg::LobbyAccept(max_players, game_type) => {
                self.lobby.lobby_open = true;
                self.lobby.max_players = max_players;
                self.lobby.game_type = game_type;
                if let Err(error) = network.blocking_broadcast_frontend(Backend2FrontendMsg::Pong) {
                    tracing::warn!(%error, "failed to broadcast frontend message from controller");
                }
            }
            Peer2PeerMsg::RequestReady => {
                let message =
                    Peer2PeerMsg::PeerReady(self.lobby.our_name.clone(), self.lobby.ready);
                if let Err(error) = network.blocking_unicast_peer(connection_id, message) {
                    tracing::warn!(%connection_id, %error, "failed to send peer message from controller");
                }
            }
            Peer2PeerMsg::PeerReady(name, ready) => {
                if let Err(error) = network
                    .blocking_broadcast_frontend(Backend2FrontendMsg::PlayerReady(name, ready))
                {
                    tracing::warn!(%error, "failed to broadcast frontend message from controller");
                }
            }
            Peer2PeerMsg::Reject(reason) => {
                if let Err(error) = network.blocking_broadcast_frontend(Backend2FrontendMsg::Error(
                    format!("Peer rejected connection: {reason}"),
                )) {
                    tracing::warn!(%error, "failed to broadcast frontend message from controller");
                }
                if let Err(error) =
                    network.blocking_close_connection(connection_id, "peer rejected connection")
                {
                    tracing::warn!(%connection_id, %error, "failed to close connection from controller");
                }
            }
            other => {
                tracing::debug!(%connection_id, %peer_id, ?other, "unhandled peer message in controller");
            }
        }
    }

    /// Removes an open connection and cleans up peer mappings if necessary.
    fn remove_connection(&mut self, connection_id: ConnectionId, network: &NetworkHandle) {
        if let Some(peer_id) = self.connection_peers.remove(&connection_id) {
            self.peer_connections.remove(&peer_id);
            if let Some(peer) = self.remove_peer_from_table(&peer_id) {
                if !peer.name.is_empty() {
                    if let Err(error) = network
                        .blocking_broadcast_frontend(Backend2FrontendMsg::RemovePlayer(peer.name))
                    {
                        tracing::warn!(%error, "failed to broadcast frontend message from controller");
                    }
                }
            }
        }
    }

    fn remove_peer_from_table(&mut self, peer_id: &PeerId) -> Option<PeerInfo> {
        let endpoint_id = Self::parse_peer_endpoint_id(peer_id)?;
        self.peers.remove(&endpoint_id)
    }

    fn parse_peer_endpoint_id(peer_id: &PeerId) -> Option<iroh::EndpointId> {
        match peer_id.as_str().parse() {
            Ok(endpoint_id) => Some(endpoint_id),
            Err(error) => {
                tracing::error!(%peer_id, %error, "peer identity is not a valid Iroh endpoint ID");
                None
            }
        }
    }

    /// Returns the public projection of the current active game state.
    pub fn current_state_public(&self) -> Option<PokerStatePublic> {
        self.lobby.game.as_ref().map(|game| game.public())
    }

    /// Prints new action logs to console.
    pub fn print_latest_changes(&mut self) {
        if let Some(gs) = self.current_state_public() {
            let already = self.lobby.last_printed_log_len;
            let total = gs.action_log.len();
            if total > already {
                for e in gs.action_log.iter().skip(already) {
                    let line =
                        pretty::format_event_human(e, &gs.players, std::io::stdout().is_terminal());
                    tracing::info!(%line);
                }
                self.lobby.last_printed_log_len = total;
            }
        }
    }

    /// Broadcasts the current game state to all frontends and updates the state watch channel.
    pub fn broadcast_state(&self, network: &NetworkHandle) {
        if let Some(gs) = self.current_state_public() {
            let current_player_name = mcg_shared::PlayerPublic::name_of(&gs.players, gs.to_act);
            tracing::info!(
                "📡 Broadcasting game state (stage: {:?}, to_act: {})",
                gs.stage,
                current_player_name
            );
            if let Some(ref tx) = self.state_watch_tx {
                let _ = tx.send_replace(Some(gs.clone()));
            }
            if let Err(error) =
                network.blocking_broadcast_frontend(Backend2FrontendMsg::UpdatePokerState(gs))
            {
                tracing::warn!(%error, "failed to broadcast frontend message from controller");
            }
        }
    }

    /// Executes a player action, validating turns and advancing game progression.
    pub fn validate_and_apply_action(
        &mut self,
        player_id: PlayerId,
        action: PlayerAction,
    ) -> Result<(), String> {
        let game = self
            .lobby
            .game
            .as_mut()
            .ok_or_else(|| "No active game. Please start a new game first.".to_string())?;

        let actor_idx = game
            .players
            .iter()
            .position(|p| p.id == player_id)
            .ok_or_else(|| "Unknown player id".to_string())?;

        if game.stage == Stage::Showdown || game.to_act != actor_idx {
            return Err("Not your turn".into());
        }

        game.apply_player_action(actor_idx, action)
            .map_err(|e| e.to_string())
    }

    /// Advances to the next hand in the current game.
    pub fn advance_to_next_hand(&mut self) -> Result<(), String> {
        let Some(game) = &mut self.lobby.game else {
            return Err("No active game. Please start a new game first.".into());
        };

        let n = game.players.len();
        if n > 0 {
            game.dealer_idx = (game.dealer_idx + 1) % n;
        }

        if let Err(e) = game.start_new_hand() {
            return Err(format!("Failed to start new hand: {e}"));
        }

        let sb = game.sb;
        let bb = game.bb;
        let gs = game.public();
        self.lobby.last_printed_log_len = gs.action_log.len();
        let header = pretty::format_table_header(&gs, sb, bb, std::io::stdout().is_terminal());
        tracing::info!("{}", header);

        Ok(())
    }

    /// Creates a new game session with the configured player list.
    pub fn create_game_session(&mut self, players: Vec<PlayerConfig>) -> Result<(), String> {
        let mut game_players = Vec::new();
        let mut bot_ids = Vec::new();
        for config in &players {
            if config.is_bot {
                bot_ids.push(config.id);
            }
            let player = Player {
                id: config.id,
                name: config.name.clone(),
                stack: 1000,
                cards: [
                    Card::new(CardRank::Ace, CardSuit::Clubs),
                    Card::new(CardRank::Ace, CardSuit::Clubs),
                ],
                has_folded: false,
                all_in: false,
                is_bot: config.is_bot,
            };
            game_players.push(player);
        }
        self.lobby.bots = bot_ids;

        match Game::with_players(game_players) {
            Ok(game) => {
                self.lobby.game = Some(game);
                self.lobby.last_printed_log_len = 0;
                Ok(())
            }
            Err(e) => Err(format!("Failed to create new game: {e}")),
        }
    }

    /// Replaces the active game state from an external serialized representation.
    pub fn import_game_state(&mut self, game_state: serde_json::Value) -> Result<(), String> {
        match serde_json::from_value::<Game>(game_state) {
            Ok(game) => {
                self.lobby.game = Some(game);
                self.lobby.last_printed_log_len = 0;
                tracing::info!("Game state replaced via PushState");
                Ok(())
            }
            Err(e) => Err(format!("Failed to deserialize game state: {e}")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::network::{NetworkSupervisor, PeerConnectionDirection, TransportKind};
    use tokio::sync::mpsc;

    #[tokio::test(flavor = "multi_thread")]
    async fn controller_creates_game_and_applies_player_actions() {
        let (event_tx, _event_rx) = mpsc::channel(16);
        let supervisor = NetworkSupervisor::new(event_tx);
        let (network, supervisor_task) = supervisor.start();

        let (state_watch_tx, state_watch_rx) = watch::channel(None);
        let net = network.clone();
        tokio::task::spawn_blocking(move || {
            let mut controller =
                Controller::new(Config::default(), None).with_state_watch(state_watch_tx);

            let players = vec![
                PlayerConfig {
                    id: PlayerId(0),
                    name: "Alice".into(),
                    is_bot: false,
                },
                PlayerConfig {
                    id: PlayerId(1),
                    name: "Bob".into(),
                    is_bot: false,
                },
            ];

            let response = controller.handle_frontend_message(
                None,
                Frontend2BackendMsg::NewGame { players },
                &net,
            );
            assert!(response.is_none());

            // Check active game state
            let state = controller
                .current_state_public()
                .expect("game should be active");
            assert_eq!(state.players.len(), 2);
            let active_player_id = state.to_act;

            // Apply valid action
            let action_res =
                controller.validate_and_apply_action(active_player_id, PlayerAction::CheckCall);
            assert!(action_res.is_ok());
            controller.print_latest_changes();
            controller.broadcast_state(&net);

            // Invalid turn action from same player should fail
            let wrong_turn_res =
                controller.validate_and_apply_action(active_player_id, PlayerAction::CheckCall);
            assert!(wrong_turn_res.is_err());
        })
        .await
        .expect("blocking task succeeded");

        assert!(state_watch_rx.borrow().is_some());

        network.shutdown().await.expect("network shutdown");
        supervisor_task.await.expect("supervisor task ok");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn controller_handles_peer_lobby_handshake() {
        use tokio::io::{duplex, split, AsyncBufReadExt, BufReader};

        let (event_tx, _event_rx) = mpsc::channel(16);
        let supervisor = NetworkSupervisor::new(event_tx);
        let (network, supervisor_task) = supervisor.start();

        let (peer_stream, peer_remote) = duplex(4096);
        let (peer_r, peer_w) = split(peer_stream);
        let (peer_rem_r, _peer_rem_w) = split(peer_remote);
        let mut peer_reader = BufReader::new(peer_rem_r);

        let endpoint_id = iroh::SecretKey::from_bytes(&[1; 32]).public();
        let peer_id = PeerId::new(endpoint_id.to_string());

        let connection_id = network
            .register_iroh_peer(peer_id.clone(), peer_r, peer_w)
            .await
            .expect("peer registered");

        let net = network.clone();
        let p_id = peer_id.clone();
        tokio::task::spawn_blocking(move || {
            let mut controller = Controller::new(Config::default(), None);

            // Open lobby for up to 3 players
            controller.lobby.lobby_open = true;
            controller.lobby.max_players = 3;
            controller.lobby.game_type = "poker".into();

            // Simulate incoming peer connection event
            controller.handle_event(
                ControllerEvent::Network(NetworkEvent::PeerConnected {
                    connection_id,
                    peer_id: p_id.clone(),
                    transport: TransportKind::Iroh,
                    direction: PeerConnectionDirection::Incoming,
                }),
                &net,
            );

            // Simulate peer connect message
            controller.handle_peer_message(
                connection_id,
                p_id.clone(),
                Peer2PeerMsg::Connect("Alice".into(), Some("alice-ticket".into())),
                &net,
            );

            assert!(controller.peers.contains_key(&endpoint_id));

            // Test Disconnect
            controller.handle_peer_message(
                connection_id,
                p_id,
                Peer2PeerMsg::Disconnect("Alice".into()),
                &net,
            );

            assert!(!controller.peers.contains_key(&endpoint_id));
        })
        .await
        .expect("blocking task ok");

        // Verify sent peer message received on remote side
        let mut line1 = String::new();
        peer_reader.read_line(&mut line1).await.expect("read line1");
        let msg1: Peer2PeerMsg = serde_json::from_str(line1.trim()).expect("parse msg1");
        assert!(matches!(
            msg1,
            Peer2PeerMsg::LobbyAccept(3, ref gt) if gt == "poker"
        ));

        network.shutdown().await.expect("network shutdown");
        supervisor_task.await.expect("supervisor task ok");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn controller_rejects_peer_when_lobby_closed() {
        use tokio::io::{duplex, split, AsyncBufReadExt, BufReader};

        let (event_tx, _event_rx) = mpsc::channel(16);
        let supervisor = NetworkSupervisor::new(event_tx);
        let (network, supervisor_task) = supervisor.start();

        let (peer_stream, peer_remote) = duplex(4096);
        let (peer_r, peer_w) = split(peer_stream);
        let (peer_rem_r, _peer_rem_w) = split(peer_remote);
        let mut peer_reader = BufReader::new(peer_rem_r);

        let endpoint_id = iroh::SecretKey::from_bytes(&[2; 32]).public();
        let peer_id = PeerId::new(endpoint_id.to_string());

        let connection_id = network
            .register_iroh_peer(peer_id.clone(), peer_r, peer_w)
            .await
            .expect("peer registered");

        let net = network.clone();
        let p_id = peer_id.clone();
        tokio::task::spawn_blocking(move || {
            let mut controller = Controller::new(Config::default(), None);
            controller.lobby.lobby_open = false;

            controller.handle_peer_message(
                connection_id,
                p_id,
                Peer2PeerMsg::Connect("Bob".into(), None),
                &net,
            );
        })
        .await
        .expect("blocking task ok");

        let mut line = String::new();
        peer_reader.read_line(&mut line).await.expect("read line");
        let msg: Peer2PeerMsg = serde_json::from_str(line.trim()).expect("parse msg");
        assert!(matches!(
            msg,
            Peer2PeerMsg::Reject(ref reason) if reason == "Lobby is closed"
        ));

        network.shutdown().await.expect("network shutdown");
        supervisor_task.await.expect("supervisor task ok");
    }
}
