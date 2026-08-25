use mcg_shared::{Backend2FrontendMsg, Frontend2BackendMsg, PlayerAction, PlayerId};
use tokio::sync::{mpsc, oneshot};

use crate::network::NetworkEvent;

use super::types::{ControllerError, ControllerEvent};

/// Cloneable handle used by async tasks (HTTP handlers, network actors, bot drivers)
/// to send events to the synchronous Controller thread.
#[derive(Clone, Debug)]
pub struct ControllerHandle {
    event_tx: mpsc::Sender<ControllerEvent>,
}

impl ControllerHandle {
    /// Creates a new controller handle wrapping an event sender channel.
    pub fn new(event_tx: mpsc::Sender<ControllerEvent>) -> Self {
        Self { event_tx }
    }

    /// Asynchronously sends a raw `ControllerEvent` to the Controller.
    pub async fn send_event(&self, event: ControllerEvent) -> Result<(), ControllerError> {
        self.event_tx
            .send(event)
            .await
            .map_err(|_| ControllerError::ControllerStopped)
    }

    /// Synchronously attempts to send an event without blocking.
    pub fn try_send_event(&self, event: ControllerEvent) -> Result<(), ControllerError> {
        match self.event_tx.try_send(event) {
            Ok(()) => Ok(()),
            Err(mpsc::error::TrySendError::Full(_)) => Err(ControllerError::QueueFull),
            Err(mpsc::error::TrySendError::Closed(_)) => Err(ControllerError::ControllerStopped),
        }
    }

    /// Forwards a network event from the network supervisor to the Controller.
    pub async fn send_network_event(&self, event: NetworkEvent) -> Result<(), ControllerError> {
        self.send_event(ControllerEvent::Network(event)).await
    }

    /// Submits a bot player action to the Controller.
    pub async fn send_bot_action(
        &self,
        player_id: PlayerId,
        action: PlayerAction,
    ) -> Result<(), ControllerError> {
        self.send_event(ControllerEvent::BotAction { player_id, action })
            .await
    }

    /// Dispatches an HTTP request message to the Controller and awaits the synchronous response.
    pub async fn send_http_request(
        &self,
        message: Frontend2BackendMsg,
    ) -> Result<Backend2FrontendMsg, ControllerError> {
        let (reply_tx, reply_rx) = oneshot::channel();
        self.send_event(ControllerEvent::HttpRequest { message, reply_tx })
            .await?;
        reply_rx.await.map_err(|_| ControllerError::ResponseDropped)
    }

    /// Requests the Controller to cleanly shut down.
    pub async fn shutdown(&self) -> Result<(), ControllerError> {
        self.send_event(ControllerEvent::Shutdown).await
    }
}
