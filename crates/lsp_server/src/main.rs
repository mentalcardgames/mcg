// Copyright 2026 Till Hoffmann
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

pub mod completion;
pub mod error_to_diagnostics;
pub mod lsp;
pub mod rope;
pub mod semantic_highlighting;
pub mod tests;
pub mod validation;

use crate::lsp::Backend;
use arc_swap::ArcSwapAny;
use dashmap::DashMap;
use std::{collections::HashMap, sync::Arc, time::Duration};
use tokio::sync::{Mutex, mpsc};
use tower_lsp::{lsp_types::Url, *};

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
            symbol_table: DashMap::new(),
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
