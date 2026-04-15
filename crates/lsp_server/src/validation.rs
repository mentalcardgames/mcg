use crate::error_to_diagnostics::*;
use front_end::{
    ast::ast_spanned::SGame,
    symbols::GameType,
    validation::{parse_document, program_validation, semantic_validation, symbol_validation},
};
use ropey::Rope;
use std::collections::HashMap;
use tower_lsp::lsp_types::Diagnostic;

/// Performs full document validation, moving from symbol resolution to semantic checks.
///
/// This function acts as the primary gatekeeper for code correctness:
/// 1. **Symbol Validation:** Ensures identifiers are declared and unique.
/// 2. **Semantic Validation:** Checks for logic errors, type mismatches, or 
///    invalid game state transitions.
///
/// ### Returns
/// * `Ok(HashMap<GameType, Vec<String>>)`: A mapped symbol table if valid.
/// * `Err(Vec<Diagnostic>)`: A collection of LSP-compatible errors found 
///   during either stage.
pub fn validate_document(ast: &SGame) -> Result<HashMap<GameType, Vec<String>>, Vec<Diagnostic>> {
    let symbol_table;

    match symbol_validation(&ast) {
        Err(errs) => {
            return Err(errs
                .iter()
                .map(|s| symbol_error_to_diagnostics(s))
                .collect());
        }
        Ok(table) => symbol_table = table,
    }

    if let Some(errs) = semantic_validation(&ast) {
        return Err(errs
            .iter()
            .map(|s| semantic_error_to_diagnostics(s))
            .collect());
    }

    return Ok(symbol_table);
}

/// Runs high-level program/game-logic validation on the AST.
///
/// Unlike `validate_document`, which focuses on symbols and semantics, this 
/// checks for structural or "game-rule" violations defined in `program_validation`.
///
/// ### Returns
/// * `Some(Vec<Diagnostic>)` if errors are found.
/// * `None` if the game logic is valid.
pub fn validate_game(ast: &SGame) -> Option<Vec<Diagnostic>> {
    if let Some(errs) = program_validation(&ast) {
        return Some(
            errs.iter()
                .map(|g| program_error_to_diagnostics(g))
                .collect(),
        );
    }

    return None;
}

/// Converts a [`Rope`] to a string and attempts to parse it into an [`SGame`] AST.
///
/// This is the first line of defense in the validation pipeline. It maps raw 
/// [Pest](https://pest.rs/) parser errors into LSP [`Diagnostic`] objects.
///
/// ### Errors
/// Returns a `Vec<Diagnostic>` containing the location and description of 
/// syntax errors if the grammar rules are violated.
pub fn validate_parsing(doc: &Rope) -> Result<SGame, Vec<Diagnostic>> {
    let result = parse_document(&doc.to_string());
    if let Err(err) = result {
        return Err(vec![pest_error_to_diagnostic(err)]);
    }

    return Ok(result.unwrap());
}
