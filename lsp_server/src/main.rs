pub mod lsp;
pub mod error_diagnostics;
pub mod rule_completion;
pub mod validation;

use std::{collections::HashMap};
use arc_swap::ArcSwapAny;
use tokio::sync::Mutex;

use tower_lsp::*;
use crate::lsp::{Backend};

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| Backend { client: client, documents: Mutex::new(HashMap::new()), last_ast: ArcSwapAny::new(None) });
    Server::new(stdin, stdout, socket).serve(service).await;
}
