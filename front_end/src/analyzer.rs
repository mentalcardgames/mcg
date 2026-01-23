use crate::{analyzer_error::AnalyzerError, ast::Game, ast_to_typed_ast::parse_ast_to_typed_ast, typed_ast, visit_typed_vars::TypedVars, visitor::Visitor};

// Find a final ast to return (right now typed ast)
pub fn analyze_ast(ast: &Game) -> Result<typed_ast::Game, AnalyzerError> {
  let mut ctx: TypedVars = vec![];
  ast.visit(&mut ctx)?;

  println!("{:?}", ctx);

  // Current return type
  parse_ast_to_typed_ast(ctx, ast)
}
