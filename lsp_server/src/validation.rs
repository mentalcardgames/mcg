use front_end::{parser::Rule, semantic::SemanticError, spans::OwnedSpan, symbols::SymbolError, validation::{parse_document, semantic_validation, symbol_validation}};
use tower_lsp::lsp_types::{Diagnostic, DiagnosticSeverity};

pub fn validate_document(text: &str) -> Option<Vec<Diagnostic>> {
  let ast;
  let result =  parse_document(text);
  if let Err(err) = result {
    return Some(vec![pest_error_to_diagnostic(err)])
  } else {
    ast = result.unwrap();
  }

  if let Some(errs) = symbol_validation(&ast) {
    return Some(errs.iter().map(|s| symbol_error_to_diagnostics(s, text)).collect())
  }

  if let Some(errs) = semantic_validation(&ast) {
    return Some(errs.iter().map(|s| semantic_error_to_diagnostics(s, text)).collect())
  }

  return None
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


fn symbol_error_to_diagnostics(symbol_error: &SymbolError, text: &str) -> Diagnostic {
  let value;
  let message;
  match symbol_error {
    SymbolError::NotInitialized { var } => {
      value = var;
      message = format!("ID: {} not initialized!", &value.id);
    },
    SymbolError::DefinedMultipleTimes { var } => {
      value = var;
      message = format!("ID: {} is defined multiple times!", &value.id);
    },
  }

  Diagnostic {
    range: to_range(&value.span, text),
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


fn semantic_error_to_diagnostics(semantic_error: &SemanticError, text: &str) -> Diagnostic {
  let spanned;
  let message;
  match semantic_error {
    SemanticError::KeyNotInPrecedence { key, precedence } => {
      message = format!("{} not in {}!", &key.id, &precedence.id);
      spanned = key;
    },
    SemanticError::KeyNoCorrToPrecedence { key, precedence} => {
      message = format!("{} does not correspond to {}!", &key.id, &precedence.id);
      spanned = key;
    },
    SemanticError::KeyNotInPointMap { key, pointmap} => {
      message = format!("{} not in {}!", &key.id, &pointmap.id);
      spanned = key;
    },
    SemanticError::KeyNoCorrToPointMap { key, pointmap} => {
      message = format!("{} does not correspond to {}!", &key.id, &pointmap.id);
      spanned = key;
    },
    SemanticError::ValueNotInKey { key, value } => {
      message = format!(" {} not in {}!", &value.id, &key.id);
      spanned = key;
    },
    SemanticError::ValueNoCorrToKey { key, value } => {
      message = format!("{} does not correspond to {}!", &value.id, &key.id);
      spanned = key;
    },
    SemanticError::KeyAndStringDontAllign { key, string_key } => {
      message = format!("{} and {} dont allign!", &key.id, &string_key.id);
      spanned = key;
    },
  }

  Diagnostic {
    range: to_range(&spanned.span, text),
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


pub fn to_range(owned_span: &OwnedSpan, source: &str) -> Range {
    let start = offset_to_position(owned_span.start, source);
    let end = offset_to_position(owned_span.end, source);

    Range { start, end }
}

pub fn offset_to_position(offset: usize, source: &str) -> Position {
    let mut line = 0;
    let mut character = 0;

    for (i, c) in source.char_indices() {
        if i >= offset {
            break;
        }

        if c == '\n' {
            line += 1;
            character = 0;
        } else {
            // LSP uses UTF-16 code units for the 'character' field
            character += c.len_utf16();
        }
    }

    Position {
        line: line as u32,
        character: character as u32,
    }
}