use crate::{analyzer::analyzer_error::AnalyzerError, asts::game_type::GameType};

pub type TypedVars = Vec<(String, GameType)>;

pub fn ambiguous(ctx: TypedVars) -> Result<(), AnalyzerError> {
  let mut new_ctx = ctx;
  let (next, _) = new_ctx.first().unwrap().clone();
  let ctx_same = new_ctx.clone().into_iter().filter(|(s, _)| *s == next).collect();
  
  if all_no_type(&ctx_same) {
    return Err(AnalyzerError::IDWithNoType { id: next })
  }

  if multiple_options(&ctx_same) {
    return Err(AnalyzerError::IDWithMultipleTypes { id: next })
  }
  
  new_ctx = new_ctx.into_iter().filter(|(s, _)| *s != next).collect::<TypedVars>();

  if new_ctx.is_empty() {
    return Ok(())
  }

  ambiguous(new_ctx)
}

fn all_no_type(ctx: &TypedVars) -> bool {
  ctx
    .iter()
    .filter(|(_, t)| *t != GameType::NoType)
    .collect::<Vec<&(String, GameType)>>()
    .is_empty()
}

fn multiple_options(ctx: &TypedVars) -> bool {
  !ctx
    .iter()
    .filter(
      |(_, ty)|
        *ty != ctx.first().unwrap().1 && *ty != GameType::NoType
    )
    .collect::<Vec<&(String, GameType)>>()
    .is_empty()
}
