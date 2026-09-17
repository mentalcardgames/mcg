use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use anyhow::{Context, Result};

use crate::config::Config;
use crate::controller::{ControllerBuilder, ControllerHandle};
use crate::network::{NetworkBuilder, NetworkHandle};
use crate::server::bot_driver::spawn_bot_driver;

/// Holds task handles for the background supervisor, bot driver, and controller thread.
pub struct BackendTasks {
    supervisor: Mutex<Option<tokio::task::JoinHandle<()>>>,
    bot_driver: Mutex<Option<tokio::task::JoinHandle<()>>>,
    controller_thread: Mutex<Option<std::thread::JoinHandle<()>>>,
}

impl Drop for BackendTasks {
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
    }
}

/// A running backend instance encapsulating the network handle, controller handle, and background tasks.
pub struct RunningBackend {
    config: Config,
    config_path: Option<PathBuf>,
    network: NetworkHandle,
    controller_handle: ControllerHandle,
    tasks: Arc<BackendTasks>,
}

impl RunningBackend {
    /// Returns a clone of the active [`NetworkHandle`].
    pub fn network(&self) -> NetworkHandle {
        self.network.clone()
    }

    /// Returns a clone of the active [`ControllerHandle`].
    pub fn controller(&self) -> ControllerHandle {
        self.controller_handle.clone()
    }

    /// Runs the HTTP/WebSocket server on `addr` and supervises the Iroh listener until `shutdown_signal` completes.
    pub async fn run_until(
        self,
        addr: SocketAddr,
        shutdown_signal: impl std::future::Future<Output = ()>,
    ) -> Result<()> {
        let bound_addr = self
            .network
            .start_axum_listener(addr)
            .await
            .context("starting Axum HTTP/WebSocket listener")?;

        let display_addr = if bound_addr.ip().is_loopback() {
            format!("localhost:{}", bound_addr.port())
        } else {
            bound_addr.to_string()
        };

        tracing::info!(display_addr = %display_addr, "MCG Server running");

        // Clickable banner for the Web UI
        println!("\n\x1b[1;36m=== Web UI Available ===\x1b[0m");
        println!(
            "\x1b[1mURL:\x1b[0m       \x1b[4;34mhttp://{}\x1b[0m",
            display_addr
        );
        println!("\x1b[1;36m========================\x1b[0m\n");

        tracing::info!("open your browser and navigate to the above URL");
        tracing::debug!("blank line");

        self.network
            .start_iroh_listener(self.config.clone(), self.config_path.clone())
            .await
            .context("starting Iroh listener")?;

        shutdown_signal.await;
        tracing::info!("Shutdown signal received, shutting down backend...");
        self.shutdown().await;
        Ok(())
    }

    /// Runs the HTTP/WebSocket server on `addr` and supervises the Iroh listener until a Ctrl+C signal is received.
    pub async fn run(self, addr: SocketAddr) -> Result<()> {
        self.run_until(addr, async {
            let _ = tokio::signal::ctrl_c().await;
        })
        .await
    }

    /// Gracefully shuts down all backend tasks.
    pub async fn shutdown(&self) {
        let _ = self.controller_handle.shutdown().await;

        let bot_driver = self
            .tasks
            .bot_driver
            .lock()
            .expect("bot driver task lock poisoned")
            .take();
        if let Some(bot_driver) = bot_driver {
            bot_driver.abort();
        }

        if let Err(error) = self.network.shutdown().await {
            tracing::warn!(%error, "network supervisor stopped before server shutdown");
        }

        let supervisor = self
            .tasks
            .supervisor
            .lock()
            .expect("network supervisor task lock poisoned")
            .take();
        if let Some(supervisor) = supervisor {
            if let Err(error) = supervisor.await {
                tracing::error!(%error, "network supervisor task failed during shutdown");
            }
        }

        let controller_thread = self
            .tasks
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

/// Builder for orchestrating backend startup including network supervision, controller thread, and bots.
pub struct BackendBuilder {
    config: Config,
    config_path: Option<PathBuf>,
    channel_capacity: usize,
    enable_bots: bool,
}

impl BackendBuilder {
    /// Creates a new builder with the given server configuration.
    pub fn new(config: Config) -> Self {
        let enable_bots = config.bots > 0;
        Self {
            config,
            config_path: None,
            channel_capacity: 256,
            enable_bots,
        }
    }

    /// Sets the configuration file path.
    pub fn with_config_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.config_path = Some(path.into());
        self
    }

    /// Sets the optional configuration file path.
    pub fn with_config_path_opt(mut self, path: Option<PathBuf>) -> Self {
        self.config_path = path;
        self
    }

    /// Sets the channel capacity for internal events.
    pub fn with_channel_capacity(mut self, capacity: usize) -> Self {
        self.channel_capacity = capacity;
        self
    }

    /// Explicitly enables or disables the bot driver task.
    pub fn with_bots(mut self, enable: bool) -> Self {
        self.enable_bots = enable;
        self
    }

    /// Builds and starts all backend components, returning a [`RunningBackend`].
    pub fn build(self) -> Result<RunningBackend> {
        let (state_watch_tx, state_watch_rx) = tokio::sync::watch::channel(None);

        let controller_builder = ControllerBuilder::new(self.config.clone())
            .with_config_path_opt(self.config_path.clone())
            .with_state_watch(state_watch_tx)
            .with_channel_capacity(self.channel_capacity);

        let controller_handle = controller_builder.handle();

        let (network, supervisor_task) = NetworkBuilder::new(controller_handle.clone()).spawn();

        let (_, controller_thread) = controller_builder.with_network(network.clone()).spawn();

        let bot_driver = if self.enable_bots {
            let bot_delay_range = self.config.bot_delay_range();
            let driver_task = spawn_bot_driver(
                controller_handle.clone(),
                state_watch_rx,
                crate::bot::BotManager::new(),
                bot_delay_range,
            );
            Some(driver_task)
        } else {
            None
        };

        let tasks = Arc::new(BackendTasks {
            supervisor: Mutex::new(Some(supervisor_task)),
            bot_driver: Mutex::new(bot_driver),
            controller_thread: Mutex::new(Some(controller_thread)),
        });

        Ok(RunningBackend {
            config: self.config,
            config_path: self.config_path,
            network,
            controller_handle,
            tasks,
        })
    }
}
