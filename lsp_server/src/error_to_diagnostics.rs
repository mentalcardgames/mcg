use front_end::{
    ir::GameFlowError, parser::Rule, semantic::SemanticError, spans::OwnedSpan,
    symbols::SymbolError,
};
use pest::error::LineColLocation;
use tower_lsp::lsp_types::{Diagnostic, DiagnosticSeverity, Position, Range};

pub fn pest_error_to_diagnostic(pest_err: pest::error::Error<Rule>) -> Diagnostic {
    let range = match pest_err.line_col {
        LineColLocation::Pos((line, col)) => {
            // LSP is 0-indexed, Pest is 1-indexed
            let pos = Position::new((line - 1) as u32, (col - 1) as u32);
            Range::new(pos, pos)
        }
        LineColLocation::Span((start_line, start_col), (end_line, end_col)) => Range::new(
            Position::new((start_line - 1) as u32, (start_col - 1) as u32),
            Position::new((end_line - 1) as u32, (end_col - 1) as u32),
        ),
    };

    Diagnostic {
        range,
        message: format!("{}", pest_err.variant.message()),
        severity: Some(tower_lsp::lsp_types::DiagnosticSeverity::ERROR),
        ..Default::default()
    }
}

pub fn symbol_error_to_diagnostics(symbol_error: &SymbolError) -> Diagnostic {
    let value;
    let message;
    match symbol_error {
        SymbolError::NotInitialized { var } => {
            value = var;
            message = format!("'{}' not initialized", &value.id);
        }
        SymbolError::DefinedMultipleTimes { var } => {
            value = var;
            message = format!("'{}' is defined multiple times", &value.id);
        }
    }

    Diagnostic {
        range: to_range(&value.span),
        severity: Some(DiagnosticSeverity::ERROR), // Defines the color/style
        code: None,
        source: Some("cgdsl-lsp".to_string()),
        message: message,
        related_information: None,
        tags: None,
        data: None,
        code_description: None,
    }
}

pub fn semantic_error_to_diagnostics(semantic_error: &SemanticError) -> Diagnostic {
    let spanned;
    let message;
    match semantic_error {
        SemanticError::KeyNotFoundForType { ty, key } => {
            message = format!("'{}' not found for '{}'", &key.id, ty);
            spanned = key;
        }
        SemanticError::NoCorrToType { ty, key } => {
            message = format!("'{}' does not correspond to '{}'", &key.id, &ty.id);
            spanned = key;
        }
        SemanticError::MemoryMismatch { memory } => {
            message = format!("'{}' does not match initialized value", &memory.id);
            spanned = memory;
        }
    }

    Diagnostic {
        range: to_range(&spanned.span),
        severity: Some(DiagnosticSeverity::ERROR), // Defines the color/style
        code: None,
        source: Some("cgdsl-lsp".to_string()),
        message: message,
        related_information: None,
        tags: None,
        data: None,
        code_description: None,
    }
}

pub fn program_error_to_diagnostics(program_error: &GameFlowError) -> Diagnostic {
    let spanned;
    let message;
    match program_error {
        GameFlowError::Unreachable { span } => {
            message = format!("Code is unreachable");
            spanned = span;
        }
        GameFlowError::NoStageToEnd { span } => {
            message = format!("There is no stage to end");
            spanned = span;
        }
        GameFlowError::FlowNotConnected { span } => {
            message = format!("The Game is not connected");
            spanned = span;
        }
        GameFlowError::FlowNotConnectedWithControl => {
            message = format!("The Game is heavily not connected");
            return Diagnostic {
                severity: Some(DiagnosticSeverity::ERROR), // Defines the color/style
                code: None,
                source: Some("cgdsl-lsp".to_string()),
                message: message,
                related_information: None,
                tags: None,
                data: None,
                code_description: None,
                ..Default::default()
            };
        }
    }

    Diagnostic {
        range: to_range(&spanned),
        severity: Some(DiagnosticSeverity::ERROR), // Defines the color/style
        code: None,
        source: Some("cgdsl-lsp".to_string()),
        message: message,
        related_information: None,
        tags: None,
        data: None,
        code_description: None,
    }
}

pub fn to_range(span: &OwnedSpan) -> Range {
    // pest position starts at 1!
    let start_pos = Position {
        line: (span.start_pos.0.clone() - 1) as u32,
        character: (span.start_pos.1.clone() - 1) as u32,
    };
    // pest position starts at 1!
    let end_pos = Position {
        line: (span.end_pos.0.clone() - 1) as u32,
        character: (span.end_pos.1.clone() - 1) as u32,
    };

    Range {
        start: start_pos,
        end: end_pos,
    }
}
