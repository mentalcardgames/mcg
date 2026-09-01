use std::sync::Arc;

use axum::extract::ws::WebSocket;
use iroh::endpoint::Endpoint;
use mcg_shared::{Backend2FrontendMsg, Peer2PeerMsg};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::sync::{mpsc, oneshot};

use crate::network::iroh::{
    IrohConnectError, IrohConnector, IrohEndpointConnector, IrohReader, IrohWriter,
};
use crate::network::{ConnectionId, NetworkCommand, NetworkError, PeerId};

/// Internal request contract sent from [`NetworkHandle`] to [`super::NetworkSupervisor`].
pub(crate) enum SupervisorRequest {
    Shutdown {
        response_tx: oneshot::Sender<()>,
    },
    ConfigureIroh {
        connector: Arc<dyn IrohConnector>,
        response_tx: oneshot::Sender<Result<(), NetworkError>>,
    },
    ConnectIrohPeer {
        ticket: String,
        response_tx: oneshot::Sender<Result<ConnectionId, NetworkError>>,
    },
    RegisterFrontendWebSocket {
        socket: Box<WebSocket>,
        response_tx: oneshot::Sender<Result<ConnectionId, NetworkError>>,
    },
    RegisterPendingPeerWebSocket {
        socket: Box<WebSocket>,
        response_tx: oneshot::Sender<Result<ConnectionId, NetworkError>>,
    },
    RegisterIncomingIrohPeer {
        peer_id: PeerId,
        reader: Box<dyn AsyncRead + Unpin + Send>,
        writer: Box<dyn AsyncWrite + Unpin + Send>,
        response_tx: oneshot::Sender<Result<ConnectionId, NetworkError>>,
    },
    RegisterIncomingIrohFrontend {
        reader: Box<dyn AsyncRead + Unpin + Send>,
        writer: Box<dyn AsyncWrite + Unpin + Send>,
        response_tx: oneshot::Sender<Result<ConnectionId, NetworkError>>,
    },
    Execute {
        command: NetworkCommand,
        response_tx: oneshot::Sender<Result<(), NetworkError>>,
    },
}

pub(crate) struct IrohConnectResult {
    pub(crate) result: Result<(PeerId, IrohReader, IrohWriter), IrohConnectError>,
    pub(crate) response_tx: oneshot::Sender<Result<ConnectionId, NetworkError>>,
}

/// Cloneable interface used by transports and application logic.
#[derive(Clone)]
pub struct NetworkHandle {
    pub(crate) request_tx: mpsc::Sender<SupervisorRequest>,
}

impl NetworkHandle {
    pub(crate) fn new(request_tx: mpsc::Sender<SupervisorRequest>) -> Self {
        Self { request_tx }
    }

    /// Requests an orderly shutdown of the supervisor and all owned child tasks.
    pub(crate) async fn shutdown(&self) -> Result<(), NetworkError> {
        let (response_tx, response_rx) = oneshot::channel();
        self.request_tx
            .send(SupervisorRequest::Shutdown { response_tx })
            .await
            .map_err(|_| NetworkError::SupervisorStopped)?;
        response_rx
            .await
            .map_err(|_| NetworkError::SupervisorStopped)
    }

    /// Configures the Iroh endpoint used for outgoing peer connections.
    pub async fn configure_iroh_endpoint(&self, endpoint: Endpoint) -> Result<(), NetworkError> {
        self.configure_iroh_connector(Arc::new(IrohEndpointConnector::new(endpoint)))
            .await
    }

    pub(crate) async fn configure_iroh_connector(
        &self,
        connector: Arc<dyn IrohConnector>,
    ) -> Result<(), NetworkError> {
        let (response_tx, response_rx) = oneshot::channel();
        self.request_tx
            .send(SupervisorRequest::ConfigureIroh {
                connector,
                response_tx,
            })
            .await
            .map_err(|_| NetworkError::SupervisorStopped)?;
        response_rx
            .await
            .map_err(|_| NetworkError::SupervisorStopped)?
    }

    /// Establishes and registers an outgoing Iroh peer connection.
    pub async fn connect_iroh_peer(
        &self,
        ticket: impl Into<String>,
    ) -> Result<ConnectionId, NetworkError> {
        let (response_tx, response_rx) = oneshot::channel();
        self.request_tx
            .send(SupervisorRequest::ConnectIrohPeer {
                ticket: ticket.into(),
                response_tx,
            })
            .await
            .map_err(|_| NetworkError::SupervisorStopped)?;
        response_rx
            .await
            .map_err(|_| NetworkError::SupervisorStopped)?
    }

    /// Registers an upgraded frontend WebSocket with the supervisor.
    pub async fn register_frontend_websocket(
        &self,
        socket: WebSocket,
    ) -> Result<ConnectionId, NetworkError> {
        let (response_tx, response_rx) = oneshot::channel();
        self.request_tx
            .send(SupervisorRequest::RegisterFrontendWebSocket {
                socket: Box::new(socket),
                response_tx,
            })
            .await
            .map_err(|_| NetworkError::SupervisorStopped)?;
        response_rx
            .await
            .map_err(|_| NetworkError::SupervisorStopped)?
    }

    /// Registers an upgraded peer WebSocket pending its identity handshake.
    pub async fn register_pending_peer_websocket(
        &self,
        socket: WebSocket,
    ) -> Result<ConnectionId, NetworkError> {
        let (response_tx, response_rx) = oneshot::channel();
        self.request_tx
            .send(SupervisorRequest::RegisterPendingPeerWebSocket {
                socket: Box::new(socket),
                response_tx,
            })
            .await
            .map_err(|_| NetworkError::SupervisorStopped)?;
        response_rx
            .await
            .map_err(|_| NetworkError::SupervisorStopped)?
    }

    /// Registers an established Iroh peer stream with the supervisor.
    pub async fn register_incoming_iroh_peer<R, W>(
        &self,
        peer_id: PeerId,
        reader: R,
        writer: W,
    ) -> Result<ConnectionId, NetworkError>
    where
        R: AsyncRead + Unpin + Send + 'static,
        W: AsyncWrite + Unpin + Send + 'static,
    {
        let (response_tx, response_rx) = oneshot::channel();
        self.request_tx
            .send(SupervisorRequest::RegisterIncomingIrohPeer {
                peer_id,
                reader: Box::new(reader),
                writer: Box::new(writer),
                response_tx,
            })
            .await
            .map_err(|_| NetworkError::SupervisorStopped)?;
        response_rx
            .await
            .map_err(|_| NetworkError::SupervisorStopped)?
    }

    /// Registers an established Iroh frontend stream with the supervisor.
    pub async fn register_incoming_iroh_frontend<R, W>(
        &self,
        reader: R,
        writer: W,
    ) -> Result<ConnectionId, NetworkError>
    where
        R: AsyncRead + Unpin + Send + 'static,
        W: AsyncWrite + Unpin + Send + 'static,
    {
        let (response_tx, response_rx) = oneshot::channel();
        self.request_tx
            .send(SupervisorRequest::RegisterIncomingIrohFrontend {
                reader: Box::new(reader),
                writer: Box::new(writer),
                response_tx,
            })
            .await
            .map_err(|_| NetworkError::SupervisorStopped)?;
        response_rx
            .await
            .map_err(|_| NetworkError::SupervisorStopped)?
    }

    /// Sends a targeted command to the supervisor and waits until it has been
    /// accepted by the destination connection queue.
    pub async fn send_command(&self, command: NetworkCommand) -> Result<(), NetworkError> {
        let (response_tx, response_rx) = oneshot::channel();
        self.request_tx
            .send(SupervisorRequest::Execute {
                command,
                response_tx,
            })
            .await
            .map_err(|_| NetworkError::SupervisorStopped)?;
        response_rx
            .await
            .map_err(|_| NetworkError::SupervisorStopped)?
    }

    /// Broadcasts a typed message across all registered frontend connections.
    pub async fn broadcast_frontend(
        &self,
        message: Backend2FrontendMsg,
    ) -> Result<(), NetworkError> {
        self.send_command(NetworkCommand::BroadcastFrontend(message))
            .await
    }

    /// Broadcasts a typed message across all registered peer connections.
    pub async fn broadcast_peer(&self, message: Peer2PeerMsg) -> Result<(), NetworkError> {
        self.send_command(NetworkCommand::BroadcastPeer(message))
            .await
    }
}
