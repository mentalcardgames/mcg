use axum::{extract::State, Json};
use crate::server::AppState;

pub async fn get_iroh_node_id(State(state): State<AppState>) -> Json<serde_json::Value> {
    Json(serde_json::json!({ "node_id": state.transport_node_id }))
}
