use std::collections::HashSet;
use crate::analyzer::analyzer_error::AnalyzerError;
use crate::analyzer::symbol::semantic_op::SemanticOp;
use crate::dsl_types::DSLType;
use crate::ast::*;


pub struct SymbolAnalyzer {
  players: HashSet<String>,
  teams: HashSet<String>,
  locations: HashSet<String>,
  keys: HashSet<String>,
  values: HashSet<String>,
  precedences: HashSet<String>,
  pointmaps: HashSet<String>,
  combos: HashSet<String>,
  tokens: HashSet<String>,
  memories: HashSet<String>,
  known_ids: HashSet<String>
}

impl Default for SymbolAnalyzer {
  fn default() -> Self {
    SymbolAnalyzer { 
      players: HashSet::new(),
      teams: HashSet::new(),
      locations: HashSet::new(),
      keys: HashSet::new(),
      values: HashSet::new(),
      precedences: HashSet::new(),
      pointmaps: HashSet::new(),
      combos: HashSet::new(),
      tokens: HashSet::new(),
      memories: HashSet::new(),
      known_ids: HashSet::new()
    }
  }
}

impl SymbolAnalyzer {
  // Initialize ID
  // =========================================================================
  // =========================================================================
  // =========================================================================
  fn initialize_id(&mut self, id: String, ty: DSLType) -> Result<(), AnalyzerError> {
    if   self.known_ids.contains(&id.clone()) {
      if ty != DSLType::Key || ty != DSLType::Value {
        return Ok(())
      }
      return Err(AnalyzerError::IdUsed)
    }

    let new_id = id.clone();
  
    match ty {
        DSLType::Player => {
          self.players.insert(id);
        },
        DSLType::Team => {
          self.teams.insert(id);
        },
        DSLType::Location => {
          self.locations.insert(id);
        },
        DSLType::Key => {
          self.keys.insert(id);
        },
        DSLType::Value => {
          self.values.insert(id);
        },
        DSLType::Precedence => {
          self.precedences.insert(id);
        },
        DSLType::PointMap => {
          self.pointmaps.insert(id);
        },
        DSLType::Combo => {
          self.combos.insert(id);
        },
        DSLType::Token => {
          self.tokens.insert(id);
        },
        DSLType::Memory => {
          self.memories.insert(id);
        },
    }

    self.known_ids.insert(new_id);

    return Ok(())
  }
  // =========================================================================
  // =========================================================================
  // =========================================================================


  // UseId
  // =========================================================================
  // =========================================================================
  // =========================================================================
  fn use_id(&mut self, id: String, ty: DSLType) -> Result<(), AnalyzerError>{
    match ty {
        DSLType::Player => {
          if !self.players.contains(&id) {
            return Err(AnalyzerError::UnknownID(id))
          }
        },
        DSLType::Team => {
          if !self.teams.contains(&id) {
            return Err(AnalyzerError::UnknownID(id))
          }
        },
        DSLType::Location => {
          if !self.locations.contains(&id) {
            return Err(AnalyzerError::UnknownID(id))
          }
        },
        DSLType::Key => {
          if !self.keys.contains(&id) {
            return Err(AnalyzerError::UnknownID(id))
          }
        },
        DSLType::Value => {
          if !self.values.contains(&id) {
            return Err(AnalyzerError::UnknownID(id))
          }
        },
        DSLType::Precedence => {
          if !self.precedences.contains(&id) {
            return Err(AnalyzerError::UnknownID(id))
          }
        },
        DSLType::PointMap => {
          if !self.pointmaps.contains(&id) {
            return Err(AnalyzerError::UnknownID(id))
          }
        },
        DSLType::Combo => {
          if !self.combos.contains(&id) {
            return Err(AnalyzerError::UnknownID(id))
          }
        },
        DSLType::Token => {
          if !self.tokens.contains(&id) {
            return Err(AnalyzerError::UnknownID(id))
          }
        },
        DSLType::Memory => {
          if !self.memories.contains(&id) {
            return Err(AnalyzerError::UnknownID(id))
          }
        },
    }

    return Ok(())
  }
  // =========================================================================
  // =========================================================================
  // =========================================================================


  // Evaluate Semantic Operator
  // =========================================================================
  // =========================================================================
  // =========================================================================
  pub fn eval_semantic_op(&mut self, semantic_op: SemanticOp) -> Result<(), AnalyzerError> {
    match semantic_op {
        SemanticOp::Initialize { id, ty } => {
            self.initialize_id(id, ty)?;
          },
        SemanticOp::InitializeVec { vec_id, ty } => {
            self.eval_initialize_vec(vec_id, ty)?;
          },
        SemanticOp::Use { id, ty } => {
            self.use_id(id, ty)?;
          },
        SemanticOp::UseVec { vec_id, ty } => {
            self.eval_use_vec(vec_id, ty)?;
          },
        SemanticOp::UseCollection { collection } => {
              self.eval_use_collection(collection)?;
          },
        SemanticOp::InitializeKeyValue { key, value } => {
            self.eval_initialize_key_value(key, value)?;
          },
        SemanticOp::UseFilter { filter } => {
            self.eval_use_filter(filter)?;
          },
        SemanticOp::UsePlayer { player } => {
            self.eval_use_player(player)?;
          },
        SemanticOp::UseCardSet { card_set } => {
            self.eval_use_card_set(card_set)?;
          },
        SemanticOp::UseInt { int_expr } => {
            self.eval_use_int(int_expr)?;
          },
        SemanticOp::UseString { string_expr } => {
            self.eval_use_string(string_expr)?;
          },
        SemanticOp::UseTeam { team } => {
            self.eval_use_team(team)?;
          },
        SemanticOp::UseGroup { group } => {
            self.eval_use_group(group)?;
          },
        SemanticOp::UseCardPosition { card_position } => {
            self.eval_use_card_position(card_position)?;
          },
        SemanticOp::UseClassicMove { classic_move } => {
            self.eval_use_classic_move(classic_move)?;
          },
        SemanticOp::UseTokenLoc { token_loc_expr } => {
          self.eval_use_token_loc(token_loc_expr)?;
        },
        SemanticOp::UseDealMove { deal_move } => {
          self.eval_use_deal_move(deal_move)?;
        },
        SemanticOp::UseExchangeMove { exchange_move } => {
          self.eval_use_exchange_move(exchange_move)?;
        },
        SemanticOp::UseTokenMove { token_move } => {
          self.eval_use_token_move(token_move)?;
        },
        SemanticOp::UseScoreRule { score_rule } => {
          self.eval_use_score_rule(score_rule)?;
        },
        SemanticOp::UseWinnerRule { winner_rule } => {
          self.eval_use_winner_rule(winner_rule)?;
        },
        SemanticOp::UseEndCondition { end_condition } => {
          self.eval_use_end_condition(end_condition)?;
        },
        SemanticOp::UseBoolExpr { bool_expr } => {
          self.eval_use_bool(bool_expr)?;
        },
    }

    return Ok(())
  }

  // UseCollection
  // =========================================================================
  fn eval_use_collection(&mut self, collection: Collection) -> Result<(), AnalyzerError> {
    match collection {
      Collection::IntCollection(int_collection) => {
        self.eval_use_int_collection(int_collection)?;
      },
      Collection::StringCollection(string_collection) => {
        self.eval_use_string_collection(string_collection)?;
      },
      Collection::LocationCollection(location_collection) => {
        self.eval_use_location_collection(location_collection)?;
      },
      Collection::PlayerCollection(player_collection) => {
        self.eval_use_player_collection(player_collection)?;
      },
      Collection::TeamCollection(team_collection) => {
        self.eval_use_team_collection(team_collection)?;
      },
      Collection::CardSet(card_set) => {
        self.eval_semantic_op(
          SemanticOp::UseCardSet { card_set: *card_set }
        )?;
      },
      Collection::Ambiguous(ids) => {
        for id in ids.iter() {
          if !self.known_ids.contains(&id.0) {
            return Err(AnalyzerError::UnknownID(id.0.clone()))
          }
        }
      },
    }

    return Ok(())
  }

  fn eval_use_int_collection(&mut self, int_collection: IntCollection) -> Result<(), AnalyzerError> {
    for int in int_collection.ints.iter() {
      self.eval_semantic_op(
        SemanticOp::UseInt { 
          int_expr: int.clone()
        }
      )?;
    }

    return Ok(())
  }

  fn eval_use_string_collection(&mut self, string_collection: StringCollection) -> Result<(), AnalyzerError> {
    for string in string_collection.strings.iter() {
      self.eval_semantic_op(
        SemanticOp::UseString { 
          string_expr: string.clone()
        }
      )?;
    }

    return Ok(())
  }

  fn eval_use_location_collection(&mut self, location_collection: LocationCollection) -> Result<(), AnalyzerError> {
    for location in location_collection.locations.iter() {
      self.use_id(location.0.clone(), DSLType::Location)?;
    }

    return Ok(())
  }

  fn eval_use_player_collection(&mut self, player_collection: PlayerCollection) -> Result<(), AnalyzerError> {
    match player_collection {
      PlayerCollection::Player(player_exprs) => {
        for player in player_exprs.iter() {
          self.eval_semantic_op(
            SemanticOp::UsePlayer { 
              player: player.clone()
            }
          )?;
        }
      },
      _ => {}
    }

    return Ok(())
  }
  
  fn eval_use_team_collection(&mut self, team_collection: TeamCollection) -> Result<(), AnalyzerError> {
    match team_collection {
      TeamCollection::Team(team_exprs) => {
        for team in team_exprs.iter() {
          self.eval_semantic_op(
            SemanticOp::UseTeam { 
              team: team.clone()
            }
          )?;
        }
      },
      _ => {},
    }

    return Ok(())
  }


  // =========================================================================


  // UseFilter
  // =========================================================================
  fn eval_use_filter(&mut self, filter_expr: FilterExpr) -> Result<(), AnalyzerError> {
    match filter_expr {
      FilterExpr::Same(key) => {
        self.use_id(key.0.clone(), DSLType::Key)?;
      },
      FilterExpr::Distinct(key) => {
        self.use_id(key.0.clone(), DSLType::Key)?;
      },
      FilterExpr::Adjacent(key, precedence) => {
        self.use_id(key.0.clone(), DSLType::Key)?;
        self.use_id(precedence.0.clone(), DSLType::Precedence)?;
      },
      FilterExpr::Higher(key, precedence) => {
        self.use_id(key.0.clone(), DSLType::Key)?;
        self.use_id(precedence.0.clone(), DSLType::Precedence)?;
      },
      FilterExpr::Lower(key, precedence) => {
        self.use_id(key.0.clone(), DSLType::Key)?;
        self.use_id(precedence.0.clone(), DSLType::Precedence)?;
      },
      FilterExpr::Size(_, int_expr) => {
        self.eval_semantic_op(
          SemanticOp::UseInt {
            int_expr: *int_expr
          }
        )?;
      },
      FilterExpr::KeyEq(key, string_expr) => {
        self.use_id(key.0.clone(), DSLType::Key)?;
        // TODO: Check if Key and String are compatible
        self.eval_semantic_op(
          SemanticOp::UseString {
            string_expr: *string_expr
          }
        )?;
      },
      FilterExpr::KeyNeq(key, string_expr) => {
        self.use_id(key.0.clone(), DSLType::Key)?;
        // TODO: Check if Key and String are compatible
        self.eval_semantic_op(
          SemanticOp::UseString {
            string_expr: *string_expr
          }
        )?;
      },
      FilterExpr::NotCombo(combo) => {
        self.use_id(combo.0.clone(), DSLType::Combo)?;
      },
      FilterExpr::Combo(combo) => {
        self.use_id(combo.0.clone(), DSLType::Combo)?;
      },
      FilterExpr::And(filter_expr, filter_expr1) => {
        self.eval_semantic_op(
          SemanticOp::UseFilter { filter: *filter_expr }
        )?;
        self.eval_semantic_op(
          SemanticOp::UseFilter { filter: *filter_expr1 }
        )?;
      },
      FilterExpr::Or(filter_expr, filter_expr1) => {
        self.eval_semantic_op(
          SemanticOp::UseFilter { filter: *filter_expr }
        )?;
        self.eval_semantic_op(
          SemanticOp::UseFilter { filter: *filter_expr1 }
        )?;
      },
    }

    return Ok(())
  }

  // =========================================================================


  // UsePlayer
  // =========================================================================
  fn eval_use_player(&mut self, player_expr: PlayerExpr) -> Result<(), AnalyzerError> {
    match player_expr {
        PlayerExpr::PlayerName(player_name) => {
          self.use_id(player_name.0.clone(), DSLType::Player)?;
        },
        PlayerExpr::Turnorder(int_expr) => {
          self.eval_semantic_op(
            SemanticOp::UseInt { int_expr }
          )?;
        },
        PlayerExpr::OwnerOf(card_position) => {
          self.eval_semantic_op(
            SemanticOp::UseCardPosition { card_position: *card_position }
          )?;
        },
        PlayerExpr::OwnerOfHighest(memory) => {
            self.use_id(memory.0.clone(), DSLType::Memory)?;
        },
        PlayerExpr::OwnerOfLowest(memory) => {
            self.use_id(memory.0.clone(), DSLType::Memory)?;
        },
        _ => {},
    }

    return Ok(())
  }

  // =========================================================================


  // UseCardSet
  // =========================================================================
  fn eval_use_card_set(&mut self, card_set: CardSet) -> Result<(), AnalyzerError> {
    match card_set {
        CardSet::Group(group) => {
          self.eval_semantic_op(
            SemanticOp::UseGroup {
              group
            }
          )?;
        },
        CardSet::GroupOfPlayer(group, player_expr) => {
          self.eval_semantic_op(
            SemanticOp::UsePlayer { player: player_expr }
          )?;
          self.eval_semantic_op(
            SemanticOp::UseGroup {
              group
            }
          )?;
        },
        CardSet::GroupOfPlayerCollection(group, player_collection) => {
          self.eval_semantic_op(
            SemanticOp::UseCollection { collection: Collection::PlayerCollection(player_collection) }
          )?;
          self.eval_semantic_op(
            SemanticOp::UseGroup {
              group
            }
          )?;
        },
    }

    return Ok(())
  }

  // =========================================================================


  // UseInt
  // =========================================================================
  fn eval_use_int(&mut self, int_expr: IntExpr) -> Result<(), AnalyzerError> {
    match int_expr {
        IntExpr::Int(_) => {},
        IntExpr::IntOp(int_expr, _, int_expr1) => {
          self.eval_semantic_op(
            SemanticOp::UseInt {
              int_expr: *int_expr
            }
          )?;
          self.eval_semantic_op(
            SemanticOp::UseInt {
              int_expr: *int_expr1
            }
          )?;
        },
        IntExpr::IntCollectionAt(int_expr) => {
          self.eval_semantic_op(
            SemanticOp::UseInt {
              int_expr: *int_expr
            }
          )?;
        },
        IntExpr::SizeOf(collection) => {
          self.eval_semantic_op(
            SemanticOp::UseCollection {
              collection: collection
            }
          )?;
        },
        IntExpr::SumOfIntCollection(int_collection) => {
          self.eval_semantic_op(
            SemanticOp::UseCollection {
              collection: Collection::IntCollection(int_collection)
            }
          )?;
        },
        IntExpr::SumOfCardSet(card_set, point_map) => {
          self.eval_semantic_op(
            SemanticOp::UseCardSet { card_set: *card_set }
          )?;
          self.use_id(point_map.0.clone(), DSLType::PointMap)?;
        },
        IntExpr::MinOf(card_set, point_map) => {
          self.eval_semantic_op(
            SemanticOp::UseCardSet { card_set: *card_set }
          )?;
          self.use_id(point_map.0.clone(), DSLType::PointMap)?;
        },
        IntExpr::MaxOf(card_set, point_map) => {
          self.eval_semantic_op(
            SemanticOp::UseCardSet { card_set: *card_set }
          )?;
          self.use_id(point_map.0.clone(), DSLType::PointMap)?;
        },
        IntExpr::MinIntCollection(int_collection) => {
          self.eval_semantic_op(
            SemanticOp::UseCollection {
              collection: Collection::IntCollection(int_collection)
            }
          )?;
        },
        IntExpr::MaxIntCollection(int_collection) => {
          self.eval_semantic_op(
            SemanticOp::UseCollection {
              collection: Collection::IntCollection(int_collection)
            }
          )?;
        },
        IntExpr::StageRoundCounter => {},
    }

    return Ok(())
  }

  // =========================================================================


  // UseString
  // =========================================================================
  fn eval_use_string(&mut self, string_expr: StringExpr) -> Result<(), AnalyzerError> {
    match string_expr {
        StringExpr::ID(_) => {},
        StringExpr::KeyOf(key, card_position) => {
          self.use_id(key.0.clone(), DSLType::Key)?;
          self.eval_semantic_op(
            SemanticOp::UseCardPosition {
              card_position: card_position
            }
          )?;
        },
        StringExpr::StringCollectionAt(string_collection, int_expr) => {
          self.eval_semantic_op(
            SemanticOp::UseCollection { collection: Collection::StringCollection(string_collection) }
          )?;
          self.eval_semantic_op(
            SemanticOp::UseInt { int_expr }
          )?;
        },
    }

    return Ok(())
  }

  // =========================================================================


  // UseTeam
  // =========================================================================
  fn eval_use_team(&mut self, team_expr: TeamExpr) -> Result<(), AnalyzerError> {
    match team_expr {
        TeamExpr::TeamName(team_name) => {
          self.use_id(team_name.0.clone(), DSLType::Team)?;
        },
        TeamExpr::TeamOf(player_expr) => {
          self.eval_semantic_op(
            SemanticOp::UsePlayer { player: player_expr }
          )?;
        },
    }

    return Ok(())
  }

  // =========================================================================


  // UseGroup
  // =========================================================================
  fn eval_use_group(&mut self, group: Group) -> Result<(), AnalyzerError> {
    match group {
        Group::Location(location) => {
          self.use_id(location.0.clone(), DSLType::Location)?;
        },
        Group::LocationWhere(location, filter_expr) => {
          self.use_id(location.0.clone(), DSLType::Location)?;
          self.eval_semantic_op(
            SemanticOp::UseFilter { filter: filter_expr }
          )?;
        },
        Group::LocationCollection(location_collection) => {
          self.eval_semantic_op(
            SemanticOp::UseCollection { collection: Collection::LocationCollection(location_collection) }
          )?;
        },
        Group::LocationCollectionWhere(location_collection, filter_expr) => {
          self.eval_semantic_op(
            SemanticOp::UseCollection { collection: Collection::LocationCollection(location_collection) }
          )?;
          self.eval_semantic_op(
            SemanticOp::UseFilter { filter: filter_expr }
          )?;
        },
        Group::ComboInLocation(combo, location) => {
          self.use_id(combo.0.clone(), DSLType::Combo)?;
          self.use_id(location.0.clone(), DSLType::Location)?;
        },
        Group::ComboInLocationCollection(combo, location_collection) => {
          self.use_id(combo.0.clone(), DSLType::Combo)?;
          self.eval_semantic_op(
            SemanticOp::UseCollection { collection: Collection::LocationCollection(location_collection) }
          )?;
        },
        Group::NotComboInLocation(combo, location) => {
          self.use_id(combo.0.clone(), DSLType::Combo)?;
          self.use_id(location.0.clone(), DSLType::Location)?;
        },
        Group::NotComboInLocationCollection(combo, location_collection) => {
          self.use_id(combo.0.clone(), DSLType::Combo)?;
          self.eval_semantic_op(
            SemanticOp::UseCollection { collection: Collection::LocationCollection(location_collection) }
          )?;
        },
        Group::CardPosition(card_position) => {
          self.eval_semantic_op(
            SemanticOp::UseCardPosition { card_position }
          )?;
        },
    }

    return Ok(())
  }
  // =========================================================================
  

  // UseCardPosition
  // =========================================================================
  fn eval_use_card_position(&mut self, card_position: CardPosition) -> Result<(), AnalyzerError> {
    match card_position {
      CardPosition::At(location, int_expr) => {
        self.use_id(location.0.clone(), DSLType::Location)?;
        self.eval_semantic_op(
          SemanticOp::UseInt { int_expr }
        )?;
      },
      CardPosition::Top(location) => {
        self.use_id(location.0.clone(), DSLType::Location)?;
      },
      CardPosition::Bottom(location) => {
        self.use_id(location.0.clone(), DSLType::Location)?;
      },
      CardPosition::Max(card_set, id) => {
        self.eval_semantic_op(
          SemanticOp::UseCardSet { card_set: *card_set }
        )?;
        
        if   !self.precedences.contains(&id.0.clone())
          && !self.pointmaps.contains(&id.0.clone()) {

          return Err(AnalyzerError::UnknownID(id.0.clone()))
        } 
      },
      CardPosition::Min(card_set, id) => {
        self.eval_semantic_op(
          SemanticOp::UseCardSet { card_set: *card_set }
        )?;

        if   !self.precedences.contains(&id.0.clone())
          && !self.pointmaps.contains(&id.0.clone()) {

          return Err(AnalyzerError::UnknownID(id.0.clone()))
        }
      },
    }

    return Ok(())
  }

  // =========================================================================


  // InitializeKeyValue
  // =========================================================================
  fn eval_initialize_key_value(&mut self, key: String, value: String) -> Result<(), AnalyzerError> {
    self.initialize_id(key.clone(), DSLType::Key)?;
    self.initialize_id(value.clone(), DSLType::Value)?;

    return Ok(())
  }

  // =========================================================================
  

  // InitializeKeyValue
  // =========================================================================
  fn eval_initialize_vec(&mut self, vec_id: Vec<String>, ty: DSLType) -> Result<(), AnalyzerError> {
    // TODO: Check for duplicates
    for id in vec_id.iter() {
      self.initialize_id(id.clone(), ty.clone())?;
    }

    return Ok(())
  }

  // =========================================================================
  

  // UseVec
  // =========================================================================
  fn eval_use_vec(&mut self, vec_id: Vec<String>, ty: DSLType) -> Result<(), AnalyzerError> {
    // TODO: MAYBE Check for duplicates
    for id in vec_id.iter() {
      self.use_id(id.clone(), ty.clone())?;
    }

    return Ok(())
  }

  // =========================================================================


  // UseClassicMove
  // =========================================================================
  fn eval_use_classic_move(&mut self, classic_move: ClassicMove) -> Result<(), AnalyzerError> {
    match classic_move {
        ClassicMove::Move(card_set, _, card_set1) => {
          self.eval_use_card_set(card_set)?;
          self.eval_use_card_set(card_set1)?;
        },
        ClassicMove::MoveQuantity(_, card_set, _, card_set1) => {
          self.eval_use_card_set(card_set)?;
          self.eval_use_card_set(card_set1)?;
        },
    }

    return Ok(())
  }

  // =========================================================================


  // UseDealMove
  // =========================================================================
  fn eval_use_deal_move(&mut self, deal_move: DealMove) -> Result<(), AnalyzerError> {
    match deal_move {
        DealMove::Deal(card_set, _, card_set1) => {
          self.eval_use_card_set(card_set)?;
          self.eval_use_card_set(card_set1)?;
        },
        DealMove::DealQuantity(_, card_set, _, card_set1) => {
          self.eval_use_card_set(card_set)?;
          self.eval_use_card_set(card_set1)?;
        },
    }

    return Ok(())
  }

  // =========================================================================


  // UseExchangeMove
  // =========================================================================
  fn eval_use_exchange_move(&mut self, exchange_move: ExchangeMove) -> Result<(), AnalyzerError> {
    match exchange_move {
        ExchangeMove::Exchange(card_set, _, card_set1) => {
          self.eval_use_card_set(card_set)?;
          self.eval_use_card_set(card_set1)?;
        },
        ExchangeMove::ExchangeQuantity(_, card_set, _, card_set1) => {
          self.eval_use_card_set(card_set)?;
          self.eval_use_card_set(card_set1)?;
        },
    }

    return Ok(())
  }

  // =========================================================================


  // UseTokenMove
  // =========================================================================
  fn eval_use_token_loc(&mut self, token_loc_expr: TokenLocExpr) -> Result<(), AnalyzerError> {
    match token_loc_expr {
        TokenLocExpr::Location(location) => {
          self.use_id(location.0, DSLType::Location)?;
        },
        TokenLocExpr::LocationCollection(location_collection) => {
          self.eval_use_location_collection(location_collection)?;
        },
        TokenLocExpr::LocationPlayer(location, player_expr) => {
          self.use_id(location.0, DSLType::Location)?;
          self.eval_use_player(player_expr)?;
        },
        TokenLocExpr::LocationCollectionPlayer(location_collection, player_expr) => {
          self.eval_use_location_collection(location_collection)?;
          self.eval_use_player(player_expr)?;
        },
        TokenLocExpr::LocationPlayerCollection(location, player_collection) => {
          self.use_id(location.0, DSLType::Location)?;
          self.eval_use_player_collection(player_collection)?;
        },
        TokenLocExpr::LocationCollectionPlayerCollection(location_collection, player_collection) => {
          self.eval_use_location_collection(location_collection)?;
          self.eval_use_player_collection(player_collection)?;
        },
    }

    return Ok(())
  }

  // =========================================================================


  // UseTokenMove
  // =========================================================================
  fn eval_use_token_move(&mut self, token_move: TokenMove) -> Result<(), AnalyzerError> {
    match token_move {
        TokenMove::Place(token_loc_expr, token_loc_expr1) => {
          self.eval_use_token_loc(token_loc_expr)?;
          self.eval_use_token_loc(token_loc_expr1)?;
        },
        TokenMove::PlaceQuantity(_, token_loc_expr, token_loc_expr1) => {
          self.eval_use_token_loc(token_loc_expr)?;
          self.eval_use_token_loc(token_loc_expr1)?;
        },
    }

    return Ok(())
  }

  // =========================================================================


  // UseScoreRule
  // =========================================================================
  fn eval_use_score_rule(&mut self, score_rule: ScoreRule) -> Result<(), AnalyzerError> {
    match score_rule {
        ScoreRule::ScorePlayer(int_expr, player_expr) => {
          self.eval_use_int(int_expr)?;
          self.eval_use_player(player_expr)?;
        },
        ScoreRule::ScorePlayerMemory(int_expr, memory, player_expr) => {
          self.eval_use_int(int_expr)?;
          self.use_id(memory.0, DSLType::Memory)?;
          self.eval_use_player(player_expr)?;
        },
        ScoreRule::ScorePlayerCollection(int_expr, player_collection) => {
          self.eval_use_int(int_expr)?;
          self.eval_use_player_collection(player_collection)?;
        },
        ScoreRule::ScorePlayerCollectionMemory(int_expr, memory, player_collection) => {
          self.eval_use_int(int_expr)?;
          self.use_id(memory.0, DSLType::Memory)?;
          self.eval_use_player_collection(player_collection)?;
        },
    }

    return Ok(())
  }

  // =========================================================================


  // UseWinnerRule
  // =========================================================================
  fn eval_use_winner_rule(&mut self, winner_rule: WinnerRule) -> Result<(), AnalyzerError> {
    match winner_rule {
        WinnerRule::WinnerPlayer(player_expr) => {
          self.eval_use_player(player_expr)?;
        },
        WinnerRule::WinnerPlayerCollection(player_collection) => {
          self.eval_use_player_collection(player_collection)?;
        },
        WinnerRule::WinnerLowestMemory(memory) => {
          self.use_id(memory.0, DSLType::Memory)?;
        },
        WinnerRule::WinnerHighestMemory(memory) => {
          self.use_id(memory.0, DSLType::Memory)?;
        },
        _ => {},
    }

    return Ok(())
  }

  // =========================================================================


  // UseBoolExpr
  // =========================================================================
  fn eval_use_bool(&mut self, bool_expr: BoolExpr) -> Result<(), AnalyzerError> {
    match bool_expr {
        BoolExpr::IntCmp(int_expr, _, int_expr1) => {
          self.eval_use_int(int_expr)?;
          self.eval_use_int(int_expr1)?;
        },
        BoolExpr::CardSetIsEmpty(card_set) => {
          self.eval_use_card_set(card_set)?;
        },
        BoolExpr::CardSetIsNotEmpty(card_set) => {
          self.eval_use_card_set(card_set)?;
        },
        BoolExpr::AmbiguousEq(id, id1) => {
          if   !self.known_ids.contains(&id.0) {
            return Err(AnalyzerError::UnknownID(id.0))
          }
          if !self.known_ids.contains(&id1.0) {
            return Err(AnalyzerError::UnknownID(id1.0))
          }
        },
        BoolExpr::AmbiguousNeq(id, id1) => {
          if   !self.known_ids.contains(&id.0) {
            return Err(AnalyzerError::UnknownID(id.0))
          }
          if !self.known_ids.contains(&id1.0) {
            return Err(AnalyzerError::UnknownID(id1.0))
          }
        },
        BoolExpr::CardSetEq(card_set, card_set1) => {
          self.eval_use_card_set(card_set)?;
          self.eval_use_card_set(card_set1)?;
        },
        BoolExpr::CardSetNeq(card_set, card_set1) => {
          self.eval_use_card_set(card_set)?;
          self.eval_use_card_set(card_set1)?;
        },
        BoolExpr::StringEq(string_expr, string_expr1) => {
          self.eval_use_string(string_expr)?;
          self.eval_use_string(string_expr1)?;
        },
        BoolExpr::StringNeq(string_expr, string_expr1) => {
          self.eval_use_string(string_expr)?;
          self.eval_use_string(string_expr1)?;
        },
        BoolExpr::PlayerEq(player_expr, player_expr1) => {
          self.eval_use_player(player_expr)?;
          self.eval_use_player(player_expr1)?;
        },
        BoolExpr::PlayerNeq(player_expr, player_expr1) => {
          self.eval_use_player(player_expr)?;
          self.eval_use_player(player_expr1)?;
        },
        BoolExpr::TeamEq(team_expr, team_expr1) => {
          self.eval_use_team(team_expr)?;
          self.eval_use_team(team_expr1)?;
        },
        BoolExpr::TeamNeq(team_expr, team_expr1) => {
          self.eval_use_team(team_expr)?;
          self.eval_use_team(team_expr1)?;
        },
        BoolExpr::And(bool_expr, bool_expr1) => {
          self.eval_use_bool(*bool_expr)?;
          self.eval_use_bool(*bool_expr1)?;
        },
        BoolExpr::Or(bool_expr, bool_expr1) => {
          self.eval_use_bool(*bool_expr)?;
          self.eval_use_bool(*bool_expr1)?;
        },
        BoolExpr::Not(bool_expr) => {
          self.eval_use_bool(*bool_expr)?;
        },
        BoolExpr::OutOfStagePlayer(player_expr) => {
          self.eval_use_player(player_expr)?;
        },
        BoolExpr::OutOfGamePlayer(player_expr) => {
          self.eval_use_player(player_expr)?;
        },
        BoolExpr::OutOfStageCollection(player_collection) => {
          self.eval_use_player_collection(player_collection)?;
        },
        BoolExpr::OutOfGameCollection(player_collection) => {
          self.eval_use_player_collection(player_collection)?;
        },
    }

    return Ok(())
  }

  // =========================================================================


  // UseRepition
  // =========================================================================
  fn eval_use_repition(&mut self, repititions: Repititions) -> Result<(), AnalyzerError> {
    self.eval_use_int(repititions.times)?;

    return Ok(())
  }
  // =========================================================================

  // UseEndCondition
  // =========================================================================
  fn eval_use_end_condition(&mut self, end_condition: EndCondition) -> Result<(), AnalyzerError> {
    match end_condition {
        EndCondition::UntilBool(bool_expr) => {
          self.eval_use_bool(bool_expr)?;
        },
        EndCondition::UntilBoolAndRep(bool_expr, repititions) => {
          self.eval_use_bool(bool_expr)?;
          self.eval_use_repition(repititions)?;

        },
        EndCondition::UntilBoolOrRep(bool_expr, repititions) => {
          self.eval_use_bool(bool_expr)?;
          self.eval_use_repition(repititions)?;
        },
        EndCondition::UntilRep(repititions) => {
          self.eval_use_repition(repititions)?;
        },
        _ => {},
    }

    return Ok(())
  }

  // =========================================================================


  // =========================================================================
  // =========================================================================
  // =========================================================================

}
