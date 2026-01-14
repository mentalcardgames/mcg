use std::fmt;
use std::collections::{HashMap, HashSet};
use std::fmt::Display;

use crate::keywords::kw as kw;
use crate::dsl_types::DSLType;

use syn::Ident;

#[derive(Debug)]
pub enum AnalyzerError {
    NoDslType,
    IdUsed,
    IdNotCapitalOrEmpty,
    InvalidInteger,
    ReservedKeyword,
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
      }
    }
}

pub struct Analyzer {
  player_ids: HashSet<Ident>,
  team_ids: HashSet<Ident>,
  location_ids: HashSet<Ident>,
  precedence_ids: HashSet<Ident>,
  pointmap_ids: HashSet<Ident>,
  combo_ids: HashSet<Ident>,
  key_ids: HashSet<Ident>,
  value_ids: HashSet<Ident>,
  value_to_key: HashMap<Ident, Ident>,
  used_ids: HashSet<Ident>,
}

impl Default for Analyzer {
  fn default() -> Self {
      Analyzer { 
        player_ids: HashSet::new(),
        team_ids: HashSet::new(),
        location_ids: HashSet::new(),
        precedence_ids: HashSet::new(),
        pointmap_ids: HashSet::new(),
        combo_ids: HashSet::new(),
        key_ids: HashSet::new(),
        value_ids: HashSet::new(),
        value_to_key: HashMap::new(),
        used_ids: HashSet::new(),
      }
  }
}

impl Analyzer {
  pub fn add_id(&mut self, id: Ident, dsl_type: DSLType) -> Result<(), AnalyzerError> {
    self.validate_id(&id)?;

    self.used_ids.insert(id.clone());

    match dsl_type {
        DSLType::Player => {
          self.player_ids.insert(id);
          return Ok(())
        },
        DSLType::Team => {
          self.team_ids.insert(id);
          return Ok(())
        },
        DSLType::Location => {
          self.location_ids.insert(id);
          return Ok(());
        },
        DSLType::Key => {
          self.key_ids.insert(id);
          return Ok(());
        },
        DSLType::Value => {
          self.value_ids.insert(id);
          return Ok(());
        },
        DSLType::Precedence => {
          self.precedence_ids.insert(id);
          return Ok(());
        },
        DSLType::PointMap => {
          self.pointmap_ids.insert(id);
          return Ok(());
        },
        DSLType::Combo => {
          self.combo_ids.insert(id);
          return Ok(());
        }
    }
  }

  fn check_id_is_int(value: &Ident) -> bool {
    // If ID is int
    if let Ok(_) = value.to_string().trim().parse::<f64>() {
      return true
    }

    return false
  }

  fn check_id_is_used(&self, value: &Ident) -> bool {
    self.used_ids.contains(value)
  }

  fn check_id_is_custom_keyword(value: &Ident) -> bool {
    return kw::in_custom_key_words(value)
  }

  fn check_id_starts_with_capital_or_empty(value: &Ident) -> bool {
    if let Some(first_letter) = value.to_string().chars().next() {
      return first_letter.is_uppercase()
    } else {
      return true
    }
  }

  fn type_of_id(&self, value: &Ident) -> Result<DSLType, AnalyzerError> {
    if self.player_ids.contains(value) {
      return Ok(DSLType::Player)
    }
    if self.team_ids.contains(value) {
      return Ok(DSLType::Team)
    }
    if self.location_ids.contains(value) {
      return Ok(DSLType::Location)
    }
    if self.precedence_ids.contains(value) {
      return Ok(DSLType::Precedence)
    }
    if self.pointmap_ids.contains(value) {
      return Ok(DSLType::PointMap)
    }
    if self.key_ids.contains(value) {
      return Ok(DSLType::Key)
    }
    if self.value_ids.contains(value) {
      return Ok(DSLType::Value)
    }

    return Err(AnalyzerError::NoDslType)
  }

  fn validate_id(&self, value: &Ident) -> Result<(), AnalyzerError> {
    if Self::check_id_is_int(value) {
      return Err(AnalyzerError::InvalidInteger)
    }
    if Self::check_id_starts_with_capital_or_empty(value) {
      return Err(AnalyzerError::IdNotCapitalOrEmpty)
    }
    if Self::check_id_is_custom_keyword(value) {
      return Err(AnalyzerError::ReservedKeyword)
    }
    if self.check_id_is_used(value) {
      return Err(AnalyzerError::IdUsed)
    }

    return Ok(())
  }

  pub fn check_id(value: &Ident) -> Result<(), AnalyzerError> {
    if Self::check_id_is_int(value) {
      return Err(AnalyzerError::InvalidInteger)
    }
    if !Self::check_id_starts_with_capital_or_empty(value) {
      return Err(AnalyzerError::IdNotCapitalOrEmpty)
    }
    if Self::check_id_is_custom_keyword(value) {
      return Err(AnalyzerError::ReservedKeyword)
    }

    return Ok(())
  }

  // Collections
}
