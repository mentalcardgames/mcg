use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::PathBuf;
use std::sync::Mutex;

use anyhow::{Context, Result};

use crate::config::Config;
use crate::controller::{ControllerBuilder, ControllerHandle};
use crate::network::{NetworkBuilder, NetworkHandle};
use mcg_poker::driver::spawn_bot_driver;

/// Holds task handles for the background supervisor, bot driver, and controller thread.
pub struct BackendTasks {
    supervisor: Mutex<Option<tokio::task::JoinHandle<()>>>,
    bot_driver: Mutex<Option<tokio::task::JoinHandle<()>>>,
    controller_thread: Mutex<Option<std::thread::JoinHandle<()>>>,
}

impl BackendTasks {
    /// Gracefully aborts or joins remaining tasks.
    pub async fn shutdown(&self) {
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

/// A cloneable handle to an actively running backend instance.
#[derive(Clone)]
pub struct BackendHandle {
    config: Config,
    config_path: Option<PathBuf>,
    network: NetworkHandle,
    controller_handle: ControllerHandle,
    bind_addr: SocketAddr,
}

impl BackendHandle {
    pub(super) fn new(
        config: Config,
        config_path: Option<PathBuf>,
        network: NetworkHandle,
        controller_handle: ControllerHandle,
        bind_addr: SocketAddr,
    ) -> Self {
        Self {
            config,
            config_path,
            network,
            controller_handle,
            bind_addr,
        }
    }

    /// Returns a clone of the active [`NetworkHandle`].
    pub fn network(&self) -> NetworkHandle {
        self.network.clone()
    }

    /// Returns a clone of the active [`ControllerHandle`].
    pub fn controller(&self) -> ControllerHandle {
        self.controller_handle.clone()
    }

    /// Returns the resolved bind address for the HTTP/WebSocket listener.
    pub fn bind_addr(&self) -> SocketAddr {
        self.bind_addr
    }

    /// Gracefully sends shutdown signals to the controller and network supervisor.
    pub async fn shutdown(&self) {
        let _ = self.controller_handle.shutdown().await;
        if let Err(error) = self.network.shutdown().await {
            tracing::warn!(%error, "network supervisor stopped before server shutdown");
        }
    }
}

/// An unstarted backend instance holding configuration and resolved network bindings.
pub struct Backend {
    config: Config,
    config_path: Option<PathBuf>,
    channel_capacity: usize,
    enable_bots: bool,
    bind_addr: SocketAddr,
}

impl Backend {
    /// Returns the resolved socket address the backend will listen on.
    pub fn bind_addr(&self) -> SocketAddr {
        self.bind_addr
    }

    /// Spawns all background components (controller thread, network supervisor, bot driver)
    /// without starting HTTP/Iroh listeners.
    pub fn spawn(self) -> Result<(BackendHandle, BackendTasks)> {
        let (state_watch_tx, state_watch_rx) = tokio::sync::watch::channel(None);

        let (controller_runner, controller_handle) = ControllerBuilder::new(self.config.clone())
            .with_config_path_opt(self.config_path.clone())
            .with_state_watch(state_watch_tx)
            .with_channel_capacity(self.channel_capacity)
            .build();

        let (network, supervisor_task) = NetworkBuilder::new(controller_handle.clone()).spawn();

        let controller_thread = controller_runner.spawn(network.clone());

        let bot_driver = if self.enable_bots {
            let bot_delay_range = self.config.bot_delay_range();
            let driver_task = spawn_bot_driver(
                controller_handle.clone(),
                state_watch_rx,
                mcg_poker::bot::BotManager::new(),
                bot_delay_range,
            );
            Some(driver_task)
        } else {
            None
        };

        let tasks = BackendTasks {
            supervisor: Mutex::new(Some(supervisor_task)),
            bot_driver: Mutex::new(bot_driver),
            controller_thread: Mutex::new(Some(controller_thread)),
        };

        let handle = BackendHandle::new(
            self.config,
            self.config_path,
            network,
            controller_handle,
            self.bind_addr,
        );

        Ok((handle, tasks))
    }

    /// Runs the HTTP/WebSocket server and supervises the Iroh listener until `shutdown_signal` completes.
    pub async fn run_until(
        self,
        shutdown_signal: impl std::future::Future<Output = ()>,
    ) -> Result<()> {
        let bind_addr = self.bind_addr;
        let (handle, tasks) = self.spawn()?;

        let bound_addr = handle
            .network
            .start_axum_listener(bind_addr)
            .await
            .context("starting Axum HTTP/WebSocket listener")?;

        // Clickable banner for the Web UI
        println!("\n\x1b[1;36m=== Web UI Available ===\x1b[0m");
        println!(
            "\x1b[1mURL:\x1b[0m       \x1b[4;34mhttp://{}\x1b[0m",
            bound_addr
        );
        println!("\x1b[1;36m========================\x1b[0m\n");

        tracing::info!("open your browser and navigate to the above URL");
        tracing::debug!("blank line");

        handle
            .network
            .start_iroh_listener(handle.config.clone(), handle.config_path.clone())
            .await
            .context("starting Iroh listener")?;

        shutdown_signal.await;
        tracing::info!("Shutdown signal received, shutting down backend...");
        handle.shutdown().await;
        tasks.shutdown().await;
        Ok(())
    }

    /// Runs the HTTP/WebSocket server and supervises the Iroh listener until a Ctrl+C signal is received.
    pub async fn run(self) -> Result<()> {
        self.run_until(async {
            let _ = tokio::signal::ctrl_c().await;
        })
        .await
    }
}

/// Builder for orchestrating backend configuration before startup.
pub struct BackendBuilder {
    config: Config,
    config_path: Option<PathBuf>,
    channel_capacity: usize,
    enable_bots: bool,
    bind_host: IpAddr,
    port: Option<u16>,
    strict_port: Option<bool>,
    bind_addr: Option<SocketAddr>,
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
            bind_host: IpAddr::V4(Ipv4Addr::UNSPECIFIED),
            port: None,
            strict_port: None,
            bind_addr: None,
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

    /// Overrides the port to listen on.
    pub fn with_port(mut self, port: u16) -> Self {
        self.port = Some(port);
        self
    }

    /// Sets whether to fail strictly if the port is unavailable instead of searching for the next free port.
    pub fn with_strict_port(mut self, strict: bool) -> Self {
        self.strict_port = Some(strict);
        self
    }

    /// Overrides the host IP address to bind to (defaults to 0.0.0.0).
    pub fn with_bind_host(mut self, host: impl Into<IpAddr>) -> Self {
        self.bind_host = host.into();
        self
    }

    /// Explicitly sets the full bind address, bypassing port search.
    pub fn with_bind_addr(mut self, addr: SocketAddr) -> Self {
        self.bind_addr = Some(addr);
        self
    }

    /// Builds the unstarted [`Backend`] instance, resolving socket bindings without starting tasks.
    pub fn build(self) -> Result<Backend> {
        let port = self.port.unwrap_or(self.config.port);
        let strict_port = self.strict_port.unwrap_or(self.config.strict_port);
        let bind_addr = match self.bind_addr {
            Some(addr) => addr,
            None => resolve_listen_addr(self.bind_host, port, strict_port)?,
        };

        Ok(Backend {
            config: self.config,
            config_path: self.config_path,
            channel_capacity: self.channel_capacity,
            enable_bots: self.enable_bots,
            bind_addr,
        })
    }

    /// Convenience method to build and spawn the backend in one step.
    pub fn spawn(self) -> Result<(BackendHandle, BackendTasks)> {
        self.build()?.spawn()
    }
}

/// Resolves the socket address to listen on, searching for the first available port if not in strict mode.
fn resolve_listen_addr(host: IpAddr, start_port: u16, strict: bool) -> Result<SocketAddr> {
    if start_port == 0 {
        return Ok(SocketAddr::new(host, 0));
    }

    if strict {
        match std::net::TcpListener::bind((host, start_port)) {
            Ok(_) => Ok(SocketAddr::new(host, start_port)),
            Err(e) => Err(anyhow::anyhow!(
                "Port {} is already in use or cannot be bound (strict mode enabled): {}",
                start_port,
                e
            )),
        }
    } else {
        for port in start_port..start_port.saturating_add(100) {
            if std::net::TcpListener::bind((host, port)).is_ok() {
                if port != start_port {
                    tracing::warn!(
                        requested_port = start_port,
                        selected_port = port,
                        "Port {} was not available, using alternative port {}",
                        start_port,
                        port
                    );
                }
                return Ok(SocketAddr::new(host, port));
            }
        }
        Err(anyhow::anyhow!(
            "No available ports found in range {}..{}",
            start_port,
            start_port.saturating_add(100)
        ))
    }
}
