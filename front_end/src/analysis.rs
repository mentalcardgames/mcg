use crate::{diagnostic::SGame, symbols::TypedVars, transform_to_typed::{TypeError, parse_ast_to_typed_ast}, typed_ast, visitor::Visitor};

#[derive(Debug)]
pub enum AnalyzerError {
    NoDslType,
    IdUsed { id: String },
    IdNotCapitalOrEmpty,
    InvalidInteger,
    ReservedKeyword,
    UnknownID(String),
    UnknownPlayerNameUsed(String),
    DuplicateIDs(Vec<String>),
    TypeError(TypeError),
    IDWithMultipleTypes { id: String },
    IDWithNoType { id: String },
    IDNotInitialized {id: String },
    NonDeterministicInitialization { created: TypedVars },
    // TODO: More precise AnalyzerErrors
    Default,
}

// Find a final ast to return (right now typed ast)
pub fn analyze_ast(ast: &SGame) -> Result<typed_ast::Game, AnalyzerError> {
  let mut ctx: TypedVars = vec![];

  ast.visit(&mut ctx)?;

  println!("{:?}", ctx);

  // Current return type
  parse_ast_to_typed_ast(ctx, ast)
}
