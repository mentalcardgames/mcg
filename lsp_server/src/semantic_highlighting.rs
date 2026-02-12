use front_end::{ast::ast::SGame, symbols::{GameType, SymbolVisitor}, walker::Walker};
use ropey::Rope;
use tower_lsp::lsp_types::*;

use crate::validation::to_range;

fn server_capabilities() -> ServerCapabilities {
    ServerCapabilities {
        text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL)),
        semantic_tokens_provider: Some(SemanticTokensServerCapabilities::SemanticTokensOptions(
            SemanticTokensOptions {
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
            }
        )),
        ..Default::default()
    }
}

pub struct AbsoluteToken {
    line: u32,
    start: u32,
    length: u32,
    token_type: u32,
    modifiers: u32,
}

pub fn calculate_deltas(mut tokens: Vec<AbsoluteToken>) -> Vec<u32> {
    // Sort tokens by line, then by start character (crucial for deltas!)
    tokens.sort_by_key(|t| (t.line, t.start));

    let mut last_line = 0;
    let mut last_start = 0;
    let mut output = Vec::with_capacity(tokens.len() * 5);

    for token in tokens {
        let delta_line = token.line - last_line;
        let delta_start = if delta_line == 0 {
            token.start - last_start
        } else {
            token.start
        };

        output.push(delta_line);
        output.push(delta_start);
        output.push(token.length);
        output.push(token.token_type);
        output.push(token.modifiers);

        last_line = token.line;
        last_start = token.start;
    }
    output
}

pub fn tokenize_ast(ast: &SGame, rope: &Rope) -> Vec<AbsoluteToken> {
    let mut symbols = SymbolVisitor::new();
    ast.walk(&mut symbols);

    let var_type = symbols.name_resolution();

    var_type
        .iter()
        .map(|(v, g_type)| {
            let range = to_range(&v.span, rope);
            
            AbsoluteToken {
                line: range.start.line,
                start: range.start.character,
                // Calculate length: handle multi-line spans by focusing on the start line length
                length: if range.end.line == range.start.line {
                    range.end.character - range.start.character
                } else {
                    // If a token spans multiple lines, LSP usually expects 
                    // a separate entry per line, but for identifiers, this is rare.
                    (v.span.end - v.span.start) as u32 
                },
                token_type: game_type_to_legend_index(g_type),
                modifiers: 0, // You can add "readonly" or "static" logic here later
            }
        })
        .collect()
}


fn game_type_to_legend_index(gt: &GameType) -> u32 {
  match gt {
    GameType::Player => 0,
    GameType::Team => 1,
    GameType::Location => 2,
    GameType::Precedence => 3,
    GameType::PointMap => 4,
    GameType::Combo => 5,
    GameType::Key => 6,
    GameType::Value => 7,
    GameType::Memory => 8,
    GameType::Token => 9,
    GameType::Stage => 10,
    GameType::NoType => 11,
  }
}