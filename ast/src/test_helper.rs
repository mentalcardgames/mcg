pub mod test_helper {
  use crate::ast::*;

  pub const CURRENT: PlayerExpr = PlayerExpr::Current;
  pub const PREVIOUS: PlayerExpr = PlayerExpr::Previous;
  pub const COMPETITOR: PlayerExpr = PlayerExpr::Competitor;

  pub fn id(id: &str) -> ID {
    ID::new(id)
  }

  pub fn stage(id: &str) -> Stage {
    Stage::new(ID::new(id))
  }

  pub fn playername(id: &str) -> PlayerName {
    PlayerName::new(ID::new(id))
  }

  pub fn teamname(id: &str) -> TeamName {
    TeamName::new(ID::new(id))
  }

  pub fn location(id: &str) -> Location {
    Location::new(ID::new(id))
  }

  pub fn token(id: &str) -> Token {
    Token::new(ID::new(id))
  }

  pub fn precedence(id: &str) -> Precedence {
    Precedence::new(ID::new(id))
  }

  pub fn pointmap(id: &str) -> PointMap {
    PointMap::new(ID::new(id))
  }

  pub fn combo(id: &str) -> Combo {
    Combo::new(ID::new(id))
  }

  pub fn memory(id: &str) -> Memory {
    Memory::new(ID::new(id))
  }

  pub fn key(id: &str) -> Key {
    Key::new(ID::new(id))
  }

  pub fn value(id: &str) -> Value {
    Value::new(ID::new(id))
  }
}
