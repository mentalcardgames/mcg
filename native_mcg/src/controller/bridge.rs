//! Async bridge connecting the synchronous Controller with the asynchronous Network layer.

use tokio::sync::mpsc;
use tokio::task::JoinHandle;

use crate::network::{NetworkCommand, NetworkEvent, NetworkHandle};
use crate::server::peer_connections::PeerConnectionService;

use super::handle::ControllerHandle;
use super::types::ControllerCommand;

/// Spawns an async task that forwards incoming [`NetworkEvent`]s to the [`ControllerHandle`],
/// updating the [`PeerConnectionService`] on connection lifecycle events.
pub fn spawn_network_event_forwarder(
    mut network_event_rx: mpsc::Receiver<NetworkEvent>,
    controller: ControllerHandle,
    peer_service: Option<PeerConnectionService>,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        while let Some(event) = network_event_rx.recv().await {
            match &event {
                NetworkEvent::PeerConnected {
                    connection_id,
                    peer_id,
                    direction,
                    ..
                } => {
                    if let Some(ref service) = peer_service {
                        service
                            .connection_opened(*connection_id, peer_id.clone(), *direction)
                            .await;
                    }
                }
                NetworkEvent::ConnectionClosed { connection_id, .. } => {
                    if let Some(ref service) = peer_service {
                        service.connection_closed(*connection_id).await;
                    }
                }
                _ => {}
            }

            if controller.send_network_event(event).await.is_err() {
                tracing::debug!(
                    "controller event receiver closed; stopping network event forwarder"
                );
                break;
            }
        }
    })
}

/// Spawns an async task that forwards outbound [`ControllerCommand`]s from the Controller
/// to [`NetworkHandle`] and [`PeerConnectionService`].
pub fn spawn_controller_command_forwarder(
    mut command_rx: mpsc::UnboundedReceiver<ControllerCommand>,
    network: NetworkHandle,
    peer_service: Option<PeerConnectionService>,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        while let Some(command) = command_rx.recv().await {
            match command {
                ControllerCommand::BroadcastFrontend(message) => {
                    if let Err(error) = network.broadcast_frontend(message).await {
                        tracing::warn!(%error, "failed to broadcast frontend message from controller");
                    }
                }
                ControllerCommand::BroadcastPeer(message) => {
                    if let Err(error) = network.broadcast_peer(message).await {
                        tracing::warn!(%error, "failed to broadcast peer message from controller");
                    }
                }
                ControllerCommand::SendFrontend {
                    connection_id,
                    message,
                } => {
                    if let Err(error) = network
                        .send_command(NetworkCommand::SendFrontend {
                            connection_id,
                            message,
                        })
                        .await
                    {
                        tracing::warn!(%connection_id, %error, "failed to send frontend message from controller");
                    }
                }
                ControllerCommand::SendPeer {
                    connection_id,
                    message,
                } => {
                    if let Err(error) = network
                        .send_command(NetworkCommand::SendPeer {
                            connection_id,
                            message,
                        })
                        .await
                    {
                        tracing::warn!(%connection_id, %error, "failed to send peer message from controller");
                    }
                }
                ControllerCommand::CloseConnection {
                    connection_id,
                    reason,
                } => {
                    if let Err(error) = network
                        .send_command(NetworkCommand::CloseConnection {
                            connection_id,
                            reason,
                        })
                        .await
                    {
                        tracing::warn!(%connection_id, %error, "failed to close connection from controller");
                    }
                }
                ControllerCommand::ConnectPeer { ticket } => {
                    if let Some(ref service) = peer_service {
                        let service = service.clone();
                        tokio::spawn(async move {
                            if let Err(error) = service.connect(ticket).await {
                                tracing::warn!(%error, "failed to connect to peer via ticket from controller");
                            }
                        });
                    } else {
                        let network = network.clone();
                        tokio::spawn(async move {
                            if let Err(error) = network.connect_iroh_peer(ticket).await {
                                tracing::warn!(%error, "failed to connect to iroh peer from controller");
                            }
                        });
                    }
                }
            }
        }
    })
}
