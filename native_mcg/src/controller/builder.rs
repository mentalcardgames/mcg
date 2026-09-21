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

/// Builder for configuring the synchronous [`Controller`].
pub struct ControllerBuilder {
    config: Config,
    config_path: Option<PathBuf>,
    network: Option<NetworkHandle>,
    state_watch_tx: Option<watch::Sender<Option<PokerStatePublic>>>,
    channel_capacity: usize,
}

impl ControllerBuilder {
    /// Creates a new builder with the specified configuration.
    pub fn new(config: Config) -> Self {
        Self {
            config,
            config_path: None,
            network: None,
            state_watch_tx: None,
            channel_capacity: DEFAULT_CONTROLLER_CHANNEL_CAPACITY,
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
        self
    }

    /// Builds the controller runner and its corresponding [`ControllerHandle`].
    ///
    /// This step sets up the event channel without starting any background threads.
    pub fn build(self) -> (ControllerRunner, ControllerHandle) {
        let (tx, rx) = mpsc::channel(self.channel_capacity);
        let handle = ControllerHandle::new(tx);
        let mut controller = Controller::new(self.config, self.config_path);
        if let Some(watch_tx) = self.state_watch_tx {
            controller = controller.with_state_watch(watch_tx);
        }
        let runner = ControllerRunner {
            controller,
            rx,
            network: self.network,
        };
        (runner, handle)
    }

    /// Spawns the dedicated `mcg-controller` OS thread directly.
    ///
    /// Requires [`Self::with_network`] to have been called.
    pub fn spawn(mut self) -> (ControllerHandle, JoinHandle<()>) {
        let network = self
            .network
            .take()
            .expect("NetworkHandle must be provided to spawn Controller thread");
        let (runner, handle) = self.build();
        let thread_handle = runner.spawn(network);
        (handle, thread_handle)
    }
}

/// Unstarted runner owning the [`Controller`] and event receiver.
pub struct ControllerRunner {
    controller: Controller,
    rx: mpsc::Receiver<ControllerEvent>,
    network: Option<NetworkHandle>,
}

impl ControllerRunner {
    /// Spawns the dedicated `mcg-controller` OS thread with the given [`NetworkHandle`].
    pub fn spawn(mut self, network: NetworkHandle) -> JoinHandle<()> {
        self.network = Some(network);
        self.spawn_thread()
    }

    /// Spawns the dedicated `mcg-controller` OS thread using the pre-configured [`NetworkHandle`].
    pub fn spawn_with_network(self) -> JoinHandle<()> {
        self.spawn_thread()
    }

    fn spawn_thread(mut self) -> JoinHandle<()> {
        let network = self
            .network
            .take()
            .expect("NetworkHandle must be set to spawn Controller thread");
        std::thread::Builder::new()
            .name("mcg-controller".into())
            .spawn(move || {
                self.run(&network);
            })
            .expect("spawning controller OS thread")
    }

    /// Runs the controller loop synchronously on the current thread.
    pub fn run(&mut self, network: &NetworkHandle) {
        tracing::info!("synchronous controller thread started");
        while let Some(event) = self.rx.blocking_recv() {
            let is_shutdown = matches!(event, ControllerEvent::Shutdown);
            self.controller.handle_event(event, network);
            if is_shutdown {
                break;
            }
        }
        tracing::info!("synchronous controller thread stopped");
    }

    /// Consumes the runner and returns the underlying event receiver.
    ///
    /// Useful for testing network supervisor behavior in isolation.
    pub fn into_event_receiver(self) -> mpsc::Receiver<ControllerEvent> {
        self.rx
    }
}
