use front_end::{ast::ast::SGame, parser::{CGDSLParser, Rule}, spans::OwnedSpan, symbols::{SymbolError, SymbolVisitor}, walker::Walker};
use front_end::parser::Result;
use pest_consume::Parser;
use tower_lsp::lsp_types::{Diagnostic, DiagnosticSeverity};

pub fn validate_document(text: &str) -> Option<Vec<Diagnostic>> {
  let ast;
  let result =  parse_document(text);
  if let Err(err) = result {
    return Some(vec![pest_error_to_diagnostic(err)])
  } else {
    ast = result.unwrap();
  }

  let mut symbol = SymbolVisitor::new();
  ast.walk(&mut symbol);
  
  if let Some(errs) = symbol.check_game_type() {
    return Some(errs.iter().map(|s| symbol_error_to_diagnostics(s.clone(), text)).collect())
  }

  // TODO: Semantic checks

  return None
}

pub fn parse_document(text: &str) -> Result<SGame> {
  // 1. Parsing: pest_consume::parse already returns Result<Nodes, Error<Rule>>
  let nodes = CGDSLParser::parse(Rule::file, text)?;

  // 2. Extract Single Node: .single() returns Result<Node, Error<Rule>>
  let node = nodes.single()?;

  // 3. Mapping: mapper returns Result<T, Error<Rule>>
  let parsed_ast = CGDSLParser::file(node)?;

  Ok(parsed_ast)
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


fn symbol_error_to_diagnostics(symbol_error: SymbolError, text: &str) -> Diagnostic {
  let value;
  let message;
  match symbol_error {
    SymbolError::NotInitialized { var } => {
      value = var;
      message = format!("ID: {} not initialized!", value.id.clone());
    },
    SymbolError::DefinedMultipleTimes { var } => {
      value = var;
      message = format!("ID: {} is defined multiple times!", value.id.clone());
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