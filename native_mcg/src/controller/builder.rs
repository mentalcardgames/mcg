use std::path::PathBuf;
use std::thread::JoinHandle;

use mcg_shared::PokerStatePublic;
use tokio::sync::{mpsc, watch};

use crate::config::Config;
use crate::network::NetworkHandle;

use super::core::Controller;
use super::handle::ControllerHandle;
use super::types::ControllerEvent;

const DEFAULT_CONTROLLER_CHANNEL_CAPACITY: usize = 256;

/// Builder for configuring and spawning the synchronous [`Controller`].
pub struct ControllerBuilder {
    config: Config,
    config_path: Option<PathBuf>,
    network: Option<NetworkHandle>,
    state_watch_tx: Option<watch::Sender<Option<PokerStatePublic>>>,
    channel_capacity: usize,
    handle: ControllerHandle,
    rx: Option<mpsc::Receiver<ControllerEvent>>,
}

impl ControllerBuilder {
    /// Creates a new builder with the specified configuration.
    pub fn new(config: Config) -> Self {
        let (tx, rx) = mpsc::channel(DEFAULT_CONTROLLER_CHANNEL_CAPACITY);
        Self {
            config,
            config_path: None,
            network: None,
            state_watch_tx: None,
            channel_capacity: DEFAULT_CONTROLLER_CHANNEL_CAPACITY,
            handle: ControllerHandle::new(tx),
            rx: Some(rx),
        }
    }

    /// Sets the path to the configuration file, used for writing public info.
    pub fn with_config_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.config_path = Some(path.into());
        self
    }

    /// Sets the optional path to the configuration file.
    pub fn with_config_path_opt(mut self, path: Option<PathBuf>) -> Self {
        self.config_path = path;
        self
    }

    /// Sets the [`NetworkHandle`] used by the controller to communicate with the network shell.
    pub fn with_network(mut self, network: NetworkHandle) -> Self {
        self.network = Some(network);
        self
    }

    /// Attaches a public state watch sender for bot observation.
    pub fn with_state_watch(
        mut self,
        state_watch_tx: watch::Sender<Option<PokerStatePublic>>,
    ) -> Self {
        self.state_watch_tx = Some(state_watch_tx);
        self
    }

    /// Sets the capacity of the [`ControllerEvent`] mpsc channel.
    pub fn with_channel_capacity(mut self, capacity: usize) -> Self {
        self.channel_capacity = capacity;
        let (tx, rx) = mpsc::channel(capacity);
        self.handle = ControllerHandle::new(tx);
        self.rx = Some(rx);
        self
    }

    /// Returns the [`ControllerHandle`] for sending events to the Controller.
    pub fn handle(&self) -> ControllerHandle {
        self.handle.clone()
    }

    /// Spawns the dedicated `mcg-controller` OS thread and returns both the [`ControllerHandle`] and thread [`JoinHandle`].
    pub fn spawn(mut self) -> (ControllerHandle, JoinHandle<()>) {
        let network = self
            .network
            .take()
            .expect("NetworkHandle must be provided to spawn Controller thread");
        let handle = self.handle();
        let mut rx = self.rx.take().expect("Controller already spawned");
        let mut controller = Controller::new(self.config, self.config_path);
        if let Some(watch_tx) = self.state_watch_tx {
            controller = controller.with_state_watch(watch_tx);
        }
        let thread_handle = std::thread::Builder::new()
            .name("mcg-controller".into())
            .spawn(move || {
                tracing::info!("synchronous controller thread started");
                while let Some(event) = rx.blocking_recv() {
                    let is_shutdown = matches!(event, ControllerEvent::Shutdown);
                    controller.handle_event(event, &network);
                    if is_shutdown {
                        break;
                    }
                }
                tracing::info!("synchronous controller thread stopped");
            })
            .expect("spawning controller OS thread");
        (handle, thread_handle)
    }
}
