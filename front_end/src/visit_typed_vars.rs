use crate::{{analyzer_error::AnalyzerError}, {ast::*, game_type::GameType}, visitor::Visitor};

pub type TypedVars = Vec<(String, GameType)>;

fn id_not_initialized(value: &TypedVars, id: &String) -> Result<(), AnalyzerError> {
  if value.iter().map(|(s, _)| s.clone()).collect::<Vec<String>>().contains(id) {
    return Ok(())
  }

  return Err(AnalyzerError::IDNotInitialized { id: id.clone() })
}

fn id_used(value: &mut TypedVars, id: &String, ty: GameType) -> Result<(), AnalyzerError> {
  if !value.iter().map(|(s, _)| s.clone()).collect::<Vec<String>>().contains(id) {
    return Ok(())
  }

  value.push((id.clone(), ty));

  return Err(AnalyzerError::IDNotInitialized { id: id.clone() })
}

impl Visitor<TypedVars> for Game {
  type Error = AnalyzerError;

  fn visit(&self, value: &mut TypedVars) -> Result<(), Self::Error> {
    self.flows.visit(value)?;

    Ok(())
  }
}

impl Visitor<TypedVars> for Vec<FlowComponent> {
  type Error = AnalyzerError;

  fn visit(&self, value: &mut TypedVars) -> Result<(), Self::Error> {
    for flow in self.iter() {
      flow.visit(value)?;
    }

    Ok(())
  }
}

impl Visitor<TypedVars> for FlowComponent {
  type Error = AnalyzerError;

  fn visit(&self, value: &mut TypedVars) -> Result<(), Self::Error> {
    match self {
      FlowComponent::Stage(seq_stage) => seq_stage.visit(value)?,
      FlowComponent::Rule(rule) => rule.visit(value)?,
      FlowComponent::IfRule(if_rule) => if_rule.visit(value)?,
      FlowComponent::ChoiceRule(choice_rule) => choice_rule.visit(value)?,
      FlowComponent::OptionalRule(optional_rule) => optional_rule.visit(value)?,
    }

    Ok(())
  }
}

impl Visitor<TypedVars> for SeqStage {
  type Error = AnalyzerError;

  fn visit(&self, value: &mut TypedVars) -> Result<(), Self::Error> {
    self.flows.visit(value)?;
    self.player.visit(value)?;
    self.end_condition.visit(value)?;

    Ok(())
  }
}

impl Visitor<TypedVars> for IfRule {
  type Error = AnalyzerError;

  fn visit(&self, value: &mut TypedVars) -> Result<(), Self::Error> {
    self.condition.visit(value)?;
    // There might be some initialization of variables.
    for flow in self.flows.iter() {
      let former_value = value.clone();
      flow.visit(value)?;

      // There has been some initialization been done.
      // However this initialization is optional!
      // Not deterministic.
      if former_value.clone() != value.clone() {
        // calculate the difference from former value and value afterwards
        value.retain(|x| !former_value.contains(x));
        return Err(AnalyzerError::NonDeterministicInitialization { created: value.clone() })
      }
    }

    Ok(())
  }
}

impl Visitor<TypedVars> for ChoiceRule {
  type Error = AnalyzerError;

  fn visit(&self, value: &mut TypedVars) -> Result<(), Self::Error> {
    // There might be some initialization of variables.
    for flow in self.options.iter() {
      let former_value = value.clone();
      flow.visit(value)?;

      // There has been some initialization been done.
      // However this initialization is optional!
      // Not deterministic.
      if former_value.clone() != value.clone() {
        // calculate the difference from former value and value afterwards
        value.retain(|x| !former_value.contains(x));
        return Err(AnalyzerError::NonDeterministicInitialization { created: value.clone() })
      }
    }

    Ok(())
  }
}

impl Visitor<TypedVars> for OptionalRule {
  type Error = AnalyzerError;

  fn visit(&self, value: &mut TypedVars) -> Result<(), Self::Error> {
    // There might be some initialization of variables.
    for flow in self.flows.iter() {
      let former_value = value.clone();
      flow.visit(value)?;

      // There has been some initialization been done.
      // However this initialization is optional!
      // Not deterministic.
      if former_value.clone() != value.clone() {
        // calculate the difference from former value and value afterwards
        value.retain(|x| !former_value.contains(x));
        return Err(AnalyzerError::NonDeterministicInitialization { created: value.clone() })
      }
    }

    Ok(())
  }
}

impl Visitor<TypedVars> for EndCondition {
  type Error = AnalyzerError;

  fn visit(&self, value: &mut TypedVars) -> Result<(), Self::Error> {
    match self {
      EndCondition::UntilBool(bool_expr) => {
        bool_expr.visit(value)?;
      },
      EndCondition::UntilBoolAndRep(bool_expr, repititions) => {
        bool_expr.visit(value)?;
        repititions.visit(value)?;
      },
      EndCondition::UntilBoolOrRep(bool_expr, repititions) => {
        bool_expr.visit(value)?;
        repititions.visit(value)?;
      },
      EndCondition::UntilRep(repititions) => {
        repititions.visit(value)?;
      },
      _ => {},
      
    }

    Ok(())
  }
}

impl Visitor<TypedVars> for Repititions {
  type Error = AnalyzerError;

  fn visit(&self, value: &mut TypedVars) -> Result<(), Self::Error> {
    self.times.visit(value)?;
    Ok(())
  }
}

impl Visitor<TypedVars> for BoolExpr {
  type Error = AnalyzerError;

  fn visit(&self, value: &mut TypedVars) -> Result<(), Self::Error> {
    match self {
      BoolExpr::IntCmp(int_expr, _, int_expr1) => {
        int_expr.visit(value)?;
        int_expr1.visit(value)?;
      },
      BoolExpr::CardSetIsEmpty(card_set) => {
        card_set.visit(value)?;
      },
      BoolExpr::CardSetIsNotEmpty(card_set) => {
        card_set.visit(value)?;
      },
      BoolExpr::CardSetEq(card_set, card_set1) => {
        card_set.visit(value)?;
        card_set1.visit(value)?;
      },
      BoolExpr::CardSetNeq(card_set, card_set1) => {
        card_set.visit(value)?;
        card_set1.visit(value)?;
      },
      BoolExpr::StringEq(string_expr, string_expr1) => {
        string_expr.visit(value)?;
        string_expr1.visit(value)?;
      },
      BoolExpr::StringNeq(string_expr, string_expr1) => {
        string_expr.visit(value)?;
        string_expr1.visit(value)?;
      },
      BoolExpr::PlayerEq(player_expr, player_expr1) => {
        player_expr.visit(value)?;
        player_expr1.visit(value)?;
      },
      BoolExpr::PlayerNeq(player_expr, player_expr1) => {
        player_expr.visit(value)?;
        player_expr1.visit(value)?;
      },
      BoolExpr::TeamEq(team_expr, team_expr1) => {
        team_expr.visit(value)?;
        team_expr1.visit(value)?;
      },
      BoolExpr::TeamNeq(team_expr, team_expr1) => {
        team_expr.visit(value)?;
        team_expr1.visit(value)?;
      },
      BoolExpr::And(bool_expr, bool_expr1) => {
        bool_expr.visit(value)?;
        bool_expr1.visit(value)?;
      },
      BoolExpr::Or(bool_expr, bool_expr1) => {
        bool_expr.visit(value)?;
        bool_expr1.visit(value)?;
      },
      BoolExpr::Not(bool_expr) => {
        bool_expr.visit(value)?;
      },
      BoolExpr::OutOfStagePlayer(player_expr) => {
        player_expr.visit(value)?;
      },
      BoolExpr::OutOfGamePlayer(player_expr) => {
        player_expr.visit(value)?;
      },
      BoolExpr::OutOfStageCollection(player_collection) => {
        player_collection.visit(value)?;
      },
      BoolExpr::OutOfGameCollection(player_collection) => {
        player_collection.visit(value)?;
      },
    }
    Ok(())
  }
}

impl Visitor<TypedVars> for IntExpr {
  type Error = AnalyzerError;

  fn visit(&self, value: &mut TypedVars) -> Result<(), Self::Error> {
    match self {
      IntExpr::IntOp(int_expr, _, int_expr1) => {
        int_expr.visit(value)?;
        int_expr1.visit(value)?;
      },
      IntExpr::IntCollectionAt(int_expr) => {
        int_expr.visit(value)?;
      },
      IntExpr::SizeOf(collection) => {
        collection.visit(value)?;
      },
      IntExpr::SumOfIntCollection(int_collection) => {
        int_collection.visit(value)?;
      },
      IntExpr::SumOfCardSet(card_set, point_map) => {
        card_set.visit(value)?;
        id_not_initialized(value, point_map)?;
      },
      IntExpr::MinOf(card_set, point_map) => {
        card_set.visit(value)?;
        id_not_initialized(value, point_map)?;
      },
      IntExpr::MaxOf(card_set, point_map) => {
        card_set.visit(value)?;
        id_not_initialized(value, point_map)?;
      },
      IntExpr::MinIntCollection(int_collection) => {
        int_collection.visit(value)?;
      },
      IntExpr::MaxIntCollection(int_collection) => {
        int_collection.visit(value)?;
      },
      _ => {},
    }
    Ok(())
  }
}

impl Visitor<TypedVars> for PlayerExpr {
  type Error = AnalyzerError;

  fn visit(&self, value: &mut TypedVars) -> Result<(), Self::Error> {
    match self {
      PlayerExpr::PlayerName(player_name) => id_not_initialized(value, player_name)?,
      _ => {},
    }
    Ok(())
  }
}

impl Visitor<TypedVars> for TeamExpr {
  type Error = AnalyzerError;

  fn visit(&self, value: &mut TypedVars) -> Result<(), Self::Error> {
    match self {
      TeamExpr::TeamName(team_name) => id_not_initialized(value, team_name)?,
      _ => {},
    }
    Ok(())
  }
}

impl Visitor<TypedVars> for StringExpr {
  type Error = AnalyzerError;

  fn visit(&self, value: &mut TypedVars) -> Result<(), Self::Error> {
    match self {
      StringExpr::KeyOf(key, card_position) => {
        card_position.visit(value)?;
        id_not_initialized(value, key)?;
      },
      StringExpr::StringCollectionAt(string_collection, int_expr) => {
        string_collection.visit(value)?;
        int_expr.visit(value)?;
      },
    }
    Ok(())
  }
}

impl Visitor<TypedVars> for CardSet {
  type Error = AnalyzerError;

  fn visit(&self, value: &mut TypedVars) -> Result<(), Self::Error> {
    match self {
      CardSet::Group(group) => {
        group.visit(value)?;
      },
      CardSet::GroupOfPlayer(group, player_expr) => {
        group.visit(value)?;
        player_expr.visit(value)?;
      },
      CardSet::GroupOfPlayerCollection(group, player_collection) => {
        group.visit(value)?;
        player_collection.visit(value)?;
      },
    }
    Ok(())
  }
}

impl Visitor<TypedVars> for Group {
  type Error = AnalyzerError;

  fn visit(&self, value: &mut TypedVars) -> Result<(), Self::Error> {
    match self {
      Group::Location(location) => {
        id_not_initialized(value, location)?;
      },
      Group::LocationWhere(location, filter_expr) => {
        id_not_initialized(value, location)?;
        filter_expr.visit(value)?;
      },
      Group::LocationCollection(location_collection) => {
        location_collection.visit(value)?;
      },
      Group::LocationCollectionWhere(location_collection, filter_expr) => {
        location_collection.visit(value)?;
        filter_expr.visit(value)?;
      },
      Group::ComboInLocation(combo, location) => {
        id_not_initialized(value, combo)?;
        id_not_initialized(value, location)?;
      },
      Group::ComboInLocationCollection(combo, location_collection) => {
        id_not_initialized(value, combo)?;
        location_collection.visit(value)?;
      },
      Group::NotComboInLocation(combo, location) => {
        id_not_initialized(value, combo)?;
        id_not_initialized(value, location)?;
      },
      Group::NotComboInLocationCollection(combo, location_collection) => {
        id_not_initialized(value, combo)?;
        location_collection.visit(value)?;
      },
      Group::CardPosition(card_position) => {
        card_position.visit(value)?;
      },
    }
    Ok(())
  }
}

impl Visitor<TypedVars> for CardPosition {
  type Error = AnalyzerError;

  fn visit(&self, value: &mut TypedVars) -> Result<(), Self::Error> {
    match self {
      CardPosition::At(location, int_expr) => {
        id_not_initialized(value, location)?;
        int_expr.visit(value)?;
      },
      CardPosition::Top(location) => {
        id_not_initialized(value, location)?;
      },
      CardPosition::Bottom(location) => {
        id_not_initialized(value, location)?;
      },
      CardPosition::Max(card_set, id) => {
        card_set.visit(value)?;
        id_not_initialized(value, id)?;
      },
      CardPosition::Min(card_set, id) => {
        card_set.visit(value)?;
        id_not_initialized(value, id)?;
      },
    }
    Ok(())
  }
}

impl Visitor<TypedVars> for FilterExpr {
  type Error = AnalyzerError;

  fn visit(&self, value: &mut TypedVars) -> Result<(), Self::Error> {
    match self {
      FilterExpr::Same(key) => {
        id_not_initialized(value, key)?;
      },
      FilterExpr::Distinct(key) => {
        id_not_initialized(value, key)?;
      },
      FilterExpr::Adjacent(key, precedence) => {
        id_not_initialized(value, key)?;
        id_not_initialized(value, precedence)?;
      },
      FilterExpr::Higher(key, precedence) => {
        id_not_initialized(value, key)?;
        id_not_initialized(value, precedence)?;
      },
      FilterExpr::Lower(key, precedence) => {
        id_not_initialized(value, key)?;
        id_not_initialized(value, precedence)?;
      },
      FilterExpr::Size(_, int_expr) => {
        int_expr.visit(value)?;
      },
      FilterExpr::KeyEqString(key, string_expr) => {
        id_not_initialized(value, key)?;
        string_expr.visit(value)?;
      },
      FilterExpr::KeyNeqString(key, string_expr) => {
        id_not_initialized(value, key)?;
        string_expr.visit(value)?;
      },
      FilterExpr::KeyEqValue(key, val) => {
        id_not_initialized(value, key)?;
        id_not_initialized(value, val)?;
      },
      FilterExpr::KeyNeqValue(key, val) => {
        id_not_initialized(value, key)?;
        id_not_initialized(value, val)?;
      },
      FilterExpr::NotCombo(combo) => {
        id_not_initialized(value, combo)?;
      },
      FilterExpr::Combo(combo) => {
        id_not_initialized(value, combo)?;
      },
      FilterExpr::And(filter_expr, filter_expr1) => {
        filter_expr.visit(value)?;
        filter_expr1.visit(value)?;
      },
      FilterExpr::Or(filter_expr, filter_expr1) => {
        filter_expr.visit(value)?;
        filter_expr1.visit(value)?;
      },
    }
    Ok(())
  }
}

impl Visitor<TypedVars> for Collection {
  type Error = AnalyzerError;

  fn visit(&self, value: &mut TypedVars) -> Result<(), Self::Error> {
    match self {
      Collection::IntCollection(int_collection) => {
        int_collection.visit(value)?;
      },
      Collection::StringCollection(string_collection) => {
        string_collection.visit(value)?;
      },
      Collection::LocationCollection(location_collection) => {
        location_collection.visit(value)?;
      },
      Collection::PlayerCollection(player_collection) => {
        player_collection.visit(value)?;
      },
      Collection::TeamCollection(team_collection) => {
        team_collection.visit(value)?;
      },
      Collection::CardSet(card_set) => {
        card_set.visit(value)?;
      },
      Collection::Ambiguous(ids) => {
        for id in ids.iter() {
          id_not_initialized(value, id)?;
        }
      },
    }
    Ok(())
  }
}

impl Visitor<TypedVars> for IntCollection {
  type Error = AnalyzerError;

  fn visit(&self, value: &mut TypedVars) -> Result<(), Self::Error> {
    for int_expr in self.ints.iter() {
      int_expr.visit(value)?;
    }
    Ok(())
  }
} 

impl Visitor<TypedVars> for StringCollection {
  type Error = AnalyzerError;

  fn visit(&self, value: &mut TypedVars) -> Result<(), Self::Error> {
    for string_expr in self.strings.iter() {
      string_expr.visit(value)?;
    }
    Ok(())
  }
}

impl Visitor<TypedVars> for LocationCollection {
  type Error = AnalyzerError;

  fn visit(&self, value: &mut TypedVars) -> Result<(), Self::Error> {
    for location in self.locations.iter() {
      id_not_initialized(value, location)?;
    }
    Ok(())
  }
}

impl Visitor<TypedVars> for PlayerCollection {
  type Error = AnalyzerError;

  fn visit(&self, value: &mut TypedVars) -> Result<(), Self::Error> {
    match self {
      PlayerCollection::Player(player_exprs) => {
        for player_expr in player_exprs.iter() {
          player_expr.visit(value)?;
        }
      },
      _ => {}
    }
    Ok(())
  }
}

impl Visitor<TypedVars> for TeamCollection {
  type Error = AnalyzerError;

  fn visit(&self, value: &mut TypedVars) -> Result<(), Self::Error> {
    match self {
      TeamCollection::Team(team_exprs) => {
        for team_expr in team_exprs.iter() {
          team_expr.visit(value)?;
        }
      },
      _ => {}
    }
    Ok(())
  }
}

impl Visitor<TypedVars> for Rule {
  type Error = AnalyzerError;

  fn visit(&self, value: &mut TypedVars) -> Result<(), Self::Error> {
    match self {
      Rule::CreatePlayer(player_names) => {
        for player in player_names.iter() {
          id_used(value, player, GameType::Player)?;
        }
      },
      Rule::CreateTeam(team_name, player_names) => {
        id_used(value, team_name, GameType::Team)?;

        for player in player_names.iter() {
          id_not_initialized(value, player)?;
        }
      },
      Rule::CreateTurnorder(player_names) => {
        for player in player_names.iter() {
          id_not_initialized(value, player)?;
        }
      },
      Rule::CreateTurnorderRandom(player_names) => {
        for player in player_names.iter() {
          id_not_initialized(value, player)?;
        }
      },
      Rule::CreateLocationOnPlayerCollection(location, player_collection) => {
        id_used(value, location, GameType::Location)?;
        player_collection.visit(value)?;
      },
      Rule::CreateLocationOnTeamCollection(location, team_collection) => {
        id_used(value, location, GameType::Location)?;
        team_collection.visit(value)?;
      },
      Rule::CreateLocationOnTable(location) => {
        id_used(value, location, GameType::Location)?;
      },
      Rule::CreateLocationCollectionOnPlayerCollection(location_collection, player_collection) => {
        for location in location_collection.locations.iter() {
          id_used(value, location, GameType::Location)?;
        } 

        player_collection.visit(value)?;
      },
      Rule::CreateLocationCollectionOnTeamCollection(location_collection, team_collection) => {
        for location in location_collection.locations.iter() {
          id_used(value, location, GameType::Location)?;
        } 
        
        team_collection.visit(value)?;
      },
      Rule::CreateLocationCollectionOnTable(location_collection) => {
        for location in location_collection.locations.iter() {
          id_used(value, location, GameType::Location)?;
        }
      },
      Rule::CreateCardOnLocation(location, types) => {
        id_used(value, location, GameType::Location)?;
        types.visit(value)?;
      },
      Rule::CreateTokenOnLocation(int_expr, token, location) => {
        int_expr.visit(value)?;
        id_used(value, token, GameType::Token)?;
        id_not_initialized(value, location)?;
      },
      Rule::CreatePrecedence(precedence, items) => {
        id_used(value, precedence, GameType::Precedence)?;

        for (k, v) in items.iter() {
          id_not_initialized(value, k)?;
          id_not_initialized(value, v)?;
        }
      },
      Rule::CreateCombo(combo, filter_expr) => {
        id_used(value, combo, GameType::Combo)?;
        filter_expr.visit(value)?;
      },
      Rule::CreateMemoryIntPlayerCollection(memory, int_expr, player_collection) => {
        id_used(value, memory, GameType::Memory)?;
        int_expr.visit(value)?;
        player_collection.visit(value)?;
      },
      Rule::CreateMemoryStringPlayerCollection(memory, string_expr, player_collection) => {
        id_used(value, memory, GameType::Memory)?;
        string_expr.visit(value)?;
        player_collection.visit(value)?;
      },
      Rule::CreateMemoryIntTable(memory, int_expr) => {
        id_used(value, memory, GameType::Memory)?;
        int_expr.visit(value)?;
      },
      Rule::CreateMemoryStringTable(memory, string_expr) => {
        id_used(value, memory, GameType::Memory)?;
        string_expr.visit(value)?;
      },
      Rule::CreateMemoryPlayerCollection(memory, player_collection) => {
        id_used(value, memory, GameType::Memory)?;
        player_collection.visit(value)?;
      },
      Rule::CreateMemoryTable(memory) => {
        id_used(value, memory, GameType::Memory)?;
      },
      Rule::CreatePointMap(point_map, items) => {
        id_used(value, point_map, GameType::PointMap)?;

        for (k, v, int_expr) in items {
          id_not_initialized(value, k)?;
          id_not_initialized(value, v)?;
          int_expr.visit(value)?;
        }
      },
      Rule::FlipAction(card_set, _) => {
        card_set.visit(value)?;
      },
      Rule::ShuffleAction(card_set) => {
        card_set.visit(value)?;
      },
      Rule::PlayerOutOfStageAction(player_expr) => {
        player_expr.visit(value)?;
      },
      Rule::PlayerOutOfGameSuccAction(player_expr) => {
        player_expr.visit(value)?;
      },
      Rule::PlayerOutOfGameFailAction(player_expr) => {
        player_expr.visit(value)?;
      },
      Rule::PlayerCollectionOutOfStageAction(player_collection) => {
        player_collection.visit(value)?;
      },
      Rule::PlayerCollectionOutOfGameSuccAction(player_collection) => {
        player_collection.visit(value)?;
      },
      Rule::PlayerCollectionOutOfGameFailAction(player_collection) => {
        player_collection.visit(value)?;
      },
      Rule::SetMemoryInt(memory, int_expr) => {
        id_not_initialized(value, memory)?;
        int_expr.visit(value)?;
      },
      Rule::SetMemoryString(memory, string_expr) => {
        id_not_initialized(value, memory)?;
        string_expr.visit(value)?;
      },
      Rule::SetMemoryCollection(memory, collection) => {
        id_not_initialized(value, memory)?;
        collection.visit(value)?;
      },
      Rule::CycleAction(player_expr) => {
        player_expr.visit(value)?;
      },
      Rule::BidActionMemory(memory, _) => {
        id_not_initialized(value, memory)?;
      },
      Rule::EndGameWithWinner(player_expr) => {
        player_expr.visit(value)?;
      },
      Rule::DemandCardPositionAction(card_position) => {
        card_position.visit(value)?;
      },
      Rule::DemandStringAction(string_expr) => {
        string_expr.visit(value)?;
      },
      Rule::DemandIntAction(int_expr) => {
        int_expr.visit(value)?;
      },
      Rule::ClassicMove(classic_move) => {
        classic_move.visit(value)?;
      },
      Rule::DealMove(deal_move) => {
        deal_move.visit(value)?;
      },
      Rule::ExchangeMove(exchange_move) => {
        exchange_move.visit(value)?;
      },
      Rule::TokenMove(token_move) => {
        token_move.visit(value)?;
      },
      Rule::ScoreRule(score_rule) => {
        score_rule.visit(value)?;
      },
      Rule::WinnerRule(winner_rule) => {
        winner_rule.visit(value)?;
      },
      _ => {},
    }
    Ok(())
  }
}

impl Visitor<TypedVars> for Types {
  type Error = AnalyzerError;

  fn visit(&self, value: &mut TypedVars) -> Result<(), Self::Error> {
    for (k, vs) in self.types.iter() {
      id_used(value, k, GameType::Key)?;
      
      for v in vs.iter() {
        id_used(value, v, GameType::Value)?;
      }
    }
    Ok(())
  }
}

impl Visitor<TypedVars> for Quantity {
  type Error = AnalyzerError;

  fn visit(&self, value: &mut TypedVars) -> Result<(), Self::Error> {
    match self {
      Quantity::Int(int_expr) => {
        int_expr.visit(value)?;
      },
      Quantity::Quantifier(quantifier) => {
        quantifier.visit(value)?;
      },
      Quantity::IntRange(int_range) => {
        int_range.visit(value)?;
      },
    }
    Ok(())
  }
}

impl Visitor<TypedVars> for Quantifier {
  type Error = AnalyzerError;

  fn visit(&self, _: &mut TypedVars) -> Result<(), Self::Error> {
    match self {
      Quantifier::All => {},
      Quantifier::Any => {},
    }
    Ok(())
  }
}

impl Visitor<TypedVars> for IntRange {
  type Error = AnalyzerError;

  fn visit(&self, value: &mut TypedVars) -> Result<(), Self::Error> {
    self.int.visit(value)?;
    Ok(())
  }
}

impl Visitor<TypedVars> for ClassicMove {
  type Error = AnalyzerError;

  fn visit(&self, value: &mut TypedVars) -> Result<(), Self::Error> {
    match self {
      ClassicMove::Move(card_set, _, card_set1) => {
        card_set.visit(value)?;
        card_set1.visit(value)?;
      },
      ClassicMove::MoveQuantity(quantity, card_set, _, card_set1) => {
        quantity.visit(value)?;
        card_set.visit(value)?;
        card_set1.visit(value)?;
      },
    }
    Ok(())
  }
}

impl Visitor<TypedVars> for DealMove {
  type Error = AnalyzerError;

  fn visit(&self, value: &mut TypedVars) -> Result<(), Self::Error> {
    match self {
      DealMove::Deal(card_set, _, card_set1) => {
        card_set.visit(value)?;
        card_set1.visit(value)?;
      },
      DealMove::DealQuantity(quantity, card_set, _, card_set1) => {
        quantity.visit(value)?;
        card_set.visit(value)?;
        card_set1.visit(value)?;
      },
    }
    Ok(())
  }
}

impl Visitor<TypedVars> for ExchangeMove {
  type Error = AnalyzerError;

  fn visit(&self, value: &mut TypedVars) -> Result<(), Self::Error> {
    match self {
      ExchangeMove::Exchange(card_set, _, card_set1) => {
        card_set.visit(value)?;
        card_set1.visit(value)?;
      },
      ExchangeMove::ExchangeQuantity(quantity, card_set, _, card_set1) => {
        quantity.visit(value)?;
        card_set.visit(value)?;
        card_set1.visit(value)?;
      },
    }
    Ok(())
  }
}

impl Visitor<TypedVars> for TokenMove {
  type Error = AnalyzerError;

  fn visit(&self, value: &mut TypedVars) -> Result<(), Self::Error> {
    match self {
      TokenMove::Place(token, token_loc_expr, token_loc_expr1) => {
        id_not_initialized(value, token)?;
        token_loc_expr.visit(value)?;
        token_loc_expr1.visit(value)?;
      },
      TokenMove::PlaceQuantity(quantity, token, token_loc_expr, token_loc_expr1) => {
        quantity.visit(value)?;
        id_not_initialized(value, token)?;
        token_loc_expr.visit(value)?;
        token_loc_expr1.visit(value)?;
      },
    }
    Ok(())
  }
} 

impl Visitor<TypedVars> for TokenLocExpr {
  type Error = AnalyzerError;

  fn visit(&self, value: &mut TypedVars) -> Result<(), Self::Error> {
    match self {
      TokenLocExpr::Location(location) => {
        id_not_initialized(value, location)?;
      },
      TokenLocExpr::LocationCollection(location_collection) => {
        location_collection.visit(value)?;
      },
      TokenLocExpr::LocationPlayer(location, player_expr) => {
        id_not_initialized(value, location)?;
        player_expr.visit(value)?;
      },
      TokenLocExpr::LocationCollectionPlayer(location_collection, player_expr) => {
        location_collection.visit(value)?;
        player_expr.visit(value)?;
      },
      TokenLocExpr::LocationPlayerCollection(location, player_collection) => {
        id_not_initialized(value, location)?;
        player_collection.visit(value)?;
      },
      TokenLocExpr::LocationCollectionPlayerCollection(location_collection, player_collection) => {
        location_collection.visit(value)?;
        player_collection.visit(value)?;
      },
    }
    Ok(())
  }
}

impl Visitor<TypedVars> for ScoreRule {
  type Error = AnalyzerError;

  fn visit(&self, value: &mut TypedVars) -> Result<(), Self::Error> {
    match self {
      ScoreRule::ScorePlayer(int_expr, player_expr) => {
        int_expr.visit(value)?;
        player_expr.visit(value)?;
      },
      ScoreRule::ScorePlayerMemory(int_expr, memory, player_expr) => {
        int_expr.visit(value)?;
        id_not_initialized(value, memory)?;
        player_expr.visit(value)?;
      },
      ScoreRule::ScorePlayerCollection(int_expr, player_collection) => {
        int_expr.visit(value)?;
        player_collection.visit(value)?;
      },
      ScoreRule::ScorePlayerCollectionMemory(int_expr, memory, player_collection) => {
        int_expr.visit(value)?;
        id_not_initialized(value, memory)?;
        player_collection.visit(value)?;
      },
    }
    Ok(())
  }
}

impl Visitor<TypedVars> for WinnerRule {
  type Error = AnalyzerError;

  fn visit(&self, value: &mut TypedVars) -> Result<(), Self::Error> {
    match self {
      WinnerRule::WinnerPlayer(player_expr) => {
        player_expr.visit(value)?;
      },
      WinnerRule::WinnerPlayerCollection(player_collection) => {
        player_collection.visit(value)?;
      },
      WinnerRule::WinnerLowestScore => {},
      WinnerRule::WinnerHighestScore => {},
      WinnerRule::WinnerLowestMemory(memory) => {
        id_not_initialized(value, memory)?;
      },
      WinnerRule::WinnerHighestMemory(memory) => {
        id_not_initialized(value, memory)?;
      },
      WinnerRule::WinnerLowestPosition => {},
      WinnerRule::WinnerHighestPosition => {},
    }
    Ok(())
  }
}