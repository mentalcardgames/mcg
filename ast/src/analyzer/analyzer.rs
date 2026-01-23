use crate::analyzer::analyzer_error::AnalyzerError;
use crate::parse::ast_to_typed_ast::{parse_ast_to_typed_ast};
use crate::analyzer::ctx::{TypedVars, ambiguous};

use crate::asts::{ast::*, typed_ast};
use crate::parse::visitor::Visitor;

// Find a final ast to return (right now typed ast)
pub fn analyze_ast(ast: &Game) -> Result<typed_ast::Game, AnalyzerError> {
  let mut ctx: TypedVars = vec![];
  ast.visit(&mut ctx)?;

  println!("{:?}", ctx);

  // Checking for Ambiguous IDs
  ambiguous(ctx.clone())?;

  // Current return type
  parse_ast_to_typed_ast(ctx, ast)
}
