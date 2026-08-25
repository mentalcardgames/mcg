//! Dedicated synchronous Controller environment and event abstractions.
//!
//! The controller executes sequentially in an isolated thread and owns the core
//! application domain state (game, lobby, player identity). It processes incoming
//! [`ControllerEvent`] messages and communicates with the async network shell via
//! [`ControllerCommand`] and the [`ControllerSink`] trait.

pub mod bridge;
mod core;
mod handle;
mod runner;
mod sink;
mod types;

pub use self::bridge::{spawn_controller_command_forwarder, spawn_network_event_forwarder};
pub use self::core::{Controller, Lobby, PeerInfo};
pub use handle::ControllerHandle;
pub use runner::{spawn_controller, start_controller};
pub use sink::{ChannelControllerSink, ControllerSink, InMemoryControllerSink};
pub use types::{ControllerCommand, ControllerError, ControllerEvent};

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;
    use crate::config::Config;
    use crate::network::ConnectionId;
    use mcg_shared::{Backend2FrontendMsg, Frontend2BackendMsg, PlayerConfig, PlayerId};
    use tokio::sync::mpsc;

    #[test]
    fn in_memory_sink_records_commands() {
        let mut sink = InMemoryControllerSink::new();
        sink.broadcast_frontend(Backend2FrontendMsg::Pong);
        sink.send_frontend(ConnectionId::new(42), Backend2FrontendMsg::Pong);
        sink.close_connection(ConnectionId::new(42), "test close".into());

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

    #[tokio::test]
    async fn dedicated_controller_thread_executes_sequential_loop() {
        let (command_tx, mut command_rx) = mpsc::unbounded_channel();
        let sink = ChannelControllerSink::new(command_tx);
        let controller = Controller::new(Config::default(), None);

        let (thread_handle, handle) = start_controller(controller, 16, sink);

        // Send HTTP request to start game
        let response = handle
            .send_http_request(Frontend2BackendMsg::NewGame {
                players: vec![
                    PlayerConfig {
                        id: PlayerId(0),
                        name: "Alice".into(),
                        is_bot: false,
                    },
                    PlayerConfig {
                        id: PlayerId(1),
                        name: "Bob".into(),
                        is_bot: true,
                    },
                ],
            })
            .await
            .expect("should get response");

        assert!(matches!(response, Backend2FrontendMsg::UpdatePokerState(_)));

        // Expect BroadcastFrontend from the controller command channel
        let command = tokio::time::timeout(Duration::from_secs(1), command_rx.recv())
            .await
            .expect("command should arrive within timeout")
            .expect("channel should be open");
        assert!(matches!(command, ControllerCommand::BroadcastFrontend(_)));

        // Shut down controller thread
        handle.shutdown().await.expect("shutdown event sent");
        thread_handle.join().expect("thread join succeeded");
    }

    #[tokio::test]
    async fn bridge_wires_controller_with_network_supervisor() {
        use crate::network::NetworkSupervisor;
        use tokio::io::{duplex, split, AsyncBufReadExt, BufReader};

        let (network_event_tx, network_event_rx) = mpsc::channel(16);
        let (supervisor, network) = NetworkSupervisor::new(network_event_tx);
        let supervisor_task = tokio::spawn(supervisor.run());

        let (command_tx, command_rx) = mpsc::unbounded_channel();
        let sink = ChannelControllerSink::new(command_tx);
        let controller = Controller::new(Config::default(), None);
        let (thread_handle, controller_handle) = start_controller(controller, 16, sink);

        let _event_forwarder =
            spawn_network_event_forwarder(network_event_rx, controller_handle.clone(), None);
        let _command_forwarder =
            spawn_controller_command_forwarder(command_rx, network.clone(), None);

        // Register a frontend stream
        let (frontend_stream, frontend_remote) = duplex(4096);
        let (fe_r, fe_w) = split(frontend_stream);
        let (fe_rem_r, _fe_rem_w) = split(frontend_remote);
        let mut fe_reader = BufReader::new(fe_rem_r);

        let _conn_id = network
            .register_incoming_iroh_frontend(fe_r, fe_w)
            .await
            .expect("frontend registration succeeded");

        // Request a new game from the controller via HTTP
        let response = controller_handle
            .send_http_request(Frontend2BackendMsg::NewGame {
                players: vec![
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
                ],
            })
            .await
            .expect("response received");
        assert!(matches!(response, Backend2FrontendMsg::UpdatePokerState(_)));

        // Read the broadcasted state update on the registered frontend stream
        let mut line = String::new();
        fe_reader
            .read_line(&mut line)
            .await
            .expect("read from frontend remote");
        let broadcasted: Backend2FrontendMsg =
            serde_json::from_str(line.trim()).expect("deserialize broadcasted message");
        assert!(matches!(
            broadcasted,
            Backend2FrontendMsg::UpdatePokerState(_)
        ));

        // Clean shutdown
        controller_handle
            .shutdown()
            .await
            .expect("controller shutdown");
        thread_handle.join().expect("thread join");
        network.shutdown().await.expect("network shutdown");
        let _ = supervisor_task.await;
    }
}
