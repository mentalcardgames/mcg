use std::collections::HashMap;
use crate::error_to_diagnostics::*;
use front_end::{ast::ast::SGame, symbols::GameType, validation::{parse_document, program_validation, semantic_validation, symbol_validation}};
use ropey::Rope;
use tower_lsp::lsp_types::Diagnostic;

pub fn validate_document(ast: &SGame) -> Result<HashMap<GameType, Vec<String>>, Vec<Diagnostic>> {
  let symbol_table;
  
  match symbol_validation(&ast) {
    Err(errs) => return Err(errs.iter().map(|s| symbol_error_to_diagnostics(s)).collect()),
    Ok(table) => {
      symbol_table = table
    },
  }

  if let Some(errs) = semantic_validation(&ast) {
    return Err(errs.iter().map(|s| semantic_error_to_diagnostics(s)).collect())
  }

  return Ok(symbol_table)
}

pub fn validate_game(ast: &SGame) -> Option<Vec<Diagnostic>> {
  if let Some(errs) = program_validation(&ast) {
    return Some(errs.iter().map(|g| program_error_to_diagnostics(g)).collect())
  }

  return None
}

pub fn validate_parsing(doc: &Rope) -> Result<SGame, Vec<Diagnostic>> {
  let result = parse_document(&doc.to_string());
  if let Err(err) = result {
    return Err(vec![pest_error_to_diagnostic(err)])
  }

  return Ok(result.unwrap())
}


// fn byte_offset_to_position(rope: &Rope, byte_offset: usize) -> Position {
//     // Clamp to ensure we don't panic on a trailing byte index
//     let safe_byte = byte_offset.min(rope.len_bytes());
//     let char_idx = rope.byte_to_char(safe_byte);

//     let line_idx = rope.char_to_line(char_idx);
//     let line_start_char = rope.line_to_char(line_idx);
    
//     // The number of characters from the start of the line to our target
//     let column_chars = char_idx.saturating_sub(line_start_char);

//     let line_slice = rope.line(line_idx);
    
//     // Sum UTF-16 units for the characters on this line up to our char_idx
//     let utf16_col: usize = line_slice
//         .chars()
//         .take(column_chars)
//         .map(|c| c.len_utf16())
//         .sum();

//     Position {
//         line: line_idx as u32,
//         character: utf16_col as u32,
//     }
// }