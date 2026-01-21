use crate::analyzer::analyzer_error::AnalyzerError;
use crate::parse::ast_to_typed_ast::{parse_ast};
use crate::analyzer::type_analyzer::{ambiguous, ctx};

use crate::asts::ast::*;

pub fn analyze_ast(ast: &Game) -> Result<(), AnalyzerError> {
  let ctx = ctx(ast);

  ambiguous(ctx.clone())?;
  parse_ast(ctx, ast)?;

  Ok(())
}
