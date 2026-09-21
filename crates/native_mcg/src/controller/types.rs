use std::error::Error;
use std::fmt;

use mcg_shared::{PlayerAction, PlayerId};

use crate::network::NetworkEvent;

/// Errors that can occur when communicating with the Controller.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ControllerError {
    /// The controller thread has stopped or closed its event receiver.
    ControllerStopped,
    /// The controller's incoming event queue is full (for bounded channels).
    QueueFull,
}

impl fmt::Display for ControllerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ControllerStopped => write!(f, "controller thread has stopped"),
            Self::QueueFull => write!(f, "controller event queue is full"),
        }
    }
}

impl Error for ControllerError {}

/// Events received by the synchronous Controller from the async network shell,
/// bot drivers, or system lifecycle.
#[derive(Debug)]
pub enum ControllerEvent {
    /// Network-level event from the NetworkSupervisor (connections, incoming messages, closures).
    Network(NetworkEvent),
    /// Bot action dispatched by an external bot driver.
    BotAction {
        player_id: PlayerId,
        action: PlayerAction,
    },
    /// Request to cleanly shut down the controller event loop.
    Shutdown,
}
