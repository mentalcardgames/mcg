use axum::{extract::ws::WebSocketUpgrade, extract::State, response::IntoResponse};

use crate::network::NetworkHandle;

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(network): State<NetworkHandle>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| async move {
        if let Err(error) = network.register_websocket(socket).await {
            tracing::error!(%error, "failed to register WebSocket connection");
        }
    })
}
