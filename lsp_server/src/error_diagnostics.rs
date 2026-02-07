use front_end::parser::Rule;
use tower_lsp::lsp_types::{Diagnostic, Range, Position};

pub fn pest_error_to_diagnostic(err: pest::error::Error<Rule>) -> Diagnostic {
    let (start, end) = match err.line_col {
        pest::error::LineColLocation::Pos((line, col)) => {
            // Pest uses 1-based indexing, LSP uses 0-based
            let pos = Position::new(line as u32 - 1, col as u32 - 1);
            (pos, pos)
        }
        pest::error::LineColLocation::Span((start_line, start_col), (end_line, end_col)) => {
            let start = Position::new(start_line as u32 - 1, start_col as u32 - 1);
            let end = Position::new(end_line as u32 - 1, end_col as u32 - 1);
            (start, end)
        }
    };

    Diagnostic {
        range: Range::new(start, end),
        severity: Some(tower_lsp::lsp_types::DiagnosticSeverity::ERROR),
        message: err.variant.message().to_string(),
        ..Default::default()
    }
}