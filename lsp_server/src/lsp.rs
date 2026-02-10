use std::collections::HashMap;

use front_end::parser::{CGDSLParser, Rule};
use front_end::ast::ast::SGame;
use crate::validation::validate_document;
use pest_consume::Parser;
use tokio::sync::Mutex;
use arc_swap::ArcSwapOption;
use std::sync::Arc;
use tower_lsp::jsonrpc::{self, Result};
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};

use crate::rule_completion::{try_auto_completion};

#[derive(Debug)]
pub struct Backend {
    pub client: Client,
    pub documents: Mutex<HashMap<Url, String>>, // stores current document text
    // No Mutex needed! ArcSwap handles thread-safety internally.
    pub last_ast: ArcSwapOption<SGame>, 
}


#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                // 1. How files are synced
                text_document_sync: Some(TextDocumentSyncCapability::Options(
                    TextDocumentSyncOptions {
                        open_close: Some(true),
                        change: Some(TextDocumentSyncKind::FULL),
                        save: Some(TextDocumentSyncSaveOptions::SaveOptions(SaveOptions {
                            include_text: Some(true),
                        })),
                        ..Default::default()
                    },
                )),

                // 2. Autocompletion configuration (Moves out of sync options)
                completion_provider: Some(CompletionOptions {
                    resolve_provider: Some(false),
                    trigger_characters: Some(vec![".".to_string(), ":".to_string(), " ".to_string()]),
                    all_commit_characters: None,
                    completion_item: None,
                    work_done_progress_options: Default::default(),
                }),

                ..Default::default()
            },
            ..Default::default()
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "server initialized!")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let uri = params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;

        let text = {
            let docs = self.documents.lock().await;
            docs.get(&uri).cloned().ok_or(jsonrpc::Error::internal_error())?
        };

        let result = CGDSLParser::parse(Rule::file, &text);

        match result {
            Err(err) => {
                // .load() gives you an atomic snapshot. No .await, no blocking.
                let last_ast_snapshot = self.last_ast.load();
                
                // last_ast_snapshot is a Guard that behaves like Option<Arc<FileAST>>
                Ok(try_auto_completion(err, position, &text, last_ast_snapshot.as_deref()))
            },
            Ok(mut nodes) => {
                if let Ok(ast) = CGDSLParser::file(nodes.next().unwrap()) {
                    // To update, we just "store" a new Arc. 
                    // This is an atomic pointer swap.
                    self.last_ast.store(Some(Arc::new(ast)));
                }
                Ok(None)
            }
        }
    }

    async fn hover(&self, _: HoverParams) -> Result<Option<Hover>> {
        Ok(Some(Hover {
            contents: HoverContents::Scalar(
                MarkedString::String("You're hovering!".to_string())
            ),
            range: None
        }))
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let mut docs = self.documents.lock().await;
        docs.insert(params.text_document.uri, params.text_document.text);
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        let uri = params.text_document.uri.clone();
        let docs = self.documents.lock().await;

        if let Some(content) = docs.get(&uri) {
            // Run validation (this should return an empty vec if no errors are found)
            let diagnostics = validate_document(content).unwrap_or_default();
            
            // This single call handles both clearing old errors (if vec is empty)
            // and showing new ones (if vec has items).
            self.client.publish_diagnostics(uri, diagnostics, None).await;
        }
    }

    async fn did_change(&self, _: DidChangeTextDocumentParams) {
        // let uri = params.text_document.uri;
        
        // Optional: For when it is optimized a lot (also works now but cant handle big files in the future)
        // // In FULL sync mode, the first item in content_changes is the entire doc
        // if let Some(change) = params.content_changes.into_iter().next() {
        //     let text = change.text;
            
        //     // 1. Update your internal cache so other features (hover/completions) work
        //     {
        //         let mut docs = self.documents.lock().await;
        //         docs.insert(uri.clone(), text.clone());
        //     }

        //     // 2. Run the diagnostics
        //     // Run validation (this should return an empty vec if no errors are found)
        //     let diagnostics = validate_document(&text).unwrap_or_default();
            
        //     // This single call handles both clearing old errors (if vec is empty)
        //     // and showing new ones (if vec has items).
        //     self.client.publish_diagnostics(uri, diagnostics, None).await;
        // }
    }
}