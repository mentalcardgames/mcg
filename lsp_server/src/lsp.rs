use std::collections::HashMap;

use dashmap::DashMap;
use front_end::ast::ast_spanned::SGame;
use front_end::symbols::GameType;
use front_end::validation::parse_document;
use ropey::{Rope};
use crate::rope::{Document, apply_change};
use crate::semantic_highlighting::{calculate_deltas, tokenize_ast};
use crate::validation::{validate_document, validate_game, validate_parsing};
use tokio::sync::{Mutex, mpsc};
use arc_swap::ArcSwapOption;
use std::sync::Arc;
use tower_lsp::jsonrpc::{self, Result};
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};

use crate::completion::{get_completions};

#[derive(Debug)]
pub struct Backend {
    pub client: Client,
    pub documents: Mutex<HashMap<Url, Document>>, // stores current document text
    // No Mutex needed! ArcSwap handles thread-safety internally.
    pub last_ast: ArcSwapOption<SGame>, 
    // Debouncer to minimize flickering
    pub analysis_tx: mpsc::UnboundedSender<Url>,
    pub symbol_table: DashMap<GameType, Vec<String>>,
}

impl Backend {
    pub async fn run_analysis(&self, uri: Url) {
        // 1. Get a snapshot of the text
        let doc_rope = {
            let docs = self.documents.lock().await;
            docs.get(&uri).map(|d| d.rope.clone())
        };

        if let Some(rope) = doc_rope {
            let diagnostics = self.get_diagnostics(&rope);

            // Standard document storage
            let mut docs = self.documents.lock().await;
            docs.insert(uri.clone(), Document { rope });

            self.client.publish_diagnostics(uri, diagnostics, None).await;
        }
    }

    pub fn get_diagnostics(&self, rope: &Rope) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        match validate_parsing(&rope) {
            Ok(ast) => {
                // Run semantic validation
                match validate_document(&ast) {
                    Ok(new_table) => {
                        // Update the symbol table
                        self.symbol_table.clear();
                        for (game_type, names) in new_table {
                            self.symbol_table.insert(game_type, names);
                        }
                    },
                    Err(v) => {
                        diagnostics = v;
                    }
                }
                self.last_ast.store(Some(Arc::new(ast)));
            }
            Err(err_diagnostics) => {
                diagnostics = err_diagnostics;
            }
        }

        diagnostics
    }

    pub fn get_did_save_diagnostics(&self, rope: &Rope) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        match validate_parsing(&rope) {
            Ok(ast) => {
                // Run semantic validation
                match validate_document(&ast) {
                    Ok(new_table) => {
                        // Update the symbol table
                        self.symbol_table.clear();
                        for (game_type, names) in new_table {
                            self.symbol_table.insert(game_type, names);
                        }
                    },
                    Err(v) => {
                        diagnostics.extend(v);
                    }
                }
                // Run game validation
                match validate_game(&ast) {
                    None => {},
                    Some(v) => {
                        diagnostics.extend(v);
                    }
                }
                self.last_ast.store(Some(Arc::new(ast)));
            }
            Err(err_diagnostics) => {
                diagnostics.extend(err_diagnostics);
            }
        }

        diagnostics
    }

    pub async fn get_rope(&self, uri: &Url) -> Rope {
        let mut docs = self.documents.lock().await;

        let doc = docs
            .get_mut(&uri)
            .expect("didSave before didOpen");

        // Materialize full text ONCE
        doc.rope.clone()
    }

    pub async fn get_rope_and_apply_change(&self, uri: &Url, params: &DidChangeTextDocumentParams) -> Rope {
        let mut docs = self.documents.lock().await;

        let doc = docs
            .get_mut(&uri)
            .expect("didChange before didOpen");

        // Apply *all* changes
        for change in params.content_changes.iter() {
            apply_change(&mut doc.rope, &change);
        }

        // Materialize full text ONCE
        doc.rope.clone()
    }


    pub fn get_semantic_tokens(&self) -> Option<Vec<SemanticToken>> {
        let tokens;
        if let Some(safe_ast) = &*self.last_ast.load() {
            tokens = tokenize_ast(safe_ast)
        } else {
            return None;
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

        Some(semantic_tokens)
    }
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

                execute_command_provider: Some(ExecuteCommandOptions {
                    commands: vec!["cgdsl.generateGraph".to_string()],
                    ..Default::default()
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
        let rope = self.get_rope(&uri).await;
        let result = parse_document(&rope.to_string());

        match result {
            Err(err) => {
                Ok(get_completions(err, &self.symbol_table))
            },
            Ok(_) => {Ok(None)}
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
        let uri = params.text_document.uri;
        let text = params.text_document.text;
        let rope = Rope::from_str(&text);
        
        let diagnostics = self.get_diagnostics(&rope);

        // Standard document storage
        let mut docs = self.documents.lock().await;
        docs.insert(uri.clone(), Document { rope });

        self.client.publish_diagnostics(uri, diagnostics, None).await;
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        let uri = params.text_document.uri;
        let rope = self.get_rope(&uri).await;
        let diagnostics = self.get_did_save_diagnostics(&rope);

        // Standard document storage
        let mut docs = self.documents.lock().await;
        docs.insert(uri.clone(), Document { rope });

        self.client.publish_diagnostics(uri, diagnostics, None).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri.clone();
        let rope = self.get_rope_and_apply_change(&uri, &params).await;
        let diagnostics = self.get_diagnostics(&rope);

        // Standard document storage
        let mut docs = self.documents.lock().await;
        docs.insert(uri.clone(), Document { rope });

        self.client.publish_diagnostics(uri, diagnostics, None).await;
    }

    async fn semantic_tokens_full(
        &self,
        _: SemanticTokensParams,
    ) -> Result<Option<SemanticTokensResult>> {
        if let Some(semantic_tokens) = self.get_semantic_tokens() {
            return Ok(Some(SemanticTokensResult::Tokens(SemanticTokens {
                result_id: None,
                data: semantic_tokens,
            })))
        }

        return Ok(None)
    }

    async fn execute_command(&self, params: ExecuteCommandParams) -> jsonrpc::Result<Option<serde_json::Value>> {
        if let Some(safe_ast) = &*self.last_ast.load() {
            if params.command == "cgdsl.generateGraph" {
                // Get your graph data
                let graph = safe_ast.to_lowered_graph(); 
                
                // 1. Get the path from TS arguments
                let path_str = params.arguments.get(0)
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| jsonrpc::Error::invalid_params("Missing path"))?;
                
                let dot_path = std::path::Path::new(path_str);

                // 3. Write the DOT file (The "Game View")
                front_end::fsm_to_dot::fsm_to_dot(&graph, dot_path).map_err(|_| {
                    jsonrpc::Error::internal_error()
                })?;
                
                // Return it as a JSON value
                let json_value = serde_json::to_value(&graph)
                    .map_err(|_| jsonrpc::Error::internal_error())?;
                    
                return Ok(Some(json_value));
            }
        } else {
            return Ok(None);
        }

        Err(jsonrpc::Error::method_not_found())
    }

    // Use this for REQUESTS (expects a response)
    async fn symbol(&self, _params: WorkspaceSymbolParams) -> jsonrpc::Result<Option<Vec<SymbolInformation>>> {
        // Standard LSP method
        Ok(None)
    }
}