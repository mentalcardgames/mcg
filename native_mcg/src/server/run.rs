use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use axum::Router;

use crate::config::Config;
use crate::controller::{start_controller, Controller, ControllerHandle, NetworkControllerSink};
use crate::network::{
    NetworkEvent, NetworkHandle, NetworkSupervisor, PeerConnectionService, RouterState,
};
use crate::server::bot_driver::spawn_bot_driver;
use anyhow::{Context, Result};
use tokio::sync::{mpsc, RwLock};

const NETWORK_EVENT_CHANNEL_CAPACITY: usize = 256;

struct NetworkTasks {
    controller_handle: ControllerHandle,
    supervisor: Mutex<Option<tokio::task::JoinHandle<()>>>,
    event_forwarder: Mutex<Option<tokio::task::JoinHandle<()>>>,
    bot_driver: Mutex<Option<tokio::task::JoinHandle<()>>>,
    controller_thread: Mutex<Option<std::thread::JoinHandle<()>>>,
}

impl NetworkTasks {
    async fn shutdown(&self) {
        let _ = self.controller_handle.shutdown().await;

        let bot_driver = self
            .bot_driver
            .lock()
            .expect("bot driver task lock poisoned")
            .take();
        if let Some(bot_driver) = bot_driver {
            bot_driver.abort();
        }

        let supervisor = self
            .supervisor
            .lock()
            .expect("network supervisor task lock poisoned")
            .take();
        if let Some(supervisor) = supervisor {
            if let Err(error) = supervisor.await {
                tracing::error!(%error, "network supervisor task failed during shutdown");
            }
        }

        let event_forwarder = self
            .event_forwarder
            .lock()
            .expect("event forwarder task lock poisoned")
            .take();
        if let Some(event_forwarder) = event_forwarder {
            if let Err(error) = event_forwarder.await {
                tracing::error!(%error, "network event forwarder failed during shutdown");
            }
        }

        let controller_thread = self
            .controller_thread
            .lock()
            .expect("controller thread lock poisoned")
            .take();
        if let Some(controller_thread) = controller_thread {
            if let Err(error) = controller_thread.join() {
                tracing::error!(?error, "controller thread panicked during shutdown");
            }
        }
    }
}

impl Drop for NetworkTasks {
    fn drop(&mut self) {
        if let Some(bot_driver) = self
            .bot_driver
            .get_mut()
            .expect("bot driver task lock poisoned")
            .take()
        {
            bot_driver.abort();
        }
        if let Some(supervisor) = self
            .supervisor
            .get_mut()
            .expect("network supervisor task lock poisoned")
            .take()
        {
            supervisor.abort();
        }
        if let Some(event_forwarder) = self
            .event_forwarder
            .get_mut()
            .expect("event forwarder task lock poisoned")
            .take()
        {
            event_forwarder.abort();
        }
    }
}

struct RunningNetwork {
    controller: ControllerHandle,
    network: NetworkHandle,
    peer_connections: PeerConnectionService,
    local_ticket: Arc<RwLock<Option<String>>>,
    network_tasks: Arc<NetworkTasks>,
}

fn start_network(config: Config, config_path: Option<PathBuf>) -> RunningNetwork {
    let local_ticket = Arc::new(RwLock::new(None));
    let (event_tx, mut event_rx) = mpsc::channel::<NetworkEvent>(NETWORK_EVENT_CHANNEL_CAPACITY);
    let supervisor = NetworkSupervisor::new(event_tx);
    let (network, supervisor_task) = supervisor.start();
    let peer_connections = PeerConnectionService::new(local_ticket.clone(), network.clone());

    let (state_watch_tx, state_watch_rx) = tokio::sync::watch::channel(None);
    let sink = NetworkControllerSink::with_state_watch(network.clone(), state_watch_tx);
    let controller = Controller::new(config.clone(), config_path.clone());
    let (controller_thread, controller_handle) = start_controller(controller, 256, sink);

    let event_forwarder = tokio::spawn({
        let controller_handle = controller_handle.clone();
        let peer_connections = peer_connections.clone();
        async move {
            while let Some(event) = event_rx.recv().await {
                match &event {
                    NetworkEvent::PeerConnected {
                        connection_id,
                        peer_id,
                        direction,
                        ..
                    } => {
                        peer_connections
                            .connection_opened(*connection_id, peer_id.clone(), *direction)
                            .await;
                    }
                    NetworkEvent::ConnectionClosed { connection_id, .. } => {
                        peer_connections.connection_closed(*connection_id).await;
                    }
                    _ => {}
                }

                if controller_handle.send_network_event(event).await.is_err() {
                    tracing::debug!(
                        "controller event receiver closed; stopping network event forwarder"
                    );
                    break;
                }
            }
        }
    });
    let bot_delay_range = config.bot_delay_range();
    let bot_driver = spawn_bot_driver(
        controller_handle.clone(),
        state_watch_rx,
        crate::bot::BotManager::new(),
        bot_delay_range,
    );

    RunningNetwork {
        controller: controller_handle.clone(),
        network,
        peer_connections,
        local_ticket,
        network_tasks: Arc::new(NetworkTasks {
            controller_handle,
            supervisor: Mutex::new(Some(supervisor_task)),
            event_forwarder: Mutex::new(Some(event_forwarder)),
            bot_driver: Mutex::new(Some(bot_driver)),
            controller_thread: Mutex::new(Some(controller_thread)),
        }),
    }
}

pub fn build_router(config: Config, config_path: Option<PathBuf>) -> Router {
    let RunningNetwork {
        controller,
        network,
        peer_connections,
        network_tasks,
        ..
    } = start_network(config, config_path);
    let router_state =
        RouterState::new(controller, network, peer_connections).with_task_guard(network_tasks);
    crate::network::build_router(router_state)
}

pub async fn run_server(
    addr: SocketAddr,
    config: Config,
    config_path: Option<PathBuf>,
) -> Result<()> {
    let RunningNetwork {
        controller,
        network,
        peer_connections,
        local_ticket,
        network_tasks,
    } = start_network(config.clone(), config_path.clone());
    let router_state = RouterState::new(controller, network.clone(), peer_connections);
    let app = crate::network::build_router(router_state);

    let display_addr = if addr.ip().is_loopback() {
        format!("localhost:{}", addr.port())
    } else {
        addr.to_string()
    };

    tracing::info!(display_addr = %display_addr, "MCG Server running");

    // Nice clickable banner for the Web UI
    println!("\n\x1b[1;36m=== Web UI Available ===\x1b[0m");
    println!(
        "\x1b[1mURL:\x1b[0m       \x1b[4;34mhttp://{}\x1b[0m",
        display_addr
    );
    println!("\x1b[1;36m========================\x1b[0m\n");

    tracing::info!("open your browser and navigate to the above URL");
    tracing::debug!("blank line");
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .with_context(|| format!("Failed to bind to {}", display_addr))?;
    // The owned task starts Iroh concurrently and is shut down after Axum stops.
    let iroh_listener =
        crate::network::spawn_iroh_listener(config, config_path, local_ticket, network.clone());
    let server_result = axum::serve(listener, app).await;
    iroh_listener.shutdown().await;
    if let Err(error) = network.shutdown().await {
        tracing::warn!(%error, "network supervisor stopped before server shutdown");
    }
    network_tasks.shutdown().await;
    server_result.context("running HTTP/WebSocket server")?;
    Ok(())
}
