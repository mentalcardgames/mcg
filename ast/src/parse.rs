use crate::{ast::*};
use crate::keywords::kw as kw;
use crate::analyzer::Analyzer;

use syn::parse::discouraged::Speculative;
use syn::parse::{Parse, ParseStream, Result};
use syn::punctuated::Punctuated;
use syn::{Ident, LitInt, Token, braced, bracketed, parenthesized};


// ------------------------
// Parsing implementations
// ------------------------

// Core of the Parser
// ===========================================================================
fn parse_with_alternatives<T>(input: ParseStream, alts: &[fn(ParseStream) -> Result<T>]) -> Result<T> {
    for alt in alts {
        let fork = input.fork();
        if let Ok(result) = alt(&fork) {
            input.advance_to(&fork);
            return Ok(result);
        }
    }
    Err(input.error("no alternative matched"))
}

// ===========================================================================

// IDs
// ===========================================================================
impl Parse for ID {
  fn parse(input: ParseStream) -> Result<Self> {
      let fork = input.fork();
      let id = fork.parse::<Ident>()?;

      // check correct "shape" of ID
      match Analyzer::check_id(&id) {
        Ok(_) => {},
        Err(err) => {
          return Err(input.error(&err))
        }
      }

      input.advance_to(&fork);

      return Ok(ID::new(id))
  }
}

impl Parse for Stage {
  fn parse(input: ParseStream) -> Result<Self> {
      let id = input.parse::<ID>()?;

      return Ok(Stage::new(id))
  }
}

impl Parse for PlayerName {
  fn parse(input: ParseStream) -> Result<Self> {
      let id = input.parse::<ID>()?;

      return Ok(PlayerName::new(id))
  }
}

impl Parse for TeamName {
  fn parse(input: ParseStream) -> Result<Self> {
      let id = input.parse::<ID>()?;

      return Ok(TeamName::new(id))
  }
}

impl Parse for Location {
  fn parse(input: ParseStream) -> Result<Self> {
      let id = input.parse::<ID>()?;

      return Ok(Location::new(id))
  }
}

impl Parse for Token {
  fn parse(input: ParseStream) -> Result<Self> {
      let id = input.parse::<ID>()?;

      return Ok(Token::new(id))
  }
}

impl Parse for Precedence {
  fn parse(input: ParseStream) -> Result<Self> {
      let id = input.parse::<ID>()?;

      return Ok(Precedence::new(id))
  }
}

impl Parse for PointMap {
  fn parse(input: ParseStream) -> Result<Self> {
      let id = input.parse::<ID>()?;

      return Ok(PointMap::new(id))
  }
}

impl Parse for Combo {
  fn parse(input: ParseStream) -> Result<Self> {
      let id = input.parse::<ID>()?;

      return Ok(Combo::new(id))
  }
}

impl Parse for Memory {
  fn parse(input: ParseStream) -> Result<Self> {
      let id = input.parse::<ID>()?;

      return Ok(Memory::new(id))
  }
}

impl Parse for Key {
  fn parse(input: ParseStream) -> Result<Self> {
      let id = input.parse::<ID>()?;

      return Ok(Key::new(id))
  }
}

impl Parse for Value {
  fn parse(input: ParseStream) -> Result<Self> {
      let id = input.parse::<ID>()?;

      return Ok(Value::new(id))
  }
}

// ===========================================================================


// Op
// ===========================================================================
impl Parse for Op {
  fn parse(input: ParseStream) -> Result<Self> {
      parse_with_alternatives(input, 
        &[
          parse_op_plus,
          parse_op_minus,
          parse_op_div,
          parse_op_mul,
          parse_op_mod,
        ]
      )
  }
}

fn parse_op_plus(input: ParseStream) -> Result<Op> {
  input.parse::<Token![+]>()?;
  
  return Ok(Op::Plus)
}

fn parse_op_minus(input: ParseStream) -> Result<Op> {
  input.parse::<Token![-]>()?;
  
  return Ok(Op::Minus)
}

fn parse_op_div(input: ParseStream) -> Result<Op> {
  input.parse::<Token![/]>()?;
  
  return Ok(Op::Div)
}

fn parse_op_mul(input: ParseStream) -> Result<Op> {
  input.parse::<Token![*]>()?;
  
  return Ok(Op::Mul)
}

fn parse_op_mod(input: ParseStream) -> Result<Op> {
  input.parse::<Token![%]>()?;
  
  return Ok(Op::Mod)
}

// ===========================================================================


// IntCmpOp
// ===========================================================================
impl Parse for IntCmpOp {
  fn parse(input: ParseStream) -> Result<Self> {
      parse_with_alternatives(input, 
        &[
          parse_intcmpop_eq,
          parse_intcmpop_neq,
          parse_intcmpop_le,
          parse_intcmpop_ge,
          parse_intcmpop_lt,
          parse_intcmpop_gt,
        ]
      )
  }
}

fn parse_intcmpop_eq(input: ParseStream) -> Result<IntCmpOp> {
  input.parse::<Token![==]>()?;
  
  return Ok(IntCmpOp::Eq)
}

fn parse_intcmpop_neq(input: ParseStream) -> Result<IntCmpOp> {
  input.parse::<Token![!=]>()?;
  
  return Ok(IntCmpOp::Neq)
}

fn parse_intcmpop_le(input: ParseStream) -> Result<IntCmpOp> {
  input.parse::<Token![<=]>()?;
  
  return Ok(IntCmpOp::Le)
}

fn parse_intcmpop_ge(input: ParseStream) -> Result<IntCmpOp> {
  input.parse::<Token![>=]>()?;
  
  return Ok(IntCmpOp::Ge)
}

fn parse_intcmpop_lt(input: ParseStream) -> Result<IntCmpOp> {
  input.parse::<Token![<]>()?;
  
  return Ok(IntCmpOp::Lt)
}

fn parse_intcmpop_gt(input: ParseStream) -> Result<IntCmpOp> {
  input.parse::<Token![>]>()?;
  
  return Ok(IntCmpOp::Gt)
}

// ===========================================================================


// Status
// ===========================================================================
impl Parse for Status {
  fn parse(input: ParseStream) -> Result<Self> {
      parse_with_alternatives(input, 
        &[
          parse_status_face_down,
          parse_status_face_up,
          parse_status_private,
        ]
      )
  }
}

fn parse_status_face_down(input: ParseStream) -> Result<Status> {
  input.parse::<kw::face>()?;
  input.parse::<kw::down>()?;

  return Ok(Status::FaceDown)
}

fn parse_status_face_up(input: ParseStream) -> Result<Status> {
  input.parse::<kw::face>()?;
  input.parse::<kw::up>()?;

  return Ok(Status::FaceUp)
}

fn parse_status_private(input: ParseStream) -> Result<Status> {
  input.parse::<kw::private>()?;

  return Ok(Status::Private)
}

// ===========================================================================


// Quantifier
// ===========================================================================
impl Parse for Quantifier {
  fn parse(input: ParseStream) -> Result<Self> {
    parse_with_alternatives(input, 
      &[
        parse_quantifier_all,
        parse_quantifier_any,
      ]
    )
  }
}
fn parse_quantifier_all(input: ParseStream) -> Result<Quantifier> {
  input.parse::<kw::all>()?;

  return Ok(Quantifier::All)
}

fn parse_quantifier_any(input: ParseStream) -> Result<Quantifier> {
  input.parse::<kw::any>()?;

  return Ok(Quantifier::Any)
}

// ===========================================================================


// PlayerExpr
// ===========================================================================
impl Parse for PlayerExpr {
  fn parse(input: ParseStream) -> Result<Self> {
      if input.peek(kw::others) {

        return Err(input.error("PlayerExpr can not be KeyWord 'others'!"))
      }

      parse_with_alternatives(input, 
        &[
          parse_player_current,
          parse_player_previous,
          parse_player_next,
          parse_player_competitor,
          parse_player_owner_of_highest,
          parse_player_owner_of_lowest,
          parse_player_at_turnorder,
          parse_player_owner_of_cardposition,
          parse_player_playername,
        ]
      )
      
  }
}

fn parse_player_current(input: ParseStream) -> Result<PlayerExpr> {
  input.parse::<kw::current>()?;
  
  return Ok(PlayerExpr::Current)
}

fn parse_player_previous(input: ParseStream) -> Result<PlayerExpr> {
  input.parse::<kw::previous>()?;
  
  return Ok(PlayerExpr::Previous)
}

fn parse_player_next(input: ParseStream) -> Result<PlayerExpr> {
  input.parse::<kw::next>()?;

  return Ok(PlayerExpr::Next)
}

fn parse_player_competitor(input: ParseStream) -> Result<PlayerExpr> {
  input.parse::<kw::competitor>()?;

  return Ok(PlayerExpr::Competitor)
}

fn parse_player_owner_of_highest(input: ParseStream) -> Result<PlayerExpr> {
  input.parse::<kw::owner>()?;
  input.parse::<kw::of>()?;
  input.parse::<kw::highest>()?;
  let memory = input.parse::<Memory>()?;

  return Ok(PlayerExpr::OwnerOfHighest(memory))
}

fn parse_player_owner_of_lowest(input: ParseStream) -> Result<PlayerExpr> {
  input.parse::<kw::owner>()?;
  input.parse::<kw::of>()?;
  input.parse::<kw::lowest>()?;
  let memory = input.parse::<Memory>()?;

  return Ok(PlayerExpr::OwnerOfLowest(memory))
}

fn parse_player_at_turnorder(input: ParseStream) -> Result<PlayerExpr> {
  input.parse::<kw::turnorder>()?;
  let content;
  parenthesized!(content in input);
  let int = content.parse::<IntExpr>()?;

  return Ok(PlayerExpr::Turnorder(int))
}

fn parse_player_owner_of_cardposition(input: ParseStream) -> Result<PlayerExpr> {
  input.parse::<kw::owner>()?;
  input.parse::<kw::of>()?;
  let cardpos = input.parse::<CardPosition>()?;

  return Ok(PlayerExpr::OwnerOf(Box::new(cardpos)))
}

fn parse_player_playername(input: ParseStream) -> Result<PlayerExpr> {
  let playername = input.parse::<PlayerName>()?;
      
  return Ok(PlayerExpr::PlayerName(playername))
}

// ===========================================================================


// TeamExpr
// ===========================================================================
impl Parse for TeamExpr {
  fn parse(input: ParseStream) -> Result<Self> {
      parse_with_alternatives(input, 
        &[
          parse_team_team_of,
          parse_team_teamname,
        ]
      )
  }
}

fn parse_team_team_of(input: ParseStream) -> Result<TeamExpr> {
  input.parse::<kw::team>()?;
  input.parse::<kw::of>()?;
  let player = input.parse::<PlayerExpr>()?;

  return Ok(TeamExpr::TeamOf(player))
}

fn parse_team_teamname(input: ParseStream) -> Result<TeamExpr> {
  let teamname = input.parse::<TeamName>()?;
      
  return Ok(TeamExpr::TeamName(teamname))
}

// ===========================================================================


// CardPosition
// ===========================================================================
impl Parse for CardPosition {
  fn parse(input: ParseStream) -> Result<Self> {
      parse_with_alternatives(input, 
        &[
          parse_cardposition_top,
          parse_cardposition_bottom,
          parse_cardposition_max,
          parse_cardposition_min,
          parse_cardposition_at,
        ]
      )
  }
}

fn parse_cardposition_top(input: ParseStream) -> Result<CardPosition> {
  input.parse::<kw::top>()?;
  let content;
  parenthesized!(content in input);
  let location = content.parse::<Location>()?;

  return Ok(CardPosition::Top(location))
}

fn parse_cardposition_bottom(input: ParseStream) -> Result<CardPosition> {
  input.parse::<kw::bottom>()?;
  let content;
  parenthesized!(content in input);
  let location = content.parse::<Location>()?;

  return Ok(CardPosition::Bottom(location))
}

fn parse_cardposition_max(input: ParseStream) -> Result<CardPosition> {
  input.parse::<kw::max>()?;        
  let content;
  parenthesized!(content in input);
  let cardset = content.parse::<CardSet>()?;
  input.parse::<kw::using>()?;
  let id = input.parse::<ID>()?;

  return Ok(CardPosition::Max(Box::new(cardset), id))
}

fn parse_cardposition_min(input: ParseStream) -> Result<CardPosition> {
  input.parse::<kw::min>()?;        
  let content;
  parenthesized!(content in input);
  let cardset = content.parse::<CardSet>()?;
  input.parse::<kw::using>()?;
  let id = input.parse::<ID>()?;

  return Ok(CardPosition::Min(Box::new(cardset), id))
}

fn parse_cardposition_at(input: ParseStream) -> Result<CardPosition> {
  let location = input.parse::<Location>()?;
  let content;
  bracketed!(content in input);
  let int = content.parse::<IntExpr>()?;

  return Ok(CardPosition::At(location, int))
}

// ===========================================================================


// IntExpr
// ===========================================================================
impl Parse for IntExpr {
  fn parse(input: ParseStream) -> Result<Self> {
      parse_with_alternatives(input, 
        &[
          parse_int_op,
          parse_int_size_of,
          parse_int_sum_of,
          parse_int_sum_int_collection,
          parse_int_min_of_cardset,
          parse_int_max_of_cardset,
          parse_int_min_of_int_collection,
          parse_int_max_of_int_collection,
          parse_int_stageroundcounter,
          parse_int_int,
        ]
      )
      
  }
}

fn parse_int_op(input: ParseStream) -> Result<IntExpr> {
  let content;
  parenthesized!(content in input);
  let left = content.parse::<IntExpr>()?;
  let op = content.parse::<Op>()?;
  let right = content.parse::<IntExpr>()?;

  return Ok(IntExpr::IntOp(Box::new(left), op, Box::new(right)))
}

fn parse_int_size_of(input: ParseStream) -> Result<IntExpr> {
  input.parse::<kw::size>()?;
  input.parse::<kw::of>()?;
  let collection = input.parse::<Collection>()?;

  return Ok(IntExpr::SizeOf(collection))
}

fn parse_int_sum_of(input: ParseStream) -> Result<IntExpr> {
  input.parse::<kw::sum>()?;
  input.parse::<kw::of>()?;
  let cardset = input.parse::<CardSet>()?;
  input.parse::<kw::using>()?;
  let pointmap = input.parse::<PointMap>()?;

  return Ok(IntExpr::SumOfCardSet(Box::new(cardset), pointmap))
}

fn parse_int_sum_int_collection(input: ParseStream) -> Result<IntExpr> {
  input.parse::<kw::sum>()?;
  let intcollection = input.parse::<IntCollection>()?;
  
  return Ok(IntExpr::SumOfIntCollection(intcollection))
}

fn parse_int_min_of_cardset(input: ParseStream) -> Result<IntExpr> {
  input.parse::<kw::min>()?;
  input.parse::<kw::of>()?;
  let cardset = input.parse::<CardSet>()?;
  input.parse::<kw::using>()?;
  let pointmap = input.parse::<PointMap>()?;

  return Ok(IntExpr::MinOf(Box::new(cardset), pointmap))
}

fn parse_int_max_of_cardset(input: ParseStream) -> Result<IntExpr> {
  input.parse::<kw::max>()?;
  input.parse::<kw::of>()?;
  let cardset = input.parse::<CardSet>()?;
  input.parse::<kw::using>()?;
  let pointmap = input.parse::<PointMap>()?;

  return Ok(IntExpr::MaxOf(Box::new(cardset), pointmap))
}

fn parse_int_min_of_int_collection(input: ParseStream) -> Result<IntExpr> {
  input.parse::<kw::min>()?;
  let intcollection = input.parse::<IntCollection>()?;

  return Ok(IntExpr::MinIntCollection(intcollection))
}

fn parse_int_max_of_int_collection(input: ParseStream) -> Result<IntExpr> {
  input.parse::<kw::max>()?;
  let intcollection = input.parse::<IntCollection>()?;

  return Ok(IntExpr::MaxIntCollection(intcollection))
}

fn parse_int_stageroundcounter(input: ParseStream) -> Result<IntExpr> {
  input.parse::<kw::stageroundcounter>()?;

  return Ok(IntExpr::StageRoundCounter)
}

fn parse_int_int(input: ParseStream) -> Result<IntExpr> {
  let int: i32 = (input.parse::<LitInt>()?).base10_parse()?;
      
  return Ok(IntExpr::Int(int))
}

// ===========================================================================


// BoolExpr
// ===========================================================================
impl Parse for BoolExpr {
  fn parse(input: ParseStream) -> Result<Self> {
        parse_with_alternatives(input, &[
            parse_bool_and,
            parse_bool_or,
            parse_bool_not,
            parse_bool_out_of_stage_player,
            parse_bool_out_of_game_player,
            parse_bool_out_of_stage_player_collection,
            parse_bool_out_of_game_player_collection,
            parse_bool_cardset_empty,
            parse_bool_cardset_not_empty,
            parse_bool_int,
            parse_bool_cardset_eq,
            parse_bool_cardset_neq,
            parse_bool_player_eq,
            parse_bool_player_neq,
            parse_bool_team_eq,
            parse_bool_team_neq,
            parse_bool_string_neq,
            parse_bool_string_eq,
            // Fall back to the Analyzer because of possible
            // ambiguous parsing.
            parse_bool_id_eq,
            parse_bool_id_neq,
        ])
    }
}


fn parse_bool_not(input: ParseStream) -> Result<BoolExpr> {
    input.parse::<kw::not>()?;
    let bool_expr = input.parse::<BoolExpr>()?;

    return Ok(BoolExpr::Not(Box::new(bool_expr)))
}

fn parse_bool_and(input: ParseStream) -> Result<BoolExpr> {
    let content;
    parenthesized!(content in input);
    let left = content.parse::<BoolExpr>()?;
    content.parse::<kw::and>()?;
    let right = content.parse::<BoolExpr>()?;
    
    return Ok(BoolExpr::And(Box::new(left), Box::new(right)))
}

fn parse_bool_or(input: ParseStream) -> Result<BoolExpr> {
    let content;
    parenthesized!(content in input);
    let left = content.parse::<BoolExpr>()?;
    content.parse::<kw::or>()?;
    let right = content.parse::<BoolExpr>()?;

    return Ok(BoolExpr::Or(Box::new(left), Box::new(right)))
}

fn parse_bool_id_eq(input: ParseStream) -> Result<BoolExpr> {
  let left = input.parse::<ID>()?;
  input.parse::<Token![==]>()?;
  let right = input.parse::<ID>()?;

  return Ok(BoolExpr::Eq(left, right))
}

fn parse_bool_id_neq(input: ParseStream) -> Result<BoolExpr> {
  let left = input.parse::<ID>()?;
  input.parse::<Token![!=]>()?;
  let right = input.parse::<ID>()?;

  return Ok(BoolExpr::Neq(left, right))
}

fn parse_bool_string_eq(input: ParseStream) -> Result<BoolExpr> {
    let left = input.parse::<StringExpr>()?;
    input.parse::<Token![==]>()?;
    let right = input.parse::<StringExpr>()?;

    if   matches!(left, StringExpr::ID(_))
      && matches!(right, StringExpr::ID(_)) {
        return Err(input.error("Ambiguous parsing!"))
    }

    return Ok(BoolExpr::StringEq(left, right))
}

fn parse_bool_string_neq(input: ParseStream) -> Result<BoolExpr> {
    let left = input.parse::<StringExpr>()?;
    input.parse::<Token![!=]>()?;
    let right = input.parse::<StringExpr>()?;

    if   matches!(left, StringExpr::ID(_))
      && matches!(right, StringExpr::ID(_)) {
        return Err(input.error("Ambiguous parsing!"))
    }

    return Ok(BoolExpr::StringNeq(left, right))
}

fn parse_bool_int(input: ParseStream) -> Result<BoolExpr> {
    let left = input.parse::<IntExpr>()?;
    let op = input.parse::<IntCmpOp>()?;
    let right = input.parse::<IntExpr>()?;

    return Ok(BoolExpr::IntCmp(left, op, right))
}

fn parse_bool_cardset_eq(input: ParseStream) -> Result<BoolExpr> {
    let left = input.parse::<CardSet>()?;
    input.parse::<Token![==]>()?;
    let right = input.parse::<CardSet>()?;

    if   matches!(left, CardSet::Group(Group::Location(_)))
      && matches!(right, CardSet::Group(Group::Location(_))) {
        return Err(input.error("Ambiguous parsing!"))
    }

    return Ok(BoolExpr::CardSetEq(left, right))
}

fn parse_bool_cardset_neq(input: ParseStream) -> Result<BoolExpr> {
    let left = input.parse::<CardSet>()?;
    input.parse::<Token![!=]>()?;
    let right = input.parse::<CardSet>()?;

    if   matches!(left, CardSet::Group(Group::Location(_)))
      && matches!(right, CardSet::Group(Group::Location(_))) {
        return Err(input.error("Ambiguous parsing!"))
    }

    return Ok(BoolExpr::CardSetNeq(left, right))
}

fn parse_bool_cardset_empty(input: ParseStream) -> Result<BoolExpr> {
    let cardset = input.parse::<CardSet>()?;
    input.parse::<kw::is>()?;
    input.parse::<kw::empty>()?;

    return Ok(BoolExpr::CardSetIsEmpty(cardset))
}

fn parse_bool_cardset_not_empty(input: ParseStream) -> Result<BoolExpr> {
    let cardset = input.parse::<CardSet>()?;
    input.parse::<kw::is>()?;
    input.parse::<kw::not>()?;
    input.parse::<kw::empty>()?;

    return Ok(BoolExpr::CardSetIsNotEmpty(cardset))
}

fn parse_bool_player_eq(input: ParseStream) -> Result<BoolExpr> {
    let left = input.parse::<PlayerExpr>()?;
    input.parse::<Token![==]>()?;
    let right = input.parse::<PlayerExpr>()?;

    if   matches!(left, PlayerExpr::PlayerName(_))
      && matches!(right, PlayerExpr::PlayerName(_)) {
        return Err(input.error("Ambiguous parsing!"))
    }

    return Ok(BoolExpr::PlayerEq(left, right))
}

fn parse_bool_player_neq(input: ParseStream) -> Result<BoolExpr> {
    let left = input.parse::<PlayerExpr>()?;
    input.parse::<Token![!=]>()?;
    let right = input.parse::<PlayerExpr>()?;

    if   matches!(left, PlayerExpr::PlayerName(_))
      && matches!(right, PlayerExpr::PlayerName(_)) {
        return Err(input.error("Ambiguous parsing!"))
    }

    return Ok(BoolExpr::PlayerNeq(left, right))
}

fn parse_bool_team_eq(input: ParseStream) -> Result<BoolExpr> {
    let left = input.parse::<TeamExpr>()?;
    input.parse::<Token![==]>()?;
    let right = input.parse::<TeamExpr>()?;

    if   matches!(left, TeamExpr::TeamName(_))
      && matches!(right, TeamExpr::TeamName(_)) {
        return Err(input.error("Ambiguous parsing!"))
    }

    return Ok(BoolExpr::TeamEq(left, right))
}

fn parse_bool_team_neq(input: ParseStream) -> Result<BoolExpr> {
    let left = input.parse::<TeamExpr>()?;
    input.parse::<Token![!=]>()?;
    let right = input.parse::<TeamExpr>()?;

    if   matches!(left, TeamExpr::TeamName(_))
      && matches!(right, TeamExpr::TeamName(_)) {
        return Err(input.error("Ambiguous parsing!"))
    }

    return Ok(BoolExpr::TeamNeq(left, right))
}

fn parse_bool_out_of_stage_player(input: ParseStream) -> Result<BoolExpr> {
    let player = input.parse::<PlayerExpr>()?;
    input.parse::<kw::out>()?;
    input.parse::<kw::of>()?;
    input.parse::<kw::stage>()?;

    return Ok(BoolExpr::OutOfStagePlayer(player))
}

fn parse_bool_out_of_game_player(input: ParseStream) -> Result<BoolExpr> {
    let player = input.parse::<PlayerExpr>()?;
    input.parse::<kw::out>()?;
    input.parse::<kw::of>()?;
    input.parse::<kw::game>()?;

    return Ok(BoolExpr::OutOfGamePlayer(player))
}

fn parse_bool_out_of_stage_player_collection(input: ParseStream) -> Result<BoolExpr> {
    let playercollection = input.parse::<PlayerCollection>()?;
    input.parse::<kw::out>()?;
    input.parse::<kw::of>()?;
    input.parse::<kw::stage>()?;

    return Ok(BoolExpr::OutOfStageCollection(playercollection))
}

fn parse_bool_out_of_game_player_collection(input: ParseStream) -> Result<BoolExpr> {
    let playercollection = input.parse::<PlayerCollection>()?;
    input.parse::<kw::out>()?;
    input.parse::<kw::of>()?;
    input.parse::<kw::game>()?;

    return Ok(BoolExpr::OutOfGameCollection(playercollection))
}

// ===========================================================================


// StringExpr
// ===========================================================================
impl Parse for StringExpr {
    fn parse(input: ParseStream) -> Result<Self> {
      parse_with_alternatives(input, 
        &[
          parse_string_expr_key_of,
          parse_string_expr_collection_at,
          parse_string_expr_id,
        ]
      )
    }
}

fn parse_string_expr_key_of(input: ParseStream) -> Result<StringExpr> {
    let key = input.parse::<Key>()?;
    input.parse::<kw::of>()?;
    let position = input.parse::<CardPosition>()?;

    return Ok(StringExpr::KeyOf(key, position))
}

fn parse_string_expr_collection_at(input: ParseStream) -> Result<StringExpr> {
    let string_collection: StringCollection = input.parse::<StringCollection>()?;
    let content;
    bracketed!(content in input);
    let int: IntExpr = content.parse()?;

    return Ok(StringExpr::StringCollectionAt(string_collection, int))
}

fn parse_string_expr_id(input: ParseStream) -> Result<StringExpr> {
  let id = input.parse::<ID>()?;
  
  return Ok(StringExpr::ID(id))
}

// ===========================================================================


// PlayerCollection
// ===========================================================================
impl Parse for PlayerCollection {
  fn parse(input: ParseStream) -> Result<Self> {
      parse_with_alternatives(input, 
        &[
          parse_player_collection_others,
          parse_player_collection_playersin,
          parse_player_collection_playersout,
          parse_player_collection_player,
          parse_player_collection_quantifier,
        ]
      )
  }
}

fn parse_player_collection_others(input: ParseStream) -> Result<PlayerCollection> {
  input.parse::<kw::others>()?;

  return Ok(PlayerCollection::Others)
}

fn parse_player_collection_playersin(input: ParseStream) -> Result<PlayerCollection> {
  input.parse::<kw::playersin>()?;

  return Ok(PlayerCollection::PlayersIn)
}

fn parse_player_collection_playersout(input: ParseStream) -> Result<PlayerCollection> {
  input.parse::<kw::playersout>()?;

  return Ok(PlayerCollection::PlayersOut)
}

fn parse_player_collection_player(input: ParseStream) -> Result<PlayerCollection> {
  let content;
  parenthesized!(content in input);
  let players: Punctuated<PlayerExpr, Token![,]> =
      content.parse_terminated(PlayerExpr::parse, Token![,])?;

  return Ok(PlayerCollection::Player(players.into_iter().collect()))
}

fn parse_player_collection_quantifier(input: ParseStream) -> Result<PlayerCollection> {
  let quantifier = input.parse::<Quantifier>()?;
  
  return Ok(PlayerCollection::Quantifier(quantifier))
}

// ===========================================================================


// FilterExpr
// ===========================================================================
impl Parse for FilterExpr {
  fn parse(input: ParseStream) -> Result<Self> {
    parse_with_alternatives(input, 
      &[
        parse_filter_adjacent,
        parse_filter_distinct,
        parse_filter_same,
        parse_filter_higher,
        parse_filter_lower,
        parse_filter_size,
        parse_filter_and,
        parse_filter_or,
        parse_filter_key_eq,
        parse_filter_key_neq,
        parse_filter_not_combo,
        parse_filter_combo,
      ]
    )
  }
}

fn parse_filter_same(input: ParseStream) -> Result<FilterExpr> {
  input.parse::<kw::same>()?;
  let key = input.parse::<Key>()?;

  return Ok(FilterExpr::Same(key))
}

fn parse_filter_distinct(input: ParseStream) -> Result<FilterExpr> {
  input.parse::<kw::distinct>()?;
  let key = input.parse::<Key>()?;

  return Ok(FilterExpr::Distinct(key))
}

fn parse_filter_adjacent(input: ParseStream) -> Result<FilterExpr> {
  input.parse::<kw::adjacent>()?;
  let key = input.parse::<Key>()?;
  input.parse::<kw::using>()?;
  let precedence = input.parse::<Precedence>()?;

  return Ok(FilterExpr::Adjacent(key, precedence))
}

fn parse_filter_higher(input: ParseStream) -> Result<FilterExpr> {
  input.parse::<kw::higher>()?;
  let key = input.parse::<Key>()?;
  input.parse::<kw::using>()?;
  let precedence = input.parse::<Precedence>()?;

  return Ok(FilterExpr::Higher(key, precedence))
}

fn parse_filter_lower(input: ParseStream) -> Result<FilterExpr> {
  input.parse::<kw::lower>()?;
  let key = input.parse::<Key>()?;
  input.parse::<kw::using>()?;
  let precedence = input.parse::<Precedence>()?;

  return Ok(FilterExpr::Lower(key, precedence))
}

fn parse_filter_size(input: ParseStream) -> Result<FilterExpr> {
  input.parse::<kw::size>()?;
  let operator = input.parse::<IntCmpOp>()?;
  let int = input.parse::<IntExpr>()?;

  return Ok(FilterExpr::Size(operator, Box::new(int)))
}

fn parse_filter_key_eq(input: ParseStream) -> Result<FilterExpr> {
  let key = input.parse::<Key>()?;
  input.parse::<Token![==]>()?;
  let string = input.parse::<StringExpr>()?;

  return Ok(FilterExpr::KeyEq(key, Box::new(string)))
}

fn parse_filter_key_neq(input: ParseStream) -> Result<FilterExpr> {
  let key = input.parse::<Key>()?;
  input.parse::<Token![!=]>()?;
  let string = input.parse::<StringExpr>()?;

  return Ok(FilterExpr::KeyNeq(key, Box::new(string)))
}

fn parse_filter_and(input: ParseStream) -> Result<FilterExpr> {
  let content;
  parenthesized!(content in input);
  let filter_left = content.parse::<FilterExpr>()?;
  content.parse::<kw::and>()?;
  let filter_right = content.parse::<FilterExpr>()?;

  return Ok(FilterExpr::And(Box::new(filter_left), Box::new(filter_right)))
}

fn parse_filter_or(input: ParseStream) -> Result<FilterExpr> {
  let content;
  parenthesized!(content in input);
  let filter_left = content.parse::<FilterExpr>()?;
  content.parse::<kw::or>()?;
  let filter_right = content.parse::<FilterExpr>()?;

  return Ok(FilterExpr::Or(Box::new(filter_left), Box::new(filter_right)))
}

fn parse_filter_not_combo(input: ParseStream) -> Result<FilterExpr> {
  input.parse::<kw::not>()?;
  let combo = input.parse::<Combo>()?;

  return Ok(FilterExpr::NotCombo(combo))
}

fn parse_filter_combo(input: ParseStream) -> Result<FilterExpr> {
  let combo = input.parse::<Combo>()?;

  return Ok(FilterExpr::Combo(combo))
}

// ===========================================================================


// Group
// ===========================================================================
impl Parse for Group {
    fn parse(input: ParseStream) -> Result<Self> {
      parse_with_alternatives(input, 
        &[
          parse_group_cardposition,
          parse_group_not_combo_in_location_collection,
          parse_group_not_combo_in_location,
          parse_group_combo_in_location_collection,
          parse_group_combo_in_location,
          parse_group_locaiton_collection_where,
          parse_group_locaiton_collection,
          parse_group_locaiton_where,
          parse_group_locaiton,
        ]
      )
    }
}

fn parse_group_cardposition(input: ParseStream) -> Result<Group> {
  let cardposition = input.parse::<CardPosition>()?;

  return Ok(Group::CardPosition(cardposition));
}

fn parse_group_not_combo_in_location(input: ParseStream) -> Result<Group> {
  let combo = input.parse::<Combo>()?;
  input.parse::<kw::not>()?;
  input.parse::<Token![in]>()?;
  let location = input.parse::<Location>()?;

  return Ok(Group::NotComboInLocation(combo, location));
}

fn parse_group_not_combo_in_location_collection(input: ParseStream) -> Result<Group> {
  let combo = input.parse::<Combo>()?;
  input.parse::<kw::not>()?;
  input.parse::<Token![in]>()?;
  let locationcollection = input.parse::<LocationCollection>()?;

  return Ok(Group::NotComboInLocationCollection(combo, locationcollection));
}

fn parse_group_combo_in_location(input: ParseStream) -> Result<Group> {
  let combo = input.parse::<Combo>()?;
  input.parse::<Token![in]>()?;
  let location = input.parse::<Location>()?;

  return Ok(Group::ComboInLocation(combo, location));
}

fn parse_group_combo_in_location_collection(input: ParseStream) -> Result<Group> {
  let combo = input.parse::<Combo>()?;
  input.parse::<Token![in]>()?;
  let locationcollection = input.parse::<LocationCollection>()?;

  return Ok(Group::ComboInLocationCollection(combo, locationcollection));
}

fn parse_group_locaiton_collection_where(input: ParseStream) -> Result<Group> {
  let locationcollection = input.parse::<LocationCollection>()?;
  input.parse::<Token![where]>()?;
  let filter: FilterExpr = input.parse()?;
  
  return Ok(Group::LocationCollectionWhere(locationcollection, filter));
}

fn parse_group_locaiton_collection(input: ParseStream) -> Result<Group> {
  let locationcollection = input.parse::<LocationCollection>()?;
  
  return Ok(Group::LocationCollection(locationcollection));
}

fn parse_group_locaiton_where(input: ParseStream) -> Result<Group> {
  let location = input.parse::<Location>()?;
  input.parse::<Token![where]>()?;
  let filter: FilterExpr = input.parse()?;
  
  return Ok(Group::LocationWhere(location, filter));
}

fn parse_group_locaiton(input: ParseStream) -> Result<Group> {
  let location = input.parse::<Location>()?;
  
  return Ok(Group::Location(location));
}

// ===========================================================================


// CardSet
// ===========================================================================
impl Parse for CardSet {
  fn parse(input: ParseStream) -> Result<Self> {
      parse_with_alternatives(input, 
        &[
          parse_cardset_group_of_player_collection,
          parse_cardset_group_of_player,
          parse_cardset_group,
        ]
      )
  }
}

fn parse_cardset_group_of_player(input: ParseStream) -> Result<CardSet> {
  let group = input.parse::<Group>()?;
  input.parse::<kw::of>()?;
  let player = input.parse::<PlayerExpr>()?;

  return Ok(CardSet::GroupOfPlayer(group, player))
}

fn parse_cardset_group_of_player_collection(input: ParseStream) -> Result<CardSet> {
  let group = input.parse::<Group>()?;
  input.parse::<kw::of>()?;
  let playercollection = input.parse::<PlayerCollection>()?;

  return Ok(CardSet::GroupOfPlayerCollection(group, playercollection))
}

fn parse_cardset_group(input: ParseStream) -> Result<CardSet> {
  let group = input.parse::<Group>()?;

  return Ok(CardSet::Group(group))
}

// ===========================================================================


// IntCollection
// ===========================================================================
impl Parse for IntCollection {
  fn parse(input: ParseStream) -> Result<Self> {
      let content;
      parenthesized!(content in input);
      let ints: Punctuated<IntExpr, Token![,]> =
          content.parse_terminated(IntExpr::parse, Token![,])?;

      return Ok(IntCollection { ints: ints.into_iter().collect() })
  }
}

// ===========================================================================


// LocationCollection
// ===========================================================================
impl Parse for LocationCollection {
  fn parse(input: ParseStream) -> Result<Self> {
    let content;
    parenthesized!(content in input);
    let locations: Punctuated<Location, Token![,]> =
        content.parse_terminated(Location::parse, Token![,])?;

    return Ok(LocationCollection { locations: locations.into_iter().collect() })
  }
}

// ===========================================================================


// TeamCollection
// ===========================================================================
impl Parse for TeamCollection {
  fn parse(input: ParseStream) -> Result<Self> {
    parse_with_alternatives(input, 
    &[
      parse_team_collection_other_teams,
      parse_team_collection_team,
    ]
  )
  }
}

fn parse_team_collection_other_teams(input: ParseStream) -> Result<TeamCollection> {
  input.parse::<kw::other>()?;
  input.parse::<kw::teams>()?;

  return Ok(TeamCollection::OtherTeams)
}

fn parse_team_collection_team(input: ParseStream) -> Result<TeamCollection> {
  let content;
  parenthesized!(content in input);
  let teams: Punctuated<TeamExpr, Token![,]> =
      content.parse_terminated(TeamExpr::parse, Token![,])?;

  return Ok(TeamCollection::Team(teams.into_iter().collect()))
}

// ===========================================================================


// StringCollection
// ===========================================================================
impl Parse for StringCollection {
  fn parse(input: ParseStream) -> Result<Self> {
    let content;
    parenthesized!(content in input);
    let strings: Punctuated<StringExpr, Token![,]> =
        content.parse_terminated(StringExpr::parse, Token![,])?;

    return Ok(StringCollection { strings: strings.into_iter().collect() })
  }
}

// ===========================================================================


// Collection
// TODO: maybe work out how to do it without ambiguity and no 'types' in the front
// ===========================================================================
impl Parse for Collection {
  fn parse(input: ParseStream) -> Result<Self> {
    parse_with_alternatives(input, 
      &[
        parse_collection_player_collection,
        parse_collection_team_collection,
        parse_collection_int_collection,
        // parse_collection_location_collection,
        parse_collection_cardset,
        parse_collection_string_collection,
        parse_collection_ambiguous,
      ]
    )
  }
}

fn parse_collection_player_collection(input: ParseStream) -> Result<Collection> {
  let playercollection = input.parse::<PlayerCollection>()?;

  match &playercollection {
    PlayerCollection::Player(players) => {
      // check if parsing is ambiguous
      for player in players.iter() {
        // break if is not ambiguous
        if !matches!(player, PlayerExpr::PlayerName(_)) {
          return Ok(Collection::PlayerCollection(playercollection))
        }
      }

      // return if parsing ambiguous
      return Err(input.error("Ambiguous parsing!"))
    },
    _ => {}
  }

  return Ok(Collection::PlayerCollection(playercollection))
}

fn parse_collection_team_collection(input: ParseStream) -> Result<Collection> {
  let teamcollection = input.parse::<TeamCollection>()?;

  match &teamcollection {
    TeamCollection::Team(teams) => {
      // check if parsing is ambiguous
      for team in teams.iter() {
        // break if is not ambiguous
        if !matches!(team, TeamExpr::TeamName(_)) {
          return Ok(Collection::TeamCollection(teamcollection))
        }
      }

      // return if parsing ambiguous
      return Err(input.error("Ambiguous parsing!"))
    },
    _ => {}
  }

  return Ok(Collection::TeamCollection(teamcollection))
}

fn parse_collection_int_collection(input: ParseStream) -> Result<Collection> {
  let intcollection = input.parse::<IntCollection>()?;

  return Ok(Collection::IntCollection(intcollection))
}

// Collection for LocationCollection is ambiguous to StringCollection
// fn parse_collection_location_collection(input: ParseStream) -> Result<Collection> {
//   let locationcollection = input.parse::<LocationCollection>()?;
//
//   // Always ambiguous with StringExpr
//   return Ok(Collection::LocationCollection(locationcollection))
// }

fn parse_collection_cardset(input: ParseStream) -> Result<Collection> {
  let cardset = input.parse::<CardSet>()?;

  match &cardset {
    CardSet::Group(group) => {
      // check if parsing is ambiguous
      if !matches!(group, Group::LocationCollection(_))
        && !matches!(group, Group::Location(_))
      {
        return Ok(Collection::CardSet(Box::new(cardset)))
      }
    
      // return if parsing ambiguous
      return Err(input.error("Ambiguous parsing!"))
    },
    _ => {}
  }

  return Ok(Collection::CardSet(Box::new(cardset)))
}

fn parse_collection_string_collection(input: ParseStream) -> Result<Collection> {
  let stringcollection = input.parse::<StringCollection>()?;

  for string in stringcollection.strings.iter() {
    // check if parsing is ambiguous
    if !matches!(string, StringExpr::ID(_)) {
      return Ok(Collection::StringCollection(stringcollection))
    }
  }

  // return if parsing ambiguous
  return Err(input.error("Ambiguous parsing!"))
}

fn parse_collection_ambiguous(input: ParseStream) -> Result<Collection> {
  let content;
  parenthesized!(content in input);
  let ids: Punctuated<ID, Token![,]> =
      content.parse_terminated(ID::parse, Token![,])?;

  return Ok(Collection::Ambiguous(ids.into_iter().collect()))
}

// ===========================================================================


// Reptitions
// ===========================================================================
impl Parse for Repititions {
  fn parse(input: ParseStream) -> Result<Self> {
      let int = input.parse::<IntExpr>()?;
      input.parse::<kw::times>()?;

      return Ok(Repititions { times: int})
  }
}

// ===========================================================================


// EndCondition
// ===========================================================================
impl Parse for EndCondition {
  fn parse(input: ParseStream) -> Result<Self> {
      parse_with_alternatives(input, 
        &[
          parse_endcondition_until_end,
          parse_endcondition_until_repition,
          parse_endcondition_until_bool_and_rep,
          parse_endcondition_until_bool_or_rep,
          parse_endcondition_until_bool,
        ]
      )
  }
}

fn parse_endcondition_until_end(input: ParseStream) -> Result<EndCondition> {
  input.parse::<kw::until>()?;
  let content;
  parenthesized!(content in input);
  content.parse::<kw::end>()?;

  return Ok(EndCondition::UntilEnd)
}

fn parse_endcondition_until_repition(input: ParseStream) -> Result<EndCondition> {
  input.parse::<kw::until>()?;
  let content;
  parenthesized!(content in input);
  let reps = content.parse::<Repititions>()?;

  return Ok(EndCondition::UntilRep(reps))
}

fn parse_endcondition_until_bool(input: ParseStream) -> Result<EndCondition> {
  input.parse::<kw::until>()?;
  let content;
  parenthesized!(content in input);
  let boolexpr = content.parse::<BoolExpr>()?;

  return Ok(EndCondition::UntilBool(boolexpr))
}

fn parse_endcondition_until_bool_and_rep(input: ParseStream) -> Result<EndCondition> {
  input.parse::<kw::until>()?;
  let content;
  parenthesized!(content in input);
  let boolexpr = content.parse::<BoolExpr>()?;
  content.parse::<kw::and>()?;
  let reps = content.parse::<Repititions>()?;

  return Ok(EndCondition::UntilBoolAndRep(boolexpr, reps))
}

fn parse_endcondition_until_bool_or_rep(input: ParseStream) -> Result<EndCondition> {
  input.parse::<kw::until>()?;
  let content;
  parenthesized!(content in input);
  let boolexpr = content.parse::<BoolExpr>()?;
  content.parse::<kw::or>()?;
  let reps = content.parse::<Repititions>()?;

  return Ok(EndCondition::UntilBoolOrRep(boolexpr, reps))
}

// ===========================================================================


// IntRange
// ===========================================================================
impl Parse for IntRange {
  fn parse(input: ParseStream) -> Result<Self> {
    // input.parse::<kw::range>()?;
    // let content;
    // parenthesized!(content in input);
    let op = input.parse::<IntCmpOp>()?;
    let int = input.parse::<IntExpr>()?;

    return Ok(IntRange {op: op, int: int})
  }
}

// ===========================================================================


// Quantity
// ===========================================================================
impl Parse for Quantity {
  fn parse(input: ParseStream) -> Result<Self> {
    parse_with_alternatives(input, 
      &[
        parse_quantity_intrange,
        parse_quantity_int,
        parse_quantity_quantifier,
      ]
    )
  }
}

fn parse_quantity_int(input: ParseStream) -> Result<Quantity> {
  let int = input.parse::<IntExpr>()?;
  
  return Ok(Quantity::Int(int))
}

fn parse_quantity_intrange(input: ParseStream) -> Result<Quantity> {
  let intrange = input.parse::<IntRange>()?;

  return Ok(Quantity::IntRange(intrange))
}

fn parse_quantity_quantifier(input: ParseStream) -> Result<Quantity> {
  let quantifier = input.parse::<Quantifier>()?;

  return Ok(Quantity::Quantifier(quantifier))
}

// ===========================================================================


// ClassicMove
// ===========================================================================
impl Parse for ClassicMove {
  fn parse(input: ParseStream) -> Result<Self> {
    parse_with_alternatives(input, 
      &[
        parse_classic_move_quantity,
        parse_classic_move_move,
      ]
    )
  }
}

fn parse_classic_move_quantity(input: ParseStream) -> Result<ClassicMove> {
  input.parse::<Token![move]>()?;
  let quantity = input.parse::<Quantity>()?;
  input.parse::<kw::from>()?;
  let from_cardset = input.parse::<CardSet>()?;
  let status = input.parse::<Status>()?;
  input.parse::<kw::to>()?;
  let to_cardset = input.parse::<CardSet>()?;

  return Ok(ClassicMove::MoveQuantity(quantity, from_cardset, status, to_cardset))
}

fn parse_classic_move_move(input: ParseStream) -> Result<ClassicMove> {
  input.parse::<Token![move]>()?;
  let from_cardset = input.parse::<CardSet>()?;
  let status = input.parse::<Status>()?;
  input.parse::<kw::to>()?;
  let to_cardset = input.parse::<CardSet>()?;

  return Ok(ClassicMove::Move(from_cardset, status, to_cardset))
}

// ===========================================================================


// DealMove
// ===========================================================================
impl Parse for DealMove {
  fn parse(input: ParseStream) -> Result<Self> {  
    parse_with_alternatives(input, 
      &[
        parse_deal_move_quantity,
        parse_deal_move_deal,
      ]
    )
  }
}

fn parse_deal_move_quantity(input: ParseStream) -> Result<DealMove> {
  input.parse::<kw::deal>()?;
  let quantity = input.parse::<Quantity>()?;
  input.parse::<kw::from>()?;
  let from_cardset = input.parse::<CardSet>()?;
  let status = input.parse::<Status>()?;
  input.parse::<kw::to>()?;
  let to_cardset = input.parse::<CardSet>()?;

  return Ok(DealMove::DealQuantity(quantity, from_cardset, status, to_cardset))
}

fn parse_deal_move_deal(input: ParseStream) -> Result<DealMove> {
  input.parse::<kw::deal>()?;
  let from_cardset = input.parse::<CardSet>()?;
  let status = input.parse::<Status>()?;
  input.parse::<kw::to>()?;
  let to_cardset = input.parse::<CardSet>()?;

  return Ok(DealMove::Deal(from_cardset, status, to_cardset))
}

// ===========================================================================


// ExchangeMove
// ===========================================================================
impl Parse for ExchangeMove {
  fn parse(input: ParseStream) -> Result<Self> {
      parse_with_alternatives(input,
      &[
        parse_exchange_move_quantity,
        parse_exchange_move_exchange,
      ]
    )
  }
}

fn parse_exchange_move_quantity(input: ParseStream) -> Result<ExchangeMove> {
  input.parse::<kw::exchange>()?;
  let quantity = input.parse::<Quantity>()?;
  input.parse::<kw::from>()?;
  let from_cardset = input.parse::<CardSet>()?;
  let status = input.parse::<Status>()?;
  input.parse::<kw::with>()?;
  let to_cardset = input.parse::<CardSet>()?;

  return Ok(ExchangeMove::ExchangeQuantity(quantity, from_cardset, status, to_cardset))
}

fn parse_exchange_move_exchange(input: ParseStream) -> Result<ExchangeMove> {
  input.parse::<kw::exchange>()?;
  let from_cardset = input.parse::<CardSet>()?;
  let status = input.parse::<Status>()?;
  input.parse::<kw::with>()?;
  let to_cardset = input.parse::<CardSet>()?;

  return Ok(ExchangeMove::Exchange(from_cardset, status, to_cardset))
}

// ===========================================================================


// TokenLocExpr
// ===========================================================================
impl Parse for TokenLocExpr {
  fn parse(input: ParseStream) -> Result<Self> {
      parse_with_alternatives(input,
        &[
          parse_token_loc_collection_location_player_collection,
          parse_token_loc_collection_location_player,
          parse_token_loc_collection_location,
          parse_token_loc_expr_location_player_collection,
          parse_token_loc_expr_location_player,
          parse_token_loc_expr_location,
        ]
      )
  }
}

fn parse_token_loc_expr_location(input: ParseStream) -> Result<TokenLocExpr> {
  let location = input.parse::<Location>()?;

  return Ok(TokenLocExpr::Location(location))
}

fn parse_token_loc_expr_location_player(input: ParseStream) -> Result<TokenLocExpr> {
  let location = input.parse::<Location>()?;
  input.parse::<kw::of>()?;
  let player = input.parse::<PlayerExpr>()?;
      
  return Ok(TokenLocExpr::LocationPlayer(location, player))
}

fn parse_token_loc_expr_location_player_collection(input: ParseStream) -> Result<TokenLocExpr> {
  let location = input.parse::<Location>()?;
  input.parse::<kw::of>()?;
  let playercollection = input.parse::<PlayerCollection>()?;
      
  return Ok(TokenLocExpr::LocationPlayerCollection(location, playercollection))
}

fn parse_token_loc_collection_location(input: ParseStream) -> Result<TokenLocExpr> {
  let locationcollection = input.parse::<LocationCollection>()?;

  return Ok(TokenLocExpr::LocationCollection(locationcollection))
}

fn parse_token_loc_collection_location_player(input: ParseStream) -> Result<TokenLocExpr> {
  let locationcollection = input.parse::<LocationCollection>()?;
  input.parse::<kw::of>()?;
  let player = input.parse::<PlayerExpr>()?;
      
  return Ok(TokenLocExpr::LocationCollectionPlayer(locationcollection, player))
}

fn parse_token_loc_collection_location_player_collection(input: ParseStream) -> Result<TokenLocExpr> {
  let locationcollection = input.parse::<LocationCollection>()?;
  input.parse::<kw::of>()?;
  let playercollection = input.parse::<PlayerCollection>()?;
      
  return Ok(TokenLocExpr::LocationCollectionPlayerCollection(locationcollection, playercollection))
}

// ===========================================================================


// TokenMove
// ===========================================================================
impl Parse for TokenMove {
  fn parse(input: ParseStream) -> Result<Self> {
      parse_with_alternatives(input,
        &[
          parse_token_move_place_quantity,
          parse_token_move_place,
        ]
      )
  }
}

fn parse_token_move_place_quantity(input: ParseStream) -> Result<TokenMove> {
  input.parse::<kw::place>()?;
  let quantity = input.parse::<Quantity>()?;
  input.parse::<kw::from>()?;
  let from_tokenloc = input.parse::<TokenLocExpr>()?;
  input.parse::<kw::to>()?;
  let to_tokenloc = input.parse::<TokenLocExpr>()?;

  return Ok(TokenMove::PlaceQuantity(quantity, from_tokenloc, to_tokenloc))
}

fn parse_token_move_place(input: ParseStream) -> Result<TokenMove> {
  input.parse::<kw::place>()?;
  let from_tokenloc = input.parse::<TokenLocExpr>()?;
  input.parse::<kw::to>()?;
  let to_tokenloc = input.parse::<TokenLocExpr>()?;

  return Ok(TokenMove::Place(from_tokenloc, to_tokenloc))
}

// ===========================================================================


// Rule
// ===========================================================================
impl Parse for Rule {
  fn parse(input: ParseStream) -> Result<Self> {
    parse_with_alternatives(input,
      &[
        parse_create_players,
        parse_create_team,
        parse_create_turnorder_random,
        parse_create_turnorder,
        parse_create_location_collection_on_player_collection,
        parse_create_location_collection_on_team_collection,
        parse_create_location_collection_on_table,
        parse_create_location_on_player_collection,
        parse_create_location_on_team_collection,
        parse_create_location_on_table,
        parse_create_card_on_location,
        parse_create_token_on_location,
        parse_create_precedence,
        parse_create_pointmap,
        parse_create_combo,
        parse_create_memory_player_collection,
        parse_create_memory_table,
        parse_create_memory_int_player_collection,
        parse_create_memory_int_table,
        parse_create_memory_string_player_collection,
        parse_create_memory_string_table,
        parse_flip_action,
        parse_shuffle_action,
        parse_set_player_out_of_stage,
        parse_set_player_out_of_game_succ,
        parse_set_player_out_of_game_fail,
        parse_set_player_collection_out_of_stage,
        parse_set_player_collection_out_of_game_succ,
        parse_set_player_collection_out_of_game_fail,
        parse_cycle_action,
        parse_bid_action_memory,
        parse_bid_action,
        parse_end_turn,
        parse_end_stage,
        parse_end_game,
        parse_demand_cardposition_action,
        parse_demand_int_action,
        parse_demand_string_action,
        parse_deal_move,
        parse_classic_move,
        parse_exchange_move,
        parse_token_move,
        parse_score_rule,
        parse_winner_rule,
        parse_set_memory_collection,
        parse_set_memory_int,
        parse_set_memory_string,
        parse_set_memory_ambiguous,
      ]
    )
  }
}

fn parse_create_players(input: ParseStream) -> Result<Rule> {
  input.parse::<kw::players>()?;
  input.parse::<Token![:]>()?;
  let content;
  parenthesized!(content in input);
  let players: Punctuated<PlayerName, Token![,]> =
      content.parse_terminated(PlayerName::parse, Token![,])?;

  return Ok(Rule::CreatePlayer(players.into_iter().collect()))
}

fn parse_create_team(input: ParseStream) -> Result<Rule> {
  input.parse::<kw::team>()?;
  let teamname = input.parse::<TeamName>()?;
  input.parse::<Token![:]>()?;
  let content;
  parenthesized!(content in input);
  let players: Punctuated<PlayerName, Token![,]> =
      content.parse_terminated(PlayerName::parse, Token![,])?;

  return Ok(Rule::CreateTeam(teamname, players.into_iter().collect()))
}

fn parse_create_turnorder_random(input: ParseStream) -> Result<Rule> {
  input.parse::<kw::random>()?;
  input.parse::<kw::turnorder>()?;
  input.parse::<Token![:]>()?;
  let content;
  parenthesized!(content in input);
  let players: Punctuated<PlayerName, Token![,]> =
      content.parse_terminated(PlayerName::parse, Token![,])?;

  return Ok(Rule::CreateTurnorderRandom(players.into_iter().collect()))
}

fn parse_create_turnorder(input: ParseStream) -> Result<Rule> {
  input.parse::<kw::turnorder>()?;
  input.parse::<Token![:]>()?;
  let content;
  parenthesized!(content in input);
  let players: Punctuated<PlayerName, Token![,]> =
      content.parse_terminated(PlayerName::parse, Token![,])?;

  return Ok(Rule::CreateTurnorder(players.into_iter().collect()))
}

fn parse_create_location_collection_on_player_collection(input: ParseStream) -> Result<Rule> {
  input.parse::<kw::location>()?;
  let locationcollection = input.parse::<LocationCollection>()?;
  input.parse::<kw::on>()?;
  input.parse::<kw::players>()?;
  let playercollection = input.parse::<PlayerCollection>()?;

  return Ok(
    Rule::CreateLocationCollectionOnPlayerCollection(
      locationcollection, playercollection
    )
  )
}

fn parse_create_location_collection_on_team_collection(input: ParseStream) -> Result<Rule> {
  input.parse::<kw::location>()?;
  let locationcollection = input.parse::<LocationCollection>()?;
  input.parse::<kw::on>()?;
  input.parse::<kw::teams>()?;
  let teamcollection = input.parse::<TeamCollection>()?;

  return Ok(
    Rule::CreateLocationCollectionOnTeamCollection(
      locationcollection, teamcollection
    )
  )
}

fn parse_create_location_collection_on_table(input: ParseStream) -> Result<Rule> {
  input.parse::<kw::location>()?;
  let locationcollection = input.parse::<LocationCollection>()?;
  input.parse::<kw::on>()?;
  input.parse::<kw::table>()?;

  return Ok(
    Rule::CreateLocationCollectionOnTable(
      locationcollection
    )
  )
}

fn parse_create_location_on_player_collection(input: ParseStream) -> Result<Rule> {
  input.parse::<kw::location>()?;
  let location = input.parse::<Location>()?;
  input.parse::<kw::on>()?;
  input.parse::<kw::players>()?;
  let playercollection = input.parse::<PlayerCollection>()?;

  return Ok(
    Rule::CreateLocationOnPlayerCollection(
      location, playercollection
    )
  )
}

fn parse_create_location_on_team_collection(input: ParseStream) -> Result<Rule> {
  input.parse::<kw::location>()?;
  let location = input.parse::<Location>()?;
  input.parse::<kw::on>()?;
  input.parse::<kw::teams>()?;
  let teamcollection = input.parse::<TeamCollection>()?;
  
  return Ok(
    Rule::CreateLocationOnTeamCollection(
      location, teamcollection
    )
  )
}

fn parse_create_location_on_table(input: ParseStream) -> Result<Rule> {
  input.parse::<kw::location>()?;
  let location = input.parse::<Location>()?;
  input.parse::<kw::on>()?;
  input.parse::<kw::table>()?;
  
  return Ok(
    Rule::CreateLocationOnTable(
      location
    )
  )
}

fn parse_create_card_on_location(input: ParseStream) -> Result<Rule> {
  input.parse::<kw::card>()?;
  input.parse::<kw::on>()?;
  let location= input.parse::<Location>()?;
  input.parse::<Token![:]>()?;
  let types = input.parse::<Types>()?;

  return Ok(Rule::CreateCardOnLocation(
    location, types)
  )
}

fn parse_create_token_on_location(input: ParseStream) -> Result<Rule> {
  input.parse::<kw::token>()?;
  let amount = input.parse::<IntExpr>()?;
  let token_type = input.parse::<Token>()?;
  input.parse::<kw::on>()?;
  let location = input.parse::<Location>()?;

  return Ok(Rule::CreateTokenOnLocation(
    amount, token_type, location
  ))
}

fn parse_create_precedence(input: ParseStream) -> Result<Rule> {
  input.parse::<kw::precedence>()?;
  let precedence = input.parse::<Precedence>()?;
  
  if input.peek(kw::on) {
    input.parse::<kw::on>()?;
    let key = input.parse::<Key>()?;
    let content;
    parenthesized!(content in input);
    let values: Punctuated<Value, Token![,]> =
      content.parse_terminated(Value::parse, Token![,])?;

    let key_value_pairs = values.into_iter().map(|v| (key.clone(), v)).collect();

    return Ok(Rule::CreatePrecedence(precedence, key_value_pairs))
  }

  let content;
  parenthesized!(content in input);
  let mut key_value_pairs = Vec::new();
  while !content.is_empty() {
    let key = content.parse::<Key>()?;
    let in_content;
    parenthesized!(in_content in content);
    let value = in_content.parse::<Value>()?;
    key_value_pairs.push((key, value));

    if content.peek(Token![,]) {
      content.parse::<Token![,]>()?;

      continue
    }
    
    break
  }

  return Ok(Rule::CreatePrecedence(precedence, key_value_pairs))
}

fn parse_create_pointmap(input: ParseStream) -> Result<Rule> {
  input.parse::<kw::pointmap>()?;
  let pointmap = input.parse::<PointMap>()?;
  
  if input.peek(kw::on) {
    input.parse::<kw::on>()?;
    let key = input.parse::<Key>()?;
    let content;
    parenthesized!(content in input);
    let mut key_value_int_triples = Vec::new();
    
    while !content.is_empty() {
      let value = content.parse::<Value>()?;
      content.parse::<Token![:]>()?;
      let int = content.parse::<IntExpr>()?;
      key_value_int_triples.push((key.clone(), value, int));

      if content.peek(Token![,]) {
        content.parse::<Token![,]>()?;
        
        continue
      }
      
      break
    }

    return Ok(Rule::CreatePointMap(pointmap, key_value_int_triples))
  }

  let content;
  parenthesized!(content in input);
  let mut key_value_int_triples = Vec::new();
  
  while !content.is_empty() {
    let key = content.parse::<Key>()?;
    let in_content;
    parenthesized!(in_content in content);
    let value = in_content.parse::<Value>()?;
    in_content.parse::<Token![:]>()?;
    let int = in_content.parse::<IntExpr>()?;
    key_value_int_triples.push((key.clone(), value, int));

    if content.peek(Token![,]) {
      content.parse::<Token![,]>()?;
      
      continue
    }
    
    break
  }

  return Ok(Rule::CreatePointMap(pointmap, key_value_int_triples))
}

fn parse_create_combo(input: ParseStream) -> Result<Rule> {
  input.parse::<kw::combo>()?;
  let combo = input.parse::<Combo>()?;
  input.parse::<Token![where]>()?;
  let filter = input.parse::<FilterExpr>()?;

  return Ok(Rule::CreateCombo(combo, filter))
}

fn parse_create_memory_table(input: ParseStream) -> Result<Rule> {
  input.parse::<kw::memory>()?;
  let memory = input.parse::<Memory>()?;
  input.parse::<kw::on>()?;
  input.parse::<kw::table>()?;

  return Ok(Rule::CreateMemoryTable(memory))
}

fn parse_create_memory_player_collection(input: ParseStream) -> Result<Rule> {
  input.parse::<kw::memory>()?;
  let memory = input.parse::<Memory>()?;
  input.parse::<kw::on>()?;
  let player_collection = input.parse::<PlayerCollection>()?;

  return Ok(Rule::CreateMemoryPlayerCollection(memory, player_collection))                
}

fn parse_create_memory_int_table(input: ParseStream) -> Result<Rule> {
  input.parse::<kw::memory>()?;
  let memory = input.parse::<Memory>()?;
  let int = input.parse::<IntExpr>()?;
  input.parse::<kw::on>()?;
  input.parse::<kw::table>()?;

  return Ok(Rule::CreateMemoryIntTable(memory, int))
}

fn parse_create_memory_int_player_collection(input: ParseStream) -> Result<Rule> {
  input.parse::<kw::memory>()?;
  let memory = input.parse::<Memory>()?;
  let int = input.parse::<IntExpr>()?;
  input.parse::<kw::on>()?;
  let player_collection = input.parse::<PlayerCollection>()?;

  return Ok(Rule::CreateMemoryIntPlayerCollection(memory, int, player_collection))
}

fn parse_create_memory_string_table(input: ParseStream) -> Result<Rule> {
  input.parse::<kw::memory>()?;
  let memory = input.parse::<Memory>()?;
  let string = input.parse::<StringExpr>()?;
  input.parse::<kw::on>()?;
  input.parse::<kw::table>()?;

  return Ok(Rule::CreateMemoryStringTable(memory, string))
}

fn parse_create_memory_string_player_collection(input: ParseStream) -> Result<Rule> {
  input.parse::<kw::memory>()?;
  let memory = input.parse::<Memory>()?;
  let string = input.parse::<StringExpr>()?;
  input.parse::<kw::on>()?;
  let player_collection = input.parse::<PlayerCollection>()?;

  return Ok(Rule::CreateMemoryStringPlayerCollection(memory, string, player_collection))
}

fn parse_flip_action(input: ParseStream) -> Result<Rule> {
  input.parse::<kw::flip>()?;
  let cardset = input.parse::<CardSet>()?;
  input.parse::<kw::to>()?;
  let status  = input.parse::<Status>()?;

  return Ok(Rule::FlipAction(cardset, status))
}

fn parse_shuffle_action(input: ParseStream) -> Result<Rule> {
  input.parse::<kw::shuffle>()?;
  let cardset = input.parse::<CardSet>()?;
  
  return Ok(Rule::ShuffleAction(cardset))
}

fn parse_set_player_out_of_stage(input: ParseStream) -> Result<Rule> {
  input.parse::<kw::set>()?;
  let player = input.parse::<PlayerExpr>()?;
  input.parse::<kw::out>()?;
  input.parse::<kw::of>()?;
  input.parse::<kw::stage>()?;

  return Ok(Rule::PlayerOutOfStageAction(player))
}

fn parse_set_player_out_of_game_succ(input: ParseStream) -> Result<Rule> {
  input.parse::<kw::set>()?;
  let player = input.parse::<PlayerExpr>()?;
  input.parse::<kw::out>()?;
  input.parse::<kw::of>()?;
  input.parse::<kw::game>()?;
  input.parse::<kw::successful>()?;

  return Ok(Rule::PlayerOutOfGameSuccAction(player))
}

fn parse_set_player_out_of_game_fail(input: ParseStream) -> Result<Rule> {
  input.parse::<kw::set>()?;
  let player = input.parse::<PlayerExpr>()?;
  input.parse::<kw::out>()?;
  input.parse::<kw::of>()?;
  input.parse::<kw::game>()?;
  input.parse::<kw::fail>()?;

  return Ok(Rule::PlayerOutOfGameFailAction(player))
}

fn parse_set_player_collection_out_of_stage(input: ParseStream) -> Result<Rule> {
  input.parse::<kw::set>()?;
  let playercollection = input.parse::<PlayerCollection>()?;
  input.parse::<kw::out>()?;
  input.parse::<kw::of>()?;
  input.parse::<kw::stage>()?;

  return Ok(Rule::PlayerCollectionOutOfStageAction(playercollection))
}

fn parse_set_player_collection_out_of_game_succ(input: ParseStream) -> Result<Rule> {
  input.parse::<kw::set>()?;
  let playercollection = input.parse::<PlayerCollection>()?;
  input.parse::<kw::out>()?;
  input.parse::<kw::of>()?;
  input.parse::<kw::game>()?;
  input.parse::<kw::successful>()?;

  return Ok(Rule::PlayerCollectionOutOfGameSuccAction(playercollection))
}

fn parse_set_player_collection_out_of_game_fail(input: ParseStream) -> Result<Rule> {
  input.parse::<kw::set>()?;
  let playercollection = input.parse::<PlayerCollection>()?;
  input.parse::<kw::out>()?;
  input.parse::<kw::of>()?;
  input.parse::<kw::game>()?;
  input.parse::<kw::fail>()?;

  return Ok(Rule::PlayerCollectionOutOfGameFailAction(playercollection))
}

fn parse_cycle_action(input: ParseStream) -> Result<Rule> {
  input.parse::<kw::cycle>()?;
  input.parse::<kw::to>()?;
  let player = input.parse::<PlayerExpr>()?;
  
  return Ok(Rule::CycleAction(player))
}

fn parse_bid_action(input: ParseStream) -> Result<Rule> {
  input.parse::<kw::bid>()?;
  let quantity = input.parse::<Quantity>()?;
  
  return Ok(Rule::BidAction(quantity))
}

fn parse_bid_action_memory(input: ParseStream) -> Result<Rule> {
  input.parse::<kw::bid>()?;
  let quantity = input.parse::<Quantity>()?;
  input.parse::<kw::on>()?;
  let memory = input.parse::<Memory>()?;
  
  return Ok(Rule::BidActionMemory(memory, quantity))
}

fn parse_end_turn(input: ParseStream) -> Result<Rule> {
  input.parse::<kw::end>()?;
  input.parse::<kw::turn>()?;

  return Ok(Rule::EndTurn)
}

fn parse_end_stage(input: ParseStream) -> Result<Rule> {
  input.parse::<kw::end>()?;
  input.parse::<kw::stage>()?;

  return Ok(Rule::EndStage)
}

fn parse_end_game(input: ParseStream) -> Result<Rule> {
  input.parse::<kw::end>()?;
  input.parse::<kw::game>()?;
  input.parse::<kw::with>()?;
  input.parse::<kw::winner>()?;
  let player = input.parse::<PlayerExpr>()?;

  return Ok(Rule::EndGameWithWinner(player))
}

fn parse_demand_cardposition_action(input: ParseStream) -> Result<Rule> {
  input.parse::<kw::demand>()?;
  let cardposition = input.parse::<CardPosition>()?;

  return Ok(Rule::DemandCardPositionAction(cardposition))
}

fn parse_demand_string_action(input: ParseStream) -> Result<Rule> {
  input.parse::<kw::demand>()?;
  let string = input.parse::<StringExpr>()?;

  return Ok(Rule::DemandStringAction(string))
}

fn parse_demand_int_action(input: ParseStream) -> Result<Rule> {
  input.parse::<kw::demand>()?;
  let int = input.parse::<IntExpr>()?;

  return Ok(Rule::DemandIntAction(int))
}

fn parse_deal_move(input: ParseStream) -> Result<Rule> {
  let dealmove = input.parse::<DealMove>()?;

  return Ok(Rule::DealMove(dealmove))
}

fn parse_classic_move(input: ParseStream) -> Result<Rule> {
  let classicmove = input.parse::<ClassicMove>()?;

  return Ok(Rule::ClassicMove(classicmove))
}

fn parse_exchange_move(input: ParseStream) -> Result<Rule> {
  let exchangemove = input.parse::<ExchangeMove>()?;

  return Ok(Rule::ExchangeMove(exchangemove))
}

fn parse_token_move(input: ParseStream) -> Result<Rule> {
  let tokenmove = input.parse::<TokenMove>()?;

  return Ok(Rule::TokenMove(tokenmove))
}

fn parse_score_rule(input: ParseStream) -> Result<Rule> {
  let scorerule = input.parse::<ScoreRule>()?;

  return Ok(Rule::ScoreRule(scorerule))
}

fn parse_winner_rule(input: ParseStream) -> Result<Rule> {
  let winnerrule = input.parse::<WinnerRule>()?;

  return Ok(Rule::WinnerRule(winnerrule))
}

fn parse_set_memory_collection(input: ParseStream) -> Result<Rule> {
  let memory = input.parse::<Memory>()?;
  input.parse::<kw::is>()?;
  let collection = input.parse::<Collection>()?;

  if matches!(collection, Collection::Ambiguous(_)) {
    return Err(input.error("Ambiguous parsing"))
  }

  return Ok(Rule::SetMemoryCollection(memory, collection))
}

fn parse_set_memory_int(input: ParseStream) -> Result<Rule> {
  let memory = input.parse::<Memory>()?;
  input.parse::<kw::is>()?;
  let int = input.parse::<IntExpr>()?;

  return Ok(Rule::SetMemoryInt(memory, int))
}

fn parse_set_memory_string(input: ParseStream) -> Result<Rule> {
  let memory = input.parse::<Memory>()?;
  input.parse::<kw::is>()?;
  let string = input.parse::<StringExpr>()?;

  if matches!(string, StringExpr::ID(_)) {
    return Err(input.error("Ambiguous parsing"))
  }

  return Ok(Rule::SetMemoryString(memory, string))
}

fn parse_set_memory_ambiguous(input: ParseStream) -> Result<Rule> {
  let memory = input.parse::<Memory>()?;
  input.parse::<kw::is>()?;
  let id = input.parse::<ID>()?;

  return Ok(Rule::SetMemoryAmbiguous(memory, id))
}

// ===========================================================================


// Types
// ===========================================================================
impl Parse for Types {
  fn parse(input: ParseStream) -> Result<Self> {
    let mut types = Vec::new();
    let key = input.parse::<Key>()?;
    let content;
    parenthesized!(content in input);

    let values: Punctuated<Value, Token![,]> =
      content.parse_terminated(Value::parse, Token![,])?;

    types.push((key, values.into_iter().collect()));

    if input.peek(Token![for]) {
      input.parse::<Token![for]>()?;
      let for_types = input.parse::<Types>()?;

      types.extend_from_slice(&for_types.types);
    }

    return Ok( Types { types: types })
  }
}

// ===========================================================================


// ScoreRule
// ===========================================================================
impl Parse for ScoreRule {
  fn parse(input: ParseStream) -> Result<Self> {
    parse_with_alternatives(input, 
      &[
        parse_score_player_collection_memory,
        parse_score_player_memory,
        parse_score_player_collection,
        parse_score_player,
      ]
    )
  }
}

fn parse_score_player_collection_memory(input: ParseStream) -> Result<ScoreRule> {
  input.parse::<kw::score>()?;
  let int = input.parse::<IntExpr>()?;
  input.parse::<kw::to>()?;
  let memory = input.parse::<Memory>()?;
  input.parse::<kw::of>()?;
  let playercollection = input.parse::<PlayerCollection>()?;

  return Ok(ScoreRule::ScorePlayerCollectionMemory(int, memory, playercollection))
}

fn parse_score_player_memory(input: ParseStream) -> Result<ScoreRule> {
  input.parse::<kw::score>()?;
  let int = input.parse::<IntExpr>()?;
  input.parse::<kw::to>()?;
  let memory = input.parse::<Memory>()?;
  input.parse::<kw::of>()?;
  let player = input.parse::<PlayerExpr>()?;

  return Ok(ScoreRule::ScorePlayerMemory(int, memory, player))
}

fn parse_score_player_collection(input: ParseStream) -> Result<ScoreRule> {
  input.parse::<kw::score>()?;
  let int = input.parse::<IntExpr>()?;
  input.parse::<kw::of>()?;
  let playercollection = input.parse::<PlayerCollection>()?;

  return Ok(ScoreRule::ScorePlayerCollection(int, playercollection))
}

fn parse_score_player(input: ParseStream) -> Result<ScoreRule> {
  input.parse::<kw::score>()?;
  let int = input.parse::<IntExpr>()?;
  input.parse::<kw::of>()?;
  let player = input.parse::<PlayerExpr>()?;

  return Ok(ScoreRule::ScorePlayer(int, player))
}

// ===========================================================================


// WinnerRule
// ===========================================================================
impl Parse for WinnerRule {
  fn parse(input: ParseStream) -> Result<Self> {
    parse_with_alternatives(input, 
      &[
        parse_winner_winner_is_lowest_score,
        parse_winner_winner_is_lowest_position,
        parse_winner_winner_is_lowest_memory,
        parse_winner_winner_is_highest_score,
        parse_winner_winner_is_highest_position,
        parse_winner_winner_is_highest_memory,
      ]
    )
  }
}

fn parse_winner_winner_is_lowest_score(input: ParseStream) -> Result<WinnerRule> {
  input.parse::<kw::winner>()?;
  input.parse::<kw::is>()?;
  input.parse::<kw::lowest>()?;
  input.parse::<kw::score>()?;

  return Ok(WinnerRule::WinnerLowestScore)
}

fn parse_winner_winner_is_lowest_position(input: ParseStream) -> Result<WinnerRule> {
  input.parse::<kw::winner>()?;
  input.parse::<kw::is>()?;
  input.parse::<kw::lowest>()?;
  input.parse::<kw::position>()?;

  return Ok(WinnerRule::WinnerLowestPosition)
}

fn parse_winner_winner_is_lowest_memory(input: ParseStream) -> Result<WinnerRule> {
  input.parse::<kw::winner>()?;
  input.parse::<kw::is>()?;
  input.parse::<kw::lowest>()?;

  let memory = input.parse::<Memory>()?;

  return Ok(WinnerRule::WinnerLowestMemory(memory))
}

fn parse_winner_winner_is_highest_score(input: ParseStream) -> Result<WinnerRule> {
  input.parse::<kw::winner>()?;
  input.parse::<kw::is>()?;
  input.parse::<kw::highest>()?;
  input.parse::<kw::score>()?;

  return Ok(WinnerRule::WinnerHighestScore)
}

fn parse_winner_winner_is_highest_position(input: ParseStream) -> Result<WinnerRule> {
  input.parse::<kw::winner>()?;
  input.parse::<kw::is>()?;
  input.parse::<kw::highest>()?;
  input.parse::<kw::position>()?;

  return Ok(WinnerRule::WinnerHighestPosition)
}

fn parse_winner_winner_is_highest_memory(input: ParseStream) -> Result<WinnerRule> {
  input.parse::<kw::winner>()?;
  input.parse::<kw::is>()?;
  input.parse::<kw::highest>()?;

  let memory = input.parse::<Memory>()?;

  return Ok(WinnerRule::WinnerHighestMemory(memory))
}

// ===========================================================================


// SeqStage
// ===========================================================================
impl Parse for SeqStage {
  fn parse(input: ParseStream) -> Result<Self> {
    input.parse::<kw::stage>()?;
    let stage = input.parse::<Stage>()?;

    input.parse::<Token![for]>()?;
    let player = input.parse::<PlayerExpr>()?;
    let endcondition = input.parse::<EndCondition>()?;

    let content;
    braced!(content in input);

    let mut flows = Vec::new();
    while !content.is_empty() {
      let flow = content.parse::<FlowComponent>()?;

      flows.push(flow);
    }

    return Ok(SeqStage {
      stage: stage,
      player: player,
      end_condition: endcondition,
      flows: flows
    })
  }
}

// ===========================================================================


// IfRule
// ===========================================================================
impl Parse for IfRule {
  fn parse(input: ParseStream) -> Result<Self> {
    input.parse::<Token![if]>()?;

    let content;
    parenthesized!(content in input);

    let condition = content.parse::<BoolExpr>()?;

    let content;
    braced!(content in input);

    let mut flows = Vec::new();
    while !content.is_empty() {
      let flow = content.parse::<FlowComponent>()?;

      flows.push(flow);
    }

    return Ok(IfRule { condition, flows })
  }
}

// ===========================================================================


// OptionalRule
// ===========================================================================
impl Parse for OptionalRule {
  fn parse(input: ParseStream) -> Result<Self> {
    input.parse::<kw::optional>()?;
    
    let content;
    braced!(content in input);

    let mut flows = Vec::new();
    while !content.is_empty() {
      let flow = content.parse::<FlowComponent>()?;

      flows.push(flow);
    }

    return Ok(OptionalRule { flows })
  }
}

// ===========================================================================


// ChoiceRule
// ===========================================================================
impl Parse for ChoiceRule {
  fn parse(input: ParseStream) -> Result<Self> {
    input.parse::<kw::choose>()?;

    let content;
    braced!(content in input);

    let options: Punctuated<FlowComponent, kw::or> =
      content.parse_terminated(FlowComponent::parse, kw::or)?;

    return Ok(ChoiceRule { options: options.into_iter().collect() })
  }
}

// ===========================================================================


// FlowComponent
// ===========================================================================
impl Parse for FlowComponent {
  fn parse(input: ParseStream) -> Result<Self> {
      parse_with_alternatives(input, 
        &[
          parse_flowcomponent_seqstage,
          parse_flowcomponent_choicerule,
          parse_flowcomponent_ifrule,
          parse_flowcomponent_optionalrule,
          parse_flowcomponent_rule,
        ]
      )
  }
}

fn parse_flowcomponent_seqstage(input: ParseStream) -> Result<FlowComponent> {
  let stage = input.parse::<SeqStage>()?;

  return Ok(FlowComponent::Stage(stage))
}

fn parse_flowcomponent_ifrule(input: ParseStream) -> Result<FlowComponent> {
  let ifrule = input.parse::<IfRule>()?;

  return Ok(FlowComponent::IfRule(ifrule))
}

fn parse_flowcomponent_choicerule(input: ParseStream) -> Result<FlowComponent> {
  let choicerule = input.parse::<ChoiceRule>()?;
  
  return Ok(FlowComponent::ChoiceRule(choicerule))
}

fn parse_flowcomponent_optionalrule(input: ParseStream) -> Result<FlowComponent> {
  let optionalrule = input.parse::<OptionalRule>()?;
  
  return Ok(FlowComponent::OptionalRule(optionalrule))
}

fn parse_flowcomponent_rule(input: ParseStream) -> Result<FlowComponent> {
  let rule = input.parse::<Rule>()?;
  input.parse::<Token![;]>()?;

  return Ok(FlowComponent::Rule(rule))
}

// ===========================================================================


// Game
// ===========================================================================
impl Parse for Game {
  fn parse(input: ParseStream) -> Result<Self> {
    let mut flows = Vec::new();

    while !input.is_empty() {
      let flow = input.parse::<FlowComponent>()?;

      flows.push(flow);
    }

    return Ok(Game { flows })
  }
}

// ===========================================================================
