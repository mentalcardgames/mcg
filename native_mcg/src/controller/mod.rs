//! Dedicated synchronous Controller environment and event abstractions.
//!
//! The controller executes sequentially in an isolated thread and owns the core
//! application domain state (game, lobby, player identity). It processes incoming
//! [`ControllerEvent`] messages and communicates with the async network shell via
//! [`NetworkHandle`].

mod builder;
mod core;
mod handle;
mod types;

pub use self::core::{Controller, Lobby, PeerInfo};
pub use builder::ControllerBuilder;
pub use handle::ControllerHandle;
pub use types::{ControllerError, ControllerEvent};

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;
    use crate::config::Config;
    use crate::network::ConnectionId;
    use mcg_shared::{Backend2FrontendMsg, Frontend2BackendMsg, PlayerConfig, PlayerId};
    use tokio::sync::mpsc;

    #[tokio::test(flavor = "multi_thread")]
    async fn dedicated_controller_thread_executes_sequential_loop() {
        let (network_event_tx, _network_event_rx) = mpsc::channel(16);
        let (network, supervisor_task) =
            crate::network::NetworkBuilder::new(ControllerHandle::new(network_event_tx)).spawn();

        let (state_watch_tx, mut state_watch_rx) = tokio::sync::watch::channel(None);
        let (handle, thread_handle) = ControllerBuilder::new(Config::default())
            .with_state_watch(state_watch_tx)
            .with_network(network.clone())
            .with_channel_capacity(16)
            .spawn();

        // Send network event to start game
        handle
            .send_network_event(crate::network::NetworkEvent::FrontendMessage {
                connection_id: ConnectionId::new(1),
                message: Frontend2BackendMsg::NewGame {
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
                },
            })
            .await
            .expect("should send event");

        // Expect state update to arrive via state_watch
        tokio::time::timeout(Duration::from_secs(1), state_watch_rx.changed())
            .await
            .expect("state update should arrive within timeout")
            .expect("watch channel should remain open");
        assert_eq!(state_watch_rx.borrow().as_ref().unwrap().players.len(), 2);

        // Shut down controller thread and network
        handle.shutdown().await.expect("shutdown event sent");
        thread_handle.join().expect("thread join succeeded");
        network.shutdown().await.expect("network shutdown");
        supervisor_task.await.expect("supervisor task ok");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn controller_wires_with_network_supervisor() {
        use crate::network::NetworkBuilder;
        use tokio::io::{duplex, split, AsyncBufReadExt, AsyncWriteExt, BufReader};

        let controller_builder =
            ControllerBuilder::new(Config::default()).with_channel_capacity(16);
        let controller_handle = controller_builder.handle();
        let (network, supervisor_task) = NetworkBuilder::new(controller_handle.clone()).spawn();
        let (_, thread_handle) = controller_builder.with_network(network.clone()).spawn();

        // Register a frontend stream
        let (frontend_stream, frontend_remote) = duplex(4096);
        let (fe_r, fe_w) = split(frontend_stream);
        let (fe_rem_r, mut fe_rem_w) = split(frontend_remote);
        let mut fe_reader = BufReader::new(fe_rem_r);

        let _conn_id = network
            .register_iroh_frontend(fe_r, fe_w)
            .await
            .expect("frontend registration succeeded");

        // Send a new game message through the frontend stream
        let new_game = Frontend2BackendMsg::NewGame {
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
        };
        let mut payload = serde_json::to_vec(&new_game).expect("serialize new game");
        payload.push(b'\n');
        fe_rem_w
            .write_all(&payload)
            .await
            .expect("write to frontend stream");
        fe_rem_w.flush().await.expect("flush frontend stream");

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
