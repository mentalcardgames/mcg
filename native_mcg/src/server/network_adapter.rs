use std::collections::HashSet;

use mcg_shared::{Backend2FrontendMsg, Frontend2BackendMsg};
use tokio::sync::{broadcast, mpsc};

use crate::network::{ConnectionId, NetworkCommand, NetworkError, NetworkEvent, NetworkHandle};

use super::state::{current_state_public, dispatch_client_message, AppState};

/// Temporary bridge between the actor-based network layer and the legacy
/// lock-based backend state.
pub(super) struct LegacyBackendAdapter {
    state: AppState,
    network: NetworkHandle,
    event_rx: mpsc::Receiver<NetworkEvent>,
    broadcast_rx: broadcast::Receiver<Backend2FrontendMsg>,
    subscribers: HashSet<ConnectionId>,
}

impl LegacyBackendAdapter {
    pub(super) fn new(
        state: AppState,
        network: NetworkHandle,
        event_rx: mpsc::Receiver<NetworkEvent>,
    ) -> Self {
        let broadcast_rx = state.broadcaster.subscribe();
        Self {
            state,
            network,
            event_rx,
            broadcast_rx,
            subscribers: HashSet::new(),
        }
    }

    pub(super) async fn run(mut self) {
        loop {
            tokio::select! {
                event = self.event_rx.recv() => {
                    let Some(event) = event else {
                        break;
                    };
                    if !self.handle_network_event(event).await {
                        break;
                    }
                }
                broadcast = self.broadcast_rx.recv() => {
                    match broadcast {
                        Ok(message) => {
                            if !self.forward_broadcast(message).await {
                                break;
                            }
                        }
                        Err(broadcast::error::RecvError::Lagged(skipped)) => {
                            tracing::warn!(skipped, "legacy network adapter missed backend broadcasts");
                        }
                        Err(broadcast::error::RecvError::Closed) => break,
                    }
                }
            }
        }
        tracing::info!("legacy backend network adapter stopped");
    }

    async fn handle_network_event(&mut self, event: NetworkEvent) -> bool {
        match event {
            NetworkEvent::ConnectionOpened { connection } => {
                tracing::debug!(
                    connection_id = %connection.id,
                    role = ?connection.role,
                    transport = ?connection.transport,
                    "network connection opened"
                );
                true
            }
            NetworkEvent::ConnectionClosed {
                connection_id,
                reason,
            } => {
                self.subscribers.remove(&connection_id);
                tracing::debug!(%connection_id, ?reason, "network connection closed");
                true
            }
            NetworkEvent::FrontendMessage {
                connection_id,
                message: Frontend2BackendMsg::Subscribe,
            } => self.subscribe(connection_id).await,
            NetworkEvent::FrontendMessage {
                connection_id,
                message,
            } => {
                let response = dispatch_client_message(&self.state, message).await;
                self.send_frontend(connection_id, response).await
            }
            NetworkEvent::PeerMessage {
                connection_id,
                message,
            } => {
                tracing::warn!(%connection_id, ?message, "legacy adapter does not handle peer events yet");
                true
            }
        }
    }

    async fn subscribe(&mut self, connection_id: ConnectionId) -> bool {
        if !self.subscribers.insert(connection_id) {
            return self
                .send_frontend(
                    connection_id,
                    Backend2FrontendMsg::Error("already subscribed".into()),
                )
                .await;
        }

        if let Some(state) = current_state_public(&self.state).await {
            self.send_frontend(connection_id, Backend2FrontendMsg::UpdatePokerState(state))
                .await
        } else {
            true
        }
    }

    async fn forward_broadcast(&mut self, message: Backend2FrontendMsg) -> bool {
        let subscribers: Vec<_> = self.subscribers.iter().copied().collect();
        for connection_id in subscribers {
            match self
                .network
                .send_command(NetworkCommand::SendFrontend {
                    connection_id,
                    message: message.clone(),
                })
                .await
            {
                Ok(()) => {}
                Err(
                    NetworkError::ConnectionNotFound(_) | NetworkError::ConnectionActorStopped(_),
                ) => {
                    self.subscribers.remove(&connection_id);
                }
                Err(NetworkError::ConnectionBackpressured(_)) => {
                    tracing::warn!(%connection_id, "dropping broadcast for backpressured frontend");
                }
                Err(NetworkError::SupervisorStopped) => return false,
                Err(error) => {
                    tracing::error!(%connection_id, %error, "failed to forward backend broadcast");
                }
            }
        }
        true
    }

    async fn send_frontend(
        &mut self,
        connection_id: ConnectionId,
        message: Backend2FrontendMsg,
    ) -> bool {
        match self
            .network
            .send_command(NetworkCommand::SendFrontend {
                connection_id,
                message,
            })
            .await
        {
            Ok(()) => true,
            Err(NetworkError::ConnectionNotFound(_) | NetworkError::ConnectionActorStopped(_)) => {
                self.subscribers.remove(&connection_id);
                true
            }
            Err(NetworkError::SupervisorStopped) => false,
            Err(error) => {
                tracing::error!(%connection_id, %error, "failed to send direct frontend response");
                true
            }
        }
    }
}
