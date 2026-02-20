pub mod lsp;
pub mod error_to_diagnostics;
pub mod completion;
pub mod validation;
pub mod rope;
pub mod semantic_highlighting;
pub mod tests;

use std::{collections::HashMap, sync::Arc, time::Duration};
use arc_swap::ArcSwapAny;
use tokio::sync::{Mutex, mpsc};
use dashmap::DashMap;
use tower_lsp::{lsp_types::Url, *};
use crate::lsp::{Backend};

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    // 1. Create the channel for URIs that need re-analysis
    let (tx, mut rx) = mpsc::unbounded_channel::<Url>();

    let (service, socket) = LspService::build(|client| {
        // 2. Initialize the backend
        let backend = Arc::new(Backend {
            client,
            documents: Mutex::new(HashMap::new()),
            last_ast: ArcSwapAny::new(None),
            analysis_tx: tx,
            symbol_table: DashMap::new()
        });

        // 3. Spawn the background worker task
        let worker_backend = Arc::clone(&backend);
        tokio::spawn(async move {
            // This map tracks the latest "version" or "intent" to avoid redundant work
            while let Some(uri) = rx.recv().await {
                // DEBOUNCE: Wait for the user to stop typing
                tokio::time::sleep(Duration::from_millis(200)).await;

                // Clear out any "stale" requests for the same URI that arrived 
                // while we were sleeping.
                while let Ok(_) = rx.try_recv() { 
                    /* Just draining the pipe to get to the freshest state */ 
                }

                // Execute the heavy analysis
                worker_backend.run_analysis(uri).await;
            }
        });

        backend
    })
    .finish();

    Server::new(stdin, stdout, socket).serve(service).await;
}
