use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use anyhow::{Context, Result};
use axum::Router;

use crate::config::Config;
use crate::controller::{spawn_controller, Controller, ControllerHandle};
use crate::network::{NetworkHandle, NetworkSupervisor, RouterState};
use crate::server::bot_driver::spawn_bot_driver;

/// Holds task handles for the background supervisor, bot driver, and controller thread.
pub struct BackendTasks {
    controller_handle: ControllerHandle,
    supervisor: Mutex<Option<tokio::task::JoinHandle<()>>>,
    bot_driver: Mutex<Option<tokio::task::JoinHandle<()>>>,
    controller_thread: Mutex<Option<std::thread::JoinHandle<()>>>,
}

impl BackendTasks {
    /// Asynchronously signals shutdown to controller and gracefully stops background tasks.
    pub async fn shutdown(&self) {
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
    /// Returns a reference to the active [`NetworkHandle`].
    pub fn network(&self) -> &NetworkHandle {
        &self.network
    }

    /// Returns a reference to the active [`ControllerHandle`].
    pub fn controller(&self) -> &ControllerHandle {
        &self.controller_handle
    }

    /// Returns a reference to the background tasks manager.
    pub fn tasks(&self) -> &Arc<BackendTasks> {
        &self.tasks
    }

    /// Constructs the Axum [`Router`] wired with the backend's [`NetworkHandle`] and task guard.
    pub fn router(&self) -> Router {
        let router_state =
            RouterState::new(self.network.clone()).with_task_guard(self.tasks.clone());
        crate::network::build_router(router_state)
    }

    /// Runs the HTTP/WebSocket server on `addr` and supervises the Iroh listener until shutdown.
    pub async fn run(self, addr: SocketAddr) -> Result<()> {
        let app = self.router();

        let display_addr = if addr.ip().is_loopback() {
            format!("localhost:{}", addr.port())
        } else {
            addr.to_string()
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
        let listener = tokio::net::TcpListener::bind(addr)
            .await
            .with_context(|| format!("Failed to bind to {}", display_addr))?;

        self.network
            .start_iroh_listener(self.config, self.config_path)
            .await
            .context("starting Iroh listener")?;

        let server_result = axum::serve(listener, app).await;
        if let Err(error) = self.network.shutdown().await {
            tracing::warn!(%error, "network supervisor stopped before server shutdown");
        }
        self.tasks.shutdown().await;
        server_result.context("running HTTP/WebSocket server")?;
        Ok(())
    }

    /// Gracefully shuts down all backend tasks.
    pub async fn shutdown(&self) {
        self.tasks.shutdown().await;
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

        let (controller, controller_handle, controller_rx) =
            Controller::builder(self.config.clone())
                .with_config_path_opt(self.config_path.clone())
                .with_state_watch(state_watch_tx)
                .with_channel_capacity(self.channel_capacity)
                .build();

        let (network, supervisor_task) =
            NetworkSupervisor::builder(controller_handle.sender()).spawn();

        let controller_thread = spawn_controller(controller, controller_rx, network.clone());

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
            controller_handle: controller_handle.clone(),
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
