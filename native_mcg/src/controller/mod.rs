//! Dedicated synchronous Controller environment and event abstractions.
//!
//! The controller executes sequentially in an isolated thread and owns the core
//! application domain state (game, lobby, player identity). It processes incoming
//! [`ControllerEvent`] messages and communicates with the async network shell via
//! [`ControllerCommand`] and the [`ControllerSink`] trait.

mod handle;
mod sink;
mod types;

pub use handle::ControllerHandle;
pub use sink::{ChannelControllerSink, ControllerSink, InMemoryControllerSink};
pub use types::{ControllerCommand, ControllerError, ControllerEvent};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::network::ConnectionId;
    use mcg_shared::Backend2FrontendMsg;
    use tokio::sync::mpsc;

    #[tokio::test]
    async fn controller_handle_sends_and_receives_http_request() {
        let (event_tx, mut event_rx) = mpsc::channel(16);
        let handle = ControllerHandle::new(event_tx);

        let request_task = tokio::spawn(async move {
            handle
                .send_http_request(mcg_shared::Frontend2BackendMsg::Ping)
                .await
        });

        let received = event_rx.recv().await.expect("should receive event");
        match received {
            ControllerEvent::HttpRequest { message, reply_tx } => {
                assert!(matches!(message, mcg_shared::Frontend2BackendMsg::Ping));
                reply_tx
                    .send(Backend2FrontendMsg::Pong)
                    .expect("should reply");
            }
            _ => panic!("unexpected event type"),
        }

        let response = request_task
            .await
            .expect("join succeeded")
            .expect("request succeeded");
        assert!(matches!(response, Backend2FrontendMsg::Pong));
    }

    #[test]
    fn in_memory_sink_records_commands() {
        let mut sink = InMemoryControllerSink::new();
        sink.broadcast_frontend(Backend2FrontendMsg::Pong);
        sink.send_frontend(ConnectionId::new(42), Backend2FrontendMsg::Pong);
        sink.close_connection(ConnectionId::new(42), "test close");

        assert_eq!(sink.commands.len(), 3);
        assert!(matches!(
            &sink.commands[0],
            ControllerCommand::BroadcastFrontend(Backend2FrontendMsg::Pong)
        ));
        assert!(matches!(
            &sink.commands[1],
            ControllerCommand::SendFrontend {
                connection_id,
                message: Backend2FrontendMsg::Pong,
            } if *connection_id == ConnectionId::new(42)
        ));
        assert!(matches!(
            &sink.commands[2],
            ControllerCommand::CloseConnection {
                connection_id,
                reason,
            } if *connection_id == ConnectionId::new(42) && reason == "test close"
        ));
    }
}
