use front_end::{
    ir::GameFlowError, parser::Rule, semantic::SemanticError, spans::OwnedSpan,
    symbols::SymbolError,
};
use pest::error::LineColLocation;
use tower_lsp::lsp_types::{Diagnostic, DiagnosticSeverity, Position, Range};

/// Taking a Pest Error and give a correct visual report for the error (red-squiggle lines under the certain text).
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

    // Give hints about PlayerExpr, TeamExpr and all Memory-Types
    let message = match &pest_err.variant {
        pest::error::ErrorVariant::ParsingError { positives, negatives: _ } => {
            let mut inside = "".to_string();
            if let Some(rule) = positives.first() {
                match rule {
                    Rule::playername => {
                        inside = String::from("Hint: Player-Names start with 'P:' except when they are initialized")
                    },
                    Rule::teamname => {
                        inside = String::from("Hint: Team-Names start with 'T:' except when they are initialized")
                    },
                    Rule::player_expr => {
                        inside = String::from("Hint: Player-Expr start with '&P:' except when they are initialized")
                    },
                    Rule::team_expr => {
                        inside = String::from("Hint: Team-Expr start with '&T:' except when they are initialized")
                    },
                    Rule::int_expr => {
                        inside = String::from("Hint: Int-Expr start with '&I:' except when they are initialized")
                    },
                    Rule::string_expr => {
                        inside = String::from("Hint: String-Expr start with '&S:' except when they are initialized")
                    },
                    Rule::int_collection => {
                        inside = String::from("Hint: Int-Collection Memories start with '&IC:' except when they are initialized")
                    },
                    Rule::player_collection => {
                        inside = String::from("Hint: Player-Collection Memories start with '&PC:' except when they are initialized")
                    },
                    Rule::team_collection => {
                        inside = String::from("Hint: Team-Collection Memories start with '&TC:' except when they are initialized")
                    },
                    Rule::string_collection => {
                        inside = String::from("Hint: String-Collection Memories start with '&SC:' except when they are initialized")
                    },
                    Rule::location_collection => {
                        inside = String::from("Hint: Location-Collection Memories start with '&LC:' except when they are initialized")
                    },
                    Rule::card_set => {
                        inside = String::from("Hint: Card-Set Memories start with '&CS:' except when they are initialized")
                    },
                    _ => {}
                }
            }

            format!("{}\n{}", format!("{}", pest_err.variant.message()), inside)
        },
        pest::error::ErrorVariant::CustomError { message } => {
            format!("{}", message)
        },
    };


    Diagnostic {
        range,
        message: message,
        severity: Some(tower_lsp::lsp_types::DiagnosticSeverity::ERROR),
        ..Default::default()
    }
}

/// Converts a symbol error (from the front_end) into a tower-lsp Diagnostic.
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

/// Converts a semantic error (from the front_end) into a tower-lsp Diagnostic.
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

/// Converts a program error (from the front_end) into a tower-lsp Diagnostic.
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

/// Takes the Owned-Span struct and converts it into tower-lsp Range.
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
