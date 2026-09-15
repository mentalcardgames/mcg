use std::path::PathBuf;
use std::thread::JoinHandle;

use mcg_shared::PokerStatePublic;
use tokio::sync::{mpsc, watch};

use crate::config::Config;
use crate::network::NetworkHandle;

use super::core::Controller;
use super::handle::ControllerHandle;
use super::runner::spawn_controller;
use super::types::ControllerEvent;

const DEFAULT_CONTROLLER_CHANNEL_CAPACITY: usize = 256;

/// Builder for configuring and spawning the synchronous [`Controller`].
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

    /// Builds the controller, handle, and event receiver without spawning a thread.
    pub fn build(
        self,
    ) -> (
        Controller,
        ControllerHandle,
        mpsc::Receiver<ControllerEvent>,
    ) {
        let (tx, rx) = mpsc::channel(self.channel_capacity);
        let handle = ControllerHandle::new(tx);
        let mut controller = Controller::new(self.config, self.config_path);
        if let Some(watch_tx) = self.state_watch_tx {
            controller = controller.with_state_watch(watch_tx);
        }
        (controller, handle, rx)
    }

    /// Spawns the dedicated `mcg-controller` OS thread and returns both the [`ControllerHandle`] and thread [`JoinHandle`].
    pub fn spawn(mut self) -> (ControllerHandle, JoinHandle<()>) {
        let network = self
            .network
            .take()
            .expect("NetworkHandle must be provided to spawn Controller thread");
        let (controller, handle, rx) = self.build();
        let thread_handle = spawn_controller(controller, rx, network);
        (handle, thread_handle)
    }
}
