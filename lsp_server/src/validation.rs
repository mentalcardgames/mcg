use crate::error_to_diagnostics::*;
use front_end::{
    ast::ast_spanned::SGame,
    symbols::GameType,
    validation::{parse_document, program_validation, semantic_validation, symbol_validation},
};
use ropey::Rope;
use std::collections::HashMap;
use tower_lsp::lsp_types::Diagnostic;

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

pub fn validate_parsing(doc: &Rope) -> Result<SGame, Vec<Diagnostic>> {
    let result = parse_document(&doc.to_string());
    if let Err(err) = result {
        return Err(vec![pest_error_to_diagnostic(err)]);
    }

    return Ok(result.unwrap());
}
