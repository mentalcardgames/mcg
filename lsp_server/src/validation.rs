use std::collections::HashMap;

use front_end::{ast::ast::SGame, parser::Rule, semantic::SemanticError, spans::OwnedSpan, symbols::{GameType, SymbolError}, validation::{parse_document, semantic_validation, symbol_validation}};
use ropey::Rope;
use tower_lsp::lsp_types::{Diagnostic, DiagnosticSeverity};

pub fn validate_document(ast: &SGame, doc: &Rope) -> Result<HashMap<GameType, Vec<String>>, Vec<Diagnostic>> {
  let symbol_table;
  
  match symbol_validation(&ast) {
    Err(errs) => return Err(errs.iter().map(|s| symbol_error_to_diagnostics(s, &doc)).collect()),
    Ok(table) => {
      symbol_table = table
    },
  }

  if let Some(errs) = semantic_validation(&ast) {
    return Err(errs.iter().map(|s| semantic_error_to_diagnostics(s, &doc)).collect())
  }

  return Ok(symbol_table)
}

pub fn validate_parsing(doc: &Rope) -> Result<SGame, Vec<Diagnostic>> {
  let result = parse_document(&doc.to_string());
  if let Err(err) = result {
    return Err(vec![pest_error_to_diagnostic(err)])
  } 

  return Ok(result.unwrap())
}


use tower_lsp::lsp_types::{Position, Range};
use pest::error::LineColLocation;

pub fn pest_error_to_diagnostic(pest_err: pest::error::Error<Rule>) -> Diagnostic {
    let range = match pest_err.line_col {
        LineColLocation::Pos((line, col)) => {
            // LSP is 0-indexed, Pest is 1-indexed
            let pos = Position::new((line - 1) as u32, (col - 1) as u32);
            Range::new(pos, pos)
        }
        LineColLocation::Span((start_line, start_col), (end_line, end_col)) => {
            Range::new(
                Position::new((start_line - 1) as u32, (start_col - 1) as u32),
                Position::new((end_line - 1) as u32, (end_col - 1) as u32),
            )
        }
    };

    Diagnostic {
        range,
        message: format!("{}", pest_err.variant.message()),
        severity: Some(tower_lsp::lsp_types::DiagnosticSeverity::ERROR),
        ..Default::default()
    }
}


fn symbol_error_to_diagnostics(symbol_error: &SymbolError, doc: &Rope) -> Diagnostic {
  let value;
  let message;
  match symbol_error {
    SymbolError::NotInitialized { var } => {
      value = var;
      message = format!("'{}' not initialized", &value.id);
    },
    SymbolError::DefinedMultipleTimes { var } => {
      value = var;
      message = format!("'{}' is defined multiple times", &value.id);
    },
  }

  Diagnostic {
    range: to_range(&value.span, doc),
    severity: Some(DiagnosticSeverity::ERROR), // Defines the color/style
    code: None,
    source: Some("my-cool-lsp".to_string()),
    message: message,
    related_information: None,
    tags: None,
    data: None,
    code_description: None,
  }
}


fn semantic_error_to_diagnostics(semantic_error: &SemanticError, doc: &Rope) -> Diagnostic {
  let spanned;
  let message;
  match semantic_error {
    SemanticError::KeyNotFoundForType { ty, key } => {
      message = format!("'{}' not found for '{}'", &key.id, ty);
      spanned = key;
    },
    SemanticError::NoCorrToType { ty, key } => {
      message = format!("'{}' does not correspond to '{}'", &key.id, &ty.id);
      spanned = key;
    },
  }

  Diagnostic {
    range: to_range(&spanned.span, doc),
    severity: Some(DiagnosticSeverity::ERROR), // Defines the color/style
    code: None,
    source: Some("my-cool-lsp".to_string()),
    message: message,
    related_information: None,
    tags: None,
    data: None,
    code_description: None,
  }
}


pub fn to_range(span: &OwnedSpan, rope: &Rope) -> Range {
    Range {
        start: byte_offset_to_position(rope, span.start),
        end: byte_offset_to_position(rope, span.end),
    }
}

use tower_lsp::lsp_types::TextDocumentContentChangeEvent;

pub fn apply_change(rope: &mut Rope, change: &TextDocumentContentChangeEvent) {
    if let Some(range) = change.range {
        // 1. Convert positions to char indices safely
        let start_char = position_to_char(rope, range.start);
        let end_char = position_to_char(rope, range.end);

        // 2. Clamp indices to the current rope length to prevent panics
        let len = rope.len_chars();
        let safe_start = start_char.min(len);
        let safe_end = end_char.min(len).max(safe_start);

        // 3. Perform the edit
        if safe_start < safe_end {
            rope.remove(safe_start..safe_end);
        }
        
        if !change.text.is_empty() {
            rope.insert(safe_start, &change.text);
        }
    } else {
        *rope = Rope::from_str(&change.text);
    }
}

pub fn position_to_char(rope: &Rope, position: Position) -> usize {
    let line_idx = position.line as usize;
    let utf16_col = position.character as usize;

    // If the line index is beyond the rope, return the very end of the document
    if line_idx >= rope.len_lines() {
        return rope.len_chars();
    }

    let line_start_char = rope.line_to_char(line_idx);
    let line_slice = rope.line(line_idx);

    let mut current_utf16 = 0;
    let mut char_offset = 0;

    for c in line_slice.chars() {
        // If we've reached the target UTF-16 column, stop.
        if current_utf16 >= utf16_col {
            break;
        }
        
        // Stop if we hit a newline character—LSP positions for a line
        // shouldn't technically include the newline itself.
        if c == '\n' || c == '\r' {
            break;
        }

        current_utf16 += c.len_utf16();
        char_offset += 1;
    }

    line_start_char + char_offset
}

fn byte_offset_to_position(rope: &Rope, byte_offset: usize) -> Position {
    // Clamp to ensure we don't panic on a trailing byte index
    let safe_byte = byte_offset.min(rope.len_bytes());
    let char_idx = rope.byte_to_char(safe_byte);

    let line_idx = rope.char_to_line(char_idx);
    let line_start_char = rope.line_to_char(line_idx);
    
    // The number of characters from the start of the line to our target
    let column_chars = char_idx.saturating_sub(line_start_char);

    let line_slice = rope.line(line_idx);
    
    // Sum UTF-16 units for the characters on this line up to our char_idx
    let utf16_col: usize = line_slice
        .chars()
        .take(column_chars)
        .map(|c| c.len_utf16())
        .sum();

    Position {
        line: line_idx as u32,
        character: utf16_col as u32,
    }
}