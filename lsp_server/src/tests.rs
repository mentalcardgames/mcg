#[tokio::test]
async fn test_initialization_flow() {
    use std::{collections::HashMap};
    use arc_swap::ArcSwapAny;
    use tokio::sync::{Mutex, mpsc};
    use dashmap::DashMap;
    use tower_lsp::{*};
    use crate::lsp::{Backend};

    // 1. Setup a real channel to intercept messages the server sends to the 'client'
    let (service, _) = LspService::build(|client| {
        Backend {
            client,
            documents: Mutex::new(HashMap::new()),
            last_ast: ArcSwapAny::new(None),
            analysis_tx: mpsc::unbounded_channel().0,
            symbol_table: DashMap::new(),
        }
    }).finish();

    let params = tower_lsp::lsp_types::InitializeParams::default();
    
    // 2. Call initialize directly on the service or the backend
    let response = service.inner().initialize(params).await.unwrap();

    // 3. Assertions
    assert!(response.capabilities.definition_provider.is_some());
    
    // Optional: Check if the server tried to log a "Server started" message
    // let log_msg = messages.next().await; 
}