use crate::{typed_ast::GameType, visitor::Visitor};
use crate::{diagnostic::*};
use crate::spanned_ast::*;

#[derive(Debug, Clone)]
pub enum AnalyzerError {
  Default
}

pub type TypedVars = Vec<(Var, GameType)>;

#[derive(Debug, Clone)]
pub struct Var {
  pub id: String,
  pub(crate) span: OwnedSpan,
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

  return Err(AnalyzerError::Default)
}

fn id_used(value: &mut TypedVars, id: &SID, ty: GameType) -> Result<(), AnalyzerError> {
  if !value.iter().map(|(s, _)| s.id.clone()).collect::<Vec<String>>().contains(&id.node) {
    let var = Var {
      id: id.node.clone(),
      span: id.span.clone(),
    };

    value.push((var, ty));

    return Ok(())
  }

  return Err(AnalyzerError::Default)
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

        return Err(AnalyzerError::Default)
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
        return Err(AnalyzerError::Default)
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
        return Err(AnalyzerError::Default)
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
      EndCondition::UntilBoolRep(bool_expr, _, repititions) => {
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

impl Visitor<TypedVars> for SCompareBool {
  type Error = AnalyzerError;

  fn visit(&self, value: &mut TypedVars) -> Result<(), Self::Error> {
    match &self.node {
        CompareBool::Int(spanned, _, spanned2) => {
          spanned.visit(value)?;
          spanned2.visit(value)?;
        },
        CompareBool::CardSet(spanned, _, spanned2) => {
          spanned.visit(value)?;
          spanned2.visit(value)?;
        },
        CompareBool::String(spanned, _, spanned2) => {
          spanned.visit(value)?;
          spanned2.visit(value)?;
        },
        CompareBool::Player(spanned, _, spanned2) => {
          spanned.visit(value)?;
          spanned2.visit(value)?;
        },
        CompareBool::Team(spanned, _, spanned2) => {
          spanned.visit(value)?;
          spanned2.visit(value)?;
        },
    }

    Ok(())
  }
}

impl Visitor<TypedVars> for SAggregateBool {
  type Error = AnalyzerError;

  fn visit(&self, value: &mut TypedVars) -> Result<(), Self::Error> {
    match &self.node {
        AggregateBool::Compare(spanned) =>{
          spanned.visit(value)?;
        },
        AggregateBool::CardSetEmpty(spanned) => {
          spanned.visit(value)?;
        },
        AggregateBool::CardSetNotEmpty(spanned) => {
          spanned.visit(value)?;
        },
        AggregateBool::OutOfPlayer(spanned, spanned1) => {
          {
          spanned.visit(value)?;
          spanned1.visit(value)?;
        }
      },
    }

    Ok(())
  }
}

impl Visitor<TypedVars> for SBoolExpr {
  type Error = AnalyzerError;

  fn visit(&self, value: &mut TypedVars) -> Result<(), Self::Error> {
    match &self.node {
      BoolExpr::Aggregate(spanned) => {
        spanned.visit(value)?;
      },
      BoolExpr::Binary(spanned, _, spanned2) => {
        spanned.visit(value)?;
        spanned2.visit(value)?;
      },
      BoolExpr::Unary(_, spanned) => {
        spanned.visit(value)?;
      },
    }
    Ok(())
  }
}

impl Visitor<TypedVars> for SOutOf {
  type Error = AnalyzerError;

  fn visit(&self, value: &mut TypedVars) -> Result<(), Self::Error> {
    match &self.node {
      OutOf::CurrentStage => {},
      OutOf::Game => {},
      OutOf::Play => {},
      OutOf::Stage(id) => id_not_initialized(value, id)?
    }

    Ok(())
  }
}

impl Visitor<TypedVars> for SAggregateInt {
  type Error = AnalyzerError;

  fn visit(&self, value: &mut TypedVars) -> Result<(), Self::Error> {
    match &self.node {
        AggregateInt::SizeOf(spanned) => {
          spanned.visit(value)?;
        },
        AggregateInt::SumOfIntCollection(spanned) => {
          spanned.visit(value)?;
        },
        AggregateInt::SumOfCardSet(spanned, spanned1) => {
          spanned.visit(value)?;
          id_not_initialized(value, spanned1)?;
        },
        AggregateInt::ExtremaCardset(_, spanned1, spanned2) => {
          spanned1.visit(value)?;
          id_not_initialized(value, spanned2)?;
        },
        AggregateInt::ExtremaIntCollection(_, spanned1) => {
          spanned1.visit(value)?;
        },
    }

    Ok(())
  }
}

impl Visitor<TypedVars> for SQueryInt {
  type Error = AnalyzerError;

  fn visit(&self, value: &mut TypedVars) -> Result<(), Self::Error> {
    match &self.node {
      QueryInt::IntCollectionAt(spanned, spanned1) => {
        spanned.visit(value)?;
        spanned1.visit(value)?;
      },
    }
  
    Ok(())
  }
}

impl Visitor<TypedVars> for SIntExpr {
  type Error = AnalyzerError;

  fn visit(&self, value: &mut TypedVars) -> Result<(), Self::Error> {
    match &self.node {
      IntExpr::Aggregate(spanned) => {
        spanned.visit(value)?
      },
      IntExpr::Binary(spanned, _, spanned2) => {
        spanned.visit(value)?;
        spanned2.visit(value)?;
      },
      IntExpr::Query(spanned) => {
        spanned.visit(value)?;
      }
      _ => {},
    }
    Ok(())
  }
}

impl Visitor<TypedVars> for SAggregatePlayer {
  type Error = AnalyzerError;

  fn visit(&self, value: &mut TypedVars) -> Result<(), Self::Error> {
    match &self.node {
        AggregatePlayer::OwnerOfCardPostion(spanned) => {
          spanned.visit(value)?;
        },
        AggregatePlayer::OwnerOfMemory(_, spanned1) => {
          id_not_initialized(value, spanned1)?;
        },
    }

    Ok(())
  }
}

impl Visitor<TypedVars> for SQueryPlayer {
  type Error = AnalyzerError;

  fn visit(&self, value: &mut TypedVars) -> Result<(), Self::Error> {
    match &self.node {
        QueryPlayer::Turnorder(spanned) => {
          spanned.visit(value)?
        },
    }

    Ok(())
  }
}

impl Visitor<TypedVars> for SPlayerExpr {
  type Error = AnalyzerError;

  fn visit(&self, value: &mut TypedVars) -> Result<(), Self::Error> {
    match &self.node {
      PlayerExpr::Literal(player_name) => id_not_initialized(value, player_name)?,
      PlayerExpr::Aggregate(spanned) => {
        spanned.visit(value)?;
      },
      PlayerExpr::Query(spanned) => {
        spanned.visit(value)?;
      },
      _ => {},
    }

    Ok(())
  }
}

impl Visitor<TypedVars> for SPlayers {
  type Error = AnalyzerError;

  fn visit(&self, value: &mut TypedVars) -> Result<(), Self::Error> {
    match &self.node {
        Players::Player(spanned) => {
          spanned.visit(value)?;
        },
        Players::PlayerCollection(spanned) => {
          spanned.visit(value)?;
        },
    }
  
    Ok(())
  }
}

impl Visitor<TypedVars> for SAggregateTeam {
  type Error = AnalyzerError;

  fn visit(&self, value: &mut TypedVars) -> Result<(), Self::Error> {
    match &self.node {
        AggregateTeam::TeamOf(spanned) => {
          spanned.visit(value)?;
        },
    }

    Ok(())
  }
}

impl Visitor<TypedVars> for STeamExpr {
  type Error = AnalyzerError;

  fn visit(&self, value: &mut TypedVars) -> Result<(), Self::Error> {
    match &self.node {
      TeamExpr::Literal(team_name) => id_not_initialized(value, team_name)?,
      TeamExpr::Aggregate(spanned) => {
        spanned.visit(value)?;
      },
    }
    Ok(())
  }
}

impl Visitor<TypedVars> for SQueryString {
  type Error = AnalyzerError;

  fn visit(&self, value: &mut TypedVars) -> Result<(), Self::Error> {
    match &self.node {
        QueryString::KeyOf(spanned, spanned1) => {
          id_not_initialized(value, spanned)?;
          spanned1.visit(value)?;
        },
        QueryString::StringCollectionAt(spanned, spanned1) => {
          spanned.visit(value)?;
          spanned1.visit(value)?;
        },
    }

    Ok(())
  }
}

impl Visitor<TypedVars> for SStringExpr {
  type Error = AnalyzerError;

  fn visit(&self, value: &mut TypedVars) -> Result<(), Self::Error> {
    match &self.node {
      StringExpr::Literal(_) => {
        // No logic at the moment
      },
      StringExpr::Query(spanned) => {
        spanned.visit(value)?;
      },
    }
    Ok(())
  }
}

impl Visitor<TypedVars> for SOwner {
  type Error = AnalyzerError;

  fn visit(&self, value: &mut TypedVars) -> Result<(), Self::Error> {
    match &self.node {
        Owner::Player(spanned) => {
          spanned.visit(value)?;
        },
        Owner::PlayerCollection(spanned) => {
          spanned.visit(value)?;
        },
        Owner::Team(spanned) => {
          spanned.visit(value)?;
        },
        Owner::TeamCollection(spanned) => {
          spanned.visit(value)?;
        },
        Owner::Table => {},
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
      CardSet::GroupOwner(group, owner) => {
        group.visit(value)?;
        owner.visit(value)?;
      },
    }
    Ok(())
  }
}

impl Visitor<TypedVars> for SGroupable {
  type Error = AnalyzerError;

  fn visit(&self, value: &mut TypedVars) -> Result<(), Self::Error> {
    match &self.node {
        Groupable::Location(spanned) => {
          id_not_initialized(value, spanned)?;
        },
        Groupable::LocationCollection(spanned) => {
          spanned.visit(value)?;
        },
    }

    Ok(())
  }  
}

impl Visitor<TypedVars> for SGroup {
  type Error = AnalyzerError;

  fn visit(&self, value: &mut TypedVars) -> Result<(), Self::Error> {
    match &self.node {
      Group::Groupable(spanned) => {
        spanned.visit(value)?;
      },
      Group::Where(spanned, spanned1) => {
        spanned.visit(value)?;
        spanned1.visit(value)?;
      },
      Group::Combo(spanned1, spanned2) => {
        id_not_initialized(value, spanned1)?;
        spanned2.visit(value)?;
      },
      Group::NotCombo(spanned1, spanned2) => {
        id_not_initialized(value, spanned1)?;
        spanned2.visit(value)?;
      },
      Group::CardPosition(card_position) => {
        card_position.visit(value)?;
      },
    }

    Ok(())
  }
}

impl Visitor<TypedVars> for SAggregateCardPosition {
  type Error = AnalyzerError;

  fn visit(&self, value: &mut TypedVars) -> Result<(), Self::Error> {
    match &self.node {
        AggregateCardPosition::Extrema(_, spanned1, spanned2) => {
          spanned1.visit(value)?;
          id_not_initialized(value, spanned2)?;
        },
    }

    Ok(())
  }
}

impl Visitor<TypedVars> for SQueryCardPosition {
  type Error = AnalyzerError;

  fn visit(&self, value: &mut TypedVars) -> Result<(), Self::Error> {
    match &self.node {
        QueryCardPosition::At(spanned, spanned1) => {
          id_not_initialized(value, spanned)?;
          spanned1.visit(value)?;
        },
        QueryCardPosition::Top(spanned) => {
          id_not_initialized(value, spanned)?;
        },
        QueryCardPosition::Bottom(spanned) => {
          id_not_initialized(value, spanned)?;
        },
    }

    Ok(())
  }
}

impl Visitor<TypedVars> for SCardPosition {
  type Error = AnalyzerError;

  fn visit(&self, value: &mut TypedVars) -> Result<(), Self::Error> {
    match &self.node {
      CardPosition::Aggregate(spanned) => {
        spanned.visit(value)?;
      },
      CardPosition::Query(spanned) => {
        spanned.visit(value)?;
      },
    }
    Ok(())
  }
}

impl Visitor<TypedVars> for SAggregateFilter {
  type Error = AnalyzerError;

  fn visit(&self, value: &mut TypedVars) -> Result<(), Self::Error> {
    match &self.node {
        AggregateFilter::Size(_, spanned1) => {
          spanned1.visit(value)?;
        },
        AggregateFilter::Same(spanned) => {
          id_not_initialized(value, spanned)?;
        },
        AggregateFilter::Distinct(spanned) => {
          id_not_initialized(value, spanned)?;
        },
        AggregateFilter::Higher(spanned, spanned1) => {
          id_not_initialized(value, spanned)?;
          id_not_initialized(value, spanned1)?;
        },
        AggregateFilter::Adjacent(spanned, spanned1) => {
          id_not_initialized(value, spanned)?;
          id_not_initialized(value, spanned1)?;
        },
        AggregateFilter::Lower(spanned, spanned1) => {
          id_not_initialized(value, spanned)?;
          id_not_initialized(value, spanned1)?;
        },
        AggregateFilter::KeyString(spanned, _, spanned2) => {
          id_not_initialized(value, spanned)?;
          spanned2.visit(value)?;
        },
        AggregateFilter::Combo(spanned) => {
          id_not_initialized(value, spanned)?;
        },
        AggregateFilter::NotCombo(spanned) => {
          id_not_initialized(value, spanned)?;
        },
    }

    Ok(())
  }
}

impl Visitor<TypedVars> for SFilterExpr {
  type Error = AnalyzerError;

  fn visit(&self, value: &mut TypedVars) -> Result<(), Self::Error> {
    match &self.node {
        FilterExpr::Aggregate(spanned) => {
          spanned.visit(value)?;
        },
        FilterExpr::Binary(spanned, _, spanned2) => {
          spanned.visit(value)?;
          spanned2.visit(value)?;
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
      PlayerCollection::Literal(player_exprs) => {
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
      TeamCollection::Literal(team_exprs) => {
        for team_expr in team_exprs.iter() {
          team_expr.visit(value)?;
        }
      },
      _ => {}
    }
    Ok(())
  }
}

impl Visitor<TypedVars> for SSetUpRule {
  type Error = AnalyzerError;

  fn visit(&self, value: &mut TypedVars) -> Result<(), Self::Error> {
    match &self.node {
        SetUpRule::CreatePlayer(spanneds) => {
          for player in spanneds.iter() {
            id_used(value, player, GameType::Player)?;
          }
        },
        SetUpRule::CreateTeams(items) => {
          for (team_name, player_collection) in items.iter() {
            id_used(value, team_name, GameType::Team)?;
            player_collection.visit(value)?;
          }
        },
        SetUpRule::CreateTurnorder(spanned) => {
          spanned.visit(value)?;
        },
        SetUpRule::CreateTurnorderRandom(spanned) => {
          spanned.visit(value)?;
        },
        SetUpRule::CreateLocation(spanneds, spanned) => {
          for id in spanneds.iter() {
            id_used(value, id, GameType::Location)?;
          }

          spanned.visit(value)?;
        },
        SetUpRule::CreateCardOnLocation(location, types) => {
          id_not_initialized(value, location)?;
          types.visit(value)?;
        },
        SetUpRule::CreateTokenOnLocation(spanned, spanned1, spanned2) => {
          spanned.visit(value)?;
          id_used(value, spanned1, GameType::Token)?;
          id_not_initialized(value, spanned2)?;
        },
        SetUpRule::CreateCombo(spanned, spanned1) => {
          id_used(value, spanned, GameType::Combo)?;
          spanned1.visit(value)?;
        },
        SetUpRule::CreateMemory(spanned, _, spanned2) => {
          id_used(value, spanned, GameType::Memory)?;
          spanned2.visit(value)?;
        },
        SetUpRule::CreatePrecedence(spanned, items) => {
          id_used(value, spanned, GameType::Precedence)?;

          for (k, v) in items.iter() {
            id_not_initialized(value, k)?;
            id_not_initialized(value, v)?;
          }
        },
        SetUpRule::CreatePointMap(spanned, items) => {
          id_used(value, spanned, GameType::PointMap)?;

          for (k, v, int_expr) in items {
            id_not_initialized(value, k)?;
            id_not_initialized(value, v)?;
            int_expr.visit(value)?;
          }
        },
    }

    Ok(())
  }
}

impl Visitor<TypedVars> for SActionRule {
  type Error = AnalyzerError;

  fn visit(&self, value: &mut TypedVars) -> Result<(), Self::Error> {
    match &self.node {
        ActionRule::FlipAction(spanned, _) => {
          spanned.visit(value)?;
        },
        ActionRule::ShuffleAction(spanned) => {
          spanned.visit(value)?;
        },
        ActionRule::PlayerOutOfStageAction(spanned) => {
          spanned.visit(value)?;
        },
        ActionRule::PlayerOutOfGameSuccAction(spanned) => {
          spanned.visit(value)?;
        },
        ActionRule::PlayerOutOfGameFailAction(spanned) => {
          spanned.visit(value)?;
        },
        ActionRule::SetMemory(spanned, spanned1) => {
          id_not_initialized(value, spanned)?;
          spanned1.visit(value)?;
        },
        ActionRule::ResetMemory(spanned) => {
          id_not_initialized(value, spanned)?;
        },
        ActionRule::CycleAction(spanned) => {
          spanned.visit(value)?;
        },
        ActionRule::BidAction(spanned) => {
          spanned.visit(value)?;
        },
        ActionRule::BidMemoryAction(spanned, spanned1) => {
          id_not_initialized(value, spanned);
          spanned1.visit(value)?;
        },
        ActionRule::EndAction(spanned) => {
          match spanned.clone().node {
            EndType::Turn => {},
            EndType::Stage => {},
            EndType::GameWithWinner(spanned) => spanned.visit(value)?,
          }
        },
        ActionRule::DemandAction(spanned) => {
          spanned.visit(value)?;
        },
        ActionRule::DemandMemoryAction(spanned, spanned1) => {
          spanned.visit(value)?;
          id_not_initialized(value, spanned1)?;
        },
        ActionRule::Move(spanned) => {
          spanned.visit(value)?;
        },
    }

    Ok(())
  }
}

impl Visitor<TypedVars> for SMemoryType {
    type Error = AnalyzerError;

  fn visit(&self, value: &mut TypedVars) -> Result<(), Self::Error> {
    match &self.node {
        MemoryType::Int(spanned) => spanned.visit(value)?,
        MemoryType::String(spanned) => spanned.visit(value)?,
        MemoryType::CardSet(spanned) => spanned.visit(value)?,
        MemoryType::Collection(spanned) => spanned.visit(value)?,
    }

    Ok(())
  }
}

impl Visitor<TypedVars> for SDemandType {
  type Error = AnalyzerError;

  fn visit(&self, value: &mut TypedVars) -> Result<(), Self::Error> {
    match &self.node {
        DemandType::CardPosition(spanned) => {
          spanned.visit(value)?;
        },
        DemandType::String(spanned) => {
          spanned.visit(value)?;
        },
        DemandType::Int(spanned) => {
          spanned.visit(value)?;
        },
    }

    Ok(())
  }
}

impl Visitor<TypedVars> for SMoveType {
  type Error = AnalyzerError;

  fn visit(&self, value: &mut TypedVars) -> Result<(), Self::Error> {
    match &self.node {
        MoveType::Deal(spanned) => {
          spanned.visit(value)?;
        },
        MoveType::Exchange(spanned) => {
          spanned.visit(value)?;
        },
        MoveType::Classic(spanned) => {
          spanned.visit(value)?;
        },
        MoveType::Place(spanned) => {
          spanned.visit(value)?;
        },
    }

    Ok(())
  }
}

impl Visitor<TypedVars> for SScoringRule {
  type Error = AnalyzerError;

  fn visit(&self, value: &mut TypedVars) -> Result<(), Self::Error> {
    match &self.node {
        ScoringRule::ScoreRule(spanned) => {
          spanned.visit(value)?;
        },
        ScoringRule::WinnerRule(spanned) => {
          spanned.visit(value)?;

        },
    }

    Ok(())
  }
}

impl Visitor<TypedVars> for SGameRule {
  type Error = AnalyzerError;

  fn visit(&self, value: &mut TypedVars) -> Result<(), Self::Error> {
    match &self.node {
        GameRule::SetUp(spanned) => {
          spanned.visit(value)?;
        },
        GameRule::Action(spanned) => {
          spanned.visit(value)?;
        },
        GameRule::Scoring(spanned) => {
          spanned.visit(value)?;
        },
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
    for (_, int) in self.node.op_int.iter() {
      int.visit(value)?;
    }

    Ok(())
  }
}

impl Visitor<TypedVars> for SMoveCardSet {
  type Error = AnalyzerError;

  fn visit(&self, value: &mut TypedVars) -> Result<(), Self::Error> {
    match &self.node {
        MoveCardSet::Move(spanned, _, spanned2) => {
          spanned.visit(value)?;
          spanned2.visit(value)?;
        },
        MoveCardSet::MoveQuantity(spanned, spanned1, _, spanned2) => {
          spanned.visit(value)?;
          spanned1.visit(value)?;
          spanned2.visit(value)?;
        },
    }

    Ok(())
  }
}

impl Visitor<TypedVars> for SClassicMove {
  type Error = AnalyzerError;

  fn visit(&self, value: &mut TypedVars) -> Result<(), Self::Error> {
    match &self.node {
        ClassicMove::MoveCardSet(spanned) => {
          spanned.visit(value)?;
        },
    }

    Ok(())
  }
}

impl Visitor<TypedVars> for SDealMove {
  type Error = AnalyzerError;

  fn visit(&self, value: &mut TypedVars) -> Result<(), Self::Error> {
    match &self.node {
      DealMove::MoveCardSet(spanned) => {
        spanned.visit(value)?;
      },
    }
    Ok(())
  }
}

impl Visitor<TypedVars> for SExchangeMove {
  type Error = AnalyzerError;

  fn visit(&self, value: &mut TypedVars) -> Result<(), Self::Error> {
    match &self.node {
      ExchangeMove::MoveCardSet(spanned) => {
        spanned.visit(value)?;
      },
    }
    Ok(())
  }
}

impl Visitor<TypedVars> for STokenMove {
  type Error = AnalyzerError;

  fn visit(&self, value: &mut TypedVars) -> Result<(), Self::Error> {
    match &self.node {
      TokenMove::PlaceQuantity(spanned, token, token_loc_expr, token_loc_expr1) => {
        spanned.visit(value)?;
        id_not_initialized(value, token)?;
        token_loc_expr.visit(value)?;
        token_loc_expr1.visit(value)?;
      },
      TokenMove::Place(token, token_loc_expr, token_loc_expr1) => {
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
        TokenLocExpr::Groupable(spanned) => {
          spanned.visit(value)?;
        },
        TokenLocExpr::GroupablePlayers(spanned, spanned1) => {
          spanned.visit(value)?;
          spanned1.visit(value)?;
        },
    }
    Ok(())
  }
}

impl Visitor<TypedVars> for SScoreRule {
  type Error = AnalyzerError;

  fn visit(&self, value: &mut TypedVars) -> Result<(), Self::Error> {
    match &self.node {
        ScoreRule::Score(spanned, spanned1) => {
          spanned.visit(value)?;
          spanned1.visit(value)?;
        },
        ScoreRule::ScoreMemory(spanned, spanned1, spanned2) => {
          spanned.visit(value)?;
          id_not_initialized(value, spanned1)?;
          spanned2.visit(value)?;
        },
    }
    Ok(())
  }
}

impl Visitor<TypedVars> for SWinnerType {
  type Error = AnalyzerError;

  fn visit(&self, value: &mut TypedVars) -> Result<(), Self::Error> {
    match &self.node {
        WinnerType::Score => {},
        WinnerType::Memory(spanned) => {
          id_not_initialized(value, spanned)?;
        },
        WinnerType::Position => {},
    }

    Ok(())
  }
}

impl Visitor<TypedVars> for SWinnerRule {
  type Error = AnalyzerError;

  fn visit(&self, value: &mut TypedVars) -> Result<(), Self::Error> {
    match &self.node {
        WinnerRule::Winner(spanned) => {
          spanned.visit(value)?;
        },
        WinnerRule::WinnerWith(_, spanned1) => {
          spanned1.visit(value)?;
        },
    }
    Ok(())
  }
}