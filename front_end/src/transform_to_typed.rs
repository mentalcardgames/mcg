use std::collections::HashMap;

use crate::{analysis::AnalyzerError, spanned_ast::{self}, symbols::TypedVars, typed_ast::{self, GameType, PlayerExpr, TeamExpr, TypedID}};

use crate::diagnostic::*;

pub fn parse_ast_to_typed_ast(ctx: TypedVars, spanned_ast: &SGame) -> Result<typed_ast::Game, AnalyzerError> {
  let lowering_ctx = LoweringCtx::new(ctx);

  match spanned_ast.lower(&lowering_ctx) {
    Ok(game) => Ok(game),
    Err(type_error) => Err(AnalyzerError::TypeError(type_error))
  }
}

fn all_same(collection: &Vec<SID>, ctx: &LoweringCtx) -> Result<GameType, TypeError> {
    match collection.split_first() {
        Some((first, rest))
          if rest.iter().all(
            |x|
            ctx.lookup_eq(x, first)
          ) => {
              ctx.lookup(first)
          }
        // TODO
        _ => Err(TypeError::AmbiguousTypes(vec![])),
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

  fn lookup(&self, id: &SID) -> Result<GameType, TypeError> {
    if let Some(ty) = self.symbols.get(&id.node) {
      return Ok(ty.clone())
    }

    Err(TypeError::SymbolNotFound(id.node.clone()))
  }

  fn lookup_eq(&self, id1: &SID, id2: &SID) -> bool {
    if let Some(ty1) = self.symbols.get(&id1.node) {
      if let Some(ty2) = self.symbols.get(&id2.node) {
        return ty1 == ty2
      }
    }

    return false
  }

  pub fn to_typed_id(&self, id: &SID, ty: GameType) -> TypedID{
    return TypedID::new(id.node.clone(), ty.clone())
  } 
}

fn resolve_collection(ctx: &LoweringCtx, collection: &Vec<SID>) -> Result<typed_ast::Collection, TypeError> {
    let ty = all_same(&collection, ctx)?;

    match ty {
      GameType::Player => {
        let mut players: Vec<PlayerExpr> = Vec::new();
        for player in collection.iter() {
          players.push(PlayerExpr::PlayerName(ctx.to_typed_id(player, GameType::Player)));
        }

        return Ok(typed_ast::Collection::PlayerCollection(typed_ast::PlayerCollection::Player(players)))
      },
      GameType::Team => {
        let mut teams: Vec<TeamExpr> = Vec::new();
        for team in collection.iter() {
          teams.push(TeamExpr::TeamName(ctx.to_typed_id(team, GameType::Team)));
        }

        return Ok(typed_ast::Collection::TeamCollection(typed_ast::TeamCollection::Team(teams)))
      },
      GameType::Location => {
        let mut locations: Vec<TypedID> = Vec::new();
        for location in collection.iter() {
          locations.push(ctx.to_typed_id(location, GameType::Location));
        }

        return Ok(typed_ast::Collection::LocationCollection(typed_ast::LocationCollection { locations: locations }))
      },
      _ => {
        return Err(
          TypeError::NoCollectionFound {
            found: ty
          }
        )
      },
    }
}

pub trait Lower<T> {
    type Error;

    fn lower(&self, ctx: &LoweringCtx) -> Result<T, Self::Error>;
}

impl Lower<typed_ast::PlayerExpr> for SPlayerExpr {
  type Error = TypeError;

  fn lower(&self, ctx: &LoweringCtx) -> Result<typed_ast::PlayerExpr, Self::Error> {
      use spanned_ast::PlayerExpr::*;
      use typed_ast::PlayerExpr as T;

      Ok(
        match &self.node {
          PlayerName(player_name) => {
            T::PlayerName(ctx.to_typed_id(player_name, GameType::Player))
          }
          Current => T::Current,
          Next => T::Next,
          Previous => T::Previous,
          Competitor => T::Competitor,
          Turnorder(int_expr) => T::Turnorder(int_expr.lower(ctx)?),
          OwnerOf(card_position) => T::OwnerOf(Box::new(card_position.lower(ctx)?)),
          OwnerOfHighest(memory) => {
            T::OwnerOfHighest(ctx.to_typed_id(memory, GameType::Memory))
          },
          OwnerOfLowest(memory) => {
            T::OwnerOfLowest(ctx.to_typed_id(memory, GameType::Memory))
          },
      }
    )
  }
}


impl Lower<typed_ast::IntExpr> for SIntExpr {
  type Error = TypeError;

  fn lower(&self, ctx: &LoweringCtx) -> Result<typed_ast::IntExpr, Self::Error> {
    use typed_ast::IntExpr as T;

    Ok(
      match &self.node {
        spanned_ast::IntExpr::Int(int) => T::Int(*int),
        spanned_ast::IntExpr::IntOp(int_expr, op, int_expr1) => {
          T::IntOp(Box::new(int_expr.lower(ctx)?), op.lower(ctx)?, Box::new(int_expr1.lower(ctx)?))
        },
        spanned_ast::IntExpr::IntCollectionAt(int_expr) => {
          T::IntCollectionAt(Box::new(int_expr.lower(ctx)?))
        },
        spanned_ast::IntExpr::SizeOf(collection) => {
          T::SizeOf(collection.lower(ctx)?)
        },
        spanned_ast::IntExpr::SumOfIntCollection(int_collection) => {
          T::SumOfIntCollection(int_collection.lower(ctx)?)
        },
        spanned_ast::IntExpr::SumOfCardSet(card_set, point_map) => {
          T::SumOfCardSet(Box::new(card_set.lower(ctx)?), ctx.to_typed_id(point_map, GameType::PointMap))
        },
        spanned_ast::IntExpr::MinOf(card_set, point_map) => {
          T::MinOf(Box::new(card_set.lower(ctx)?), ctx.to_typed_id(point_map, GameType::PointMap))
        },
        spanned_ast::IntExpr::MaxOf(card_set, point_map) => {
          T::MaxOf(Box::new(card_set.lower(ctx)?), ctx.to_typed_id(point_map, GameType::PointMap))
        },
        spanned_ast::IntExpr::MinIntCollection(int_collection) => {
          T::MinIntCollection(int_collection.lower(ctx)?)
        },
        spanned_ast::IntExpr::MaxIntCollection(int_collection) => {
          T::MaxIntCollection(int_collection.lower(ctx)?)
        },
        spanned_ast::IntExpr::StageRoundCounter => T::StageRoundCounter,
      }  
    )
  }
}

impl Lower<typed_ast::Op> for SOp {
  type Error = TypeError;

  fn lower(&self, _: &LoweringCtx) -> Result<typed_ast::Op, Self::Error> {
    use typed_ast::Op as T;

    Ok(
      match &self.node {
        spanned_ast::Op::Plus  => T::Plus,
        spanned_ast::Op::Minus => T::Minus,
        spanned_ast::Op::Mul   => T::Mul,
        spanned_ast::Op::Div   => T::Div,
        spanned_ast::Op::Mod   => T::Mod,
      }
    )
  }
}

impl Lower<typed_ast::IntCmpOp> for SIntCmpOp {
  type Error = TypeError;
  
  fn lower(&self, _: &LoweringCtx) -> Result<typed_ast::IntCmpOp, Self::Error> {
    use typed_ast::IntCmpOp as T;

    Ok(
      match &self.node {
        spanned_ast::IntCmpOp::Eq  => T::Eq,
        spanned_ast::IntCmpOp::Neq => T::Neq,
        spanned_ast::IntCmpOp::Gt  => T::Gt,
        spanned_ast::IntCmpOp::Lt  => T::Lt,
        spanned_ast::IntCmpOp::Ge  => T::Ge,
        spanned_ast::IntCmpOp::Le  => T::Le,
      }
    )
  }
}

impl Lower<typed_ast::Collection> for SCollection {
  type Error = TypeError;

  fn lower(&self, ctx: &LoweringCtx) -> Result<typed_ast::Collection, Self::Error> {
    use typed_ast::Collection as T;

    Ok(
      match &self.node {
        spanned_ast::Collection::IntCollection(int_collection) => {
          T::IntCollection(int_collection.lower(ctx)?)
        },
        spanned_ast::Collection::StringCollection(string_collection) => {
          T::StringCollection(string_collection.lower(ctx)?)
        },
        spanned_ast::Collection::LocationCollection(location_collection) => {
          T::LocationCollection(location_collection.lower(ctx)?)
        },
        spanned_ast::Collection::PlayerCollection(player_collection) => {
          T::PlayerCollection(player_collection.lower(ctx)?)
        },
        spanned_ast::Collection::TeamCollection(team_collection) => {
          T::TeamCollection(team_collection.lower(ctx)?)
        },
        spanned_ast::Collection::CardSet(card_set) => {
          T::CardSet(Box::new(card_set.lower(ctx)?))
        },
        spanned_ast::Collection::Ambiguous(items) => {
          let collection = resolve_collection(ctx, items)?;
          return Ok(collection)
        },
      }
    )
  }
}

impl Lower<typed_ast::IntCollection> for SIntCollection {
  type Error = TypeError;

  fn lower(&self, ctx: &LoweringCtx) -> Result<typed_ast::IntCollection, Self::Error> {
    use typed_ast::IntCollection as T;

    let mut ints = Vec::new();
    for int in self.node.ints.iter() {
      ints.push(int.lower(ctx)?);
    }

    Ok(
      T {
        ints: ints
      }
    )
  }
}

impl Lower<typed_ast::StringCollection> for SStringCollection {
  type Error = TypeError;
  
  fn lower(&self, ctx: &LoweringCtx) -> Result<typed_ast::StringCollection, Self::Error> {
    use typed_ast::StringCollection as T;

    let mut strings = Vec::new();
    for str in self.node.strings.iter() {
      strings.push(str.lower(ctx)?);
    }


    Ok(
      T {
        strings: strings
      }
    )

  }
}

impl Lower<typed_ast::PlayerCollection> for SPlayerCollection {
  type Error = TypeError;

  fn lower(&self, ctx: &LoweringCtx) -> Result<typed_ast::PlayerCollection, Self::Error> {
    use typed_ast::PlayerCollection as T;
    
    Ok(  
      match &self.node {
        spanned_ast::PlayerCollection::Player(player_exprs) => {
          let mut players = Vec::new();
          for player in player_exprs.iter() {
            players.push(player.lower(ctx)?);
          }

          T::Player(players)
        },
        spanned_ast::PlayerCollection::Others => {
          T::Others
        },
        spanned_ast::PlayerCollection::Quantifier(quantifier) => {
          T::Quantifier(quantifier.lower(ctx)?)
        },
        spanned_ast::PlayerCollection::PlayersOut => {
          T::PlayersOut
        },
        spanned_ast::PlayerCollection::PlayersIn => {
          T::PlayersIn
        },
      }
    )
  }
}

impl Lower<typed_ast::Quantifier> for SQuantifier {
  type Error = TypeError;
  
  fn lower(&self, _: &LoweringCtx) -> Result<typed_ast::Quantifier, Self::Error> {
    use typed_ast::Quantifier as T;

    Ok(
      match &self.node {
        spanned_ast::Quantifier::All => {
          T::All
        },
        spanned_ast::Quantifier::Any => {
          T::Any
        },
      }
    ) 
  }
}

impl Lower<typed_ast::Quantity> for SQuantity {
  type Error = TypeError;

  fn lower(&self, ctx: &LoweringCtx) -> Result<typed_ast::Quantity, Self::Error> {
    use typed_ast::Quantity as T;
    
    Ok(
      match &self.node {
        spanned_ast::Quantity::Int(int_expr) => {
          T::Int(int_expr.lower(ctx)?)
        },
        spanned_ast::Quantity::Quantifier(quantifier) => {
          T::Quantifier(quantifier.lower(ctx)?)
        },
        spanned_ast::Quantity::IntRange(int_range) => {
          T::IntRange(int_range.lower(ctx)?)
        },
      }
    )
  }
}

impl Lower<typed_ast::IntRange> for SIntRange {
  type Error = TypeError;
  
  fn lower(&self, ctx: &LoweringCtx) -> Result<typed_ast::IntRange, Self::Error> {
    use typed_ast::IntRange as T;
    
    Ok(
      T {
        op: self.node.op.lower(ctx)?,
        int: self.node.int.lower(ctx)?
      }
    )
  }
}

impl Lower<typed_ast::LocationCollection> for SLocationCollection {
  type Error = TypeError;
  
  fn lower(&self, ctx: &LoweringCtx) -> Result<typed_ast::LocationCollection, Self::Error> {
    use typed_ast::LocationCollection as T;

    let mut locations = Vec::new();
    for location in self.node.locations.iter() {
      locations.push(ctx.to_typed_id(location, GameType::Location));
    }

    Ok(
      T {
        locations: locations
      }
    )
  }
}

impl Lower<typed_ast::TeamCollection> for STeamCollection {
  type Error = TypeError;

  fn lower(&self, ctx: &LoweringCtx) -> Result<typed_ast::TeamCollection, Self::Error> {
    use typed_ast::TeamCollection as T;

    Ok(
      match &self.node {
        spanned_ast::TeamCollection::Team(team_exprs) => {
          let mut teams = Vec::new();
          for team in team_exprs.iter() {
            teams.push(team.lower(ctx)?);
          }

          T::Team(teams)
        },
        spanned_ast::TeamCollection::OtherTeams => {
          T::OtherTeams
        },
      }
    ) 
  }
}

impl Lower<typed_ast::CardSet> for SCardSet {
  type Error = TypeError;

  fn lower(&self, ctx: &LoweringCtx) -> Result<typed_ast::CardSet, Self::Error> {
    use typed_ast::CardSet as T;
    
    Ok(
      match &self.node {
        spanned_ast::CardSet::Group(group) => {
          T::Group(group.lower(ctx)?)
        },
        spanned_ast::CardSet::GroupOfPlayer(group, player_expr) => {
          T::GroupOfPlayer(group.lower(ctx)?, player_expr.lower(ctx)?)
        },
        spanned_ast::CardSet::GroupOfPlayerCollection(group, player_collection) => {
          T::GroupOfPlayerCollection(group.lower(ctx)?, player_collection.lower(ctx)?)
        },
      }
    )
  }
}

impl Lower<typed_ast::Group> for SGroup {
  type Error = TypeError;

  fn lower(&self, ctx: &LoweringCtx) -> Result<typed_ast::Group, Self::Error> {
    use typed_ast::Group as T;

    Ok(
      match &self.node {
        spanned_ast::Group::Location(location) => {
          T::Location(ctx.to_typed_id(location, GameType::Location))
        },
        spanned_ast::Group::LocationWhere(location, filter_expr) => {
          T::LocationWhere(ctx.to_typed_id(location, GameType::Location), filter_expr.lower(ctx)?)
        },
        spanned_ast::Group::LocationCollection(location_collection) => {
          T::LocationCollection(location_collection.lower(ctx)?)
        },
        spanned_ast::Group::LocationCollectionWhere(location_collection, filter_expr) => {
          T::LocationCollectionWhere(location_collection.lower(ctx)?, filter_expr.lower(ctx)?)
        },
        spanned_ast::Group::ComboInLocation(combo, location) => {
          T::ComboInLocation(
            ctx.to_typed_id(combo, GameType::Combo),
            ctx.to_typed_id(location, GameType::Location)
          )
        },
        spanned_ast::Group::ComboInLocationCollection(combo, location_collection) => {
          T::ComboInLocationCollection(
            ctx.to_typed_id(combo, GameType::Combo),
            location_collection.lower(ctx)?
          )
        },
        spanned_ast::Group::NotComboInLocation(combo, location) => {
          T::NotComboInLocation(
            ctx.to_typed_id(combo, GameType::Combo),
            ctx.to_typed_id(location, GameType::Location)
          )
        },
        spanned_ast::Group::NotComboInLocationCollection(combo, location_collection) => {
          T::NotComboInLocationCollection(
            ctx.to_typed_id(combo, GameType::Combo),
            location_collection.lower(ctx)?
          )
        },
        spanned_ast::Group::CardPosition(card_position) => {
          T::CardPosition(card_position.lower(ctx)?)
        },
      }
    )
  }
}

impl Lower<typed_ast::FilterExpr> for SFilterExpr {
  type Error = TypeError;
  
  fn lower(&self, ctx: &LoweringCtx) -> Result<typed_ast::FilterExpr, Self::Error> {
    use typed_ast::FilterExpr as T;

    Ok(
      match &self.node {
        spanned_ast::FilterExpr::Same(key) => {
          T::Same(ctx.to_typed_id(key, GameType::Key))
        },
        spanned_ast::FilterExpr::Distinct(key) => {
          T::Distinct(ctx.to_typed_id(key, GameType::Key))
        },
        spanned_ast::FilterExpr::Adjacent(key, precedence) => {
          T::Adjacent(
            ctx.to_typed_id(key, GameType::Key),
            ctx.to_typed_id(precedence, GameType::Precedence),
          )
        },
        spanned_ast::FilterExpr::Higher(key, precedence) => {
          T::Higher(
            ctx.to_typed_id(key, GameType::Key),
            ctx.to_typed_id(precedence, GameType::Precedence),
          )
        },
        spanned_ast::FilterExpr::Lower(key, precedence) => {
          T::Lower(
            ctx.to_typed_id(key, GameType::Key),
            ctx.to_typed_id(precedence, GameType::Precedence),
          )
        },
        spanned_ast::FilterExpr::Size(int_cmp_op, int_expr) => {
          T::Size(
            int_cmp_op.lower(ctx)?,
            Box::new(int_expr.lower(ctx)?),
          )
        },
        spanned_ast::FilterExpr::KeyEqString(key, string_expr) => {
          T::KeyEqString(
            ctx.to_typed_id(key, GameType::Key),
            Box::new(string_expr.lower(ctx)?),
          )
        },
        spanned_ast::FilterExpr::KeyNeqString(key, string_expr) => {
          T::KeyNeqString(
            ctx.to_typed_id(key, GameType::Key),
            Box::new(string_expr.lower(ctx)?),
          )
        },
        spanned_ast::FilterExpr::KeyEqValue(key, value) => {
          T::KeyEqValue(
            ctx.to_typed_id(key, GameType::Key),
            ctx.to_typed_id(value, GameType::Value),
          )
        },
        spanned_ast::FilterExpr::KeyNeqValue(key, value) => {
          T::KeyNeqValue(
            ctx.to_typed_id(key, GameType::Key),
            ctx.to_typed_id(value, GameType::Value),
          )
        },
        spanned_ast::FilterExpr::NotCombo(combo) => {
          T::NotCombo(
            ctx.to_typed_id(combo, GameType::Combo),
          )
        },
        spanned_ast::FilterExpr::Combo(combo) => {
          T::Combo(
            ctx.to_typed_id(combo, GameType::Combo),
          )
        },
        spanned_ast::FilterExpr::And(filter_expr, filter_expr1) => {
          T::And(
            Box::new(filter_expr.lower(ctx)?),
            Box::new(filter_expr1.lower(ctx)?),
          )
        },
        spanned_ast::FilterExpr::Or(filter_expr, filter_expr1) => {
          T::Or(
            Box::new(filter_expr.lower(ctx)?),
            Box::new(filter_expr1.lower(ctx)?),
          )
        },
      }
    )
  }
}

impl Lower<typed_ast::CardPosition> for SCardPosition {
  type Error = TypeError;

  fn lower(&self, ctx: &LoweringCtx) -> Result<typed_ast::CardPosition, Self::Error> {
    use typed_ast::CardPosition as T;

    Ok(
      match &self.node {
        spanned_ast::CardPosition::At(location, int_expr) => {
          T::At(
            ctx.to_typed_id(location, GameType::Location), 
            int_expr.lower(ctx)?
          )
        },
        spanned_ast::CardPosition::Top(location) => {
          T::Top(ctx.to_typed_id(location, GameType::Location))
        },
        spanned_ast::CardPosition::Bottom(location) => {
          T::Bottom(ctx.to_typed_id(location, GameType::Location))
        },
        spanned_ast::CardPosition::Max(card_set, id) => {
          let ty = ctx.lookup(id)?;

          match ty {
            GameType::Precedence => {
              T::Max(Box::new(card_set.lower(ctx)?), ctx.to_typed_id(id, GameType::Precedence))
            },
            GameType::PointMap => {
              T::Max(Box::new(card_set.lower(ctx)?), ctx.to_typed_id(id, GameType::PointMap))
            },
            _ => {
              return Err(TypeError::PrecedenceOrPointMap { found: ty })
            },
          }
        },
        spanned_ast::CardPosition::Min(card_set, id) => {
          let ty = ctx.lookup(id)?;

          match ty {
            GameType::Precedence => {
              T::Min(Box::new(card_set.lower(ctx)?), ctx.to_typed_id(id, GameType::Precedence))
            },
            GameType::PointMap => {
              T::Min(Box::new(card_set.lower(ctx)?), ctx.to_typed_id(id, GameType::PointMap))
            },
            _ => {
              return Err(TypeError::PrecedenceOrPointMap { found: ty })
            },
          }
        },
      }
    )
  }
}

impl Lower<typed_ast::TeamExpr> for STeamExpr {
  type Error = TypeError;
  
  fn lower(&self, ctx: &LoweringCtx) -> Result<typed_ast::TeamExpr, Self::Error> {
    use typed_ast::TeamExpr as T;
    
    Ok(
      match &self.node {
        spanned_ast::TeamExpr::TeamName(team_name) => {
          T::TeamName(ctx.to_typed_id(team_name, GameType::Team))
        },
        spanned_ast::TeamExpr::TeamOf(player_expr) => {
          T::TeamOf(player_expr.lower(ctx)?)
        },
      }
    )
  }
}

impl Lower<typed_ast::StringExpr> for SStringExpr {
  type Error = TypeError;
  
  fn lower(&self, ctx: &LoweringCtx) -> Result<typed_ast::StringExpr, Self::Error> {
    use typed_ast::StringExpr as T;

    Ok(
      match &self.node {
        spanned_ast::StringExpr::KeyOf(key, card_position) => {
          T::KeyOf(
            ctx.to_typed_id(key, GameType::Key),
            card_position.lower(ctx)?
          )
        },
        spanned_ast::StringExpr::StringCollectionAt(string_collection, int_expr) => {
          T::StringCollectionAt(
            string_collection.lower(ctx)?,
            int_expr.lower(ctx)?
          )
        },
      }
    )
  }
}

impl Lower<typed_ast::BoolExpr> for SBoolExpr {
  type Error = TypeError;

  fn lower(&self, ctx: &LoweringCtx) -> Result<typed_ast::BoolExpr, Self::Error> {
    use typed_ast::BoolExpr as T;

    Ok(
      match &self.node {
        spanned_ast::BoolExpr::IntCmp(int_expr, int_cmp_op, int_expr1) => {
          T::IntCmp(
            int_expr.lower(ctx)?,
            int_cmp_op.lower(ctx)?,
            int_expr1.lower(ctx)?,
          )
        },
        spanned_ast::BoolExpr::CardSetIsEmpty(card_set) => {
          T::CardSetIsEmpty(
            card_set.lower(ctx)?
          )
        },
        spanned_ast::BoolExpr::CardSetIsNotEmpty(card_set) => {
          T::CardSetIsNotEmpty(
            card_set.lower(ctx)?
          )
        },
        spanned_ast::BoolExpr::CardSetEq(card_set, card_set1) => {
          T::CardSetEq(
            card_set.lower(ctx)?,
            card_set1.lower(ctx)?,
          )
        },
        spanned_ast::BoolExpr::CardSetNeq(card_set, card_set1) => {
          T::CardSetNeq(
            card_set.lower(ctx)?,
            card_set1.lower(ctx)?,
          )
        },
        spanned_ast::BoolExpr::StringEq(string_expr, string_expr1) => {
          T::StringEq(
            string_expr.lower(ctx)?,
            string_expr1.lower(ctx)?,
          )
        },
        spanned_ast::BoolExpr::StringNeq(string_expr, string_expr1) => {
          T::StringNeq(
            string_expr.lower(ctx)?,
            string_expr1.lower(ctx)?,
          )
        },
        spanned_ast::BoolExpr::PlayerEq(player_expr, player_expr1) => {
          T::PlayerEq(
            player_expr.lower(ctx)?,
            player_expr1.lower(ctx)?,
          )
        },
        spanned_ast::BoolExpr::PlayerNeq(player_expr, player_expr1) => {
          T::PlayerNeq(
            player_expr.lower(ctx)?,
            player_expr1.lower(ctx)?,
          )
        },
        spanned_ast::BoolExpr::TeamEq(team_expr, team_expr1) => {
          T::TeamEq(
            team_expr.lower(ctx)?,
            team_expr1.lower(ctx)?
          )
        },
        spanned_ast::BoolExpr::TeamNeq(team_expr, team_expr1) => {
          T::TeamNeq(
            team_expr.lower(ctx)?,
            team_expr1.lower(ctx)?
          )
        },
        spanned_ast::BoolExpr::And(bool_expr, bool_expr1) => {
          T::And(
            Box::new(bool_expr.lower(ctx)?),
            Box::new(bool_expr1.lower(ctx)?)
          )
        },
        spanned_ast::BoolExpr::Or(bool_expr, bool_expr1) => {
          T::Or(
            Box::new(bool_expr.lower(ctx)?),
            Box::new(bool_expr1.lower(ctx)?)
          )
        },
        spanned_ast::BoolExpr::Not(bool_expr) => {
          T::Not(Box::new(bool_expr.lower(ctx)?))
        },
        spanned_ast::BoolExpr::OutOfStagePlayer(player_expr) => {
          T::OutOfStagePlayer(player_expr.lower(ctx)?)
        },
        spanned_ast::BoolExpr::OutOfGamePlayer(player_expr) => {
          T::OutOfGamePlayer(player_expr.lower(ctx)?)
        },
        spanned_ast::BoolExpr::OutOfStageCollection(player_collection) => {
          T::OutOfStageCollection(player_collection.lower(ctx)?)
        },
        spanned_ast::BoolExpr::OutOfGameCollection(player_collection) => {
          T::OutOfGameCollection(player_collection.lower(ctx)?)
        },
      }
    )
  }
}

impl Lower<typed_ast::Rule> for SRule {
  type Error = TypeError;

  fn lower(&self, ctx: &LoweringCtx) -> Result<typed_ast::Rule, Self::Error> {
    use typed_ast::Rule as T;

    Ok(
      match &self.node {
        spanned_ast::Rule::CreatePlayer(items) => {
          let mut players = Vec::new();
          for player in items.iter() {
            players.push(ctx.to_typed_id(player, GameType::Player));
          }

          T::CreatePlayer(
            players
          )
        },
        spanned_ast::Rule::CreateTeam(team_name, items) => {
          let mut players = Vec::new();
          for player in items.iter() {
            players.push(ctx.to_typed_id(player, GameType::Player));
          }

          T::CreateTeam(
            ctx.to_typed_id(team_name, GameType::Team),
            players
          )
        },
        spanned_ast::Rule::CreateTurnorder(items) => {
          let mut players = Vec::new();
          for player in items.iter() {
            players.push(ctx.to_typed_id(player, GameType::Player));
          }

          T::CreateTurnorder(
            players
          )
        },
        spanned_ast::Rule::CreateTurnorderRandom(items) => {
          let mut players = Vec::new();
          for player in items.iter() {
            players.push(ctx.to_typed_id(player, GameType::Player));
          }
        
          T::CreateTurnorderRandom(
            players
          )
        },
        spanned_ast::Rule::CreateLocationOnPlayerCollection(location, player_collection) => {
          T::CreateLocationOnPlayerCollection(
            ctx.to_typed_id(location, GameType::Location),
            player_collection.lower(ctx)?
          )
        },
        spanned_ast::Rule::CreateLocationOnTeamCollection(location, team_collection) => {
          T::CreateLocationOnTeamCollection(
            ctx.to_typed_id(location, GameType::Location),
            team_collection.lower(ctx)?
          )
        },
        spanned_ast::Rule::CreateLocationOnTable(location) => {
          T::CreateLocationOnTable(
            ctx.to_typed_id(location, GameType::Location),
          )
        },
        spanned_ast::Rule::CreateLocationCollectionOnPlayerCollection(location_collection, player_collection) => {
          T::CreateLocationCollectionOnPlayerCollection(
            location_collection.lower(ctx)?,
            player_collection.lower(ctx)?
          )
        },
        spanned_ast::Rule::CreateLocationCollectionOnTeamCollection(location_collection, team_collection) => {
          T::CreateLocationCollectionOnTeamCollection(
            location_collection.lower(ctx)?,
            team_collection.lower(ctx)?
          )
        },
        spanned_ast::Rule::CreateLocationCollectionOnTable(location_collection) => {
          T::CreateLocationCollectionOnTable(location_collection.lower(ctx)?)
        },
        spanned_ast::Rule::CreateCardOnLocation(location, types) => {
          T::CreateCardOnLocation(
            ctx.to_typed_id(location, GameType::Location),
            types.lower(ctx)?
          )
        },
        spanned_ast::Rule::CreateTokenOnLocation(int_expr, token, location) => {
          T::CreateTokenOnLocation(
            int_expr.lower(ctx)?, ctx.to_typed_id(token, GameType::Token),
            ctx.to_typed_id(location, GameType::Location)
          )
        },
        spanned_ast::Rule::CreatePrecedence(precedence, items) => {
          let mut typed_items = Vec::new();
          for (k, v) in items.iter() {
            typed_items.push((ctx.to_typed_id(k, GameType::Key), ctx.to_typed_id(v, GameType::Value)))
          }

          T::CreatePrecedence(
            ctx.to_typed_id(precedence, GameType::Precedence), 
            typed_items
          )
        },
        spanned_ast::Rule::CreateCombo(combo, filter_expr) => {
          T::CreateCombo(ctx.to_typed_id(combo, GameType::Combo), filter_expr.lower(ctx)?)
        },
        spanned_ast::Rule::CreateMemoryIntPlayerCollection(memory, int_expr, player_collection) => {
          T::CreateMemoryIntPlayerCollection(
            ctx.to_typed_id(memory, GameType::Memory),
            int_expr.lower(ctx)?,
            player_collection.lower(ctx)?
          )
        },
        spanned_ast::Rule::CreateMemoryStringPlayerCollection(memory, string_expr, player_collection) => {
          T::CreateMemoryStringPlayerCollection(
            ctx.to_typed_id(memory, GameType::Memory),
            string_expr.lower(ctx)?,
            player_collection.lower(ctx)?
          )
        },
        spanned_ast::Rule::CreateMemoryIntTable(memory, int_expr) => {
          T::CreateMemoryIntTable(
            ctx.to_typed_id(memory, GameType::Memory),
            int_expr.lower(ctx)?
          )
        },
        spanned_ast::Rule::CreateMemoryStringTable(memory, string_expr) => {
          T::CreateMemoryStringTable(
            ctx.to_typed_id(memory, GameType::Memory),
            string_expr.lower(ctx)?
          )
        },
        spanned_ast::Rule::CreateMemoryPlayerCollection(memory, player_collection) => {
          T::CreateMemoryPlayerCollection(
            ctx.to_typed_id(memory, GameType::Memory),
            player_collection.lower(ctx)?
          )
        },
        spanned_ast::Rule::CreateMemoryTable(memory) => {
          T::CreateMemoryTable(
            ctx.to_typed_id(memory, GameType::Memory)
          )
        },
        spanned_ast::Rule::CreatePointMap(point_map, items) => {
          let mut typed_items = Vec::new();
          for (k, v, i) in items.iter() {
            typed_items.push((ctx.to_typed_id(k, GameType::Key), ctx.to_typed_id(v, GameType::Value), i.lower(ctx)?));
          }

          T::CreatePointMap(
            ctx.to_typed_id(point_map, GameType::PointMap),
            typed_items
          )
        },
        spanned_ast::Rule::FlipAction(card_set, status) => {
          T::FlipAction(
            card_set.lower(ctx)?,
            status.lower(ctx)?
          )
        },
        spanned_ast::Rule::ShuffleAction(card_set) => {
          T::ShuffleAction(card_set.lower(ctx)?)
        },
        spanned_ast::Rule::PlayerOutOfStageAction(player_expr) => {
          T::PlayerOutOfStageAction(player_expr.lower(ctx)?)
        },
        spanned_ast::Rule::PlayerOutOfGameSuccAction(player_expr) => {
          T::PlayerOutOfGameSuccAction(player_expr.lower(ctx)?)
        },
        spanned_ast::Rule::PlayerOutOfGameFailAction(player_expr) => {
          T::PlayerOutOfGameFailAction(player_expr.lower(ctx)?)
        },
        spanned_ast::Rule::PlayerCollectionOutOfStageAction(player_collection) => {
          T::PlayerCollectionOutOfStageAction(player_collection.lower(ctx)?)
        },
        spanned_ast::Rule::PlayerCollectionOutOfGameSuccAction(player_collection) => {
          T::PlayerCollectionOutOfGameSuccAction(player_collection.lower(ctx)?)
        },
        spanned_ast::Rule::PlayerCollectionOutOfGameFailAction(player_collection) => {
          T::PlayerCollectionOutOfGameFailAction(player_collection.lower(ctx)?)
        },
        spanned_ast::Rule::SetMemoryInt(memory, int_expr) => {
          T::SetMemoryInt(
            ctx.to_typed_id(memory, GameType::Memory),
            int_expr.lower(ctx)?
          )
        },
        spanned_ast::Rule::SetMemoryString(memory, string_expr) => {
          T::SetMemoryString(
            ctx.to_typed_id(memory, GameType::Memory),
            string_expr.lower(ctx)?
          )
        },
        spanned_ast::Rule::SetMemoryCollection(memory, collection) => {
          T::SetMemoryCollection(
            ctx.to_typed_id(memory, GameType::Memory),
            collection.lower(ctx)?
          )
        },
        spanned_ast::Rule::CycleAction(player_expr) => {
          T::CycleAction(player_expr.lower(ctx)?)
        },
        spanned_ast::Rule::BidAction(quantity) => {
          T::BidAction(quantity.lower(ctx)?)
        },
        spanned_ast::Rule::BidActionMemory(memory, quantity) => {
          T::BidActionMemory(ctx.to_typed_id(memory, GameType::Memory), quantity.lower(ctx)?)
        },
        spanned_ast::Rule::EndTurn => {
          T::EndTurn
        },
        spanned_ast::Rule::EndStage => {
          T::EndStage
        },
        spanned_ast::Rule::EndGameWithWinner(player_expr) => {
          T::EndGameWithWinner(player_expr.lower(ctx)?)
        },
        spanned_ast::Rule::DemandCardPositionAction(card_position) => {
          T::DemandCardPositionAction(card_position.lower(ctx)?)
        },
        spanned_ast::Rule::DemandStringAction(string_expr) => {
          T::DemandStringAction(string_expr.lower(ctx)?)
        },
        spanned_ast::Rule::DemandIntAction(int_expr) => {
          T::DemandIntAction(int_expr.lower(ctx)?)
        },
        spanned_ast::Rule::ClassicMove(classic_move) => {
          T::ClassicMove(classic_move.lower(ctx)?)
        },
        spanned_ast::Rule::DealMove(deal_move) => {
          T::DealMove(deal_move.lower(ctx)?)
        },
        spanned_ast::Rule::ExchangeMove(exchange_move) => {
          T::ExchangeMove(exchange_move.lower(ctx)?)
        },
        spanned_ast::Rule::TokenMove(token_move) => {
          T::TokenMove(token_move.lower(ctx)?)
        },
        spanned_ast::Rule::ScoreRule(score_rule) => {
          T::ScoreRule(score_rule.lower(ctx)?)
        },
        spanned_ast::Rule::WinnerRule(winner_rule) => {
          T::WinnerRule(winner_rule.lower(ctx)?)
        },
      }
    )
  }
}

impl Lower<typed_ast::ClassicMove> for SClassicMove {
  type Error = TypeError;
  
  fn lower(&self, ctx: &LoweringCtx) -> Result<typed_ast::ClassicMove, Self::Error> {
    use typed_ast::ClassicMove as T;

    Ok(
      match &self.node {
        spanned_ast::ClassicMove::Move(card_set, status, card_set1) => {
          T::Move(card_set.lower(ctx)?, status.lower(ctx)?, card_set1.lower(ctx)?)
        },
        spanned_ast::ClassicMove::MoveQuantity(quantity, card_set, status, card_set1) => {
          T::MoveQuantity(quantity.lower(ctx)?, card_set.lower(ctx)?, status.lower(ctx)?, card_set1.lower(ctx)?)
        },
      }
    )
  }
}

impl Lower<typed_ast::DealMove> for SDealMove {
  type Error = TypeError;
  
  fn lower(&self, ctx: &LoweringCtx) -> Result<typed_ast::DealMove, Self::Error> {
    use typed_ast::DealMove as T;

    Ok(
      match &self.node {
        spanned_ast::DealMove::Deal(card_set, status, card_set1) => {
          T::Deal(card_set.lower(ctx)?, status.lower(ctx)?, card_set1.lower(ctx)?)
        },
        spanned_ast::DealMove::DealQuantity(quantity, card_set, status, card_set1) => {
          T::DealQuantity(quantity.lower(ctx)?, card_set.lower(ctx)?, status.lower(ctx)?, card_set1.lower(ctx)?)
        },
      }
    )
  }
}

impl Lower<typed_ast::ExchangeMove> for SExchangeMove {
  type Error = TypeError;
  
  fn lower(&self, ctx: &LoweringCtx) -> Result<typed_ast::ExchangeMove, Self::Error> {
    use typed_ast::ExchangeMove as T;

    Ok(
      match &self.node {
        spanned_ast::ExchangeMove::Exchange(card_set, status, card_set1) => {
          T::Exchange(card_set.lower(ctx)?, status.lower(ctx)?, card_set1.lower(ctx)?)
        },
        spanned_ast::ExchangeMove::ExchangeQuantity(quantity, card_set, status, card_set1) => {
          T::ExchangeQuantity(quantity.lower(ctx)?, card_set.lower(ctx)?, status.lower(ctx)?, card_set1.lower(ctx)?)
        },
      }
    )
  }
}

impl Lower<typed_ast::TokenMove> for STokenMove {
  type Error = TypeError;
  
  fn lower(&self, ctx: &LoweringCtx) -> Result<typed_ast::TokenMove, Self::Error> {
    use typed_ast::TokenMove as T;

    Ok(
      match &self.node {
        spanned_ast::TokenMove::Place(token, token_loc_expr, token_loc_expr1) => {
          T::Place(
            ctx.to_typed_id(token, GameType::Token),
            token_loc_expr.lower(ctx)?,
            token_loc_expr1.lower(ctx)?
          )
        },
        spanned_ast::TokenMove::PlaceQuantity(quantity, token, token_loc_expr, token_loc_expr1) => {
          T::PlaceQuantity(
            quantity.lower(ctx)?,
            ctx.to_typed_id(token, GameType::Token),
            token_loc_expr.lower(ctx)?,
            token_loc_expr1.lower(ctx)?
          )
        },
      }
    ) 
  }
}

impl Lower<typed_ast::TokenLocExpr> for STokenLocExpr {
  type Error = TypeError;
  
  fn lower(&self, ctx: &LoweringCtx) -> Result<typed_ast::TokenLocExpr, Self::Error> {
    use typed_ast::TokenLocExpr as T;
    
    Ok(
      match &self.node {
        spanned_ast::TokenLocExpr::Location(location) => {
          T::Location(ctx.to_typed_id(location, GameType::Location))
        },
        spanned_ast::TokenLocExpr::LocationCollection(location_collection) => {
          T::LocationCollection(location_collection.lower(ctx)?)
        },
        spanned_ast::TokenLocExpr::LocationPlayer(location, player_expr) => {
          T::LocationPlayer(ctx.to_typed_id(location, GameType::Location), player_expr.lower(ctx)?)
        },
        spanned_ast::TokenLocExpr::LocationCollectionPlayer(location_collection, player_expr) => {
          T::LocationCollectionPlayer(location_collection.lower(ctx)?, player_expr.lower(ctx)?)
        },
        spanned_ast::TokenLocExpr::LocationPlayerCollection(location, player_collection) => {
          T::LocationPlayerCollection(ctx.to_typed_id(location, GameType::Location), player_collection.lower(ctx)?)
        },
        spanned_ast::TokenLocExpr::LocationCollectionPlayerCollection(location_collection, player_collection) => {
          T::LocationCollectionPlayerCollection(location_collection.lower(ctx)?, player_collection.lower(ctx)?)
        },
      }
    )
  }
}

impl Lower<typed_ast::ScoreRule> for SScoreRule {
  type Error = TypeError;
  
  fn lower(&self, ctx: &LoweringCtx) -> Result<typed_ast::ScoreRule, Self::Error> {
    use typed_ast::ScoreRule as T;

    Ok(
      match &self.node {
        spanned_ast::ScoreRule::ScorePlayer(int_expr, player_expr) => {
          T::ScorePlayer(int_expr.lower(ctx)?, player_expr.lower(ctx)?)
        },
        spanned_ast::ScoreRule::ScorePlayerMemory(int_expr, memory, player_expr) => {
          T::ScorePlayerMemory(
            int_expr.lower(ctx)?,
            ctx.to_typed_id(memory, GameType::Memory),
            player_expr.lower(ctx)?
          )
        },
        spanned_ast::ScoreRule::ScorePlayerCollection(int_expr, player_collection) => {
          T::ScorePlayerCollection(int_expr.lower(ctx)?, player_collection.lower(ctx)?)
        },
        spanned_ast::ScoreRule::ScorePlayerCollectionMemory(int_expr, memory, player_collection) => {
          T::ScorePlayerCollectionMemory(
            int_expr.lower(ctx)?,
            ctx.to_typed_id(memory, GameType::Memory),
            player_collection.lower(ctx)?
          )
        },
      }
    )
  }
}

impl Lower<typed_ast::WinnerRule> for SWinnerRule {
  type Error = TypeError;
  
  fn lower(&self, ctx: &LoweringCtx) -> Result<typed_ast::WinnerRule, Self::Error> {
    use typed_ast::WinnerRule as T;
      
    Ok(
      match &self.node {
        spanned_ast::WinnerRule::WinnerPlayer(player_expr) => {
          T::WinnerPlayer(player_expr.lower(ctx)?)
        },
        spanned_ast::WinnerRule::WinnerPlayerCollection(player_collection) => {
          T::WinnerPlayerCollection(player_collection.lower(ctx)?)
        },
        spanned_ast::WinnerRule::WinnerLowestScore => {
          T::WinnerHighestScore
        },
        spanned_ast::WinnerRule::WinnerHighestScore => {
          T::WinnerHighestScore
        },
        spanned_ast::WinnerRule::WinnerLowestMemory(memory) => {
          T::WinnerLowestMemory(ctx.to_typed_id(memory, GameType::Memory))
        },
        spanned_ast::WinnerRule::WinnerHighestMemory(memory) => {
          T::WinnerHighestMemory(ctx.to_typed_id(memory, GameType::Memory))
        },
        spanned_ast::WinnerRule::WinnerLowestPosition => {
          T::WinnerLowestPosition
        },
        spanned_ast::WinnerRule::WinnerHighestPosition => {
          T::WinnerHighestPosition
        },
          }
    )
  }
}

impl Lower<typed_ast::Status> for SStatus {
  type Error = TypeError;

  fn lower(&self, _: &LoweringCtx) -> Result<typed_ast::Status, Self::Error> {
    use typed_ast::Status as T;

    Ok(
      match &self.node {
        spanned_ast::Status::FaceUp => T::FaceUp,
        spanned_ast::Status::FaceDown => T::FaceDown,
        spanned_ast::Status::Private => T::Private,
      }
    )
  }
}

impl Lower<typed_ast::Types> for STypes {
  type Error = TypeError;
  
  fn lower(&self, _: &LoweringCtx) -> Result<typed_ast::Types, Self::Error> {
    use typed_ast::Types as T;

    // Only used when initialized!
    let types = self.node.types
      .iter()
      .map(
        |(k, vs)|
          {
            let key = TypedID::new(k.node.clone(), GameType::Key);
            let values = vs
              .iter()
              .map(
                |v|
                  TypedID::new(v.node.clone(), GameType::Value)
              ).collect::<Vec<TypedID>>();
            
            (key, values)
          }
      ).collect();

      Ok(
        T {
          types: types
        }
      )
  }
}

impl Lower<typed_ast::SeqStage> for SSeqStage {
  type Error = TypeError;
  
  fn lower(&self, ctx: &LoweringCtx) -> Result<typed_ast::SeqStage, Self::Error> {
    return Ok(
      typed_ast::SeqStage { 
        stage: TypedID::new(self.node.stage.node.clone(), GameType::Stage),
        player: self.node.player.lower(ctx)?,
        end_condition: self.node.end_condition.lower(ctx)?,
        flows: self.node.flows.iter().map(|f| f.lower(ctx)).collect::<Result<Vec<typed_ast::FlowComponent>, TypeError>>()?
      }
    )
  }
}


impl Lower<typed_ast::IfRule> for SIfRule {
  type Error = TypeError;
  
  fn lower(&self, ctx: &LoweringCtx) -> Result<typed_ast::IfRule, Self::Error> {
    return Ok(
      typed_ast::IfRule {
        condition: self.node.condition.lower(ctx)?,
        flows: self.node.flows.iter().map(|f| f.lower(ctx)).collect::<Result<Vec<typed_ast::FlowComponent>, TypeError>>()?
      }
    )
  }
}


impl Lower<typed_ast::ChoiceRule> for SChoiceRule {
  type Error = TypeError;
  
  fn lower(&self, ctx: &LoweringCtx) -> Result<typed_ast::ChoiceRule, Self::Error> {
    return Ok(
      typed_ast::ChoiceRule {
        options: self.node.options.iter().map(|f| f.lower(ctx)).collect::<Result<Vec<typed_ast::FlowComponent>, TypeError>>()?
      }
    )
  }
}


impl Lower<typed_ast::OptionalRule> for SOptionalRule {
  type Error = TypeError;
  
  fn lower(&self, ctx: &LoweringCtx) -> Result<typed_ast::OptionalRule, Self::Error> {
    return Ok(
      typed_ast::OptionalRule {
        flows: self.node.flows.iter().map(|f| f.lower(ctx)).collect::<Result<Vec<typed_ast::FlowComponent>, TypeError>>()?
      }
    )
  }
}


impl Lower<typed_ast::FlowComponent> for SFlowComponent {
  type Error = TypeError;

  fn lower(&self, ctx: &LoweringCtx) -> Result<typed_ast::FlowComponent, Self::Error> {   
    use typed_ast::FlowComponent as T;

    Ok(
      match &self.node {
        spanned_ast::FlowComponent::Stage(seq_stage) => {
          T::Stage(
            seq_stage.lower(ctx)?
          )
        },
        spanned_ast::FlowComponent::Rule(rule) => {
          T::Rule(rule.lower(ctx)?)
        },
        spanned_ast::FlowComponent::IfRule(if_rule) => {
          T::IfRule(
            if_rule.lower(ctx)?
          )
        },
        spanned_ast::FlowComponent::ChoiceRule(choice_rule) => {
          T::ChoiceRule(
            choice_rule.lower(ctx)?
          )
        },
        spanned_ast::FlowComponent::OptionalRule(optional_rule) => {
          T::OptionalRule(
            optional_rule.lower(ctx)?
          )
        },
      }
    )
  }
}

impl Lower<typed_ast::EndCondition> for SEndCondition {
  type Error = TypeError;
  
  fn lower(&self, ctx: &LoweringCtx) -> Result<typed_ast::EndCondition, Self::Error> {
    use typed_ast::EndCondition as T;
    
    Ok(
      match &self.node {
        spanned_ast::EndCondition::UntilBool(bool_expr) => {
          T::UntilBool(bool_expr.lower(ctx)?)
        },
        spanned_ast::EndCondition::UntilBoolAndRep(bool_expr, repititions) => {
          T::UntilBoolAndRep(bool_expr.lower(ctx)?, repititions.lower(ctx)?)
        },
        spanned_ast::EndCondition::UntilBoolOrRep(bool_expr, repititions) => {
          T::UntilBoolOrRep(bool_expr.lower(ctx)?, repititions.lower(ctx)?)
        },
        spanned_ast::EndCondition::UntilRep(repititions) => {
          T::UntilRep(repititions.lower(ctx)?)
        },
        spanned_ast::EndCondition::UntilEnd => {
          T::UntilEnd
        },
      }
    )
  }
}

impl Lower<typed_ast::Repititions> for SRepititions {
  type Error = TypeError;
  
  fn lower(&self, ctx: &LoweringCtx) -> Result<typed_ast::Repititions, Self::Error> {
    use typed_ast::Repititions as T;

    Ok(
      T {
        times: self.node.times.lower(ctx)?
      }
    )
  }
}

impl Lower<typed_ast::Game> for SGame {
  type Error = TypeError;
  
  fn lower(&self, ctx: &LoweringCtx) -> Result<typed_ast::Game, Self::Error> {
    use typed_ast::Game as T;

    Ok(
      T {
        flows: self.node.flows.iter().map(|f| f.lower(ctx)).collect::<Result<Vec<typed_ast::FlowComponent>, TypeError>>()?,
      }
    ) 
  }
}
