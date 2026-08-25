use std::thread::{Builder, JoinHandle};
use tokio::sync::mpsc;

use super::core::Controller;
use super::handle::ControllerHandle;
use super::sink::ControllerSink;
use super::types::ControllerEvent;

/// Spawns the Controller in a dedicated standard OS thread and returns its thread [`JoinHandle`].
pub fn spawn_controller<S: ControllerSink + 'static>(
    mut controller: Controller,
    mut event_rx: mpsc::Receiver<ControllerEvent>,
    mut sink: S,
) -> JoinHandle<()> {
    Builder::new()
        .name("mcg-controller".into())
        .spawn(move || {
            tracing::info!("synchronous controller thread started");
            while let Some(event) = event_rx.blocking_recv() {
                let is_shutdown = matches!(event, ControllerEvent::Shutdown);
                controller.handle_event(event, &mut sink);
                if is_shutdown {
                    break;
                }
            }
            tracing::info!("synchronous controller thread stopped");
        })
        .expect("spawning controller OS thread")
}

/// Helper function to create an event channel, spawn the controller thread, and return both the thread [`JoinHandle`] and a cloneable [`ControllerHandle`].
pub fn start_controller<S: ControllerSink + 'static>(
    controller: Controller,
    channel_capacity: usize,
    sink: S,
) -> (JoinHandle<()>, ControllerHandle) {
    let (event_tx, event_rx) = mpsc::channel(channel_capacity);
    let handle = ControllerHandle::new(event_tx);
    let thread_handle = spawn_controller(controller, event_rx, sink);
    (thread_handle, handle)
}
