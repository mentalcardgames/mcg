pub mod test_helper {
  use crate::asts::ast::*;

  pub const CURRENT: PlayerExpr = PlayerExpr::Current;
  pub const PREVIOUS: PlayerExpr = PlayerExpr::Previous;
  pub const COMPETITOR: PlayerExpr = PlayerExpr::Competitor;

  pub fn id(id: &str) -> String {
    id.to_string()
  }

  pub fn stage(id: &str) -> Stage {
    id.to_string()
  }

  pub fn playername(id: &str) -> PlayerName {
    id.to_string()
  }

  pub fn teamname(id: &str) -> TeamName {
    id.to_string()
  }

  pub fn location(id: &str) -> Location {
    id.to_string()
  }

  pub fn token(id: &str) -> Token {
    id.to_string()
  }

  pub fn precedence(id: &str) -> Precedence {
    id.to_string()
  }

  pub fn pointmap(id: &str) -> PointMap {
    id.to_string()
  }

  pub fn combo(id: &str) -> Combo {
    id.to_string()
  }

  pub fn memory(id: &str) -> Memory {
    id.to_string()
  }

  pub fn key(id: &str) -> Key {
    id.to_string()
  }

  pub fn value(id: &str) -> Value {
    id.to_string()
  }
}
