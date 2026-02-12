use std::collections::HashMap;

use front_end::ast::ast::SGame;
use front_end::validation::parse_document;
use ropey::{Rope};
use crate::rope::Document;
use crate::semantic_highlighting::{calculate_deltas, tokenize_ast};
use crate::validation::{apply_change, validate_document, validate_parsing};
use tokio::sync::Mutex;
use arc_swap::ArcSwapOption;
use std::sync::Arc;
use tower_lsp::jsonrpc::{self, Error, Result};
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};

use crate::rule_completion::{try_auto_completion};

#[derive(Debug)]
pub struct Backend {
    pub client: Client,
    pub documents: Mutex<HashMap<Url, Document>>, // stores current document text
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

                // 2. Autocompletion configuration
                completion_provider: Some(CompletionOptions {
                    resolve_provider: Some(false),
                    trigger_characters: Some(vec![".".to_string(), ":".to_string(), " ".to_string()]),
                    ..Default::default()
                }),

                // 3. ADDED: Semantic Tokens configuration
                semantic_tokens_provider: Some(
                    SemanticTokensServerCapabilities::SemanticTokensOptions(SemanticTokensOptions {
                        legend: SemanticTokensLegend {
                            token_types: vec![
                                SemanticTokenType::from("player"),
                                SemanticTokenType::from("team"),
                                SemanticTokenType::from("location"),
                                SemanticTokenType::from("precedence"),
                                SemanticTokenType::from("pointmap"),
                                SemanticTokenType::from("combo"),
                                SemanticTokenType::from("key"),
                                SemanticTokenType::from("value"),
                                SemanticTokenType::from("memory"),
                                SemanticTokenType::from("token"),
                                SemanticTokenType::from("stage"),
                                SemanticTokenType::from("notype"),
                            ],
                            token_modifiers: vec![],
                        },
                        full: Some(SemanticTokensFullOptions::Bool(true)),
                        range: None,
                        work_done_progress_options: Default::default(),
                    }),
                ),

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

        // Dirty fix right now
        let text = &text.rope.to_string();

        let result = parse_document(text);

        match result {
            Err(v) => {
                // .load() gives you an atomic snapshot. No .await, no blocking.
                let last_ast_snapshot = self.last_ast.load();
                
                // last_ast_snapshot is a Guard that behaves like Option<Arc<FileAST>>
                return Ok(try_auto_completion(v, position, &text, last_ast_snapshot.as_deref()))
            },
            Ok(ast) => {
                self.last_ast.store(Some(Arc::new(ast)));
            }
        }

        Ok(None)
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
        let uri = params.text_document.uri;
        let text = params.text_document.text;
        let rope = Rope::from_str(&text);
        
        let diagnostics;
    
        // 1. Attempt to parse
        match validate_parsing(&rope) {
            Ok(ast) => {
                // 2. If parsing is successful, run your semantic validation/diagnostics
                diagnostics = validate_document(&ast, &rope).unwrap_or_default();
                self.last_ast.store(Some(Arc::new(ast)));
            }
            Err(err_diagnostics) => {
                // 3. If parsing fails, the diagnostics ARE the parse errors
                diagnostics = err_diagnostics;
            }
        }

        // 4. Update your document map with BOTH the rope and the AST
        let mut docs = self.documents.lock().await;
        docs.insert(
            uri.clone(),
            Document {
                rope
            },
        );

        // 5. Send results to the client
        self.client.publish_diagnostics(uri, diagnostics, None).await;
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        // let uri = params.text_document.uri.clone();
        // let docs = self.documents.lock().await;

        // if let Some(content) = docs.get(&uri) {
        //     // Run validation (this should return an empty vec if no errors are found)
        //     let diagnostics = validate_document(&content.rope).unwrap_or_default();
            
        //     // This single call handles both clearing old errors (if vec is empty)
        //     // and showing new ones (if vec has items).
        //     self.client.publish_diagnostics(uri, diagnostics, None).await;
        // }
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri.clone();

        let doc_rope = {
            let mut docs = self.documents.lock().await;

            let doc = docs
                .get_mut(&uri)
                .expect("didChange before didOpen");

            // Apply *all* changes
            for change in params.content_changes {
                apply_change(&mut doc.rope, &change);
            }

            // Materialize full text ONCE
            doc.rope.clone()
        };

        let diagnostics;

        match validate_parsing(&doc_rope) {
            Ok(ast) => {
                // Run diagnostics on the full updated document
                diagnostics = validate_document(&ast, &doc_rope)
                    .unwrap_or_default();                

                self.last_ast.store(Some(Arc::new(ast)));
            },
            Err(v) => {
                diagnostics = v;
            }
        }

        self.client
            .publish_diagnostics(uri, diagnostics, None)
            .await;
    }

    async fn semantic_tokens_full(
        &self,
        params: SemanticTokensParams,
    ) -> Result<Option<SemanticTokensResult>> {
        let uri = params.text_document.uri;

        let map = self.documents.lock().await;
        let doc = map.get(&uri).ok_or_else(|| Error::invalid_params(""))?;
        let tokens;
        if let Some(safe_ast) = &*self.last_ast.load() {
            tokens = tokenize_ast(safe_ast, &doc.rope)
        } else {
            return Ok(None);
        }
        // Convert absolute tokens to LSP Relative (Delta) format
        let delta_u32s = calculate_deltas(tokens);

        // Wrap the u32s into the SemanticToken struct required by the library
        let semantic_tokens: Vec<SemanticToken> = delta_u32s
            .chunks_exact(5)
            .map(|c| SemanticToken {
                delta_line: c[0],
                delta_start: c[1],
                length: c[2],
                token_type: c[3],
                token_modifiers_bitset: c[4],
            })
            .collect();

        Ok(Some(SemanticTokensResult::Tokens(SemanticTokens {
            result_id: None,
            data: semantic_tokens,
        })))
    }
}