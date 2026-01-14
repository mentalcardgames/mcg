use std::fmt;
use std::collections::{HashMap, HashSet};
use std::fmt::Display;

use crate::keywords::kw as kw;
use crate::dsl_types::DSLType;
use crate::ast::*;

#[derive(Debug)]
pub enum AnalyzerError {
    NoDslType,
    IdUsed,
    IdNotCapitalOrEmpty,
    InvalidInteger,
    ReservedKeyword,
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
      AnalyzerError::DuplicateIDs(ids) => 
          write!(f, "Duplicate IDs in {:?}", ids),
    }
  }
}

pub struct Analyzer {
  player_ids: HashSet<String>,
  team_ids: HashSet<String>,
  location_ids: HashSet<String>,
  precedence_ids: HashSet<String>,
  pointmap_ids: HashSet<String>,
  combo_ids: HashSet<String>,
  key_ids: HashSet<String>,
  value_ids: HashSet<String>,
  value_to_key: HashMap<String, String>,
  used_ids: HashSet<String>,
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
  pub fn add_id<T: ToString>(&mut self, id: T, dsl_type: DSLType) -> Result<(), AnalyzerError> {
    self.validate_id(&id)?;

    let id = id.to_string();

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

  fn check_id_is_int<T: ToString>(value: &T) -> bool {
    // If ID is int
    if let Ok(_) = value.to_string().trim().parse::<f64>() {
      return true
    }

    return false
  }

  fn check_id_is_used<T: ToString>(&self, value: &T) -> bool {
    self.used_ids.contains(&value.to_string())
  }

  fn check_id_is_custom_keyword<T: ToString>(value: &T) -> bool {
    return kw::in_custom_key_words(value)
  }

  fn check_id_starts_with_capital_or_empty<T: ToString>(value: &T) -> bool {
    if let Some(first_letter) = value.to_string().chars().next() {
      return first_letter.is_uppercase()
    } else {
      return true
    }
  }

  fn type_of_id<T: ToString>(&self, value: &T) -> Result<DSLType, AnalyzerError> {

    let value = &value.to_string(); 

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

  fn validate_id<T: ToString>(&self, value: &T) -> Result<(), AnalyzerError> {
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

  pub fn check_id<T: ToString>(value: &T) -> Result<(), AnalyzerError> {
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


  /// AMBIGIUTY IN:
  /// > Collection
  /// > BoolExpr
  /// > SetMemory
  /// 
  /// CHECKING IF ID IS USED CORRECTLY:
  /// > every ID is unique
  /// > assigned to its corresponding type
  ///
  /// CHECKING IF KEY AND VALUE RULES:
  /// > rule like: "Rank == Ace" need to be checked
  /// 
  /// IF ERRORS OCCUR: 
  /// > find the closest matching rule for the Error
  /// > check for "mis-spells" / "mis-types" and give propable Error
  ///
  /// ELSE (if something new needs to be analyzed):
  /// 
  pub fn analyze(&mut self, game: &Game) -> Result<Game, AnalyzerError> {


    todo!()
  }

  fn analyze_flow(&mut self, flow: &FlowComponent) -> Result<FlowComponent, AnalyzerError> {
    

    todo!()
  }

  fn analyze_rule(&mut self, rule: &Rule) -> Result<Rule, AnalyzerError> {
    match rule {
        Rule::CreatePlayer(player_names) => {
          self.validate_create_player(player_names)?;
        },
        Rule::CreateTeam(team_name, player_names) => {
          self.validate_create_team(team_name, player_names)?;
        },
        Rule::CreateTurnorder(player_names) => {
          self.validate_turnorder(player_names)?;
        },
        Rule::CreateTurnorderRandom(player_names) => {
          self.validate_turnorder(player_names)?;
        },
        Rule::CreateLocationOnPlayerCollection(location, player_collection) => {
          self.validate_create_location_on_player_collection(location, player_collection)?;
        },
        Rule::CreateLocationOnTeamCollection(location, team_collection) => {
          self.validate_create_location_on_team_collection(location, team_collection)?;
        },
        Rule::CreateLocationOnTable(location) => todo!(),
        Rule::CreateLocationCollectionOnPlayerCollection(location_collection, player_collection) => todo!(),
        Rule::CreateLocationCollectionOnTeamCollection(location_collection, team_collection) => todo!(),
        Rule::CreateLocationCollectionOnTable(location_collection) => todo!(),
        Rule::CreateCardOnLocation(location, types) => todo!(),
        Rule::CreateTokenOnLocation(int_expr, token, location) => todo!(),
        Rule::CreatePrecedence(precedence, items) => todo!(),
        Rule::CreateCombo(combo, filter_expr) => todo!(),
        Rule::CreateMemoryIntPlayerCollection(memory, int_expr, player_collection) => todo!(),
        Rule::CreateMemoryStringPlayerCollection(memory, string_expr, player_collection) => todo!(),
        Rule::CreateMemoryIntTable(memory, int_expr) => todo!(),
        Rule::CreateMemoryStringTable(memory, string_expr) => todo!(),
        Rule::CreateMemoryPlayerCollection(memory, player_collection) => todo!(),
        Rule::CreateMemoryTable(memory) => todo!(),
        Rule::CreatePointMap(point_map, items) => todo!(),
        Rule::FlipAction(card_set, status) => todo!(),
        Rule::ShuffleAction(card_set) => todo!(),
        Rule::PlayerOutOfStageAction(player_expr) => todo!(),
        Rule::PlayerOutOfGameSuccAction(player_expr) => todo!(),
        Rule::PlayerOutOfGameFailAction(player_expr) => todo!(),
        Rule::PlayerCollectionOutOfStageAction(player_collection) => todo!(),
        Rule::PlayerCollectionOutOfGameSuccAction(player_collection) => todo!(),
        Rule::PlayerCollectionOutOfGameFailAction(player_collection) => todo!(),
        Rule::SetMemoryInt(memory, int_expr) => todo!(),
        Rule::SetMemoryString(memory, string_expr) => todo!(),
        Rule::SetMemoryCollection(memory, collection) => todo!(),
        Rule::SetMemoryAmbiguous(memory, id) => todo!(),
        Rule::CycleAction(player_expr) => todo!(),
        Rule::BidAction(quantity) => todo!(),
        Rule::BidActionMemory(memory, quantity) => todo!(),
        Rule::EndTurn => todo!(),
        Rule::EndStage => todo!(),
        Rule::EndGameWithWinner(player_expr) => todo!(),
        Rule::DemandCardPositionAction(card_position) => todo!(),
        Rule::DemandStringAction(string_expr) => todo!(),
        Rule::DemandIntAction(int_expr) => todo!(),
        Rule::ClassicMove(classic_move) => todo!(),
        Rule::DealMove(deal_move) => todo!(),
        Rule::ExchangeMove(exchange_move) => todo!(),
        Rule::TokenMove(token_move) => todo!(),
        Rule::ScoreRule(score_rule) => todo!(),
        Rule::WinnerRule(winner_rule) => todo!(),
    }

    // TODO: Check the return type
    return Ok(rule.clone())
  }

  fn player_known(&self, player_name: &PlayerName) -> Result<(), AnalyzerError> {
    if !self.player_ids.contains(&player_name.0) {
      return Err(AnalyzerError::UnknownPlayerNameUsed(player_name.0.clone()))
    }

    return Ok(())
  }

  fn players_known(&self, player_names: &Vec<PlayerName>) -> Result<(), AnalyzerError> {
    for player in player_names.iter() {
      if !self.player_ids.contains(&player.0) {
        return Err(AnalyzerError::UnknownPlayerNameUsed(player.0.clone()))
      }
    }

    return Ok(())
  }

  fn has_duplicates(&self, v: &Vec<String>) -> Result<(), AnalyzerError> {
    let mut set = HashSet::new();
    if v.iter().any(|item| !set.insert(item)) {
      return Err(AnalyzerError::DuplicateIDs(v.clone()))
    }

    return Ok(())
  }

  fn has_duplicate_players(&self, player_names: &Vec<PlayerName>) -> Result<(), AnalyzerError> {
    self.has_duplicates(
      &player_names
        .iter()
        .map(|p| p.0.clone())
        .collect()
    )?;

    return Ok(())
  }

  // CreatePlayer
  // =========================================================================

  fn validate_create_player(&mut self, player_names: &Vec<PlayerName>) -> Result<(), AnalyzerError> {
    for player in player_names.iter() {
      self.validate_id(&player.0)?;
      self.add_id(&player.0, DSLType::Player)?;
    }

    return Ok(())
  }

  // =========================================================================


  // CreateTeam
  // =========================================================================

  fn validate_create_team(&mut self, team_name: &TeamName, player_names: &Vec<PlayerName>) -> Result<(), AnalyzerError> {
    self.has_duplicate_players(player_names)?;
    self.players_known(player_names)?;
    self.validate_id(&team_name.0)?;
    self.add_id(&team_name.0, DSLType::Team)?;

    return Ok(())
  }

  // =========================================================================


  // CreateTurnorder
  // =========================================================================

  fn validate_turnorder(&self, player_names: &Vec<PlayerName>) -> Result<(), AnalyzerError> {
    self.has_duplicate_players(player_names)?;
    self.players_known(player_names)?;

    return Ok(())
  }

  // =========================================================================


  // CreateLocationOnPlayerCollection
  // =========================================================================

  fn validate_player_expr(&self, player_expr: &PlayerExpr) -> Result<(), AnalyzerError> {
    match player_expr {
        PlayerExpr::PlayerName(player_name) => {
          self.player_known(player_name)?;
        },
        _ => {},
    }

    return Ok(())
  }

  fn validate_player_collection(&self, player_collection: &PlayerCollection) -> Result<(), AnalyzerError> {
    match player_collection {
        PlayerCollection::Player(player_exprs) => {
          for player_expr in player_exprs.iter() {
            self.validate_player_expr(player_expr)?;
          }
        },
        _ => {},
    }

    return Ok(())
  }

  fn validate_create_location_on_player_collection(&mut self, location: &Location, player_collection: &PlayerCollection) -> Result<(), AnalyzerError> {
    self.validate_id(&location.0)?;
    self.validate_player_collection(&player_collection)?;

    return Ok(())
  }

  // =========================================================================


  // CreateLocationOnTeamCollection
  // =========================================================================

  fn team_known(&self, team_name: &TeamName) -> Result<(), AnalyzerError> {
    if !self.team_ids.contains(&team_name.0) {
      return Err(AnalyzerError::UnknownPlayerNameUsed(team_name.0.clone()))
    }

    return Ok(())
  }

  fn teams_known(&self, team_names: &Vec<TeamName>) -> Result<(), AnalyzerError> {
    for team in team_names.iter() {
      if !self.team_ids.contains(&team.0) {
        return Err(AnalyzerError::UnknownPlayerNameUsed(team.0.clone()))
      }
    }

    return Ok(())
  }


  fn validate_team_expr(&self, team_expr: &TeamExpr) -> Result<(), AnalyzerError> {
    match team_expr {
        TeamExpr::TeamName(team_name) => {
          self.team_known(team_name)?;
        },
        TeamExpr::TeamOf(player_expr) => {
          self.validate_player_expr(player_expr)?;
        },
    }

    return Ok(())
  }

  fn validate_team_collection(&self, team_collection: &TeamCollection) -> Result<(), AnalyzerError> {
    match team_collection {
        TeamCollection::Team(team_exprs) => {
          for team_expr in team_exprs.iter() {
            self.validate_team_expr(team_expr)?;
          }
        },
        _ => {},
    }

    return Ok(())
  }

  fn has_duplicate_teams(&self, team_names: &Vec<TeamName>) -> Result<(), AnalyzerError> {
    self.has_duplicates(
      &team_names
        .iter()
        .map(|p| p.0.clone())
        .collect()
    )?;

    return Ok(())
  }


  fn validate_create_location_on_team_collection(&mut self, location: &Location, team_collection: &TeamCollection) -> Result<(), AnalyzerError> {
    self.validate_id(&location.0)?;
    self.validate_team_collection(&team_collection)?;

    return Ok(())
  }

  // =========================================================================


  // CreateLocationOnTable
  // =========================================================================

  // =========================================================================

}
