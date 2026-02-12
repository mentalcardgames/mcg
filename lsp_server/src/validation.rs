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
    SemanticError::KeyNotInPrecedence { key, precedence } => {
      message = format!("'{}' not in '{}'", &key.id, &precedence.id);
      spanned = key;
    },
    SemanticError::KeyNoCorrToPrecedence { key, precedence} => {
      message = format!("'{}' does not correspond to '{}'", &key.id, &precedence.id);
      spanned = key;
    },
    SemanticError::KeyNotInPointMap { key, pointmap} => {
      message = format!("'{}' not in '{}'", &key.id, &pointmap.id);
      spanned = key;
    },
    SemanticError::KeyNoCorrToPointMap { key, pointmap} => {
      message = format!("'{}' does not correspond to '{}'", &key.id, &pointmap.id);
      spanned = key;
    },
    SemanticError::ValueNotInKey { key, value } => {
      message = format!("'{}' not in '{}'", &value.id, &key.id);
      spanned = key;
    },
    SemanticError::ValueNoCorrToKey { key, value } => {
      message = format!("'{}' does not correspond to '{}'", &value.id, &key.id);
      spanned = key;
    },
    SemanticError::KeyAndStringDontAllign { key, string_key } => {
      message = format!("'{}' and '{}' dont allign", &key.id, &string_key.id);
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
        let start = position_to_char(rope, range.start);
        let end = position_to_char(rope, range.end);

        rope.remove(start..end);
        rope.insert(start, &change.text);
    } else {
        // Full document replace
        *rope = Rope::from_str(&change.text);
    }
}

pub fn position_to_char(rope: &Rope, position: Position) -> usize {
    let line = position.line as usize;
    let utf16_col = position.character as usize;

    // Clamp line to document
    let line = line.min(rope.len_lines().saturating_sub(1));

    let line_start = rope.line_to_char(line);
    let line_text = rope.line(line);

    let mut current_utf16 = 0;
    let mut char_offset = 0;

    for c in line_text.chars() {
        if current_utf16 >= utf16_col {
            break;
        }
        current_utf16 += c.len_utf16();
        char_offset += 1;
    }

    line_start + char_offset
}

fn byte_offset_to_position(rope: &Rope, byte_offset: usize) -> Position {
    // Convert byte → char index
    let char_idx = rope.byte_to_char(byte_offset);

    // Get line
    let line = rope.char_to_line(char_idx);

    // Get column in chars
    let line_start = rope.line_to_char(line);
    let column_chars = char_idx - line_start;

    // Convert char column → UTF-16 column
    let line_text = rope.line(line);
    let utf16_col = line_text
      .chars()
      .take(column_chars)
      .map(|c| c.len_utf16())
      .sum::<usize>();

    Position {
        line: line as u32,
        character: utf16_col as u32,
    }
}