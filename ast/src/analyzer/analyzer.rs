use crate::analyzer::analyzer_error::AnalyzerError;
use crate::parse::ast_to_typed_ast::{parse_ast_to_typed_ast};
use crate::analyzer::type_analyzer::{ambiguous, ctx};

use crate::asts::{ast::*, typed_ast};

// Find a final ast to return (right now typed ast)
pub fn analyze_ast(ast: &Game) -> Result<typed_ast::Game, AnalyzerError> {
  let ctx = ctx(ast);

  // Checking for Ambiguous IDs
  ambiguous(ctx.clone())?;

  // Current return type
  parse_ast_to_typed_ast(ctx, ast)
}
