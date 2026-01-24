use crate::{analysis::AnalyzerError, typed_ast::GameType, visitor::Visitor};
use crate::diagnostic::*;
use crate::spanned_ast::*;

pub type TypedVars = Vec<(Var, GameType)>;

#[derive(Debug, Clone)]
pub struct Var {
  pub id: String,
  pub(crate) span: proc_macro2::Span,
}

impl PartialEq for Var {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

fn id_not_initialized(value: &TypedVars, id: &SID) -> Result<(), AnalyzerError> {
  if value.iter().map(|(s, _)| s.id.clone()).collect::<Vec<String>>().contains(&id.node) {
    return Ok(())
  }

  return Err(AnalyzerError::IDNotInitialized { id: id.node.clone() })
}

fn id_used(value: &mut TypedVars, id: &SID, ty: GameType) -> Result<(), AnalyzerError> {
  if !value.iter().map(|(s, _)| s.id.clone()).collect::<Vec<String>>().contains(&id.node) {
    let var = Var {
      id: id.node.clone(),
      span: id.span,
    };

    value.push((var, ty));

    return Ok(())
  }

  return Err(AnalyzerError::IdUsed { id: id.node.clone() })
}

impl Visitor<TypedVars> for SGame {
  type Error = AnalyzerError;

  fn visit(&self, value: &mut TypedVars) -> Result<(), Self::Error> {
    self.node.flows.visit(value)?;

    Ok(())
  }
}

impl Visitor<TypedVars> for Vec<SFlowComponent> {
  type Error = AnalyzerError;

  fn visit(&self, value: &mut TypedVars) -> Result<(), Self::Error> {
    for flow in self.iter() {
      flow.visit(value)?;
    }

    Ok(())
  }
}

impl Visitor<TypedVars> for SFlowComponent {
  type Error = AnalyzerError;

  fn visit(&self, value: &mut TypedVars) -> Result<(), Self::Error> {
    match &self.node {
      FlowComponent::Stage(seq_stage) => seq_stage.visit(value)?,
      FlowComponent::Rule(rule) => rule.visit(value)?,
      FlowComponent::IfRule(if_rule) => if_rule.visit(value)?,
      FlowComponent::ChoiceRule(choice_rule) => choice_rule.visit(value)?,
      FlowComponent::OptionalRule(optional_rule) => optional_rule.visit(value)?,
    }

    Ok(())
  }
}

impl Visitor<TypedVars> for SSeqStage {
  type Error = AnalyzerError;

  fn visit(&self, value: &mut TypedVars) -> Result<(), Self::Error> {
    self.node.flows.visit(value)?;
    self.node.player.visit(value)?;
    self.node.end_condition.visit(value)?;

    Ok(())
  }
}

impl Visitor<TypedVars> for SIfRule {
  type Error = AnalyzerError;

  fn visit(&self, value: &mut TypedVars) -> Result<(), Self::Error> {
    self.node.condition.visit(value)?;
    // There might be some initialization of variables.
    for flow in self.node.flows.iter() {
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

impl Visitor<TypedVars> for SChoiceRule {
  type Error = AnalyzerError;

  fn visit(&self, value: &mut TypedVars) -> Result<(), Self::Error> {
    // There might be some initialization of variables.
    for flow in self.node.options.iter() {
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

impl Visitor<TypedVars> for SOptionalRule {
  type Error = AnalyzerError;

  fn visit(&self, value: &mut TypedVars) -> Result<(), Self::Error> {
    // There might be some initialization of variables.
    for flow in self.node.flows.iter() {
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

impl Visitor<TypedVars> for SEndCondition {
  type Error = AnalyzerError;

  fn visit(&self, value: &mut TypedVars) -> Result<(), Self::Error> {
    match &self.node {
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

impl Visitor<TypedVars> for SRepititions {
  type Error = AnalyzerError;

  fn visit(&self, value: &mut TypedVars) -> Result<(), Self::Error> {
    self.node.times.visit(value)?;
    Ok(())
  }
}

impl Visitor<TypedVars> for SBoolExpr {
  type Error = AnalyzerError;

  fn visit(&self, value: &mut TypedVars) -> Result<(), Self::Error> {
    match &self.node {
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

impl Visitor<TypedVars> for SIntExpr {
  type Error = AnalyzerError;

  fn visit(&self, value: &mut TypedVars) -> Result<(), Self::Error> {
    match &self.node {
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

impl Visitor<TypedVars> for SPlayerExpr {
  type Error = AnalyzerError;

  fn visit(&self, value: &mut TypedVars) -> Result<(), Self::Error> {
    match &self.node {
      PlayerExpr::PlayerName(player_name) => id_not_initialized(value, player_name)?,
      _ => {},
    }
    Ok(())
  }
}

impl Visitor<TypedVars> for STeamExpr {
  type Error = AnalyzerError;

  fn visit(&self, value: &mut TypedVars) -> Result<(), Self::Error> {
    match &self.node {
      TeamExpr::TeamName(team_name) => id_not_initialized(value, team_name)?,
      _ => {},
    }
    Ok(())
  }
}

impl Visitor<TypedVars> for SStringExpr {
  type Error = AnalyzerError;

  fn visit(&self, value: &mut TypedVars) -> Result<(), Self::Error> {
    match &self.node {
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

impl Visitor<TypedVars> for SCardSet {
  type Error = AnalyzerError;

  fn visit(&self, value: &mut TypedVars) -> Result<(), Self::Error> {
    match &self.node {
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

impl Visitor<TypedVars> for SGroup {
  type Error = AnalyzerError;

  fn visit(&self, value: &mut TypedVars) -> Result<(), Self::Error> {
    match &self.node {
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

impl Visitor<TypedVars> for SCardPosition {
  type Error = AnalyzerError;

  fn visit(&self, value: &mut TypedVars) -> Result<(), Self::Error> {
    match &self.node {
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

impl Visitor<TypedVars> for SFilterExpr {
  type Error = AnalyzerError;

  fn visit(&self, value: &mut TypedVars) -> Result<(), Self::Error> {
    match &self.node {
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

impl Visitor<TypedVars> for SCollection {
  type Error = AnalyzerError;

  fn visit(&self, value: &mut TypedVars) -> Result<(), Self::Error> {
    match &self.node {
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
        Collection::Ambiguous(spanneds) => {
          for s in spanneds.iter() {
            id_not_initialized(value, s)?;
          }
        },
    }
    Ok(())
  }
}

impl Visitor<TypedVars> for SIntCollection {
  type Error = AnalyzerError;

  fn visit(&self, value: &mut TypedVars) -> Result<(), Self::Error> {
    for int_expr in self.node.ints.iter() {
      int_expr.visit(value)?;
    }
    Ok(())
  }
} 

impl Visitor<TypedVars> for SStringCollection {
  type Error = AnalyzerError;

  fn visit(&self, value: &mut TypedVars) -> Result<(), Self::Error> {
    for string_expr in self.node.strings.iter() {
      string_expr.visit(value)?;
    }
    Ok(())
  }
}

impl Visitor<TypedVars> for SLocationCollection {
  type Error = AnalyzerError;

  fn visit(&self, value: &mut TypedVars) -> Result<(), Self::Error> {
    for location in self.node.locations.iter() {
      id_not_initialized(value, location)?;
    }
    Ok(())
  }
}

impl Visitor<TypedVars> for SPlayerCollection {
  type Error = AnalyzerError;

  fn visit(&self, value: &mut TypedVars) -> Result<(), Self::Error> {
    match &self.node {
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

impl Visitor<TypedVars> for STeamCollection {
  type Error = AnalyzerError;

  fn visit(&self, value: &mut TypedVars) -> Result<(), Self::Error> {
    match &self.node {
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

impl Visitor<TypedVars> for SRule {
  type Error = AnalyzerError;

  fn visit(&self, value: &mut TypedVars) -> Result<(), Self::Error> {
    match &self.node {
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
        for location in location_collection.node.locations.iter() {
          id_used(value, location, GameType::Location)?;
        } 

        player_collection.visit(value)?;
      },
      Rule::CreateLocationCollectionOnTeamCollection(location_collection, team_collection) => {
        for location in location_collection.node.locations.iter() {
          id_used(value, location, GameType::Location)?;
        } 
        
        team_collection.visit(value)?;
      },
      Rule::CreateLocationCollectionOnTable(location_collection) => {
        for location in location_collection.node.locations.iter() {
          id_used(value, location, GameType::Location)?;
        }
      },
      Rule::CreateCardOnLocation(location, types) => {
        id_not_initialized(value, location)?;
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

impl Visitor<TypedVars> for STypes {
  type Error = AnalyzerError;

  fn visit(&self, value: &mut TypedVars) -> Result<(), Self::Error> {
    for (k, vs) in self.node.types.iter() {
      id_used(value, k, GameType::Key)?;
      
      for v in vs.iter() {
        id_used(value, v, GameType::Value)?;
      }
    }
    Ok(())
  }
}

impl Visitor<TypedVars> for SQuantity {
  type Error = AnalyzerError;

  fn visit(&self, value: &mut TypedVars) -> Result<(), Self::Error> {
    match &self.node {
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

impl Visitor<TypedVars> for SQuantifier {
  type Error = AnalyzerError;

  fn visit(&self, _: &mut TypedVars) -> Result<(), Self::Error> {
    match &self.node {
      Quantifier::All => {},
      Quantifier::Any => {},
    }
    Ok(())
  }
}

impl Visitor<TypedVars> for SIntRange {
  type Error = AnalyzerError;

  fn visit(&self, value: &mut TypedVars) -> Result<(), Self::Error> {
    self.node.int.visit(value)?;
    Ok(())
  }
}

impl Visitor<TypedVars> for SClassicMove {
  type Error = AnalyzerError;

  fn visit(&self, value: &mut TypedVars) -> Result<(), Self::Error> {
    match &self.node {
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

impl Visitor<TypedVars> for SDealMove {
  type Error = AnalyzerError;

  fn visit(&self, value: &mut TypedVars) -> Result<(), Self::Error> {
    match &self.node {
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

impl Visitor<TypedVars> for SExchangeMove {
  type Error = AnalyzerError;

  fn visit(&self, value: &mut TypedVars) -> Result<(), Self::Error> {
    match &self.node {
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

impl Visitor<TypedVars> for STokenMove {
  type Error = AnalyzerError;

  fn visit(&self, value: &mut TypedVars) -> Result<(), Self::Error> {
    match &self.node {
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

impl Visitor<TypedVars> for STokenLocExpr {
  type Error = AnalyzerError;

  fn visit(&self, value: &mut TypedVars) -> Result<(), Self::Error> {
    match &self.node {
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

impl Visitor<TypedVars> for SScoreRule {
  type Error = AnalyzerError;

  fn visit(&self, value: &mut TypedVars) -> Result<(), Self::Error> {
    match &self.node {
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

impl Visitor<TypedVars> for SWinnerRule {
  type Error = AnalyzerError;

  fn visit(&self, value: &mut TypedVars) -> Result<(), Self::Error> {
    match &self.node {
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