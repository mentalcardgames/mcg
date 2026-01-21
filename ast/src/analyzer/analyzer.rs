use crate::analyzer::analyzer_error::AnalyzerError;
use crate::parse::ast_to_typed_ast::{Lower, LoweringCtx};
use crate::analyzer::type_analyzer::{check_type as ct, ctx};

use crate::asts::ast::*;

pub struct Analyzer {
}

impl Default for Analyzer {
  fn default() -> Self {
      Analyzer {
      }
  }
}

impl Analyzer {
  pub fn analyze_game(&mut self, game: &Game) -> Result<(), AnalyzerError> {
    if ct(game) {
      return Err(AnalyzerError::Default)
    }

    let ctx = ctx(game);
    let lowering_ctx = LoweringCtx::new(ctx);

    match game.lower(&lowering_ctx) {
      Ok(game) => println!("{:?}", game),
      Err(e) => return Err(AnalyzerError::TypeError(e))
    }
    

    Ok(())
  }
}
