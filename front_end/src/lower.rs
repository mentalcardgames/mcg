use std::collections::HashMap;

use crate::{symbols::{AnalyzerError, TypedVars}, typed_ast::*};

use crate::diagnostic::*;

pub fn parse_ast_to_typed_ast(ctx: TypedVars, spanned_ast: &SGame) -> Result<Game, AnalyzerError> {
  let lowering_ctx = LoweringCtx::new(ctx);

  match spanned_ast.lower(&lowering_ctx) {
    Ok(game) => Ok(game),
    Err(type_error) => Err(AnalyzerError::Default)
  }
}

fn all_same(collection: &Vec<SID>, ctx: &LoweringCtx) -> Result<GameType, AnalyzerError> {
    match collection.split_first() {
        Some((first, rest))
          if rest.iter().all(
            |x|
            ctx.lookup_eq(x, first)
          ) => {
              ctx.lookup(first)
          }
        _ => Err(
          AnalyzerError::Default
          // TypeError::AmbiguousTypes(vec![])
        ),
    }
}

#[derive(Debug, PartialEq)]
pub enum TypeError {
  WrongType { expected: GameType, found: GameType },
  NotMatchingTypes { first: GameType, second: GameType },
  PrecedenceOrPointMap { found: GameType},
  SymbolNotFound(String),
  AmbiguousTypes(Vec<GameType>),
  EmptyCollection(Vec<GameType>),
  NoCollectionFound { found: GameType },
  NoBoolExpr { first: GameType, second: GameType},
}

pub struct LoweringCtx {
    symbols: HashMap<String, GameType>,
}

impl LoweringCtx {
  pub fn new(ctx: TypedVars) -> LoweringCtx {
    let symbols = ctx.into_iter().map(|(id, ty)| (id.id.clone(), ty)).collect::<HashMap<String, GameType>>();

    LoweringCtx { symbols }
  }

  fn lookup(&self, id: &SID) -> Result<GameType, AnalyzerError> {
    if let Some(ty) = self.symbols.get(&id.node) {
      return Ok(ty.clone())
    }

    Err(
      AnalyzerError::Default
      // TypeError::SymbolNotFound(id.node.clone())
    )
  }

  fn lookup_eq(&self, id1: &SID, id2: &SID) -> bool {
    if let Some(ty1) = self.symbols.get(&id1.node) {
      if let Some(ty2) = self.symbols.get(&id2.node) {
        return ty1 == ty2
      }
    }

    return false
  } 
}

fn to_typed_id(id: &SID, ty: GameType) -> TypedID{
  return TypedID::new(id.node.clone(), ty.clone())
}

fn resolve_collection(ctx: &LoweringCtx, collection: &Vec<SID>) -> Result<Collection, AnalyzerError> {
    let ty = all_same(&collection, ctx)?;

    match ty {
      GameType::Player => {
        let mut players: Vec<PlayerExpr> = Vec::new();
        for player in collection.iter() {
          players.push(PlayerExpr::Literal(to_typed_id(player, GameType::Player)));
        }

        return Ok(Collection::PlayerCollection(PlayerCollection::Literal(players)))
      },
      GameType::Team => {
        let mut teams: Vec<TeamExpr> = Vec::new();
        for team in collection.iter() {
          teams.push(TeamExpr::Literal(to_typed_id(team, GameType::Team)));
        }

        return Ok(Collection::TeamCollection(TeamCollection::Literal(teams)))
      },
      GameType::Location => {
        let mut locations: Vec<TypedID> = Vec::new();
        for location in collection.iter() {
          locations.push(to_typed_id(location, GameType::Location));
        }

        return Ok(Collection::LocationCollection(LocationCollection { locations: locations }))
      },
      _ => {
        return Err(
          AnalyzerError::Default
          // TypeError::NoCollectionFound {
          //   found: ty
          // }
        )
      },
    }
}

pub trait Lower<T> {
    type Error;

    fn lower(&self, ctx: &LoweringCtx) -> Result<T, Self::Error>;
}

impl Lower<Game> for SGame {
  type Error = AnalyzerError;

  fn lower(&self, ctx: &LoweringCtx) -> Result<Game, Self::Error> {
    let flows = self.node.flows.lower(ctx)?; 
    Ok(Game { flows: flows })
  }
}

impl Lower<Vec<FlowComponent>> for Vec<SFlowComponent> {
  type Error = AnalyzerError;

  fn lower(&self, ctx: &LoweringCtx) -> Result<Vec<FlowComponent>, Self::Error> {
    let mut flows = Vec::new();
    for flow in self.iter() {
      flows.push(flow.lower(ctx)?);
    }

    Ok(flows)
  }
}

impl Lower<FlowComponent> for SFlowComponent {
  type Error = AnalyzerError;

  fn lower(&self, ctx: &LoweringCtx) -> Result<FlowComponent, Self::Error> {
    use crate::spanned_ast::FlowComponent as T;

    Ok(
      match &self.node {
        T::Stage(seq_stage) => {
          FlowComponent::Stage(seq_stage.lower(ctx)?)
        },
        T::Rule(rule) => {
          FlowComponent::Rule(rule.lower(ctx)?)
        },
        T::IfRule(if_rule) => {
          FlowComponent::IfRule(if_rule.lower(ctx)?)
        },
        T::ChoiceRule(choice_rule) => {
          FlowComponent::ChoiceRule(choice_rule.lower(ctx)?)
        },
        T::OptionalRule(optional_rule) => {
          FlowComponent::OptionalRule(optional_rule.lower(ctx)?)
        },
      }
    )
  }
}

impl Lower<SeqStage> for SSeqStage {
  type Error = AnalyzerError;

  fn lower(&self, ctx: &LoweringCtx) -> Result<SeqStage, Self::Error> {
    Ok(
      SeqStage {
        stage: to_typed_id(&self.node.stage, GameType::Stage),
        flows: self.node.flows.lower(ctx)?,
        player: self.node.player.lower(ctx)?,
        end_condition: self.node.end_condition.lower(ctx)?,
      }
    )
  }
}

impl Lower<IfRule> for SIfRule {
  type Error = AnalyzerError;

  fn lower(&self, ctx: &LoweringCtx) -> Result<IfRule, Self::Error> {
    Ok(
      IfRule { 
        condition: self.node.condition.lower(ctx)?,
        flows: self.node.flows.lower(ctx)? 
      }
    )
  }
}

impl Lower<ChoiceRule> for SChoiceRule {
  type Error = AnalyzerError;

  fn lower(&self, ctx: &LoweringCtx) -> Result<ChoiceRule, Self::Error> {
    Ok(
      ChoiceRule { 
        options: self.node.options.lower(ctx)? 
      }
    )
  }
}

impl Lower<OptionalRule> for SOptionalRule {
  type Error = AnalyzerError;

  fn lower(&self, ctx: &LoweringCtx) -> Result<OptionalRule, Self::Error> {
    Ok(
      OptionalRule { flows: self.node.flows.lower(ctx)? }
    )
  }
}

impl Lower<LogicBinOp> for SLogicBinOp {
  type Error = AnalyzerError;

  fn lower(&self, ctx: &LoweringCtx) -> Result<LogicBinOp, Self::Error> {
    use crate::spanned_ast::LogicBinOp as T;

    Ok(
      match &self.node {
        T::And => LogicBinOp::And,
        T::Or => LogicBinOp::Or,
      }
    )
  }
}

impl Lower<EndCondition> for SEndCondition {
  type Error = AnalyzerError;

  fn lower(&self, ctx: &LoweringCtx) -> Result<EndCondition, Self::Error> {
    use crate::spanned_ast::EndCondition as T;

    Ok(
      match &self.node {
        T::UntilBool(bool_expr) => {
          EndCondition::UntilBool(bool_expr.lower(ctx)?)
        },
        T::UntilBoolRep(bool_expr, op, repititions) => {
          EndCondition::UntilBoolRep(
            bool_expr.lower(ctx)?,
            op.lower(ctx)?,
            repititions.lower(ctx)?
          )
        },
        T::UntilRep(repititions) => {
          EndCondition::UntilRep(repititions.lower(ctx)?)
        },
        T::UntilEnd => {
          EndCondition::UntilEnd
        }
      }
    )
  }
}

impl Lower<Repititions> for SRepititions {
  type Error = AnalyzerError;

  fn lower(&self, ctx: &LoweringCtx) -> Result<Repititions, Self::Error> {
    Ok(Repititions { times: self.node.times.lower(ctx)? })
  }
}

impl Lower<IntCompare> for SIntCompare {
  type Error = AnalyzerError;

  fn lower(&self, ctx: &LoweringCtx) -> Result<IntCompare, Self::Error> {
    use crate::spanned_ast::IntCompare as T;

    Ok(
      match &self.node {
        T::Eq => IntCompare::Eq,
        T::Neq => IntCompare::Neq,
        T::Gt => IntCompare::Gt,
        T::Lt => IntCompare::Lt,
        T::Ge => IntCompare::Ge,
        T::Le => IntCompare::Le,
      }
    )
  }
}

impl Lower<CardSetCompare> for SCardSetCompare {
  type Error = AnalyzerError;

  fn lower(&self, ctx: &LoweringCtx) -> Result<CardSetCompare, Self::Error> {
    use crate::spanned_ast::CardSetCompare as T;

    Ok(
      match &self.node {
        T::Eq => CardSetCompare::Eq,
        T::Neq => CardSetCompare::Neq,
      }
    )
  }
}

impl Lower<PlayerCompare> for SPlayerCompare {
  type Error = AnalyzerError;

  fn lower(&self, ctx: &LoweringCtx) -> Result<PlayerCompare, Self::Error> {
    use crate::spanned_ast::PlayerCompare as T;

    Ok(
      match &self.node {
        T::Eq => PlayerCompare::Eq,
        T::Neq => PlayerCompare::Neq,
      }
    )
  }
}

impl Lower<TeamCompare> for STeamCompare {
  type Error = AnalyzerError;

  fn lower(&self, ctx: &LoweringCtx) -> Result<TeamCompare, Self::Error> {
    use crate::spanned_ast::TeamCompare as T;

    Ok(
      match &self.node {
        T::Eq => TeamCompare::Eq,
        T::Neq => TeamCompare::Neq,
      }
    )
  }
}

impl Lower<StringCompare> for SStringCompare {
  type Error = AnalyzerError;

  fn lower(&self, ctx: &LoweringCtx) -> Result<StringCompare, Self::Error> {
    use crate::spanned_ast::StringCompare as T;

    Ok(
      match &self.node {
        T::Eq => StringCompare::Eq,
        T::Neq => StringCompare::Neq,
      }
    )
  }
}

impl Lower<CompareBool> for SCompareBool {
  type Error = AnalyzerError;

  fn lower(&self, ctx: &LoweringCtx) -> Result<CompareBool, Self::Error> {
    use crate::spanned_ast::CompareBool as T;

    Ok(
      match &self.node {
          T::Int(spanned, spanned1, spanned2) => {
            CompareBool::Int(
              spanned.lower(ctx)?,
              spanned1.lower(ctx)?,
              spanned2.lower(ctx)?,
            )
          },
          T::CardSet(spanned, spanned1, spanned2) => {
            CompareBool::CardSet(
              spanned.lower(ctx)?,
              spanned1.lower(ctx)?,
              spanned2.lower(ctx)?,
            )
          },
          T::String(spanned, spanned1, spanned2) => {
            CompareBool::String(
              spanned.lower(ctx)?,
              spanned1.lower(ctx)?,
              spanned2.lower(ctx)?,
            )
          },
          T::Player(spanned, spanned1, spanned2) => {
            CompareBool::Player(
              spanned.lower(ctx)?,
              spanned1.lower(ctx)?,
              spanned2.lower(ctx)?,
            )
          },
          T::Team(spanned, spanned1, spanned2) => {
            CompareBool::Team(
              spanned.lower(ctx)?,
              spanned1.lower(ctx)?,
              spanned2.lower(ctx)?,
            )
          },
      }
    )
  }
}

impl Lower<AggregateBool> for SAggregateBool {
  type Error = AnalyzerError;

  fn lower(&self, ctx: &LoweringCtx) -> Result<AggregateBool, Self::Error> {
    use crate::spanned_ast::AggregateBool as T;

    Ok(
      match &self.node {
        T::Compare(spanned) =>{
          AggregateBool::Compare(
            spanned.lower(ctx)?
          )
        },
        T::CardSetEmpty(spanned) => {
          AggregateBool::CardSetEmpty(
            spanned.lower(ctx)?,
          )
        },
        T::CardSetNotEmpty(spanned) => {
          AggregateBool::CardSetNotEmpty(
            spanned.lower(ctx)?,
          )
        },
        T::OutOfPlayer(spanned, spanned1) => {
          AggregateBool::OutOfPlayer(
            spanned.lower(ctx)?,
            spanned1.lower(ctx)?
          )
        },
      }
    )
  }
}

impl Lower<BoolOp> for SBoolOp {
  type Error = AnalyzerError;

  fn lower(&self, ctx: &LoweringCtx) -> Result<BoolOp, Self::Error> {
    use crate::spanned_ast::BoolOp as T;

    Ok(
      match &self.node {
        T::And => BoolOp::And,
        T::Or => BoolOp::Or,
      }
    )
  }
}

impl Lower<UnaryOp> for SUnaryOp {
  type Error = AnalyzerError;

  fn lower(&self, ctx: &LoweringCtx) -> Result<UnaryOp, Self::Error> {
    use crate::spanned_ast::UnaryOp as T;

    Ok(
      match &self.node {
          T::Not => {
            UnaryOp::Not
          },
      }
    )
  }
}

impl Lower<BoolExpr> for SBoolExpr {
  type Error = AnalyzerError;

  fn lower(&self, ctx: &LoweringCtx) -> Result<BoolExpr, Self::Error> {
    use crate::spanned_ast::BoolExpr as T;

    Ok(
      match &self.node {
        T::Aggregate(spanned) => {
          BoolExpr::Aggregate(
            spanned.lower(ctx)?
          )
        },
        T::Binary(spanned, spanned1, spanned2) => {
          BoolExpr::Binary(
            Box::new(spanned.lower(ctx)?), spanned1.lower(ctx)?, Box::new(spanned2.lower(ctx)?)
          )
        },
        T::Unary(spanned, spanned1) => {
            BoolExpr::Unary(spanned.lower(ctx)?, Box::new(spanned1.lower(ctx)?))
        },
      }
    )
  }
}

impl Lower<OutOf> for SOutOf {
  type Error = AnalyzerError;

  fn lower(&self, ctx: &LoweringCtx) -> Result<OutOf, Self::Error> {
    use crate::spanned_ast::OutOf as T;

    Ok(
      match &self.node {
        T::CurrentStage => OutOf::CurrentStage,
        T::Game => OutOf::Game,
        T::Play => OutOf::Play,
        T::Stage(id) => OutOf::Stage(to_typed_id(id, GameType::Stage)),
      }
    )
  }
}

impl Lower<AggregateInt> for SAggregateInt {
  type Error = AnalyzerError;

  fn lower(&self, ctx: &LoweringCtx) -> Result<AggregateInt, Self::Error> {
    use crate::spanned_ast::AggregateInt as T;

    Ok(
      match &self.node {
        T::SizeOf(spanned) => {
          AggregateInt::SizeOf(
            spanned.lower(ctx)?
          )
        },
        T::SumOfIntCollection(spanned) => {
          AggregateInt::SumOfIntCollection(
            spanned.lower(ctx)?
          )
        },
        T::SumOfCardSet(spanned, spanned1) => {
          AggregateInt::SumOfCardSet(
            Box::new(spanned.lower(ctx)?),
            to_typed_id(spanned1, GameType::PointMap)
          )
        },
        T::ExtremaCardset(spanned, spanned1, spanned2) => {
          AggregateInt::ExtremaCardset(
            spanned.lower(ctx)?,
            Box::new(spanned1.lower(ctx)?),
            to_typed_id(spanned2, GameType::PointMap)
          )
        },
        T::ExtremaIntCollection(spanned, spanned1) => {
          AggregateInt::ExtremaIntCollection(
            spanned.lower(ctx)?,
            spanned1.lower(ctx)?,
          )
        },
      }
    )
  }
}

impl Lower<Extrema> for SExtrema {
  type Error = AnalyzerError;

  fn lower(&self, ctx: &LoweringCtx) -> Result<Extrema, Self::Error> {
    use crate::spanned_ast::Extrema as T;
    
    Ok(
      match self.node {
        T::Min => Extrema::Min,
        T::Max => Extrema::Max,
      }
    )

  }
}

impl Lower<QueryInt> for SQueryInt {
  type Error = AnalyzerError;

  fn lower(&self, ctx: &LoweringCtx) -> Result<QueryInt, Self::Error> {
    use crate::spanned_ast::QueryInt as T;

    Ok(
      match &self.node {
        T::IntCollectionAt(spanned, spanned1) => {
          QueryInt::IntCollectionAt(
            Box::new(spanned.lower(ctx)?),
            Box::new(spanned1.lower(ctx)?),
          )
        },
      }
    )  
  }
}

impl Lower<IntOp> for SIntOp {
  type Error = AnalyzerError;

  fn lower(&self, ctx: &LoweringCtx) -> Result<IntOp, Self::Error> {
    use crate::spanned_ast::IntOp as T;

    Ok(
      match &self.node {
        T::Plus => IntOp::Plus,
        T::Minus => IntOp::Minus,
        T::Mul => IntOp::Mul,
        T::Div => IntOp::Div,
        T::Mod => IntOp::Mod,
      }
    )
  }
}

impl Lower<RuntimeInt> for SRuntimeInt {
  type Error = AnalyzerError;

  fn lower(&self, ctx: &LoweringCtx) -> Result<RuntimeInt, Self::Error> {
    use crate::spanned_ast::RuntimeInt as T;

    Ok(
      match &self.node {
        T::StageRoundCounter => RuntimeInt::StageRoundCounter,
        T::PlayRoundCounter => RuntimeInt::PlayRoundCounter,
      }
    )
  }
}

impl Lower<IntExpr> for SIntExpr {
  type Error = AnalyzerError;

  fn lower(&self, ctx: &LoweringCtx) -> Result<IntExpr, Self::Error> {
    use crate::spanned_ast::IntExpr as T;

    Ok(
      match &self.node {
        T::Aggregate(spanned ) => {
          IntExpr::Aggregate(
            spanned.lower(ctx)?
          )
        },
        T::Binary(spanned, spanned1, spanned2) => {
          IntExpr::Binary(
            Box::new(spanned.lower(ctx)?),
            spanned1.lower(ctx)?,
            Box::new(spanned2.lower(ctx)?),
          )
        },
        T::Query(spanned) => {
          IntExpr::Query(
            spanned.lower(ctx)?
          )
        }
        T::Literal(int) => IntExpr::Literal(int.clone()),
        T::Runtime(spanned) => IntExpr::Runtime(spanned.lower(ctx)?),
      }
    )
  }
}

impl Lower<AggregatePlayer> for SAggregatePlayer {
  type Error = AnalyzerError;

  fn lower(&self, ctx: &LoweringCtx) -> Result<AggregatePlayer, Self::Error> {
    use crate::spanned_ast::AggregatePlayer as T;

    Ok(
      match &self.node {
        T::OwnerOfCardPostion(spanned) => {
          AggregatePlayer::OwnerOfCardPostion(
            Box::new(spanned.lower(ctx)?)
          )
        },
        T::OwnerOfMemory(spanned, spanned1) => {
          AggregatePlayer::OwnerOfMemory(
            spanned.lower(ctx)?,
            to_typed_id(spanned1, GameType::Memory),
          )
        },
      }
    )
  }
}

impl Lower<QueryPlayer> for SQueryPlayer {
  type Error = AnalyzerError;

  fn lower(&self, ctx: &LoweringCtx) -> Result<QueryPlayer, Self::Error> {
    use crate::spanned_ast::QueryPlayer as T;
    
    Ok(
      match &self.node {
        T::Turnorder(spanned) => {
          QueryPlayer::Turnorder(spanned.lower(ctx)?)
        },
      }
    )
  }
}

impl Lower<RuntimePlayer> for SRuntimePlayer {
  type Error = AnalyzerError;

  fn lower(&self, ctx: &LoweringCtx) -> Result<RuntimePlayer, Self::Error> {
    use crate::spanned_ast::RuntimePlayer as T;

    Ok(
      match &self.node {
        T::Current => RuntimePlayer::Current,
        T::Next => RuntimePlayer::Next,
        T::Previous => RuntimePlayer::Previous,
        T::Competitor => RuntimePlayer::Competitor,
      }
    )
  }
}

impl Lower<PlayerExpr> for SPlayerExpr {
  type Error = AnalyzerError;

  fn lower(&self, ctx: &LoweringCtx) -> Result<PlayerExpr, Self::Error> {
    use crate::spanned_ast::PlayerExpr as T;

    Ok(
      match &self.node {
        T::Literal(player_name) => {
          PlayerExpr::Literal(to_typed_id(player_name, GameType::Player))
        },
        T::Aggregate(spanned) => {
          PlayerExpr::Aggregate(spanned.lower(ctx)?)
        },
        T::Query(spanned) => {
          PlayerExpr::Query(spanned.lower(ctx)?)
        },
        T::Runtime(spanned) => PlayerExpr::Runtime(spanned.lower(ctx)?),
      }
    )
  }
}

impl Lower<Players> for SPlayers {
  type Error = AnalyzerError;

  fn lower(&self, ctx: &LoweringCtx) -> Result<Players, Self::Error> {
    use crate::spanned_ast::Players as T;

    Ok(
      match &self.node {
          T::Player(spanned) => {
            Players::Player(spanned.lower(ctx)?)
          },
          T::PlayerCollection(spanned) => {
            Players::PlayerCollection(spanned.lower(ctx)?)
          },
      }
    )  
  }
}

impl Lower<AggregateTeam> for SAggregateTeam {
  type Error = AnalyzerError;

  fn lower(&self, ctx: &LoweringCtx) -> Result<AggregateTeam, Self::Error> {
    use crate::spanned_ast::AggregateTeam as T;

    Ok(
      match &self.node {
        T::TeamOf(spanned) => {
          AggregateTeam::TeamOf(spanned.lower(ctx)?)
        },
      }
    )
  }
}

impl Lower<TeamExpr> for STeamExpr {
  type Error = AnalyzerError;

  fn lower(&self, ctx: &LoweringCtx) -> Result<TeamExpr, Self::Error> {
    use crate::spanned_ast::TeamExpr as T;
    
    Ok(
      match &self.node {
        T::Literal(team_name) => TeamExpr::Literal(to_typed_id(team_name, GameType::Team)),
        T::Aggregate(spanned) => {
          TeamExpr::Aggregate(
            spanned.lower(ctx)?
          )
        },
      }
    )
  }
}

impl Lower<QueryString> for SQueryString {
  type Error = AnalyzerError;

  fn lower(&self, ctx: &LoweringCtx) -> Result<QueryString, Self::Error> {
    use crate::spanned_ast::QueryString as T;
    
    Ok(
      match &self.node {
        T::KeyOf(spanned, spanned1) => {
          QueryString::KeyOf(
            to_typed_id(spanned, GameType::Key),
            spanned1.lower(ctx)?
          )
        },
        T::StringCollectionAt(spanned, spanned1) => {
          QueryString::StringCollectionAt(
            spanned.lower(ctx)?,
            spanned1.lower(ctx)?
          )
        },
      }
    )
  }
}

impl Lower<StringExpr> for SStringExpr {
  type Error = AnalyzerError;

  fn lower(&self, ctx: &LoweringCtx) -> Result<StringExpr, Self::Error> {
    use crate::spanned_ast::StringExpr as T;
    
    Ok(
      match &self.node {
        T::Literal(spanned) => {
          StringExpr::Literal(to_typed_id(spanned, GameType::String))
        },
        T::Query(spanned) => {
          StringExpr::Query(spanned.lower(ctx)?)
        },
      }
    )
  }
}

impl Lower<Owner> for SOwner {
  type Error = AnalyzerError;

  fn lower(&self, ctx: &LoweringCtx) -> Result<Owner, Self::Error> {
    use crate::spanned_ast::Owner as T;

    Ok(
      match &self.node {
        T::Player(spanned) => {
          Owner::Player(spanned.lower(ctx)?)
        },
        T::PlayerCollection(spanned) => {
          Owner::PlayerCollection(spanned.lower(ctx)?)
        },
        T::Team(spanned) => {
          Owner::Team(spanned.lower(ctx)?)
        },
        T::TeamCollection(spanned) => {
          Owner::TeamCollection(spanned.lower(ctx)?)
        },
        T::Table => Owner::Table,
      }
    )
  }
}

impl Lower<CardSet> for SCardSet {
  type Error = AnalyzerError;

  fn lower(&self, ctx: &LoweringCtx) -> Result<CardSet, Self::Error> {
    use crate::spanned_ast::CardSet as T;

    Ok(
      match &self.node {
        T::Group(group) => {
          CardSet::Group(
            group.lower(ctx)?
          )
        },
        T::GroupOwner(group, owner) => {
          CardSet::GroupOwner(
            group.lower(ctx)?,
            owner.lower(ctx)?,
          )
        },
      }
    )
  }
}

impl Lower<Groupable> for SGroupable {
  type Error = AnalyzerError;

  fn lower(&self, ctx: &LoweringCtx) -> Result<Groupable, Self::Error> {
    use crate::spanned_ast::Groupable as T;

    Ok(
      match &self.node {
        T::Location(spanned) => {
          Groupable::Location(to_typed_id(spanned, GameType::Location))
        },
        T::LocationCollection(spanned) => {
          Groupable::LocationCollection(spanned.lower(ctx)?)
        },
      }
    )
  }  
}

impl Lower<Group> for SGroup {
  type Error = AnalyzerError;

  fn lower(&self, ctx: &LoweringCtx) -> Result<Group, Self::Error> {
    use crate::spanned_ast::Group as T;

    Ok(
      match &self.node {
        T::Groupable(spanned) => {
          Group::Groupable(
            spanned.lower(ctx)?,
          )
        },
        T::Where(spanned, spanned1) => {
          Group::Where(
            spanned.lower(ctx)?,
            spanned1.lower(ctx)?
          )
        },
        T::Combo(spanned, spanned1) => {
          Group::Combo(
            to_typed_id(spanned, GameType::Combo),
            spanned1.lower(ctx)?
          )
        },
        T::NotCombo(spanned, spanned1) => {
          Group::NotCombo(
            to_typed_id(spanned, GameType::Combo),
            spanned1.lower(ctx)?
          )
        },
        T::CardPosition(card_position) => {
          Group::CardPosition(
            card_position.lower(ctx)?
          )
        },
      }
    )
  }
}

impl Lower<AggregateCardPosition> for SAggregateCardPosition {
  type Error = AnalyzerError;

  fn lower(&self, ctx: &LoweringCtx) -> Result<AggregateCardPosition, Self::Error> {
    use crate::spanned_ast::AggregateCardPosition as T;

    Ok(
      match &self.node {
          T::Extrema(spanned, spanned1, spanned2) => {
            // LookUp if it is pointmap or precedence
            let ty = ctx.lookup(spanned2)?;

            AggregateCardPosition::Extrema(
              spanned.lower(ctx)?,
              Box::new(spanned1.lower(ctx)?),
              to_typed_id(spanned2, ty)
            )
          },
      }
    )
  }
}

impl Lower<QueryCardPosition> for SQueryCardPosition {
  type Error = AnalyzerError;

  fn lower(&self, ctx: &LoweringCtx) -> Result<QueryCardPosition, Self::Error> {
    use crate::spanned_ast::QueryCardPosition as T;

    Ok(
      match &self.node {
        T::At(spanned, spanned1) => {
          QueryCardPosition::At(
            to_typed_id(spanned, GameType::Location),
            spanned1.lower(ctx)?,
          )
        },
        T::Top(spanned) => {
          QueryCardPosition::Top(
            to_typed_id(spanned, GameType::Location)
          )
        },
        T::Bottom(spanned) => {
          QueryCardPosition::Bottom(
            to_typed_id(spanned, GameType::Location)
          )
        },
      }
    )
  }
}

impl Lower<CardPosition> for SCardPosition {
  type Error = AnalyzerError;

  fn lower(&self, ctx: &LoweringCtx) -> Result<CardPosition, Self::Error> {
    use crate::spanned_ast::CardPosition as T;
    
    Ok(
      match &self.node {
        T::Aggregate(spanned) => {
          CardPosition::Aggregate(
            spanned.lower(ctx)?
          )
        },
        T::Query(spanned) => {
          CardPosition::Query(
            spanned.lower(ctx)?
          )
        },
      }
    )
  }
}

impl Lower<AggregateFilter> for SAggregateFilter {
  type Error = AnalyzerError;

  fn lower(&self, ctx: &LoweringCtx) -> Result<AggregateFilter, Self::Error> {
    use crate::spanned_ast::AggregateFilter as T;

    Ok(
      match &self.node {
        T::Size(spanned, spanned1) => {
          AggregateFilter::Size(
            spanned.lower(ctx)?,
            Box::new(spanned1.lower(ctx)?)
          )
        },
        T::Same(spanned) => {
          AggregateFilter::Same(
            to_typed_id(spanned, GameType::Key)
          )
        },
        T::Distinct(spanned) => {
          AggregateFilter::Same(
            to_typed_id(spanned, GameType::Key)
          )
        },
        T::Higher(spanned, spanned1) => {
          AggregateFilter::Higher(
            to_typed_id(spanned, GameType::Key),
            to_typed_id(spanned1, GameType::Precedence)
          )
        },
        T::Lower(spanned, spanned1) => {
          AggregateFilter::Lower(
            to_typed_id(spanned, GameType::Key),
            to_typed_id(spanned1, GameType::Precedence)
          )
        },
        T::Adjacent(spanned, spanned1) => {
          AggregateFilter::Adjacent(
            to_typed_id(spanned, GameType::Key),
            to_typed_id(spanned1, GameType::Precedence)
          )
        },
        T::KeyString(spanned, spanned1, spanned2) => {
          AggregateFilter::KeyString(
            to_typed_id(spanned, GameType::Key),
            spanned1.lower(ctx)?,
            Box::new(spanned2.lower(ctx)?),
          )
        },
        T::Combo(spanned) => {
          AggregateFilter::Combo(
            to_typed_id(spanned, GameType::Combo),
          )
        },
        T::NotCombo(spanned) => {
          AggregateFilter::Combo(
            to_typed_id(spanned, GameType::Combo),
          )
        },
      }
    )
  }
}

impl Lower<FilterOp> for SFilterOp {
  type Error = AnalyzerError;

  fn lower(&self, ctx: &LoweringCtx) -> Result<FilterOp, Self::Error> {
    use crate::spanned_ast::FilterOp as T;

    Ok(
      match &self.node {
        T::And => FilterOp::And,
        T::Or => FilterOp::Or,
      }
    )
  }
}

impl Lower<FilterExpr> for SFilterExpr {
  type Error = AnalyzerError;

  fn lower(&self, ctx: &LoweringCtx) -> Result<FilterExpr, Self::Error> {
    use crate::spanned_ast::FilterExpr as T;

    Ok(
      match &self.node {
        T::Aggregate(spanned) => {
          FilterExpr::Aggregate(spanned.lower(ctx)?)
        },
        T::Binary(spanned, spanned1, spanned2) => {
          FilterExpr::Binary(
            Box::new(spanned.lower(ctx)?),
            spanned1.lower(ctx)?,
            Box::new(spanned2.lower(ctx)?),
          )
        },
      }
    )
  }
}

impl Lower<Collection> for SCollection {
  type Error = AnalyzerError;

  fn lower(&self, ctx: &LoweringCtx) -> Result<Collection, Self::Error> {
    use crate::spanned_ast::Collection as T;

    Ok(
      match &self.node {
        T::IntCollection(int_collection) => {
            Collection::IntCollection(int_collection.lower(ctx)?)
          },
        T::StringCollection(string_collection) => {
            Collection::StringCollection(string_collection.lower(ctx)?)
          },
        T::LocationCollection(location_collection) => {
            Collection::LocationCollection(location_collection.lower(ctx)?)
          },
        T::PlayerCollection(player_collection) => {
            Collection::PlayerCollection(player_collection.lower(ctx)?)
          },
        T::TeamCollection(team_collection) => {
            Collection::TeamCollection(team_collection.lower(ctx)?)
          },
        T::CardSet(card_set) => {
            Collection::CardSet(Box::new(card_set.lower(ctx)?))
          },
        T::Ambiguous(spanneds) => {
          resolve_collection(ctx, spanneds)?
        },
      }
    )
  }
}

impl Lower<IntCollection> for SIntCollection {
  type Error = AnalyzerError;

  fn lower(&self, ctx: &LoweringCtx) -> Result<IntCollection, Self::Error> {
    let mut ints = Vec::new();
    for int_expr in self.node.ints.iter() {
      ints.push(int_expr.lower(ctx)?)
    }

    Ok(IntCollection { ints: ints })
  }
} 

impl Lower<StringCollection> for SStringCollection {
  type Error = AnalyzerError;

  fn lower(&self, ctx: &LoweringCtx) -> Result<StringCollection, Self::Error> {
    let mut strings = Vec::new();
    for string_expr in self.node.strings.iter() {
      strings.push(string_expr.lower(ctx)?)
    }

    Ok(StringCollection { strings: strings })
  }
}

impl Lower<LocationCollection> for SLocationCollection {
  type Error = AnalyzerError;

  fn lower(&self, ctx: &LoweringCtx) -> Result<LocationCollection, Self::Error> {
    let mut locations = Vec::new();
    for location in self.node.locations.iter() {
      locations.push(to_typed_id(location, GameType::Location))
    }

    Ok(LocationCollection { locations: locations })
  }
}

impl Lower<AggregatePlayerCollection> for SAggregatePlayerCollection {
  type Error = AnalyzerError;

  fn lower(&self, ctx: &LoweringCtx) -> Result<AggregatePlayerCollection, Self::Error> {
    use crate::spanned_ast::AggregatePlayerCollection as T;

    Ok(
      match &self.node {
        T::Quantifier(spanned) => AggregatePlayerCollection::Quantifier(spanned.lower(ctx)?),
      }
    )
  }
}

impl Lower<RuntimePlayerCollection> for SRuntimePlayerCollection {
  type Error = AnalyzerError;

  fn lower(&self, ctx: &LoweringCtx) -> Result<RuntimePlayerCollection, Self::Error> {
    use crate::spanned_ast::RuntimePlayerCollection as T;

    Ok(
      match &self.node {
        T::PlayersOut => RuntimePlayerCollection::PlayersOut,
        T::PlayersIn => RuntimePlayerCollection::PlayersIn,
        T::Others => RuntimePlayerCollection::Others,
      }
    )
  }
}

impl Lower<PlayerCollection> for SPlayerCollection {
  type Error = AnalyzerError;

  fn lower(&self, ctx: &LoweringCtx) -> Result<PlayerCollection, Self::Error> {
    use crate::spanned_ast::PlayerCollection as T;

    Ok(
      match &self.node {
        T::Literal(player_exprs) => {
          let mut players = Vec::new();
          for player_expr in player_exprs.iter() {
            players.push(player_expr.lower(ctx)?)
          }

          PlayerCollection::Literal(players)
        },
        T::Aggregate(spanned) => {
          PlayerCollection::Aggregate(spanned.lower(ctx)?)
        },
        T::Runtime(spanned) => {
          PlayerCollection::Runtime(spanned.lower(ctx)?)
        },
      }
    )
  }
}

impl Lower<RuntimeTeamCollection> for SRuntimeTeamCollection {
  type Error = AnalyzerError;

  fn lower(&self, ctx: &LoweringCtx) -> Result<RuntimeTeamCollection, Self::Error> {
    use crate::spanned_ast::RuntimeTeamCollection as T;

    Ok(
      match &self.node {
        T::OtherTeams => RuntimeTeamCollection::OtherTeams,
      }
    )
  }
}

impl Lower<TeamCollection> for STeamCollection {
  type Error = AnalyzerError;

  fn lower(&self, ctx: &LoweringCtx) -> Result<TeamCollection, Self::Error> {
    use crate::spanned_ast::TeamCollection as T;

    Ok(
      match &self.node {
        T::Literal(team_exprs) => {
          let mut teams = Vec::new();
          for team_expr in team_exprs.iter() {
            teams.push(team_expr.lower(ctx)?)
          }

          TeamCollection::Literal(teams)
        },
        T::Runtime(spanned) => {
          TeamCollection::Runtime(spanned.lower(ctx)?)
        },
      }
    )
  }
}

impl Lower<MemoryType> for SMemoryType {
  type Error = AnalyzerError;

  fn lower(&self, ctx: &LoweringCtx) -> Result<MemoryType, Self::Error> {
    use crate::spanned_ast::MemoryType as T;

    Ok(
      match &self.node {
        T::Int(spanned) => {
          MemoryType::Int(
            spanned.lower(ctx)?
          )
        },
        T::String(spanned) => {
          MemoryType::String(
            spanned.lower(ctx)?
          )
        },
        T::CardSet(spanned) => {
          MemoryType::CardSet(
            spanned.lower(ctx)?
          )
        },
        T::Collection(spanned) => {
          MemoryType::Collection(
            spanned.lower(ctx)?
          )
        },
      }
    )
  }
}

impl Lower<SetUpRule> for SSetUpRule {
  type Error = AnalyzerError;

  fn lower(&self, ctx: &LoweringCtx) -> Result<SetUpRule, Self::Error> {
    use crate::spanned_ast::SetUpRule as T;

    Ok(
      match &self.node {
        T::CreatePlayer(spanneds) => {
          let mut player_names = Vec::new();
          for player in spanneds.iter() {
            player_names.push(to_typed_id(player, GameType::Player));
          }

          SetUpRule::CreatePlayer(player_names)
        },
        T::CreateTeams(items) => {
          let mut teams = Vec::new();
          for (team_name, player_collection) in items.iter() {
            teams.push(
              (
                to_typed_id(team_name, GameType::Team),
                player_collection.lower(ctx)?
              )
            )
          }

          SetUpRule::CreateTeams(teams)
        },
        T::CreateTurnorder(spanned) => {
          SetUpRule::CreateTurnorder(spanned.lower(ctx)?)
        },
        T::CreateTurnorderRandom(spanned) => {
          SetUpRule::CreateTurnorderRandom(spanned.lower(ctx)?)
        },
        T::CreateLocation(spanneds, spanned) => {
          let mut locations = Vec::new();
          for id in spanneds.iter() {
            locations.push(to_typed_id(id, GameType::Location));
          }

          SetUpRule::CreateLocation(locations, spanned.lower(ctx)?)
        },
        T::CreateCardOnLocation(location, types) => {
          SetUpRule::CreateCardOnLocation(
            to_typed_id(location, GameType::Location),
            types.lower(ctx)?
          )
        },
        T::CreateTokenOnLocation(spanned, spanned1, spanned2) => {
          SetUpRule::CreateTokenOnLocation(
            spanned.lower(ctx)?,
            to_typed_id(spanned1, GameType::Token),
            to_typed_id(spanned2, GameType::Location),
          )
        },
        T::CreateCombo(spanned, spanned1) => {
          SetUpRule::CreateCombo(
            to_typed_id(spanned, GameType::Combo),
            spanned1.lower(ctx)?
          )
        },
        T::CreateMemory(spanned, spanned1, spanned2) => {
          SetUpRule::CreateMemory(
            to_typed_id(spanned, GameType::Memory),
            spanned1.lower(ctx)?,
            spanned2.lower(ctx)?,
          )
        },
        T::CreatePrecedence(spanned, items) => {
          let mut kv = Vec::new();
          for (k, v) in items.iter() {
            kv.push(
              (
                to_typed_id(k, GameType::Key),
                to_typed_id(v, GameType::Value),
              )
            )
          }

          SetUpRule::CreatePrecedence(
            to_typed_id(spanned, GameType::Precedence),
            kv
          )
        },
        T::CreatePointMap(spanned, items) => {
          let mut kvi = Vec::new();
          for (k, v, i) in items.iter() {
            kvi.push(
              (
                to_typed_id(k, GameType::Key),
                to_typed_id(v, GameType::Value),
                i.lower(ctx)?
              )
            )
          }

          SetUpRule::CreatePointMap(
            to_typed_id(spanned, GameType::PointMap),
            kvi
          )
        },
      }
    )
  }
}

impl Lower<Status> for SStatus {
  type Error = AnalyzerError;

  fn lower(&self, ctx: &LoweringCtx) -> Result<Status, Self::Error> {
    use crate::spanned_ast::Status as T;

    Ok(
      match &self.node {
        T::FaceUp => Status::FaceUp,
        T::FaceDown => Status::FaceDown,
        T::Private => Status::Private,
      }
    )
  }
}

impl Lower<EndType> for SEndType {
  type Error = AnalyzerError;

  fn lower(&self, ctx: &LoweringCtx) -> Result<EndType, Self::Error> {
    use crate::spanned_ast::EndType as T;

    Ok(
      match &self.node {
        T::Turn => EndType::Turn,
        T::Stage => EndType::Stage,
        T::GameWithWinner(spanned) => EndType::GameWithWinner(spanned.lower(ctx)?),
      }
    )
  }
}

impl Lower<ActionRule> for SActionRule {
  type Error = AnalyzerError;

  fn lower(&self, ctx: &LoweringCtx) -> Result<ActionRule, Self::Error> {
    use crate::spanned_ast::ActionRule as T;

    Ok(
      match &self.node {
        T::FlipAction(spanned, spanned1) => {
          ActionRule::FlipAction(
            spanned.lower(ctx)?,
            spanned1.lower(ctx)?,
          )
        },
        T::ShuffleAction(spanned) => {
          ActionRule::ShuffleAction(
            spanned.lower(ctx)?,
          )
        },
        T::PlayerOutOfStageAction(spanned) => {
          ActionRule::PlayerOutOfStageAction(
            spanned.lower(ctx)?,
          )
        },
        T::PlayerOutOfGameSuccAction(spanned) => {
          ActionRule::PlayerOutOfGameSuccAction(
            spanned.lower(ctx)?,
          )
        },
        T::PlayerOutOfGameFailAction(spanned) => {
          ActionRule::PlayerOutOfGameFailAction(
            spanned.lower(ctx)?,
          )
        },
        T::SetMemory(spanned, spanned1) => {
          ActionRule::SetMemory(
            to_typed_id(spanned, GameType::Memory),
            spanned1.lower(ctx)?
          )
        },
        T::ResetMemory(spanned) => {
          ActionRule::ResetMemory(
            to_typed_id(spanned, GameType::Memory),
          )
        },
        T::CycleAction(spanned) => {
          ActionRule::CycleAction(
            spanned.lower(ctx)?,
          )
        },
        T::BidAction(spanned) => {
          ActionRule::BidAction(
            spanned.lower(ctx)?,
          )
        },
        T::BidMemoryAction(spanned, spanned1) => {
          ActionRule::BidMemoryAction(
            to_typed_id(spanned, GameType::Memory),
            spanned1.lower(ctx)?,
          )
        },
        T::EndAction(spanned) => {
          ActionRule::EndAction(spanned.lower(ctx)?)
        },
        T::DemandAction(spanned) => {
          ActionRule::DemandAction(
            spanned.lower(ctx)?,
          )
        },
        T::DemandMemoryAction(spanned, spanned1) => {
          ActionRule::DemandMemoryAction(
            spanned.lower(ctx)?,
            to_typed_id(spanned1, GameType::Memory),
          )
        },
        T::Move(spanned) => {
          ActionRule::Move(
            spanned.lower(ctx)?,
          )
        },
      }
    )
  }
}

impl Lower<DemandType> for SDemandType {
  type Error = AnalyzerError;

  fn lower(&self, ctx: &LoweringCtx) -> Result<DemandType, Self::Error> {
    use crate::spanned_ast::DemandType as T;

    Ok(
      match &self.node {
          T::CardPosition(spanned) => {
            DemandType::CardPosition(
              spanned.lower(ctx)?,
            )
          },
          T::String(spanned) => {
            DemandType::String(
              spanned.lower(ctx)?,
            )
          },
          T::Int(spanned) => {
            DemandType::Int(
              spanned.lower(ctx)?,
            )
          },
      }
    )
  }
}

impl Lower<MoveType> for SMoveType {
  type Error = AnalyzerError;

  fn lower(&self, ctx: &LoweringCtx) -> Result<MoveType, Self::Error> {
    use crate::spanned_ast::MoveType as T;
    
    Ok(
      match &self.node {
        T::Deal(spanned) => {
          MoveType::Deal(
            spanned.lower(ctx)?,
          )
        },
        T::Exchange(spanned) => {
          MoveType::Exchange(
            spanned.lower(ctx)?,
          )
        },
        T::Classic(spanned) => {
          MoveType::Classic(
            spanned.lower(ctx)?,
          )
        },
        T::Place(spanned) => {
          MoveType::Place(
            spanned.lower(ctx)?,
          )
        },
      }
    )
  }
}

impl Lower<ScoringRule> for SScoringRule {
  type Error = AnalyzerError;

  fn lower(&self, ctx: &LoweringCtx) -> Result<ScoringRule, Self::Error> {
    use crate::spanned_ast::ScoringRule as T;
  
    Ok(
      match &self.node {
        T::ScoreRule(spanned) => {
          ScoringRule::ScoreRule(
            spanned.lower(ctx)?,
          )
        },
        T::WinnerRule(spanned) => {
          ScoringRule::WinnerRule(
            spanned.lower(ctx)?,
          )
        },
      }
    )
  }
}

impl Lower<GameRule> for SGameRule {
  type Error = AnalyzerError;

  fn lower(&self, ctx: &LoweringCtx) -> Result<GameRule, Self::Error> {
    use crate::spanned_ast::GameRule as T;

    Ok(
      match &self.node {
        T::SetUp(spanned) => {
          GameRule::SetUp(
            spanned.lower(ctx)?,
          )
        },
        T::Action(spanned) => {
          GameRule::Action(
            spanned.lower(ctx)?,
          )
        },
        T::Scoring(spanned) => {
          GameRule::Scoring(
            spanned.lower(ctx)?,
          )
        },
      }
    )
  }
}

impl Lower<Types> for STypes {
  type Error = AnalyzerError;

  fn lower(&self, ctx: &LoweringCtx) -> Result<Types, Self::Error> {
    let mut k_vs = Vec::new();

    for (k, vs) in self.node.types.iter() {
      let mut typed_vs = Vec::new();
      for v in vs.iter() {
        typed_vs.push(to_typed_id(v, GameType::Value));
      }

      k_vs.push((to_typed_id(k, GameType::Key), typed_vs))
    }

    Ok(Types { types: k_vs })
  }
}

impl Lower<Quantity> for SQuantity {
  type Error = AnalyzerError;

  fn lower(&self, ctx: &LoweringCtx) -> Result<Quantity, Self::Error> {
    use crate::spanned_ast::Quantity as T;

    Ok(
      match &self.node {
        T::Int(int_expr) => {
          Quantity::Int(
            int_expr.lower(ctx)?
          )
        },
        T::Quantifier(quantifier) => {
          Quantity::Quantifier(
            quantifier.lower(ctx)?
          )
        },
        T::IntRange(int_range) => {
          Quantity::IntRange(
            int_range.lower(ctx)?
          )
        },
      }
    )
  }
}

impl Lower<Quantifier> for SQuantifier {
  type Error = AnalyzerError;

  fn lower(&self, ctx: &LoweringCtx) -> Result<Quantifier, Self::Error> {
    use crate::spanned_ast::Quantifier as T;

    Ok(
      match &self.node {
        T::All => {
          Quantifier::All
        },
        T::Any => {
          Quantifier::Any
        },
      }
    )
  }
}

impl Lower<IntRange> for SIntRange {
  type Error = AnalyzerError;

  fn lower(&self, ctx: &LoweringCtx) -> Result<IntRange, Self::Error> {
    let mut op_int = Vec::new();
    for (o, i) in self.node.op_int.iter() {
      op_int.push((o.lower(ctx)?, i.lower(ctx)?))
    }
    Ok(IntRange { op_int: op_int })
  }
}

impl Lower<MoveCardSet> for SMoveCardSet {
  type Error = AnalyzerError;

  fn lower(&self, ctx: &LoweringCtx) -> Result<MoveCardSet, Self::Error> {
    use crate::spanned_ast::MoveCardSet as T;

    Ok(
      match &self.node {
        T::MoveQuantity(spanned, spanned1,spanned2, spanned3) => {
          MoveCardSet::MoveQuantity(
            spanned.lower(ctx)?,
            spanned1.lower(ctx)?,
            spanned2.lower(ctx)?,
            spanned3.lower(ctx)?,
          )
        },
        T::Move(spanned, spanned1,spanned2) => {
          MoveCardSet::Move(
            spanned.lower(ctx)?,
            spanned1.lower(ctx)?,
            spanned2.lower(ctx)?,
          )
        },
      }
    )
  }
}

impl Lower<ClassicMove> for SClassicMove {
  type Error = AnalyzerError;

  fn lower(&self, ctx: &LoweringCtx) -> Result<ClassicMove, Self::Error> {
    use crate::spanned_ast::ClassicMove as T;

    Ok(
      match &self.node {
        T::MoveCardSet(spanned) => {
          ClassicMove::MoveCardSet(
            spanned.lower(ctx)?
          )
        },
      }
    )
  }
}

impl Lower<DealMove> for SDealMove {
  type Error = AnalyzerError;

  fn lower(&self, ctx: &LoweringCtx) -> Result<DealMove, Self::Error> {
    use crate::spanned_ast::DealMove as T;

    Ok(
      match &self.node {
        T::MoveCardSet(spanned) => {
          DealMove::MoveCardSet(
            spanned.lower(ctx)?
          )
        },
      }
    )
  }
}

impl Lower<ExchangeMove> for SExchangeMove {
  type Error = AnalyzerError;

  fn lower(&self, ctx: &LoweringCtx) -> Result<ExchangeMove, Self::Error> {
    use crate::spanned_ast::ExchangeMove as T;

    Ok(
      match &self.node {
        T::MoveCardSet(spanned) => {
          ExchangeMove::MoveCardSet(
            spanned.lower(ctx)?
          )
        },
      }
    )
  }
}

impl Lower<TokenMove> for STokenMove {
  type Error = AnalyzerError;

  fn lower(&self, ctx: &LoweringCtx) -> Result<TokenMove, Self::Error> {
    use crate::spanned_ast::TokenMove as T;

    Ok(
      match &self.node {
        T::PlaceQuantity(spanned, token, token_loc_expr, token_loc_expr1) => {
          TokenMove::PlaceQuantity(
            spanned.lower(ctx)?,
            to_typed_id(token, GameType::Token),
            token_loc_expr.lower(ctx)?,
            token_loc_expr1.lower(ctx)?,
          )
        },
        T::Place(token, token_loc_expr, token_loc_expr1) => {
          TokenMove::Place(
            to_typed_id(token, GameType::Token),
            token_loc_expr.lower(ctx)?,
            token_loc_expr1.lower(ctx)?,
          )
        },
      }
    )
  }
} 

impl Lower<TokenLocExpr> for STokenLocExpr {
  type Error = AnalyzerError;

  fn lower(&self, ctx: &LoweringCtx) -> Result<TokenLocExpr, Self::Error> {
    use crate::spanned_ast::TokenLocExpr as T;

    Ok(
      match &self.node {
        T::Groupable(spanned) => {
          TokenLocExpr::Groupable(spanned.lower(ctx)?)
        },
        T::GroupablePlayers(spanned, spanned1) => {
          TokenLocExpr::GroupablePlayers(
            spanned.lower(ctx)?,
            spanned1.lower(ctx)?,
          )
        },
      }
    )
  }
}

impl Lower<ScoreRule> for SScoreRule {
  type Error = AnalyzerError;

  fn lower(&self, ctx: &LoweringCtx) -> Result<ScoreRule, Self::Error> {
    use crate::spanned_ast::ScoreRule as T;
    
    Ok(
      match &self.node {
          T::Score(spanned, spanned1) => {
            ScoreRule::Score(
              spanned.lower(ctx)?,
              spanned1.lower(ctx)?
            )
          },
          T::ScoreMemory(spanned, spanned1, spanned2) => {
            ScoreRule::ScoreMemory(
              spanned.lower(ctx)?,
              to_typed_id(spanned1, GameType::Memory),
              spanned2.lower(ctx)?,
            )
          },
      }
    )
  }
}

impl Lower<WinnerType> for SWinnerType {
  type Error = AnalyzerError;

  fn lower(&self, ctx: &LoweringCtx) -> Result<WinnerType, Self::Error> {
    use crate::spanned_ast::WinnerType as T;

    Ok(
      match &self.node {
        T::Score => {
          WinnerType::Score
        },
        T::Memory(spanned) => {
          WinnerType::Memory(to_typed_id(spanned, GameType::Memory))
        },
        T::Position => {
          WinnerType::Position
        },
      }
    )
  }
}

impl Lower<WinnerRule> for SWinnerRule {
  type Error = AnalyzerError;

  fn lower(&self, ctx: &LoweringCtx) -> Result<WinnerRule, Self::Error> {
    use crate::spanned_ast::WinnerRule as T;

    Ok(
      match &self.node {
        T::Winner(spanned) => {
          WinnerRule::Winner(spanned.lower(ctx)?)
        },
        T::WinnerWith(spanned, spanned1) => {
          WinnerRule::WinnerWith(spanned.lower(ctx)?, spanned1.lower(ctx)?)
        },
      }
    )
  }
}