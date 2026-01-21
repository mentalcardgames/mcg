use std::collections::HashMap;

use crate::{{asts::game_type::GameType, analyzer::type_analyzer::TypedVars, asts::typed_ast::{self, IntExpr, PlayerExpr, StringExpr, TeamExpr, TypedID}}, asts::ast::{self}};

fn all_same(v: &Vec<GameType>) -> Result<GameType, TypeError> {
    match v.split_first() {
        None => Err(TypeError::EmptyCollection(v.clone())),
        Some((first, rest)) if rest.iter().all(|x| x == first) => {
            Ok(first.clone())
        }
        _ => Err(TypeError::AmbiguousTypes(v.clone())),
    }
}

#[derive(Debug)]
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
    let filtered_ctx = ctx.into_iter().filter(|(_, ty)| *ty != GameType::NoType).collect::<Vec<(String, GameType)>>();
    let symbols = filtered_ctx.into_iter().collect::<HashMap<String, GameType>>();

    LoweringCtx { symbols }
  }

  fn lookup(&self, id: &String) -> Result<GameType, TypeError> {
    if let Some(ty) = self.symbols.get(id) {
      return Ok(ty.clone())
    }

    Err(TypeError::SymbolNotFound(id.to_string()))
  }

  pub fn find(&self, id: &String, expected_ty: GameType) -> Result<TypedID, TypeError> {
    let ty = self.lookup(id)?;
    if ty != expected_ty {
      return Err(TypeError::WrongType {
          expected: expected_ty,
          found: ty.clone(),
      });
    }

    return Ok(TypedID::new(id.clone(), ty.clone()))
  } 
}

pub trait Lower<T> {
    type Error;

    fn lower(&self, ctx: &LoweringCtx) -> Result<T, Self::Error>;
}

impl Lower<typed_ast::PlayerExpr> for ast::PlayerExpr {
  type Error = TypeError;

  fn lower(&self, ctx: &LoweringCtx) -> Result<typed_ast::PlayerExpr, Self::Error> {
      use ast::PlayerExpr::*;
      use typed_ast::PlayerExpr as T;

      Ok(
        match self {
          PlayerName(player_name) => {
            T::PlayerName(ctx.find(player_name, GameType::Player)?)
          }
          Current => T::Current,
          Next => T::Next,
          Previous => T::Previous,
          Competitor => T::Competitor,
          Turnorder(int_expr) => T::Turnorder(int_expr.lower(ctx)?),
          OwnerOf(card_position) => T::OwnerOf(Box::new(card_position.lower(ctx)?)),
          OwnerOfHighest(memory) => {
            T::OwnerOfHighest(ctx.find(memory, GameType::Memory)?)
          },
          OwnerOfLowest(memory) => {
            T::OwnerOfLowest(ctx.find(memory, GameType::Memory)?)
          },
      }
    )
  }
}


impl Lower<typed_ast::IntExpr> for ast::IntExpr {
  type Error = TypeError;

  fn lower(&self, ctx: &LoweringCtx) -> Result<typed_ast::IntExpr, Self::Error> {
    use typed_ast::IntExpr as T;

    Ok(
      match self {
        ast::IntExpr::Int(int) => T::Int(*int),
        ast::IntExpr::IntOp(int_expr, op, int_expr1) => {
          T::IntOp(Box::new(int_expr.lower(ctx)?), op.lower(ctx)?, Box::new(int_expr1.lower(ctx)?))
        },
        ast::IntExpr::IntCollectionAt(int_expr) => {
          T::IntCollectionAt(Box::new(int_expr.lower(ctx)?))
        },
        ast::IntExpr::SizeOf(collection) => {
          T::SizeOf(collection.lower(ctx)?)
        },
        ast::IntExpr::SumOfIntCollection(int_collection) => {
          T::SumOfIntCollection(int_collection.lower(ctx)?)
        },
        ast::IntExpr::SumOfCardSet(card_set, point_map) => {
          T::SumOfCardSet(Box::new(card_set.lower(ctx)?), ctx.find(point_map, GameType::PointMap)?)
        },
        ast::IntExpr::MinOf(card_set, point_map) => {
          T::MinOf(Box::new(card_set.lower(ctx)?), ctx.find(point_map, GameType::PointMap)?)
        },
        ast::IntExpr::MaxOf(card_set, point_map) => {
          T::MaxOf(Box::new(card_set.lower(ctx)?), ctx.find(point_map, GameType::PointMap)?)
        },
        ast::IntExpr::MinIntCollection(int_collection) => {
          T::MinIntCollection(int_collection.lower(ctx)?)
        },
        ast::IntExpr::MaxIntCollection(int_collection) => {
          T::MaxIntCollection(int_collection.lower(ctx)?)
        },
        ast::IntExpr::StageRoundCounter => T::StageRoundCounter,
      }  
    )
  }
}

impl Lower<typed_ast::Op> for ast::Op {
  type Error = TypeError;

  fn lower(&self, _: &LoweringCtx) -> Result<typed_ast::Op, Self::Error> {
    use typed_ast::Op as T;

    Ok(
      match self {
        ast::Op::Plus  => T::Plus,
        ast::Op::Minus => T::Minus,
        ast::Op::Mul   => T::Mul,
        ast::Op::Div   => T::Div,
        ast::Op::Mod   => T::Mod,
      }
    )
  }
}

impl Lower<typed_ast::IntCmpOp> for ast::IntCmpOp {
  type Error = TypeError;
  
  fn lower(&self, _: &LoweringCtx) -> Result<typed_ast::IntCmpOp, Self::Error> {
    use typed_ast::IntCmpOp as T;

    Ok(
      match self {
        ast::IntCmpOp::Eq  => T::Eq,
        ast::IntCmpOp::Neq => T::Neq,
        ast::IntCmpOp::Gt  => T::Gt,
        ast::IntCmpOp::Lt  => T::Lt,
        ast::IntCmpOp::Ge  => T::Ge,
        ast::IntCmpOp::Le  => T::Le,
      }
    )
  }
}

impl Lower<typed_ast::Collection> for ast::Collection {
  type Error = TypeError;

  fn lower(&self, ctx: &LoweringCtx) -> Result<typed_ast::Collection, Self::Error> {
    use typed_ast::Collection as T;
    use typed_ast::PlayerCollection as PC;
    use typed_ast::PlayerExpr as PE;
    use typed_ast::TeamCollection as TC;
    use typed_ast::TeamExpr as TE;
    use typed_ast::LocationCollection as LC;

    Ok(
      match self {
        ast::Collection::IntCollection(int_collection) => {
          T::IntCollection(int_collection.lower(ctx)?)
        },
        ast::Collection::StringCollection(string_collection) => {
          T::StringCollection(string_collection.lower(ctx)?)
        },
        ast::Collection::LocationCollection(location_collection) => {
          T::LocationCollection(location_collection.lower(ctx)?)
        },
        ast::Collection::PlayerCollection(player_collection) => {
          T::PlayerCollection(player_collection.lower(ctx)?)
        },
        ast::Collection::TeamCollection(team_collection) => {
          T::TeamCollection(team_collection.lower(ctx)?)
        },
        ast::Collection::CardSet(card_set) => {
          T::CardSet(Box::new(card_set.lower(ctx)?))
        },
        ast::Collection::Ambiguous(items) => {
          let types = items
                                      .into_iter()
                                      .map(
                                        |t|
                                          ctx.lookup(t)  
                                      ).collect::<Result<Vec<GameType>, TypeError>>()?;
          let ty = all_same(&types)?;
          match ty {
            GameType::Player => {
              let ids = items
                .iter()
                .map(|id|
                  ctx.find(id, GameType::Player)
                )
                .collect::<Result<Vec<TypedID>, TypeError>>()?;

              let players = ids
                  .into_iter()
                  .map(
                    |id|
                      PE::PlayerName(id)
                  ).collect();
              T::PlayerCollection(
                PC::Player(
                  players
                )
              )
            },
            GameType::Team => {
              let ids = items
                .iter()
                .map(|id|
                  ctx.find(id, GameType::Player)
                )
                .collect::<Result<Vec<TypedID>, TypeError>>()?;

              T::TeamCollection(
                TC::Team(
                  ids
                  .iter()
                  .map(
                    |id|
                      TE::TeamName(id.clone())
                  ).collect()
                )
              )
            },
            GameType::Location => {
              let ids = items
                .iter()
                .map(|id|
                  ctx.find(id, GameType::Player)
                )
                .collect::<Result<Vec<TypedID>, TypeError>>()?;

              T::LocationCollection(
                LC {
                  locations: ids
                }
              )
            },
            _ => {
              return Err(
                TypeError::NoCollectionFound {
                  found: ty
                }
              )
            },
          }
        },
      }
    )
  }
}

impl Lower<typed_ast::IntCollection> for ast::IntCollection {
  type Error = TypeError;

  fn lower(&self, ctx: &LoweringCtx) -> Result<typed_ast::IntCollection, Self::Error> {
    use typed_ast::IntCollection as T;

    let ints = self.ints
          .iter()
          .map(
            |int_expr|
              int_expr.lower(ctx)
          ).collect::<Result<Vec<IntExpr>, TypeError>>()?; 

    Ok(
      T {
        ints: ints
      }
    )
  }
}

impl Lower<typed_ast::StringCollection> for ast::StringCollection {
  type Error = TypeError;
  
  fn lower(&self, ctx: &LoweringCtx) -> Result<typed_ast::StringCollection, Self::Error> {
    use typed_ast::StringCollection as T;

    let strings = self.strings
          .iter()
          .map(
            |string_expr|
              string_expr.lower(ctx)
          ).collect::<Result<Vec<StringExpr>, TypeError>>()?; 

    Ok(
      T {
        strings: strings
      }
    )

  }
}

impl Lower<typed_ast::PlayerCollection> for ast::PlayerCollection {
  type Error = TypeError;

  fn lower(&self, ctx: &LoweringCtx) -> Result<typed_ast::PlayerCollection, Self::Error> {
    use typed_ast::PlayerCollection as T;
    
    Ok(  
      match self {
        ast::PlayerCollection::Player(player_exprs) => {
          let players = player_exprs.iter().map(|p| p.lower(ctx)).collect::<Result<Vec<PlayerExpr>, TypeError>>()?;
          T::Player(players)
        },
        ast::PlayerCollection::Others => {
          T::Others
        },
        ast::PlayerCollection::Quantifier(quantifier) => {
          T::Quantifier(quantifier.lower(ctx)?)
        },
        ast::PlayerCollection::PlayersOut => {
          T::PlayersOut
        },
        ast::PlayerCollection::PlayersIn => {
          T::PlayersIn
        },
      }
    )
  }
}

impl Lower<typed_ast::Quantifier> for ast::Quantifier {
  type Error = TypeError;
  
  fn lower(&self, _: &LoweringCtx) -> Result<typed_ast::Quantifier, Self::Error> {
    use typed_ast::Quantifier as T;

    Ok(
      match self {
        ast::Quantifier::All => {
          T::All
        },
        ast::Quantifier::Any => {
          T::Any
        },
      }
    ) 
  }
}

impl Lower<typed_ast::Quantity> for ast::Quantity {
  type Error = TypeError;

  fn lower(&self, ctx: &LoweringCtx) -> Result<typed_ast::Quantity, Self::Error> {
    use typed_ast::Quantity as T;
    
    Ok(
      match self {
        ast::Quantity::Int(int_expr) => {
          T::Int(int_expr.lower(ctx)?)
        },
        ast::Quantity::Quantifier(quantifier) => {
          T::Quantifier(quantifier.lower(ctx)?)
        },
        ast::Quantity::IntRange(int_range) => {
          T::IntRange(int_range.lower(ctx)?)
        },
      }
    )
  }
}

impl Lower<typed_ast::IntRange> for ast::IntRange {
  type Error = TypeError;
  
  fn lower(&self, ctx: &LoweringCtx) -> Result<typed_ast::IntRange, Self::Error> {
    use typed_ast::IntRange as T;
    
    Ok(
      T {
        op: self.op.lower(ctx)?,
        int: self.int.lower(ctx)?
      }
    )
  }

  
}

impl Lower<typed_ast::LocationCollection> for ast::LocationCollection {
  type Error = TypeError;
  
  fn lower(&self, ctx: &LoweringCtx) -> Result<typed_ast::LocationCollection, Self::Error> {
    use typed_ast::LocationCollection as T;

    let locations = self.locations
      .iter()
      .map(
        |l|
          ctx.find(l, GameType::Location)
      ).collect::<Result<Vec<TypedID>, TypeError>>()?; 

    Ok(
      T {
        locations: locations
      }
    )
  }
}

impl Lower<typed_ast::TeamCollection> for ast::TeamCollection {
  type Error = TypeError;

  fn lower(&self, ctx: &LoweringCtx) -> Result<typed_ast::TeamCollection, Self::Error> {
    use typed_ast::TeamCollection as T;

    Ok(
      match self {
        ast::TeamCollection::Team(team_exprs) => {
          let teams = team_exprs
            .iter()
            .map(|t| t.lower(ctx))
            .collect::<Result<Vec<TeamExpr>, TypeError>>()?;
          T::Team(teams)
        },
        ast::TeamCollection::OtherTeams => {
          T::OtherTeams
        },
      }
    ) 
  }
}

impl Lower<typed_ast::CardSet> for ast::CardSet {
  type Error = TypeError;

  fn lower(&self, ctx: &LoweringCtx) -> Result<typed_ast::CardSet, Self::Error> {
    use typed_ast::CardSet as T;
    
    Ok(
      match self {
        ast::CardSet::Group(group) => {
          T::Group(group.lower(ctx)?)
        },
        ast::CardSet::GroupOfPlayer(group, player_expr) => {
          T::GroupOfPlayer(group.lower(ctx)?, player_expr.lower(ctx)?)
        },
        ast::CardSet::GroupOfPlayerCollection(group, player_collection) => {
          T::GroupOfPlayerCollection(group.lower(ctx)?, player_collection.lower(ctx)?)
        },
      }
    )
  }
}

impl Lower<typed_ast::Group> for ast::Group {
  type Error = TypeError;

  fn lower(&self, ctx: &LoweringCtx) -> Result<typed_ast::Group, Self::Error> {
    use typed_ast::Group as T;

    Ok(
      match self {
        ast::Group::Location(location) => {
          T::Location(ctx.find(location, GameType::Location)?)
        },
        ast::Group::LocationWhere(location, filter_expr) => {
          T::LocationWhere(ctx.find(location, GameType::Location)?, filter_expr.lower(ctx)?)
        },
        ast::Group::LocationCollection(location_collection) => {
          T::LocationCollection(location_collection.lower(ctx)?)
        },
        ast::Group::LocationCollectionWhere(location_collection, filter_expr) => {
          T::LocationCollectionWhere(location_collection.lower(ctx)?, filter_expr.lower(ctx)?)
        },
        ast::Group::ComboInLocation(combo, location) => {
          T::ComboInLocation(
            ctx.find(combo, GameType::Combo)?,
            ctx.find(location, GameType::Location)?
          )
        },
        ast::Group::ComboInLocationCollection(combo, location_collection) => {
          T::ComboInLocationCollection(
            ctx.find(combo, GameType::Combo)?,
            location_collection.lower(ctx)?
          )
        },
        ast::Group::NotComboInLocation(combo, location) => {
          T::NotComboInLocation(
            ctx.find(combo, GameType::Combo)?,
            ctx.find(location, GameType::Location)?
          )
        },
        ast::Group::NotComboInLocationCollection(combo, location_collection) => {
          T::NotComboInLocationCollection(
            ctx.find(combo, GameType::Combo)?,
            location_collection.lower(ctx)?
          )
        },
        ast::Group::CardPosition(card_position) => {
          T::CardPosition(card_position.lower(ctx)?)
        },
      }
    )
  }
}

impl Lower<typed_ast::FilterExpr> for ast::FilterExpr {
  type Error = TypeError;
  
  fn lower(&self, ctx: &LoweringCtx) -> Result<typed_ast::FilterExpr, Self::Error> {
    use typed_ast::FilterExpr as T;

    Ok(
      match self {
        ast::FilterExpr::Same(key) => {
          T::Same(ctx.find(key, GameType::Key)?)
        },
        ast::FilterExpr::Distinct(key) => {
          T::Distinct(ctx.find(key, GameType::Key)?)
        },
        ast::FilterExpr::Adjacent(key, precedence) => {
          T::Adjacent(
            ctx.find(key, GameType::Key)?,
            ctx.find(precedence, GameType::Precedence)?,
          )
        },
        ast::FilterExpr::Higher(key, precedence) => {
          T::Higher(
            ctx.find(key, GameType::Key)?,
            ctx.find(precedence, GameType::Precedence)?,
          )
        },
        ast::FilterExpr::Lower(key, precedence) => {
          T::Lower(
            ctx.find(key, GameType::Key)?,
            ctx.find(precedence, GameType::Precedence)?,
          )
        },
        ast::FilterExpr::Size(int_cmp_op, int_expr) => {
          T::Size(
            int_cmp_op.lower(ctx)?,
            Box::new(int_expr.lower(ctx)?),
          )
        },
        ast::FilterExpr::KeyEqString(key, string_expr) => {
          T::KeyEqString(
            ctx.find(key, GameType::Key)?,
            Box::new(string_expr.lower(ctx)?),
          )
        },
        ast::FilterExpr::KeyNeqString(key, string_expr) => {
          T::KeyNeqString(
            ctx.find(key, GameType::Key)?,
            Box::new(string_expr.lower(ctx)?),
          )
        },
        ast::FilterExpr::KeyEqValue(key, value) => {
          T::KeyEqValue(
            ctx.find(key, GameType::Key)?,
            ctx.find(value, GameType::Value)?,
          )
        },
        ast::FilterExpr::KeyNeqValue(key, value) => {
          T::KeyNeqValue(
            ctx.find(key, GameType::Key)?,
            ctx.find(value, GameType::Value)?,
          )
        },
        ast::FilterExpr::NotCombo(combo) => {
          T::NotCombo(
            ctx.find(combo, GameType::Combo)?,
          )
        },
        ast::FilterExpr::Combo(combo) => {
          T::NotCombo(
            ctx.find(combo, GameType::Combo)?,
          )
        },
        ast::FilterExpr::And(filter_expr, filter_expr1) => {
          T::And(
            Box::new(filter_expr.lower(ctx)?),
            Box::new(filter_expr1.lower(ctx)?),
          )
        },
        ast::FilterExpr::Or(filter_expr, filter_expr1) => {
          T::Or(
            Box::new(filter_expr.lower(ctx)?),
            Box::new(filter_expr1.lower(ctx)?),
          )
        },
      }
    )
  }
}

impl Lower<typed_ast::CardPosition> for ast::CardPosition {
  type Error = TypeError;

  fn lower(&self, ctx: &LoweringCtx) -> Result<typed_ast::CardPosition, Self::Error> {
    use typed_ast::CardPosition as T;

    Ok(
      match self {
        ast::CardPosition::At(location, int_expr) => {
          T::At(
            ctx.find(location, GameType::Location)?, 
            int_expr.lower(ctx)?
          )
        },
        ast::CardPosition::Top(location) => {
          T::Top(ctx.find(location, GameType::Location)?)
        },
        ast::CardPosition::Bottom(location) => {
          T::Bottom(ctx.find(location, GameType::Location)?)
        },
        ast::CardPosition::Max(card_set, id) => {
          let ty = ctx.lookup(id)?;

          match ty {
            GameType::Precedence => {
              T::MaxPrecedence(Box::new(card_set.lower(ctx)?), ctx.find(id, GameType::Precedence)?)
            },
            GameType::PointMap => {
              T::MaxPointMap(Box::new(card_set.lower(ctx)?), ctx.find(id, GameType::PointMap)?)
            },
            _ => {
              return Err(TypeError::PrecedenceOrPointMap { found: ty })
            },
          }
        },
        ast::CardPosition::Min(card_set, id) => {
          let ty = ctx.lookup(id)?;

          match ty {
            GameType::Precedence => {
              T::MinPrecedence(Box::new(card_set.lower(ctx)?), ctx.find(id, GameType::Precedence)?)
            },
            GameType::PointMap => {
              T::MinPointMap(Box::new(card_set.lower(ctx)?), ctx.find(id, GameType::PointMap)?)
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

impl Lower<typed_ast::TeamExpr> for ast::TeamExpr {
  type Error = TypeError;
  
  fn lower(&self, ctx: &LoweringCtx) -> Result<typed_ast::TeamExpr, Self::Error> {
    use typed_ast::TeamExpr as T;
    
    Ok(
      match self {
        ast::TeamExpr::TeamName(team_name) => {
          T::TeamName(ctx.find(team_name, GameType::Team)?)
        },
        ast::TeamExpr::TeamOf(player_expr) => {
          T::TeamOf(player_expr.lower(ctx)?)
        },
      }
    )
  }
}

impl Lower<typed_ast::StringExpr> for ast::StringExpr {
  type Error = TypeError;
  
  fn lower(&self, ctx: &LoweringCtx) -> Result<typed_ast::StringExpr, Self::Error> {
    use typed_ast::StringExpr as T;

    Ok(
      match self {
        ast::StringExpr::KeyOf(key, card_position) => {
          T::KeyOf(
            ctx.find(key, GameType::Key)?,
            card_position.lower(ctx)?
          )
        },
        ast::StringExpr::StringCollectionAt(string_collection, int_expr) => {
          T::StringCollectionAt(
            string_collection.lower(ctx)?,
            int_expr.lower(ctx)?
          )
        },
      }
    )
  }
}

impl Lower<typed_ast::BoolExpr> for ast::BoolExpr {
  type Error = TypeError;

  fn lower(&self, ctx: &LoweringCtx) -> Result<typed_ast::BoolExpr, Self::Error> {
    use typed_ast::BoolExpr as T;

    Ok(
      match self {
        ast::BoolExpr::IntCmp(int_expr, int_cmp_op, int_expr1) => {
          T::IntCmp(
            int_expr.lower(ctx)?,
            int_cmp_op.lower(ctx)?,
            int_expr1.lower(ctx)?,
          )
        },
        ast::BoolExpr::CardSetIsEmpty(card_set) => {
          T::CardSetIsEmpty(
            card_set.lower(ctx)?
          )
        },
        ast::BoolExpr::CardSetIsNotEmpty(card_set) => {
          T::CardSetIsNotEmpty(
            card_set.lower(ctx)?
          )
        },
        ast::BoolExpr::CardSetEq(card_set, card_set1) => {
          T::CardSetEq(
            card_set.lower(ctx)?,
            card_set1.lower(ctx)?,
          )
        },
        ast::BoolExpr::CardSetNeq(card_set, card_set1) => {
          T::CardSetNeq(
            card_set.lower(ctx)?,
            card_set1.lower(ctx)?,
          )
        },
        ast::BoolExpr::StringEq(string_expr, string_expr1) => {
          T::StringEq(
            string_expr.lower(ctx)?,
            string_expr1.lower(ctx)?,
          )
        },
        ast::BoolExpr::StringNeq(string_expr, string_expr1) => {
          T::StringNeq(
            string_expr.lower(ctx)?,
            string_expr1.lower(ctx)?,
          )
        },
        ast::BoolExpr::PlayerEq(player_expr, player_expr1) => {
          T::PlayerEq(
            player_expr.lower(ctx)?,
            player_expr1.lower(ctx)?,
          )
        },
        ast::BoolExpr::PlayerNeq(player_expr, player_expr1) => {
          T::PlayerNeq(
            player_expr.lower(ctx)?,
            player_expr1.lower(ctx)?,
          )
        },
        ast::BoolExpr::TeamEq(team_expr, team_expr1) => {
          T::TeamEq(
            team_expr.lower(ctx)?,
            team_expr1.lower(ctx)?
          )
        },
        ast::BoolExpr::TeamNeq(team_expr, team_expr1) => {
          T::TeamNeq(
            team_expr.lower(ctx)?,
            team_expr1.lower(ctx)?
          )
        },
        ast::BoolExpr::And(bool_expr, bool_expr1) => {
          T::And(
            Box::new(bool_expr.lower(ctx)?),
            Box::new(bool_expr1.lower(ctx)?)
          )
        },
        ast::BoolExpr::Or(bool_expr, bool_expr1) => {
          T::Or(
            Box::new(bool_expr.lower(ctx)?),
            Box::new(bool_expr1.lower(ctx)?)
          )
        },
        ast::BoolExpr::Not(bool_expr) => {
          T::Not(Box::new(bool_expr.lower(ctx)?))
        },
        ast::BoolExpr::OutOfStagePlayer(player_expr) => {
          T::OutOfStagePlayer(player_expr.lower(ctx)?)
        },
        ast::BoolExpr::OutOfGamePlayer(player_expr) => {
          T::OutOfGamePlayer(player_expr.lower(ctx)?)
        },
        ast::BoolExpr::OutOfStageCollection(player_collection) => {
          T::OutOfStageCollection(player_collection.lower(ctx)?)
        },
        ast::BoolExpr::OutOfGameCollection(player_collection) => {
          T::OutOfGameCollection(player_collection.lower(ctx)?)
        },
        ast::BoolExpr::AmbiguousEq(id, id1) => {
          let ty = ctx.lookup(id)?;
          let ty1 = ctx.lookup(id1)?;

          if ty != ty1 {
            return Err(TypeError::NotMatchingTypes { first: ty, second: ty1 })
          }

          match ty {
            GameType::Player => {
              T::PlayerEq(
                typed_ast::PlayerExpr::PlayerName(ctx.find(id, GameType::Player)?),
                typed_ast::PlayerExpr::PlayerName(ctx.find(id1, GameType::Player)?)
              )
            },
            GameType::Team => {
              T::TeamEq(
                typed_ast::TeamExpr::TeamName(ctx.find(id, GameType::Team)?),
                typed_ast::TeamExpr::TeamName(ctx.find(id1, GameType::Team)?)
              )
            },
            GameType::Location => {
              T::CardSetEq(
                typed_ast::CardSet::Group(typed_ast::Group::Location(ctx.find(id, GameType::Location)?)),
                typed_ast::CardSet::Group(typed_ast::Group::Location(ctx.find(id1, GameType::Location)?)),
              )
            },
            _ => {
              return Err(TypeError::NoBoolExpr { first: ty, second: ty1 })
            },
          }
        },
        ast::BoolExpr::AmbiguousNeq(id, id1) => {
          let ty = ctx.lookup(id)?;
          let ty1 = ctx.lookup(id1)?;

          if ty != ty1 {
            return Err(TypeError::NotMatchingTypes { first: ty, second: ty1 })
          }

          match ty {
            GameType::Player => {
              T::PlayerNeq(
                typed_ast::PlayerExpr::PlayerName(ctx.find(id, GameType::Player)?),
                typed_ast::PlayerExpr::PlayerName(ctx.find(id1, GameType::Player)?)
              )
            },
            GameType::Team => {
              T::TeamNeq(
                typed_ast::TeamExpr::TeamName(ctx.find(id, GameType::Team)?),
                typed_ast::TeamExpr::TeamName(ctx.find(id1, GameType::Team)?)
              )
            },
            GameType::Location => {
              T::CardSetNeq(
                typed_ast::CardSet::Group(typed_ast::Group::Location(ctx.find(id, GameType::Location)?)),
                typed_ast::CardSet::Group(typed_ast::Group::Location(ctx.find(id1, GameType::Location)?)),
              )
            },
            _ => {
              return Err(TypeError::NoBoolExpr { first: ty, second: ty1 })
            },
          }
        },
      }
    )
  }
}

impl Lower<typed_ast::Rule> for ast::Rule {
  type Error = TypeError;

  fn lower(&self, ctx: &LoweringCtx) -> Result<typed_ast::Rule, Self::Error> {
    use typed_ast::Rule as T;

    Ok(
      match self {
        ast::Rule::CreatePlayer(items) => {
          T::CreatePlayer(
            items
              .iter()
              .map(|p| ctx.find(p, GameType::Player))
              .collect::<Result<Vec<TypedID>, TypeError>>()?
          )
        },
        ast::Rule::CreateTeam(team_name, items) => {
          T::CreateTeam(
            ctx.find(team_name, GameType::Team)?,
            items
              .iter()
              .map(|p| ctx.find(p, GameType::Player))
              .collect::<Result<Vec<TypedID>, TypeError>>()?
          )
        },
        ast::Rule::CreateTurnorder(items) => {
          T::CreateTurnorder(
            items
              .iter()
              .map(|p| ctx.find(p, GameType::Player))
              .collect::<Result<Vec<TypedID>, TypeError>>()?
          )
        },
        ast::Rule::CreateTurnorderRandom(items) => {
          T::CreateTurnorderRandom(
            items
              .iter()
              .map(|p| ctx.find(p, GameType::Player))
              .collect::<Result<Vec<TypedID>, TypeError>>()?
          )
        },
        ast::Rule::CreateLocationOnPlayerCollection(location, player_collection) => {
          T::CreateLocationOnPlayerCollection(
            ctx.find(location, GameType::Location)?,
            player_collection.lower(ctx)?
          )
        },
        ast::Rule::CreateLocationOnTeamCollection(location, team_collection) => {
          T::CreateLocationOnTeamCollection(
            ctx.find(location, GameType::Location)?,
            team_collection.lower(ctx)?
          )
        },
        ast::Rule::CreateLocationOnTable(location) => {
          T::CreateLocationOnTable(
            ctx.find(location, GameType::Location)?,
          )
        },
        ast::Rule::CreateLocationCollectionOnPlayerCollection(location_collection, player_collection) => {
          T::CreateLocationCollectionOnPlayerCollection(
            location_collection.lower(ctx)?,
            player_collection.lower(ctx)?
          )
        },
        ast::Rule::CreateLocationCollectionOnTeamCollection(location_collection, team_collection) => {
          T::CreateLocationCollectionOnTeamCollection(
            location_collection.lower(ctx)?,
            team_collection.lower(ctx)?
          )
        },
        ast::Rule::CreateLocationCollectionOnTable(location_collection) => {
          T::CreateLocationCollectionOnTable(location_collection.lower(ctx)?)
        },
        ast::Rule::CreateCardOnLocation(location, types) => {
          T::CreateCardOnLocation(
            ctx.find(location, GameType::Location)?,
            types.lower(ctx)?
          )
        },
        ast::Rule::CreateTokenOnLocation(int_expr, token, location) => {
          T::CreateTokenOnLocation(
            int_expr.lower(ctx)?, ctx.find(token, GameType::Token)?,
            ctx.find(location, GameType::Location)?
          )
        },
        ast::Rule::CreatePrecedence(precedence, items) => {
          let mut typed_items = Vec::new();
          for (k, v) in items.iter() {
            typed_items.push((ctx.find(k, GameType::Key)?, ctx.find(v, GameType::Value)?))
          }

          T::CreatePrecedence(
            ctx.find(precedence, GameType::Precedence)?, 
            typed_items
          )
        },
        ast::Rule::CreateCombo(combo, filter_expr) => {
          T::CreateCombo(ctx.find(combo, GameType::Combo)?, filter_expr.lower(ctx)?)
        },
        ast::Rule::CreateMemoryIntPlayerCollection(memory, int_expr, player_collection) => {
          T::CreateMemoryIntPlayerCollection(
            ctx.find(memory, GameType::Memory)?,
            int_expr.lower(ctx)?,
            player_collection.lower(ctx)?
          )
        },
        ast::Rule::CreateMemoryStringPlayerCollection(memory, string_expr, player_collection) => {
          T::CreateMemoryStringPlayerCollection(
            ctx.find(memory, GameType::Memory)?,
            string_expr.lower(ctx)?,
            player_collection.lower(ctx)?
          )
        },
        ast::Rule::CreateMemoryIntTable(memory, int_expr) => {
          T::CreateMemoryIntTable(
            ctx.find(memory, GameType::Memory)?,
            int_expr.lower(ctx)?
          )
        },
        ast::Rule::CreateMemoryStringTable(memory, string_expr) => {
          T::CreateMemoryStringTable(
            ctx.find(memory, GameType::Memory)?,
            string_expr.lower(ctx)?
          )
        },
        ast::Rule::CreateMemoryPlayerCollection(memory, player_collection) => {
          T::CreateMemoryPlayerCollection(
            ctx.find(memory, GameType::Memory)?,
            player_collection.lower(ctx)?
          )
        },
        ast::Rule::CreateMemoryTable(memory) => {
          T::CreateMemoryTable(
            ctx.find(memory, GameType::Memory)?
          )
        },
        ast::Rule::CreatePointMap(point_map, items) => {
          let mut typed_items = Vec::new();
          for (k, v, i) in items.iter() {
            typed_items.push((ctx.find(k, GameType::Key)?, ctx.find(v, GameType::Value)?, i.lower(ctx)?));
          }

          T::CreatePointMap(
            ctx.find(point_map, GameType::PointMap)?,
            typed_items
          )
        },
        ast::Rule::FlipAction(card_set, status) => {
          T::FlipAction(
            card_set.lower(ctx)?,
            status.lower(ctx)?
          )
        },
        ast::Rule::ShuffleAction(card_set) => {
          T::ShuffleAction(card_set.lower(ctx)?)
        },
        ast::Rule::PlayerOutOfStageAction(player_expr) => {
          T::PlayerOutOfStageAction(player_expr.lower(ctx)?)
        },
        ast::Rule::PlayerOutOfGameSuccAction(player_expr) => {
          T::PlayerOutOfGameSuccAction(player_expr.lower(ctx)?)
        },
        ast::Rule::PlayerOutOfGameFailAction(player_expr) => {
          T::PlayerOutOfGameFailAction(player_expr.lower(ctx)?)
        },
        ast::Rule::PlayerCollectionOutOfStageAction(player_collection) => {
          T::PlayerCollectionOutOfStageAction(player_collection.lower(ctx)?)
        },
        ast::Rule::PlayerCollectionOutOfGameSuccAction(player_collection) => {
          T::PlayerCollectionOutOfGameSuccAction(player_collection.lower(ctx)?)
        },
        ast::Rule::PlayerCollectionOutOfGameFailAction(player_collection) => {
          T::PlayerCollectionOutOfGameFailAction(player_collection.lower(ctx)?)
        },
        ast::Rule::SetMemoryInt(memory, int_expr) => {
          T::SetMemoryInt(
            ctx.find(memory, GameType::Memory)?,
            int_expr.lower(ctx)?
          )
        },
        ast::Rule::SetMemoryString(memory, string_expr) => {
          T::SetMemoryString(
            ctx.find(memory, GameType::Memory)?,
            string_expr.lower(ctx)?
          )
        },
        ast::Rule::SetMemoryCollection(memory, collection) => {
          T::SetMemoryCollection(
            ctx.find(memory, GameType::Memory)?,
            collection.lower(ctx)?
          )
        },
        ast::Rule::CycleAction(player_expr) => {
          T::CycleAction(player_expr.lower(ctx)?)
        },
        ast::Rule::BidAction(quantity) => {
          T::BidAction(quantity.lower(ctx)?)
        },
        ast::Rule::BidActionMemory(memory, quantity) => {
          T::BidActionMemory(ctx.find(memory, GameType::Memory)?, quantity.lower(ctx)?)
        },
        ast::Rule::EndTurn => {
          T::EndTurn
        },
        ast::Rule::EndStage => {
          T::EndStage
        },
        ast::Rule::EndGameWithWinner(player_expr) => {
          T::EndGameWithWinner(player_expr.lower(ctx)?)
        },
        ast::Rule::DemandCardPositionAction(card_position) => {
          T::DemandCardPositionAction(card_position.lower(ctx)?)
        },
        ast::Rule::DemandStringAction(string_expr) => {
          T::DemandStringAction(string_expr.lower(ctx)?)
        },
        ast::Rule::DemandIntAction(int_expr) => {
          T::DemandIntAction(int_expr.lower(ctx)?)
        },
        ast::Rule::ClassicMove(classic_move) => {
          T::ClassicMove(classic_move.lower(ctx)?)
        },
        ast::Rule::DealMove(deal_move) => {
          T::DealMove(deal_move.lower(ctx)?)
        },
        ast::Rule::ExchangeMove(exchange_move) => {
          T::ExchangeMove(exchange_move.lower(ctx)?)
        },
        ast::Rule::TokenMove(token_move) => {
          T::TokenMove(token_move.lower(ctx)?)
        },
        ast::Rule::ScoreRule(score_rule) => {
          T::ScoreRule(score_rule.lower(ctx)?)
        },
        ast::Rule::WinnerRule(winner_rule) => {
          T::WinnerRule(winner_rule.lower(ctx)?)
        },
      }
    )
  }
}

impl Lower<typed_ast::ClassicMove> for ast::ClassicMove {
  type Error = TypeError;
  
  fn lower(&self, ctx: &LoweringCtx) -> Result<typed_ast::ClassicMove, Self::Error> {
    use typed_ast::ClassicMove as T;

    Ok(
      match self {
        ast::ClassicMove::Move(card_set, status, card_set1) => {
          T::Move(card_set.lower(ctx)?, status.lower(ctx)?, card_set1.lower(ctx)?)
        },
        ast::ClassicMove::MoveQuantity(quantity, card_set, status, card_set1) => {
          T::MoveQuantity(quantity.lower(ctx)?, card_set.lower(ctx)?, status.lower(ctx)?, card_set1.lower(ctx)?)
        },
      }
    )
  }
}

impl Lower<typed_ast::DealMove> for ast::DealMove {
  type Error = TypeError;
  
  fn lower(&self, ctx: &LoweringCtx) -> Result<typed_ast::DealMove, Self::Error> {
    use typed_ast::DealMove as T;

    Ok(
      match self {
        ast::DealMove::Deal(card_set, status, card_set1) => {
          T::Deal(card_set.lower(ctx)?, status.lower(ctx)?, card_set1.lower(ctx)?)
        },
        ast::DealMove::DealQuantity(quantity, card_set, status, card_set1) => {
          T::DealQuantity(quantity.lower(ctx)?, card_set.lower(ctx)?, status.lower(ctx)?, card_set1.lower(ctx)?)
        },
      }
    )
  }
}

impl Lower<typed_ast::ExchangeMove> for ast::ExchangeMove {
  type Error = TypeError;
  
  fn lower(&self, ctx: &LoweringCtx) -> Result<typed_ast::ExchangeMove, Self::Error> {
    use typed_ast::ExchangeMove as T;

    Ok(
      match self {
        ast::ExchangeMove::Exchange(card_set, status, card_set1) => {
          T::Exchange(card_set.lower(ctx)?, status.lower(ctx)?, card_set1.lower(ctx)?)
        },
        ast::ExchangeMove::ExchangeQuantity(quantity, card_set, status, card_set1) => {
          T::ExchangeQuantity(quantity.lower(ctx)?, card_set.lower(ctx)?, status.lower(ctx)?, card_set1.lower(ctx)?)
        },
      }
    )
  }
}

impl Lower<typed_ast::TokenMove> for ast::TokenMove {
  type Error = TypeError;
  
  fn lower(&self, ctx: &LoweringCtx) -> Result<typed_ast::TokenMove, Self::Error> {
    use typed_ast::TokenMove as T;

    Ok(
      match self {
        ast::TokenMove::Place(token, token_loc_expr, token_loc_expr1) => {
          T::Place(
            ctx.find(token, GameType::Token)?,
            token_loc_expr.lower(ctx)?,
            token_loc_expr1.lower(ctx)?
          )
        },
        ast::TokenMove::PlaceQuantity(quantity, token, token_loc_expr, token_loc_expr1) => {
          T::PlaceQuantity(
            quantity.lower(ctx)?,
            ctx.find(token, GameType::Token)?,
            token_loc_expr.lower(ctx)?,
            token_loc_expr1.lower(ctx)?
          )
        },
      }
    ) 
  }
}

impl Lower<typed_ast::TokenLocExpr> for ast::TokenLocExpr {
  type Error = TypeError;
  
  fn lower(&self, ctx: &LoweringCtx) -> Result<typed_ast::TokenLocExpr, Self::Error> {
    use typed_ast::TokenLocExpr as T;
    
    Ok(
      match self {
        ast::TokenLocExpr::Location(location) => {
          T::Location(ctx.find(location, GameType::Location)?)
        },
        ast::TokenLocExpr::LocationCollection(location_collection) => {
          T::LocationCollection(location_collection.lower(ctx)?)
        },
        ast::TokenLocExpr::LocationPlayer(location, player_expr) => {
          T::LocationPlayer(ctx.find(location, GameType::Location)?, player_expr.lower(ctx)?)
        },
        ast::TokenLocExpr::LocationCollectionPlayer(location_collection, player_expr) => {
          T::LocationCollectionPlayer(location_collection.lower(ctx)?, player_expr.lower(ctx)?)
        },
        ast::TokenLocExpr::LocationPlayerCollection(location, player_collection) => {
          T::LocationPlayerCollection(ctx.find(location, GameType::Location)?, player_collection.lower(ctx)?)
        },
        ast::TokenLocExpr::LocationCollectionPlayerCollection(location_collection, player_collection) => {
          T::LocationCollectionPlayerCollection(location_collection.lower(ctx)?, player_collection.lower(ctx)?)
        },
      }
    )
  }
}

impl Lower<typed_ast::ScoreRule> for ast::ScoreRule {
  type Error = TypeError;
  
  fn lower(&self, ctx: &LoweringCtx) -> Result<typed_ast::ScoreRule, Self::Error> {
    use typed_ast::ScoreRule as T;

    Ok(
      match self {
        ast::ScoreRule::ScorePlayer(int_expr, player_expr) => {
          T::ScorePlayer(int_expr.lower(ctx)?, player_expr.lower(ctx)?)
        },
        ast::ScoreRule::ScorePlayerMemory(int_expr, memory, player_expr) => {
          T::ScorePlayerMemory(
            int_expr.lower(ctx)?,
            ctx.find(memory, GameType::Memory)?,
            player_expr.lower(ctx)?
          )
        },
        ast::ScoreRule::ScorePlayerCollection(int_expr, player_collection) => {
          T::ScorePlayerCollection(int_expr.lower(ctx)?, player_collection.lower(ctx)?)
        },
        ast::ScoreRule::ScorePlayerCollectionMemory(int_expr, memory, player_collection) => {
          T::ScorePlayerCollectionMemory(
            int_expr.lower(ctx)?,
            ctx.find(memory, GameType::Memory)?,
            player_collection.lower(ctx)?
          )
        },
      }
    )
  }
}

impl Lower<typed_ast::WinnerRule> for ast::WinnerRule {
  type Error = TypeError;
  
  fn lower(&self, ctx: &LoweringCtx) -> Result<typed_ast::WinnerRule, Self::Error> {
    use typed_ast::WinnerRule as T;
      
    Ok(
      match self {
        ast::WinnerRule::WinnerPlayer(player_expr) => {
          T::WinnerPlayer(player_expr.lower(ctx)?)
        },
        ast::WinnerRule::WinnerPlayerCollection(player_collection) => {
          T::WinnerPlayerCollection(player_collection.lower(ctx)?)
        },
        ast::WinnerRule::WinnerLowestScore => {
          T::WinnerHighestScore
        },
        ast::WinnerRule::WinnerHighestScore => {
          T::WinnerHighestScore
        },
        ast::WinnerRule::WinnerLowestMemory(memory) => {
          T::WinnerLowestMemory(ctx.find(memory, GameType::Memory)?)
        },
        ast::WinnerRule::WinnerHighestMemory(memory) => {
          T::WinnerHighestMemory(ctx.find(memory, GameType::Memory)?)
        },
        ast::WinnerRule::WinnerLowestPosition => {
          T::WinnerLowestPosition
        },
        ast::WinnerRule::WinnerHighestPosition => {
          T::WinnerHighestPosition
        },
          }
    )
  }
}

impl Lower<typed_ast::Status> for ast::Status {
  type Error = TypeError;

  fn lower(&self, _: &LoweringCtx) -> Result<typed_ast::Status, Self::Error> {
    use typed_ast::Status as T;

    Ok(
      match self {
        ast::Status::FaceUp => T::FaceUp,
        ast::Status::FaceDown => T::FaceDown,
        ast::Status::Private => T::Private,
      }
    )
  }
}

impl Lower<typed_ast::Types> for ast::Types {
  type Error = TypeError;
  
  fn lower(&self, _: &LoweringCtx) -> Result<typed_ast::Types, Self::Error> {
    use typed_ast::Types as T;

    // Only used when initialized!
    let types = self.types
      .iter()
      .map(
        |(k, vs)|
          {
            let key = TypedID::new(k.clone(), GameType::Key);
            let values = vs
              .iter()
              .map(
                |v|
                  TypedID::new(v.clone(), GameType::Value)
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

impl Lower<typed_ast::FlowComponent> for ast::FlowComponent {
  type Error = TypeError;

  fn lower(&self, ctx: &LoweringCtx) -> Result<typed_ast::FlowComponent, Self::Error> {   
    use typed_ast::FlowComponent as T;

    Ok(
      match self {
        ast::FlowComponent::Stage(seq_stage) => {
          T::Stage(
            typed_ast::SeqStage { 
              stage: TypedID::new(seq_stage.stage.clone(), GameType::Stage),
              player: seq_stage.player.lower(ctx)?,
              end_condition: seq_stage.end_condition.lower(ctx)?,
              flows: seq_stage.flows.iter().map(|f| f.lower(ctx)).collect::<Result<Vec<typed_ast::FlowComponent>, TypeError>>()?
            }
          )
        },
        ast::FlowComponent::Rule(rule) => {
          T::Rule(rule.lower(ctx)?)
        },
        ast::FlowComponent::IfRule(if_rule) => {
          T::IfRule(
            typed_ast::IfRule {
              condition: if_rule.condition.lower(ctx)?,
              flows: if_rule.flows.iter().map(|f| f.lower(ctx)).collect::<Result<Vec<typed_ast::FlowComponent>, TypeError>>()?
            }
          )
        },
        ast::FlowComponent::ChoiceRule(choice_rule) => {
          T::ChoiceRule(
            typed_ast::ChoiceRule {
              options: choice_rule.options.iter().map(|f| f.lower(ctx)).collect::<Result<Vec<typed_ast::FlowComponent>, TypeError>>()?
            }
          )
        },
        ast::FlowComponent::OptionalRule(optional_rule) => {
          T::OptionalRule(
            typed_ast::OptionalRule {
              flows: optional_rule.flows.iter().map(|f| f.lower(ctx)).collect::<Result<Vec<typed_ast::FlowComponent>, TypeError>>()?
            }
          )
        },
      }
    )
  }
}

impl Lower<typed_ast::EndCondition> for ast::EndCondition {
  type Error = TypeError;
  
  fn lower(&self, ctx: &LoweringCtx) -> Result<typed_ast::EndCondition, Self::Error> {
    use typed_ast::EndCondition as T;
    
    Ok(
      match self {
        ast::EndCondition::UntilBool(bool_expr) => {
          T::UntilBool(bool_expr.lower(ctx)?)
        },
        ast::EndCondition::UntilBoolAndRep(bool_expr, repititions) => {
          T::UntilBoolAndRep(bool_expr.lower(ctx)?, repititions.lower(ctx)?)
        },
        ast::EndCondition::UntilBoolOrRep(bool_expr, repititions) => {
          T::UntilBoolOrRep(bool_expr.lower(ctx)?, repititions.lower(ctx)?)
        },
        ast::EndCondition::UntilRep(repititions) => {
          T::UntilRep(repititions.lower(ctx)?)
        },
        ast::EndCondition::UntilEnd => {
          T::UntilEnd
        },
      }
    )
  }
}

impl Lower<typed_ast::Repititions> for ast::Repititions {
  type Error = TypeError;
  
  fn lower(&self, ctx: &LoweringCtx) -> Result<typed_ast::Repititions, Self::Error> {
    use typed_ast::Repititions as T;

    Ok(
      T {
        times: self.times.lower(ctx)?
      }
    )
  }
}

impl Lower<typed_ast::Game> for ast::Game {
  type Error = TypeError;
  
  fn lower(&self, ctx: &LoweringCtx) -> Result<typed_ast::Game, Self::Error> {
    use typed_ast::Game as T;

    Ok(
      T {
        flows: self.flows.iter().map(|f| f.lower(ctx)).collect::<Result<Vec<typed_ast::FlowComponent>, TypeError>>()?,
      }
    ) 
  }
}
