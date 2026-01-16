use std::fmt;
use std::fmt::Display;

#[derive(Debug)]
pub enum AnalyzerError {
    NoDslType,
    IdUsed,
    IdNotCapitalOrEmpty,
    InvalidInteger,
    ReservedKeyword,
    UnknownID(String),
    UnknownPlayerNameUsed(String),
    DuplicateIDs(Vec<String>),
}

impl Display for AnalyzerError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      AnalyzerError::NoDslType =>
          write!(f, "no DSL type specified"),
      AnalyzerError::IdUsed =>
          write!(f, "identifier is already used"),
      AnalyzerError::IdNotCapitalOrEmpty =>
          write!(f, "identifier must be non-empty and start with a capital letter"),
      AnalyzerError::InvalidInteger =>
          write!(f, "invalid integer"),
      AnalyzerError::ReservedKeyword =>
          write!(f, "identifier is a reserved keyword"),
      AnalyzerError::UnknownPlayerNameUsed(player) =>
          write!(f, "Player {} unknown", player),
      AnalyzerError::UnknownID(id) =>
          write!(f, "ID {} unknown", id),
      AnalyzerError::DuplicateIDs(ids) => 
          write!(f, "Duplicate IDs in {:?}", ids),
    }
  }
}
