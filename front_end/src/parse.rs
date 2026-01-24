use syn::parse::discouraged::Speculative;
use syn::parse::{Parse, ParseStream, Result};
use syn::punctuated::Punctuated;
use syn::{Ident, LitInt, Token, braced, bracketed, parenthesized};
use syn::spanned::Spanned as _;

use crate::spanned_ast::*;
use crate::keywords::kw::{self};
use crate::diagnostic::*;

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
impl Parse for SID {
  fn parse(input: ParseStream) -> Result<Self> {
      let fork = input.fork();
      let ident = fork.parse::<Ident>()?;
      let span = ident.span();
      let id = ident.to_string();

      // check correct "shape" of ID
      if kw::in_custom_key_words(&id) {
        return Err(input.error(&format!("ID: {} is custom_keyword", id)))
      }

      input.advance_to(&fork);

      return Ok(
        SID {
          node: id,
          span
        }
      )
  }
}
// ===========================================================================


// Op
// ===========================================================================
impl Parse for SOp {
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

fn parse_op_plus(input: ParseStream) -> Result<SOp> {
  let plus_token: syn::Token![+] = input.parse()?;
  let span = plus_token.span();
  
  return Ok(
    SOp {
      node: Op::Plus,
      span
    }
  )
}

fn parse_op_minus(input: ParseStream) -> Result<SOp> {
  let minus_token: syn::Token![-] = input.parse()?;
  let span = minus_token.span();
  
  return Ok(
    SOp {
      node: Op::Minus,
      span
    }
  )
}

fn parse_op_div(input: ParseStream) -> Result<SOp> {
  let div_token: syn::Token![/] = input.parse()?;
  let span = div_token.span();
  
  return Ok(
    SOp {
      node: Op::Div,
      span
    }
  )
}

fn parse_op_mul(input: ParseStream) -> Result<SOp> {
 let mul_token: syn::Token![*] = input.parse()?;
  let span = mul_token.span();
  
  return Ok(
    SOp {
      node: Op::Mul,
      span
    }
  )
}

fn parse_op_mod(input: ParseStream) -> Result<SOp> {
  let mod_token: syn::Token![%] = input.parse()?;
  let span = mod_token.span();
  
  return Ok(
    SOp {
      node: Op::Mod,
      span
    }
  )
}

// ===========================================================================


// IntCmpOp
// ===========================================================================
impl Parse for SIntCmpOp {
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

fn parse_intcmpop_eq(input: ParseStream) -> Result<SIntCmpOp> {
  let eq_token: syn::Token![==] = input.parse()?;
  let span = eq_token.span();
  
  return Ok(
    SIntCmpOp {
      node: IntCmpOp::Eq,
      span
    }
  )
}

fn parse_intcmpop_neq(input: ParseStream) -> Result<SIntCmpOp> {
  let neq_token: syn::Token![!=] = input.parse()?;
  let span = neq_token.span();
  
  return Ok(
    SIntCmpOp {
      node: IntCmpOp::Neq,
      span
    }
  )
}

fn parse_intcmpop_le(input: ParseStream) -> Result<SIntCmpOp> {
  let le_token: syn::Token![<=] = input.parse()?;
  let span = le_token.span();
  
  return Ok(
    SIntCmpOp {
      node: IntCmpOp::Le,
      span
    }
  )
}

fn parse_intcmpop_ge(input: ParseStream) -> Result<SIntCmpOp> {
  let ge_token = input.parse::<Token![>=]>()?;
  let span = ge_token.span();
  
  return Ok(
    SIntCmpOp {
      node: IntCmpOp::Ge,
      span
    }
  )
}

fn parse_intcmpop_lt(input: ParseStream) -> Result<SIntCmpOp> {
  let lt_token = input.parse::<Token![<]>()?;
  let span = lt_token.span();
  
  return Ok(
    SIntCmpOp {
      node: IntCmpOp::Lt,
      span
    }
  )
}

fn parse_intcmpop_gt(input: ParseStream) -> Result<SIntCmpOp> {
  let gt_token = input.parse::<Token![>]>()?;
  let span = gt_token.span();
  
  return Ok(
    SIntCmpOp {
      node: IntCmpOp::Gt,
      span
    }
  )
}

// ===========================================================================


// Status
// ===========================================================================
impl Parse for SStatus {
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

fn parse_status_face_down(input: ParseStream) -> Result<SStatus> {
  let face = input.parse::<kw::face>()?;
  let down = input.parse::<kw::down>()?;

  let span = face.span().join(down.span()).unwrap();

  return Ok(
    SStatus {
      node: Status::FaceDown,
      span  
    }
  )
}

fn parse_status_face_up(input: ParseStream) -> Result<SStatus> {
  let face = input.parse::<kw::face>()?;
  let up = input.parse::<kw::up>()?;

  let span = face.span().join(up.span()).unwrap();

  return Ok(
    SStatus {
      node: Status::FaceUp,
      span
    }
  )
}

fn parse_status_private(input: ParseStream) -> Result<SStatus> {
  let private = input.parse::<kw::private>()?;

  let span = private.span();

  return Ok(
    SStatus {
      node: Status::Private,
      span
    }
  )
}

// ===========================================================================


// Quantifier
// ===========================================================================
impl Parse for SQuantifier {
  fn parse(input: ParseStream) -> Result<Self> {
    parse_with_alternatives(input, 
      &[
        parse_quantifier_all,
        parse_quantifier_any,
      ]
    )
  }
}
fn parse_quantifier_all(input: ParseStream) -> Result<SQuantifier> {
  let all = input.parse::<kw::all>()?;

  let span = all.span();

  return Ok(
    SQuantifier {
      node: Quantifier::All,
      span
    }
  )
}

fn parse_quantifier_any(input: ParseStream) -> Result<SQuantifier> {
  let any = input.parse::<kw::any>()?;

  let span = any.span();

  return Ok(
    SQuantifier {
      node: Quantifier::Any,
      span
    }
  )
}

// ===========================================================================


// PlayerExpr
// ===========================================================================
impl Parse for SPlayerExpr {
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

fn parse_player_current(input: ParseStream) -> Result<SPlayerExpr> {
  let current = input.parse::<kw::current>()?;
  
  let span = current.span();

  return Ok(
    SPlayerExpr {
      node: PlayerExpr::Current,
      span
    }
  )
}

fn parse_player_previous(input: ParseStream) -> Result<SPlayerExpr> {
  let previous = input.parse::<kw::previous>()?;
  
  let span = previous.span();

  return Ok(
    SPlayerExpr {
      node: PlayerExpr::Previous,
      span
    }
  )
}

fn parse_player_next(input: ParseStream) -> Result<SPlayerExpr> {
  let next = input.parse::<kw::next>()?;
  
  let span = next.span();

  return Ok(
    SPlayerExpr {
      node: PlayerExpr::Next,
      span
    }
  )
}

fn parse_player_competitor(input: ParseStream) -> Result<SPlayerExpr> {
  let competitor = input.parse::<kw::competitor>()?;
  
  let span = competitor.span();

  return Ok(
    SPlayerExpr {
      node: PlayerExpr::Competitor,
      span
    }
  )
}

fn parse_player_owner_of_highest(input: ParseStream) -> Result<SPlayerExpr> {
  let owner = input.parse::<kw::owner>()?;
  input.parse::<kw::of>()?;
  input.parse::<kw::highest>()?;
  let memory = input.parse::<SID>()?;

  let span = owner
    .span()
    .join(memory.span)
    .unwrap_or(owner.span());

  return Ok(
    SPlayerExpr {
      node: PlayerExpr::OwnerOfHighest(memory),
      span
    }
  )
}

fn parse_player_owner_of_lowest(input: ParseStream) -> Result<SPlayerExpr> {
  let owner = input.parse::<kw::owner>()?;
  input.parse::<kw::of>()?;
  input.parse::<kw::lowest>()?;
  let memory = input.parse::<SID>()?;

  let span = owner
    .span()
    .join(memory.span)
    .unwrap_or(owner.span());

  return Ok(
    SPlayerExpr {
      node: PlayerExpr::OwnerOfLowest(memory),
      span
    }
  )
}

fn parse_player_at_turnorder(input: ParseStream) -> Result<SPlayerExpr> {
    // Parse the 'turnorder' keyword and capture its span
    let turnorder_kw = input.parse::<kw::turnorder>()?;
    let start_span = turnorder_kw.span();

    // Parse the parenthesized content
    let content;
    parenthesized!(content in input);
    let int: SIntExpr = content.parse()?; // assumes IntExpr is already spanned

    // Compute the combined span: start of keyword → end of inner expression
    let span = start_span.join(int.span).unwrap_or(start_span);

    Ok(SPlayerExpr {
        node: PlayerExpr::Turnorder(int),
        span,
    })
}

fn parse_player_owner_of_cardposition(input: ParseStream) -> Result<SPlayerExpr> {
  let owner = input.parse::<kw::owner>()?;
  let start_span = owner.span();
  input.parse::<kw::of>()?;
  let cardpos = input.parse::<SCardPosition>()?;

  let span = start_span
    .join(cardpos.span)
    .unwrap_or(start_span);


  return Ok(
    SPlayerExpr {
      node: PlayerExpr::OwnerOf(Box::new(cardpos)),
      span
    }
  )
}

fn parse_player_playername(input: ParseStream) -> Result<SPlayerExpr> {
  let playername = input.parse::<SID>()?;

  let span = playername.span;
      
  return Ok(
    SPlayerExpr {
      node: PlayerExpr::PlayerName(playername),
      span
    }
)
}

// ===========================================================================


// TeamExpr
// ===========================================================================
impl Parse for STeamExpr {
  fn parse(input: ParseStream) -> Result<Self> {
      parse_with_alternatives(input, 
        &[
          parse_team_team_of,
          parse_team_teamname,
        ]
      )
  }
}

fn parse_team_team_of(input: ParseStream) -> Result<STeamExpr> {
  let team = input.parse::<kw::team>()?;
  let start_span = team.span();
  input.parse::<kw::of>()?;
  let player = input.parse::<SPlayerExpr>()?;

  let span = start_span
    .join(player.span)
    .unwrap_or(start_span);

  return Ok(
    STeamExpr {
      node: TeamExpr::TeamOf(player),
      span
    }
  )
}

fn parse_team_teamname(input: ParseStream) -> Result<STeamExpr> {
  let teamname = input.parse::<SID>()?;
    
  let span = teamname.span;

  return Ok(
    STeamExpr {
      node: TeamExpr::TeamName(teamname),
      span
    }
  )
}

// ===========================================================================


// CardPosition
// ===========================================================================
impl Parse for SCardPosition {
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

fn parse_cardposition_top(input: ParseStream) -> Result<SCardPosition> {
  let top = input.parse::<kw::top>()?;
  let start_span = top.span();

  let content;
  parenthesized!(content in input);
  let location = content.parse::<SID>()?;

  let span = start_span
    .join(location.span)
    .unwrap_or(start_span);

  return Ok(
    SCardPosition {
      node: CardPosition::Top(location),
      span
    }
  )
}

fn parse_cardposition_bottom(input: ParseStream) -> Result<SCardPosition> {
  let bottom = input.parse::<kw::bottom>()?;
  let start_span = bottom.span();

  let content;
  parenthesized!(content in input);
  let location = content.parse::<SID>()?;

  let span = start_span
    .join(location.span)
    .unwrap_or(start_span);

  return Ok(
    SCardPosition {
      node: CardPosition::Bottom(location),
      span
    }
  )
}

fn parse_cardposition_max(input: ParseStream) -> Result<SCardPosition> {
  let max = input.parse::<kw::max>()?;        
  let start_span = max.span();

  let content;
  parenthesized!(content in input);
  let cardset = content.parse::<SCardSet>()?;
  input.parse::<kw::using>()?;
  let id = input.parse::<SID>()?;

  let span = start_span
    .join(id.span)
    .unwrap_or(start_span);

  return Ok(
    SCardPosition {
      node: CardPosition::Max(Box::new(cardset), id),
      span
    }
  )
}

fn parse_cardposition_min(input: ParseStream) -> Result<SCardPosition> {
  let min = input.parse::<kw::min>()?;        
  let start_span = min.span();

  let content;
  parenthesized!(content in input);
  let cardset = content.parse::<SCardSet>()?;
  input.parse::<kw::using>()?;
  let id = input.parse::<SID>()?;

  let span = start_span
    .join(id.span)
    .unwrap_or(start_span);

  return Ok(
    SCardPosition {
      node: CardPosition::Min(Box::new(cardset), id),
      span
    }
  )
}

fn parse_cardposition_at(input: ParseStream) -> Result<SCardPosition> {
  let location = input.parse::<SID>()?;
  let start_span = location.span;
  let content;
  bracketed!(content in input);
  let int = content.parse::<SIntExpr>()?;

  let span = start_span
    .join(int.span)
    .unwrap_or(start_span);

  return Ok(
    SCardPosition {
      node: CardPosition::At(location, int),
      span
    }
  )
}

// ===========================================================================


// IntExpr
// ===========================================================================
impl Parse for SIntExpr {
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

fn parse_int_op(input: ParseStream) -> Result<SIntExpr> {
  let content;
  parenthesized!(content in input);
  let left = content.parse::<SIntExpr>()?;
  let start_span = left.span;
  let op = content.parse::<SOp>()?;
  let right = content.parse::<SIntExpr>()?;
  
  let span = start_span
    .join(right.span)
    .unwrap_or(start_span);

  return Ok(
    SIntExpr {
      node: IntExpr::IntOp(Box::new(left), op, Box::new(right)),
      span
    }
  )
}

fn parse_int_size_of(input: ParseStream) -> Result<SIntExpr> {
  let size = input.parse::<kw::size>()?;
  let start_span = size.span();
  input.parse::<kw::of>()?;
  let collection = input.parse::<SCollection>()?;

  let span = start_span
    .join(collection.span)
    .unwrap_or(start_span);

  return Ok(
    SIntExpr {
      node: IntExpr::SizeOf(collection),
      span
    }
  )
}

fn parse_int_sum_of(input: ParseStream) -> Result<SIntExpr> {
  let sum = input.parse::<kw::sum>()?;
  let start_span = sum.span();
  input.parse::<kw::of>()?;
  let cardset = input.parse::<SCardSet>()?;
  input.parse::<kw::using>()?;
  let pointmap = input.parse::<SID>()?;

  let span = start_span
    .join(pointmap.span)
    .unwrap_or(start_span);

  return Ok(
    SIntExpr {
      node: IntExpr::SumOfCardSet(Box::new(cardset), pointmap),
      span
    }
  )
}

fn parse_int_sum_int_collection(input: ParseStream) -> Result<SIntExpr> {
  let sum = input.parse::<kw::sum>()?;
  let start_span = sum.span();
  let intcollection = input.parse::<SIntCollection>()?;
  
  let span = start_span
    .join(intcollection.span)
    .unwrap_or(start_span);

  return Ok(
    SIntExpr {
      node: IntExpr::SumOfIntCollection(intcollection),
      span
    }
  )
}

fn parse_int_min_of_cardset(input: ParseStream) -> Result<SIntExpr> {
  let min = input.parse::<kw::min>()?;
  let start_span = min.span();

  input.parse::<kw::of>()?;
  let cardset = input.parse::<SCardSet>()?;
  input.parse::<kw::using>()?;
  let pointmap = input.parse::<SID>()?;

  let span = start_span
    .join(pointmap.span)
    .unwrap_or(start_span);

  return Ok(
    SIntExpr {
      node: IntExpr::MinOf(Box::new(cardset), pointmap),
      span
    }
  )
}

fn parse_int_max_of_cardset(input: ParseStream) -> Result<SIntExpr> {
  let max = input.parse::<kw::max>()?;
  let start_span = max.span();
  input.parse::<kw::of>()?;
  let cardset = input.parse::<SCardSet>()?;
  input.parse::<kw::using>()?;
  let pointmap = input.parse::<SID>()?;

  let span = start_span
    .join(pointmap.span)
    .unwrap_or(start_span);

  return Ok(
    SIntExpr {
      node: IntExpr::MaxOf(Box::new(cardset), pointmap),
      span
    }
  )
}

fn parse_int_min_of_int_collection(input: ParseStream) -> Result<SIntExpr> {
  let min = input.parse::<kw::min>()?;
  let start_span = min.span();

  let intcollection = input.parse::<SIntCollection>()?;

  let span = start_span
    .join(intcollection.span)
    .unwrap_or(start_span);

  return Ok(
    SIntExpr {
      node: IntExpr::MinIntCollection(intcollection),
      span
    }
  )
}

fn parse_int_max_of_int_collection(input: ParseStream) -> Result<SIntExpr> {
  let max = input.parse::<kw::max>()?;
  let start_span = max.span();

  let intcollection = input.parse::<SIntCollection>()?;

  let span = start_span
    .join(intcollection.span)
    .unwrap_or(start_span);

  return Ok(
    SIntExpr {
      node: IntExpr::MaxIntCollection(intcollection),
      span
    }
  )
}

fn parse_int_stageroundcounter(input: ParseStream) -> Result<SIntExpr> {
  let stageroundcounter = input.parse::<kw::stageroundcounter>()?;
  let start_span = stageroundcounter.span();

  let span = start_span;

  return Ok(
    SIntExpr {
      node: IntExpr::StageRoundCounter,
      span
    }
  )
}

fn parse_int_int(input: ParseStream) -> Result<SIntExpr> {
  let litint: LitInt = input.parse::<LitInt>()?;
  let span = litint.span();
  let int = litint.base10_parse()?;
      
  return Ok(
    SIntExpr {
      node: IntExpr::Int(int),
      span
    } 
  )
}

// ===========================================================================


// BoolExpr
// ===========================================================================
impl Parse for SBoolExpr {
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
            // parse_bool_id_eq_ambiguous,
            // parse_bool_id_neq_ambiguous,
        ])
    }
}


fn parse_bool_not(input: ParseStream) -> Result<SBoolExpr> {
    let not = input.parse::<kw::not>()?;
    let start_span = not.span();
    let bool_expr = input.parse::<SBoolExpr>()?;

    let span = start_span
      .join(bool_expr.span)
      .unwrap_or(start_span);

    return Ok(
      SBoolExpr {
        node: BoolExpr::Not(Box::new(bool_expr)),
        span
      }
    )
}

fn parse_bool_and(input: ParseStream) -> Result<SBoolExpr> {
    let content;
    parenthesized!(content in input);
    let left = content.parse::<SBoolExpr>()?;
    let start_span = left.span;
    content.parse::<kw::and>()?;
    let right = content.parse::<SBoolExpr>()?;
    
    let span = start_span
      .join(right.span)
      .unwrap_or(start_span);

    return Ok(
      SBoolExpr {
        node: BoolExpr::And(Box::new(left), Box::new(right)),
        span
      }
    )
}

fn parse_bool_or(input: ParseStream) -> Result<SBoolExpr> {
    let content;
    parenthesized!(content in input);
    let left = content.parse::<SBoolExpr>()?;
    let start_span = left.span;
    content.parse::<kw::or>()?;
    let right = content.parse::<SBoolExpr>()?;
    
    let span = start_span
      .join(right.span)
      .unwrap_or(start_span);

    return Ok(
      SBoolExpr {
        node: BoolExpr::Or(Box::new(left), Box::new(right)),
        span
      }
    )
}

// fn parse_bool_id_eq_ambiguous(input: ParseStream) -> Result<BoolExpr> {
//   let left = input.parse::<ID>()?.to_string();
//   input.parse::<Token![==]>()?;
//   let right = input.parse::<ID>()?.to_string();

//   return Ok(BoolExpr::AmbiguousEq(left, right))
// }

// fn parse_bool_id_neq_ambiguous(input: ParseStream) -> Result<BoolExpr> {
//   let left = input.parse::<ID>()?.to_string();
//   input.parse::<Token![!=]>()?;
//   let right = input.parse::<ID>()?.to_string();

//   return Ok(BoolExpr::AmbiguousNeq(left, right))
// }

fn parse_bool_string_eq(input: ParseStream) -> Result<SBoolExpr> {
    let left = input.parse::<SStringExpr>()?;
    input.parse::<Token![==]>()?;
    let right = input.parse::<SStringExpr>()?;

    let span = left.span
      .join(right.span)
      .unwrap_or(left.span);

    return Ok(
      SBoolExpr {
        node: BoolExpr::StringEq(left, right),
        span
      }
    )
}

fn parse_bool_string_neq(input: ParseStream) -> Result<SBoolExpr> {
    let left = input.parse::<SStringExpr>()?;
    input.parse::<Token![!=]>()?;
    let right = input.parse::<SStringExpr>()?;

    let span = left.span
      .join(right.span)
      .unwrap_or(left.span);

    return Ok(
      SBoolExpr {
        node: BoolExpr::StringNeq(left, right),
        span
      }
    )
}

fn parse_bool_int(input: ParseStream) -> Result<SBoolExpr> {
    let left = input.parse::<SIntExpr>()?;
    let op = input.parse::<SIntCmpOp>()?;
    let right = input.parse::<SIntExpr>()?;

    let span = left.span
      .join(right.span)
      .unwrap_or(left.span);

    return Ok(
      SBoolExpr {
        node: BoolExpr::IntCmp(left, op, right),
        span
      }
    )
}

fn parse_bool_cardset_eq(input: ParseStream) -> Result<SBoolExpr> {
    let left = input.parse::<SCardSet>()?;
    input.parse::<Token![==]>()?;
    let right = input.parse::<SCardSet>()?;


    // This is the only thing making sense semantically
    // if   matches!(left, CardSet::Group(Group::Location(_)))
    //   && matches!(right, CardSet::Group(Group::Location(_))) {
    //     return Err(input.error("Ambiguous parsing!"))
    // }

    let span = left.span
      .join(right.span)
      .unwrap_or(left.span);

    return Ok(
      SBoolExpr {
        node: BoolExpr::CardSetEq(left, right),
        span
      }
    )
}

fn parse_bool_cardset_neq(input: ParseStream) -> Result<SBoolExpr> {
    let left = input.parse::<SCardSet>()?;
    input.parse::<Token![!=]>()?;
    let right = input.parse::<SCardSet>()?;

    // This is the only thing making sense semantically
    // if   matches!(left, CardSet::Group(Group::Location(_)))
    //   && matches!(right, CardSet::Group(Group::Location(_))) {
    //     return Err(input.error("Ambiguous parsing!"))
    // }

    let span = left.span
      .join(right.span)
      .unwrap_or(left.span);

    return Ok(
      SBoolExpr {
        node: BoolExpr::CardSetNeq(left, right),
        span
      }
    )
}

fn parse_bool_cardset_empty(input: ParseStream) -> Result<SBoolExpr> {
    let cardset = input.parse::<SCardSet>()?;
    let start_span = cardset.span;
    input.parse::<kw::is>()?;
    let empty = input.parse::<kw::empty>()?;

    let span = start_span
      .join(empty.span())
      .unwrap_or(start_span);

    return Ok(
      SBoolExpr {
        node: BoolExpr::CardSetIsEmpty(cardset),
        span
      } 
    )
}

fn parse_bool_cardset_not_empty(input: ParseStream) -> Result<SBoolExpr> {
    let cardset = input.parse::<SCardSet>()?;
    let start_span = cardset.span;
    input.parse::<kw::is>()?;
    input.parse::<kw::not>()?;
    let empty = input.parse::<kw::empty>()?;

    let span = start_span
      .join(empty.span())
      .unwrap_or(start_span);

    return Ok(
      SBoolExpr {
        node: BoolExpr::CardSetIsNotEmpty(cardset),
        span
      } 
    )
}

fn parse_bool_player_eq(input: ParseStream) -> Result<SBoolExpr> {
    let left = input.parse::<SPlayerExpr>()?;
    input.parse::<Token![==]>()?;
    let right = input.parse::<SPlayerExpr>()?;

    if   matches!(left.node, PlayerExpr::PlayerName(_))
      && matches!(right.node, PlayerExpr::PlayerName(_)) {
        return Err(input.error("No Semantic Value!"))
    }

    let span = left.span
      .join(right.span)
      .unwrap_or(left.span);

    return Ok(
      SBoolExpr {
        node: BoolExpr::PlayerEq(left, right),
        span
      }
    )
}

fn parse_bool_player_neq(input: ParseStream) -> Result<SBoolExpr> {
    let left = input.parse::<SPlayerExpr>()?;
    input.parse::<Token![!=]>()?;
    let right = input.parse::<SPlayerExpr>()?;

    if   matches!(left.node, PlayerExpr::PlayerName(_))
      && matches!(right.node, PlayerExpr::PlayerName(_)) {
        return Err(input.error("No Semantic Value!"))
    }

    let span = left.span
      .join(right.span)
      .unwrap_or(left.span);

    return Ok(
      SBoolExpr {
        node: BoolExpr::PlayerNeq(left, right),
        span
      }
    )
}

fn parse_bool_team_eq(input: ParseStream) -> Result<SBoolExpr> {
    let left = input.parse::<STeamExpr>()?;
    input.parse::<Token![==]>()?;
    let right = input.parse::<STeamExpr>()?;

    if   matches!(left.node, TeamExpr::TeamName(_))
      && matches!(right.node, TeamExpr::TeamName(_)) {
        return Err(input.error("No Semantic Value!"))
    }

    let span = left.span
      .join(right.span)
      .unwrap_or(left.span);

    return Ok(
      SBoolExpr {
        node: BoolExpr::TeamEq(left, right),
        span
      }
    )
}

fn parse_bool_team_neq(input: ParseStream) -> Result<SBoolExpr> {
    let left = input.parse::<STeamExpr>()?;
    input.parse::<Token![!=]>()?;
    let right = input.parse::<STeamExpr>()?;

    if   matches!(left.node, TeamExpr::TeamName(_))
      && matches!(right.node, TeamExpr::TeamName(_)) {
        return Err(input.error("No Semantic Value!"))
    }

    let span = left.span
      .join(right.span)
      .unwrap_or(left.span);

    return Ok(
      SBoolExpr {
        node: BoolExpr::TeamNeq(left, right),
        span
      }
    )
}

fn parse_bool_out_of_stage_player(input: ParseStream) -> Result<SBoolExpr> {
    let player = input.parse::<SPlayerExpr>()?;
    let start_span = player.span;
    input.parse::<kw::out>()?;
    input.parse::<kw::of>()?;
    let stage = input.parse::<kw::stage>()?;

    let span = start_span
      .join(stage.span())
      .unwrap_or(start_span);

    return Ok(
      SBoolExpr {
        node: BoolExpr::OutOfStagePlayer(player),
        span
      }
    )
}

fn parse_bool_out_of_game_player(input: ParseStream) -> Result<SBoolExpr> {
    let player = input.parse::<SPlayerExpr>()?;
    let start_span = player.span;
    input.parse::<kw::out>()?;
    input.parse::<kw::of>()?;
    let game = input.parse::<kw::game>()?;

    let span = start_span
      .join(game.span())
      .unwrap_or(start_span);

    return Ok(
      SBoolExpr {
        node: BoolExpr::OutOfGamePlayer(player),
        span
      }
    )
}

fn parse_bool_out_of_stage_player_collection(input: ParseStream) -> Result<SBoolExpr> {
    let playercollection = input.parse::<SPlayerCollection>()?;
    let start_span = playercollection.span;
    input.parse::<kw::out>()?;
    input.parse::<kw::of>()?;
    let stage = input.parse::<kw::stage>()?;

    let span = start_span
      .join(stage.span())
      .unwrap_or(start_span);

    return Ok(
      SBoolExpr {
        node: BoolExpr::OutOfStageCollection(playercollection),
        span
      }
    )
}

fn parse_bool_out_of_game_player_collection(input: ParseStream) -> Result<SBoolExpr> {
    let playercollection = input.parse::<SPlayerCollection>()?;
    let start_span = playercollection.span;
    input.parse::<kw::out>()?;
    input.parse::<kw::of>()?;
    let game = input.parse::<kw::game>()?;

    let span = start_span
      .join(game.span())
      .unwrap_or(start_span);

    return Ok(
      SBoolExpr {
        node: BoolExpr::OutOfGameCollection(playercollection),
        span
      }
    )
}

// ===========================================================================


// StringExpr
// ===========================================================================
impl Parse for SStringExpr {
    fn parse(input: ParseStream) -> Result<Self> {
      parse_with_alternatives(input, 
        &[
          parse_string_expr_key_of,
          parse_string_expr_collection_at,
        ]
      )
    }
}

fn parse_string_expr_key_of(input: ParseStream) -> Result<SStringExpr> {
    let key = input.parse::<SID>()?;
    let start_span = key.span;
    input.parse::<kw::of>()?;
    let position = input.parse::<SCardPosition>()?;

    let span = start_span
      .join(position.span)
      .unwrap_or(start_span);

    return Ok(
      SStringExpr {
        node: StringExpr::KeyOf(key, position),
        span
      }
    )
}

fn parse_string_expr_collection_at(input: ParseStream) -> Result<SStringExpr> {
    let string_collection: SStringCollection = input.parse::<SStringCollection>()?;
    let start_span = string_collection.span;

    let content;
    bracketed!(content in input);
    let int: SIntExpr = content.parse()?;

    let span = start_span
      .join(int.span)
      .unwrap_or(start_span);

    return Ok(
      SStringExpr {
        node: StringExpr::StringCollectionAt(string_collection, int),
        span
      }
    )
}

// ===========================================================================


// PlayerCollection
// ===========================================================================
impl Parse for SPlayerCollection {
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

fn parse_player_collection_others(input: ParseStream) -> Result<SPlayerCollection> {
  let others = input.parse::<kw::others>()?;

  let span = others.span();

  return Ok(
    SPlayerCollection {
      node: PlayerCollection::Others,
      span
    }
  )
}

fn parse_player_collection_playersin(input: ParseStream) -> Result<SPlayerCollection> {
  let playersin = input.parse::<kw::playersin>()?;

  let span = playersin.span();

  return Ok(
    SPlayerCollection {
      node: PlayerCollection::PlayersIn,
      span
    }
  )
}

fn parse_player_collection_playersout(input: ParseStream) -> Result<SPlayerCollection> {
  let playersout = input.parse::<kw::playersout>()?;

  let span = playersout.span();

  return Ok(
    SPlayerCollection {
      node: PlayerCollection::PlayersOut,
      span
    }
  )
}

fn parse_player_collection_player(input: ParseStream) -> Result<SPlayerCollection> {
  let content;
  parenthesized!(content in input);

  let players: Punctuated<SPlayerExpr, Token![,]> =
      content.parse_terminated(SPlayerExpr::parse, Token![,])?;

  let span = content.span();

  return Ok(
    SPlayerCollection {
      node: PlayerCollection::Player(
          players.into_iter().collect()
      ),
      span,
    }
  )
}

fn parse_player_collection_quantifier(input: ParseStream) -> Result<SPlayerCollection> {
  let quantifier = input.parse::<SQuantifier>()?;
  
  let span = quantifier.span;

  return Ok(
    SPlayerCollection {
      node: PlayerCollection::Quantifier(quantifier),
      span
    }
  )
}

// ===========================================================================


// FilterExpr
// ===========================================================================
impl Parse for SFilterExpr {
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
        parse_filter_key_eq_string,
        parse_filter_key_neq_string,
        parse_filter_key_eq_value,
        parse_filter_key_neq_value,
        parse_filter_not_combo,
        parse_filter_combo,
      ]
    )
  }
}

fn parse_filter_same(input: ParseStream) -> Result<SFilterExpr> {
  let same = input.parse::<kw::same>()?;
  let start_span = same.span();
  let key = input.parse::<SID>()?;

  let span = start_span
    .join(key.span)
    .unwrap_or(start_span);

  return Ok(
    SFilterExpr {
      node: FilterExpr::Same(key),
      span
    }
  )
}

fn parse_filter_distinct(input: ParseStream) -> Result<SFilterExpr> {
  let distinct = input.parse::<kw::distinct>()?;
  let start_span = distinct.span();
  let key = input.parse::<SID>()?;

  let span = start_span
    .join(key.span)
    .unwrap_or(start_span);

  return Ok(
    SFilterExpr {
      node: FilterExpr::Distinct(key),
      span
    }
  )
}

fn parse_filter_adjacent(input: ParseStream) -> Result<SFilterExpr> {
  let adjacent = input.parse::<kw::adjacent>()?;
  let start_span = adjacent.span();
  let key = input.parse::<SID>()?;
  input.parse::<kw::using>()?;
  let precedence = input.parse::<SID>()?;

  let span = start_span
    .join(precedence.span)
    .unwrap_or(start_span);

  return Ok(
    SFilterExpr {
      node: FilterExpr::Adjacent(key, precedence),
      span
    }
  )
}

fn parse_filter_higher(input: ParseStream) -> Result<SFilterExpr> {
  let higher = input.parse::<kw::higher>()?;
  let start_span = higher.span();
  let key = input.parse::<SID>()?;
  input.parse::<kw::using>()?;
  let precedence = input.parse::<SID>()?;

  let span = start_span
    .join(precedence.span)
    .unwrap_or(start_span);

  return Ok(
    SFilterExpr {
      node: FilterExpr::Higher(key, precedence),
      span
    }
  )
}

fn parse_filter_lower(input: ParseStream) -> Result<SFilterExpr> {
  let lower = input.parse::<kw::lower>()?;
  let start_span = lower.span();
  let key = input.parse::<SID>()?;
  input.parse::<kw::using>()?;
  let precedence = input.parse::<SID>()?;

  let span = start_span
    .join(precedence.span)
    .unwrap_or(start_span);

  return Ok(
    SFilterExpr {
      node: FilterExpr::Lower(key, precedence),
      span
    }
  )
}

fn parse_filter_size(input: ParseStream) -> Result<SFilterExpr> {
  let size = input.parse::<kw::size>()?;
  let start_span = size.span();
  let operator = input.parse::<SIntCmpOp>()?;
  let int = input.parse::<SIntExpr>()?;

  let span = start_span
    .join(int.span)
    .unwrap_or(start_span);

  return Ok(
    SFilterExpr {
      node: FilterExpr::Size(operator, Box::new(int)),
      span
    }
  )
}

fn parse_filter_key_eq_string(input: ParseStream) -> Result<SFilterExpr> {
  let key = input.parse::<SID>()?;
  let start_span = key.span;
  input.parse::<Token![==]>()?;
  let string = input.parse::<SStringExpr>()?;

  let span = start_span
    .join(string.span)
    .unwrap_or(start_span);

  return Ok(
    SFilterExpr {
      node: FilterExpr::KeyEqString(key, Box::new(string)),
      span
    }
  )
}

fn parse_filter_key_neq_string(input: ParseStream) -> Result<SFilterExpr> {
  let key = input.parse::<SID>()?;
  let start_span = key.span;
  input.parse::<Token![!=]>()?;
  let string = input.parse::<SStringExpr>()?;

  let span = start_span
    .join(string.span)
    .unwrap_or(start_span);

  return Ok(
    SFilterExpr {
      node: FilterExpr::KeyNeqString(key, Box::new(string)),
      span
    }
  )
}

fn parse_filter_key_eq_value(input: ParseStream) -> Result<SFilterExpr> {
  let key = input.parse::<SID>()?;
  let start_span = key.span;
  input.parse::<Token![==]>()?;
  let value = input.parse::<SID>()?;

  let span = start_span
    .join(value.span)
    .unwrap_or(start_span);

  return Ok(
    SFilterExpr {
      node: FilterExpr::KeyEqValue(key, value),
      span
    }
  )
}

fn parse_filter_key_neq_value(input: ParseStream) -> Result<SFilterExpr> {
  let key = input.parse::<SID>()?;
  let start_span = key.span;
  input.parse::<Token![!=]>()?;
  let value = input.parse::<SID>()?;

  let span = start_span
    .join(value.span)
    .unwrap_or(start_span);

  return Ok(
    SFilterExpr {
      node: FilterExpr::KeyNeqValue(key, value),
      span
    }
  )
}

fn parse_filter_and(input: ParseStream) -> Result<SFilterExpr> {
  let content;
  parenthesized!(content in input);
  let filter_left = content.parse::<SFilterExpr>()?;
  content.parse::<kw::and>()?;
  let filter_right = content.parse::<SFilterExpr>()?;

  let span = filter_left.span
    .join(filter_right.span)
    .unwrap_or(filter_left.span);

  return Ok(
    SFilterExpr {
      node: FilterExpr::And(Box::new(filter_left), Box::new(filter_right)),
      span
    }
  )
}

fn parse_filter_or(input: ParseStream) -> Result<SFilterExpr> {
  let content;
  parenthesized!(content in input);
  let filter_left = content.parse::<SFilterExpr>()?;
  content.parse::<kw::or>()?;
  let filter_right = content.parse::<SFilterExpr>()?;

  let span = filter_left.span
    .join(filter_right.span)
    .unwrap_or(filter_left.span);

  return Ok(
    SFilterExpr {
      node: FilterExpr::Or(Box::new(filter_left), Box::new(filter_right)),
      span
    }
  )
}

fn parse_filter_not_combo(input: ParseStream) -> Result<SFilterExpr> {
  let not = input.parse::<kw::not>()?;
  let start_span = not.span();
  let combo = input.parse::<SID>()?;

  let span = start_span
    .join(combo.span)
    .unwrap_or(start_span);

  return Ok(
    SFilterExpr {
      node: FilterExpr::NotCombo(combo),
      span
    }
  )
}

fn parse_filter_combo(input: ParseStream) -> Result<SFilterExpr> {
  let combo = input.parse::<SID>()?;

  let span = combo.span;

  return Ok(
    SFilterExpr {
      node: FilterExpr::Combo(combo),
      span
    }
  )
}


// ===========================================================================


// Group
// ===========================================================================
impl Parse for SGroup {
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

fn parse_group_cardposition(input: ParseStream) -> Result<SGroup> {
  let cardposition = input.parse::<SCardPosition>()?;

  let span = cardposition.span;

  return Ok(
    SGroup {
      node: Group::CardPosition(cardposition),
      span
    }
  );
}

fn parse_group_not_combo_in_location(input: ParseStream) -> Result<SGroup> {
  let combo = input.parse::<SID>()?;
  let start_span = combo.span;
  input.parse::<kw::not>()?;
  input.parse::<Token![in]>()?;
  let location = input.parse::<SID>()?;

  let span = start_span
    .join(location.span)
    .unwrap_or(start_span);

  return Ok(
    SGroup {
      node: Group::NotComboInLocation(combo, location),
      span
    }
  );
}

fn parse_group_not_combo_in_location_collection(input: ParseStream) -> Result<SGroup> {
  let combo = input.parse::<SID>()?;
  let start_span = combo.span;

  input.parse::<kw::not>()?;
  input.parse::<Token![in]>()?;
  let locationcollection = input.parse::<SLocationCollection>()?;

  let span = start_span
    .join(locationcollection.span)
    .unwrap_or(start_span);

  return Ok(
    SGroup {
      node: Group::NotComboInLocationCollection(combo, locationcollection),
      span
    }
  );
}

fn parse_group_combo_in_location(input: ParseStream) -> Result<SGroup> {
  let combo = input.parse::<SID>()?;
  let start_span = combo.span;
  input.parse::<Token![in]>()?;
  let location = input.parse::<SID>()?;


  let span = start_span
    .join(location.span)
    .unwrap_or(start_span);

  return Ok(
    SGroup {
      node: Group::ComboInLocation(combo, location),
      span
    }
  );
}

fn parse_group_combo_in_location_collection(input: ParseStream) -> Result<SGroup> {
  let combo = input.parse::<SID>()?;
  let start_span = combo.span;

  input.parse::<Token![in]>()?;
  let locationcollection = input.parse::<SLocationCollection>()?;

  let span = start_span
    .join(locationcollection.span)
    .unwrap_or(start_span);

  return Ok(
    SGroup {
      node: Group::ComboInLocationCollection(combo, locationcollection),
      span
    }
  );
}

fn parse_group_locaiton_collection_where(input: ParseStream) -> Result<SGroup> {
  let locationcollection = input.parse::<SLocationCollection>()?;
  let start_span = locationcollection.span;
  input.parse::<Token![where]>()?;
  let filter: SFilterExpr = input.parse()?;

  let span = start_span
    .join(filter.span)
    .unwrap_or(start_span);
  
  return Ok(
    SGroup {
      node: Group::LocationCollectionWhere(locationcollection, filter),
      span
    }
  );
}

fn parse_group_locaiton_collection(input: ParseStream) -> Result<SGroup> {
  let locationcollection = input.parse::<SLocationCollection>()?;
  let span = locationcollection.span;
  
  return Ok(
    SGroup {
      node: Group::LocationCollection(locationcollection),
      span
    }
  );
}

fn parse_group_locaiton_where(input: ParseStream) -> Result<SGroup> {
  let location = input.parse::<SID>()?;
  let start_span = location.span;
  input.parse::<Token![where]>()?;
  let filter: SFilterExpr = input.parse()?;

  let span = start_span
    .join(filter.span)
    .unwrap_or(start_span);

  return Ok(
    SGroup {
      node: Group::LocationWhere(location, filter),
      span
    }
  );
}

fn parse_group_locaiton(input: ParseStream) -> Result<SGroup> {
  let location = input.parse::<SID>()?;
  
  let span = location.span;

  return Ok(
    SGroup {
      node: Group::Location(location),
      span
    }
  )
}

// ===========================================================================


// CardSet
// ===========================================================================
impl Parse for SCardSet {
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

fn parse_cardset_group_of_player(input: ParseStream) -> Result<SCardSet> {
  let group = input.parse::<SGroup>()?;
  let start_span = group.span;
  input.parse::<kw::of>()?;
  let player = input.parse::<SPlayerExpr>()?;

  let span = start_span
    .join(player.span)
    .unwrap_or(start_span);

  return Ok(
    SCardSet {
      node: CardSet::GroupOfPlayer(group, player),
      span
    }
  )
}

fn parse_cardset_group_of_player_collection(input: ParseStream) -> Result<SCardSet> {
  let group = input.parse::<SGroup>()?;
  let start_span = group.span;
  input.parse::<kw::of>()?;
  let playercollection = input.parse::<SPlayerCollection>()?;

  let span = start_span
    .join(playercollection.span)
    .unwrap_or(start_span);

  return Ok(
    SCardSet {
      node: CardSet::GroupOfPlayerCollection(group, playercollection),
      span
    }
  )
}

fn parse_cardset_group(input: ParseStream) -> Result<SCardSet> {
  let group = input.parse::<SGroup>()?;

  let span = group.span;

  return Ok(
    SCardSet {
      node: CardSet::Group(group),
      span
    }
  )
}

// ===========================================================================


// IntCollection
// ===========================================================================
impl Parse for SIntCollection {
  fn parse(input: ParseStream) -> Result<Self> {
      let content;
      parenthesized!(content in input);
      let ints: Punctuated<SIntExpr, Token![,]> =
          content.parse_terminated(SIntExpr::parse, Token![,])?;

      let span = content.span();

      return Ok(
        SIntCollection {
          node: 
            IntCollection {
              ints: ints.into_iter().collect()
            },
          span,
        }
      )
  }
}

// ===========================================================================


// LocationCollection
// ===========================================================================
impl Parse for SLocationCollection {
  fn parse(input: ParseStream) -> Result<Self> {
    let content;
    parenthesized!(content in input);
    let locations: Punctuated<SID, Token![,]> =
        content.parse_terminated(SID::parse, Token![,])?;

    let span = content.span();

    return Ok(
      SLocationCollection {
        node: 
          LocationCollection {
            locations: locations.into_iter().collect()
          },
        span,
      }
    )
  }
}

// ===========================================================================


// TeamCollection
// ===========================================================================
impl Parse for STeamCollection {
  fn parse(input: ParseStream) -> Result<Self> {
    parse_with_alternatives(input, 
    &[
      parse_team_collection_other_teams,
      parse_team_collection_team,
    ]
  )
  }
}

fn parse_team_collection_other_teams(input: ParseStream) -> Result<STeamCollection> {
  let other = input.parse::<kw::other>()?;
  let start_span = other.span;
  let teams = input.parse::<kw::teams>()?;

  let span = start_span
    .join(teams.span)
    .unwrap_or(start_span);

  return Ok(
    STeamCollection {
      node: TeamCollection::OtherTeams,
      span
    }
  )
}

fn parse_team_collection_team(input: ParseStream) -> Result<STeamCollection> {
  let content;
  parenthesized!(content in input);
  let teams: Punctuated<STeamExpr, Token![,]> =
      content.parse_terminated(STeamExpr::parse, Token![,])?;

  let span = content.span();

  return Ok(
    STeamCollection {
      node: TeamCollection::Team(teams.into_iter().collect()),
      span,
    }
  )
}

// ===========================================================================


// StringCollection
// ===========================================================================
impl Parse for SStringCollection {
  fn parse(input: ParseStream) -> Result<Self> {
    let content;
    parenthesized!(content in input);
    let strings: Punctuated<SStringExpr, Token![,]> =
        content.parse_terminated(SStringExpr::parse, Token![,])?;

   let span = content.span();

    return Ok(
      SStringCollection {
        node: 
          StringCollection {
            strings: strings.into_iter().collect()
          },
        span,
      }
    )
  }
}

// ===========================================================================


// Collection
// TODO: maybe work out how to do it without ambiguity and no 'types' in the front
// ===========================================================================
impl Parse for SCollection {
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

fn parse_collection_player_collection(input: ParseStream) -> Result<SCollection> {
  let playercollection = input.parse::<SPlayerCollection>()?;

  let span = playercollection.span;

  match &playercollection.node {
    PlayerCollection::Player(players) => {
      // check if parsing is ambiguous
      for player in players.iter().map(|p| p.node.clone()) {
        // break if is not ambiguous
        if !matches!(player, PlayerExpr::PlayerName(_)) {
          return Ok(
            SCollection {
              node: Collection::PlayerCollection(playercollection),
              span
            }
          )
        }
      }

      // return if parsing ambiguous
      return Err(input.error("Ambiguous parsing!"))
    },
    _ => {}
  }

  return Ok(
    SCollection {
      node: Collection::PlayerCollection(playercollection),
      span
    }
  )
}

fn parse_collection_team_collection(input: ParseStream) -> Result<SCollection> {
  let teamcollection = input.parse::<STeamCollection>()?;

  let span = teamcollection.span;

  match &teamcollection.node {
    TeamCollection::Team(teams) => {
      // check if parsing is ambiguous
      for team in teams.iter().map(|t| t.node.clone()) {
        // break if is not ambiguous
        if !matches!(team, TeamExpr::TeamName(_)) {
          return Ok(
            SCollection {
              node: Collection::TeamCollection(teamcollection),
              span
            }
          )
        }
      }

      // return if parsing ambiguous
      return Err(input.error("Ambiguous parsing!"))
    },
    _ => {}
  }

  return Ok(
    SCollection {
      node: Collection::TeamCollection(teamcollection),
      span
    }
  )
}

fn parse_collection_int_collection(input: ParseStream) -> Result<SCollection> {
  let intcollection = input.parse::<SIntCollection>()?;

  let span = intcollection.span;

  return Ok(
    SCollection {
      node: Collection::IntCollection(intcollection),
      span
    }
  )
}

// Collection for LocationCollection is ambiguous to StringCollection
// fn parse_collection_location_collection(input: ParseStream) -> Result<Collection> {
//   let locationcollection = input.parse::<LocationCollection>()?;
//
//   // Always ambiguous with StringExpr
//   return Ok(Collection::LocationCollection(locationcollection))
// }

fn parse_collection_cardset(input: ParseStream) -> Result<SCollection> {
  let cardset = input.parse::<SCardSet>()?;

  let span = cardset.span;

  match &cardset.node {
    CardSet::Group(group) => {
      // check if parsing is ambiguous
      if !matches!(group.node, Group::LocationCollection(_))
        && !matches!(group.node, Group::Location(_))
      {
        return Ok(
          SCollection {
            node: Collection::CardSet(Box::new(cardset)),
            span
          }
        )
      }
    
      // return if parsing ambiguous
      return Err(input.error("Ambiguous parsing!"))
    },
    _ => {}
  }

  return Ok(
    SCollection {
      node: Collection::CardSet(Box::new(cardset)),
      span
    }
  )
}

fn parse_collection_string_collection(input: ParseStream) -> Result<SCollection> {
  let stringcollection = input.parse::<SStringCollection>()?;

  let span =  stringcollection.span;

  return Ok(
    SCollection {
      node: Collection::StringCollection(stringcollection),
      span
    }
  )
}

fn parse_collection_ambiguous(input: ParseStream) -> Result<SCollection> {
  let content;
  parenthesized!(content in input);
  let ids: Punctuated<SID, Token![,]> =
      content.parse_terminated(SID::parse, Token![,])?;

  let span = content.span();

  return Ok(
    SCollection {
      node: Collection::Ambiguous(ids.into_iter().map(|id| id).collect()),
      span
    }
  )
}

// ===========================================================================


// Reptitions
// ===========================================================================
impl Parse for SRepititions {
  fn parse(input: ParseStream) -> Result<Self> {
      let int = input.parse::<SIntExpr>()?;
      let start_span = int.span;

      let times = input.parse::<kw::times>()?;

      let span = start_span
        .join(times.span)
        .unwrap_or(start_span);

      return Ok(
        SRepititions {
          node: Repititions { times: int},
          span
        }
      )
  }
}

// ===========================================================================


// EndCondition
// ===========================================================================
impl Parse for SEndCondition {
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

fn parse_endcondition_until_end(input: ParseStream) -> Result<SEndCondition> {
  let until = input.parse::<kw::until>()?;
  let start_span = until.span();
  let content;
  parenthesized!(content in input);
  let end = content.parse::<kw::end>()?;

  let span = start_span
    .join(end.span())
    .unwrap_or(start_span);

  return Ok(
    SEndCondition {
      node: EndCondition::UntilEnd,
      span
    }
  )
}

fn parse_endcondition_until_repition(input: ParseStream) -> Result<SEndCondition> {
  let until = input.parse::<kw::until>()?;
  let start_span = until.span();

  let content;
  parenthesized!(content in input);
  let reps = content.parse::<SRepititions>()?;

  let span = start_span
    .join(reps.span)
    .unwrap_or(start_span);

  return Ok(
    SEndCondition {
      node: EndCondition::UntilRep(reps),
      span
    }
  )
}

fn parse_endcondition_until_bool(input: ParseStream) -> Result<SEndCondition> {
  let until = input.parse::<kw::until>()?;
  let start_span = until.span();
  let content;
  parenthesized!(content in input);
  let boolexpr = content.parse::<SBoolExpr>()?;

  let span = start_span
    .join(boolexpr.span)
    .unwrap_or(start_span);

  return Ok(
    SEndCondition {
      node: EndCondition::UntilBool(boolexpr),
      span
    } 
  )
}

fn parse_endcondition_until_bool_and_rep(input: ParseStream) -> Result<SEndCondition> {
  let until = input.parse::<kw::until>()?;
  let start_span = until.span();
  let content;
  parenthesized!(content in input);
  let boolexpr = content.parse::<SBoolExpr>()?;
  content.parse::<kw::and>()?;
  let reps = content.parse::<SRepititions>()?;

  let span = start_span
    .join(reps.span)
    .unwrap_or(start_span);

  return Ok(
    SEndCondition {
      node: EndCondition::UntilBoolAndRep(boolexpr, reps),
      span
    }
  )
}

fn parse_endcondition_until_bool_or_rep(input: ParseStream) -> Result<SEndCondition> {
  let until = input.parse::<kw::until>()?;
  let start_span = until.span();
  let content;
  parenthesized!(content in input);
  let boolexpr = content.parse::<SBoolExpr>()?;
  content.parse::<kw::or>()?;
  let reps = content.parse::<SRepititions>()?;

  let span = start_span
    .join(reps.span)
    .unwrap_or(start_span);

  return Ok(
    SEndCondition {
      node: EndCondition::UntilBoolOrRep(boolexpr, reps),
      span
    }
  )
}

// ===========================================================================


// IntRange
// ===========================================================================
impl Parse for SIntRange {
  fn parse(input: ParseStream) -> Result<Self> {
    // input.parse::<kw::range>()?;
    // let content;
    // parenthesized!(content in input);
    let op = input.parse::<SIntCmpOp>()?;
    let start_span = op.span;
    let int = input.parse::<SIntExpr>()?;

    let span = start_span
      .join(int.span)
      .unwrap_or(start_span);

    return Ok(
      SIntRange {
        node: IntRange {op: op, int: int},
        span
      }
    )
  }
}

// ===========================================================================


// Quantity
// ===========================================================================
impl Parse for SQuantity {
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

fn parse_quantity_int(input: ParseStream) -> Result<SQuantity> {
  let int = input.parse::<SIntExpr>()?;
  
  let span = int.span;

  return Ok(
    SQuantity {
      node: Quantity::Int(int),
      span
    }
  )
}

fn parse_quantity_intrange(input: ParseStream) -> Result<SQuantity> {
  let intrange = input.parse::<SIntRange>()?;
  
  let span = intrange.span;

  return Ok(
    SQuantity {
      node: Quantity::IntRange(intrange),
      span
    }
  )
}

fn parse_quantity_quantifier(input: ParseStream) -> Result<SQuantity> {
  let quantifier = input.parse::<SQuantifier>()?;
  
  let span = quantifier.span;

  return Ok(
    SQuantity {
      node: Quantity::Quantifier(quantifier),
      span
    }
  )
}

// ===========================================================================


// ClassicMove
// ===========================================================================
impl Parse for SClassicMove {
  fn parse(input: ParseStream) -> Result<Self> {
    parse_with_alternatives(input, 
      &[
        parse_classic_move_quantity,
        parse_classic_move_move,
      ]
    )
  }
}

fn parse_classic_move_quantity(input: ParseStream) -> Result<SClassicMove> {
  let mv = input.parse::<Token![move]>()?;
  let start_span = mv.span();
  let quantity = input.parse::<SQuantity>()?;
  input.parse::<kw::from>()?;
  let from_cardset = input.parse::<SCardSet>()?;
  let status = input.parse::<SStatus>()?;
  input.parse::<kw::to>()?;
  let to_cardset = input.parse::<SCardSet>()?;

  let span = start_span
    .join(to_cardset.span)
    .unwrap_or(start_span);

  return Ok(
    SClassicMove {
      node: ClassicMove::MoveQuantity(quantity, from_cardset, status, to_cardset),
      span
    }
  )
}

fn parse_classic_move_move(input: ParseStream) -> Result<SClassicMove> {
  let mv = input.parse::<Token![move]>()?;
  let start_span = mv.span();
  let from_cardset = input.parse::<SCardSet>()?;
  let status = input.parse::<SStatus>()?;
  input.parse::<kw::to>()?;
  let to_cardset = input.parse::<SCardSet>()?;

  let span = start_span
    .join(to_cardset.span)
    .unwrap_or(start_span);

  return Ok(
    SClassicMove {
      node: ClassicMove::Move(from_cardset, status, to_cardset),
      span
    }
  )
}

// ===========================================================================


// DealMove
// ===========================================================================
impl Parse for SDealMove {
  fn parse(input: ParseStream) -> Result<Self> {  
    parse_with_alternatives(input, 
      &[
        parse_deal_move_quantity,
        parse_deal_move_deal,
      ]
    )
  }
}

fn parse_deal_move_quantity(input: ParseStream) -> Result<SDealMove> {
  let deal = input.parse::<kw::deal>()?;
  let start_span = deal.span();
  let quantity = input.parse::<SQuantity>()?;
  input.parse::<kw::from>()?;
  let from_cardset = input.parse::<SCardSet>()?;
  let status = input.parse::<SStatus>()?;
  input.parse::<kw::to>()?;
  let to_cardset = input.parse::<SCardSet>()?;

  let span = start_span
    .join(to_cardset.span)
    .unwrap_or(start_span);

  return Ok(
    SDealMove {
      node: DealMove::DealQuantity(quantity, from_cardset, status, to_cardset),
      span
    }
  )
}

fn parse_deal_move_deal(input: ParseStream) -> Result<SDealMove> {
  let deal = input.parse::<kw::deal>()?;
  let start_span = deal.span();
  let from_cardset = input.parse::<SCardSet>()?;
  let status = input.parse::<SStatus>()?;
  input.parse::<kw::to>()?;
  let to_cardset = input.parse::<SCardSet>()?;

  let span = start_span
    .join(to_cardset.span)
    .unwrap_or(start_span);

  return Ok(
    SDealMove {
      node: DealMove::Deal(from_cardset, status, to_cardset),
      span
    }
  )
}

// ===========================================================================


// ExchangeMove
// ===========================================================================
impl Parse for SExchangeMove {
  fn parse(input: ParseStream) -> Result<Self> {
      parse_with_alternatives(input,
      &[
        parse_exchange_move_quantity,
        parse_exchange_move_exchange,
      ]
    )
  }
}

fn parse_exchange_move_quantity(input: ParseStream) -> Result<SExchangeMove> {
  let exchange = input.parse::<kw::exchange>()?;
  let start_span = exchange.span();

  let quantity = input.parse::<SQuantity>()?;
  input.parse::<kw::from>()?;
  let from_cardset = input.parse::<SCardSet>()?;
  let status = input.parse::<SStatus>()?;
  input.parse::<kw::with>()?;
  let to_cardset = input.parse::<SCardSet>()?;

  let span = start_span
    .join(to_cardset.span)
    .unwrap_or(start_span);

  return Ok(
    SExchangeMove {
      node: ExchangeMove::ExchangeQuantity(quantity, from_cardset, status, to_cardset),
      span
    }
  )
}

fn parse_exchange_move_exchange(input: ParseStream) -> Result<SExchangeMove> {
  let exchange = input.parse::<kw::exchange>()?;
  let start_span = exchange.span();

  let from_cardset = input.parse::<SCardSet>()?;
  let status = input.parse::<SStatus>()?;
  input.parse::<kw::with>()?;
  let to_cardset = input.parse::<SCardSet>()?;

  let span = start_span
    .join(to_cardset.span)
    .unwrap_or(start_span);

  return Ok(
    SExchangeMove {
      node: ExchangeMove::Exchange(from_cardset, status, to_cardset),
      span
    }
  )
}

// ===========================================================================


// TokenLocExpr
// ===========================================================================
impl Parse for STokenLocExpr {
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

fn parse_token_loc_expr_location(input: ParseStream) -> Result<STokenLocExpr> {
  let location = input.parse::<SID>()?;

  let span = location.span;

  return Ok(
    STokenLocExpr {
      node: TokenLocExpr::Location(location),
      span
    }
  )
}

fn parse_token_loc_expr_location_player(input: ParseStream) -> Result<STokenLocExpr> {
  let location = input.parse::<SID>()?;
  let start_span = location.span;

  input.parse::<kw::of>()?;
  let player = input.parse::<SPlayerExpr>()?;

  let span = start_span
    .join(player.span)
    .unwrap_or(start_span);
      
  return Ok(
    STokenLocExpr {
      node: TokenLocExpr::LocationPlayer(location, player),
      span
    }
  )
}

fn parse_token_loc_expr_location_player_collection(input: ParseStream) -> Result<STokenLocExpr> {
  let location = input.parse::<SID>()?;
  let start_span = location.span;

  input.parse::<kw::of>()?;
  let player_collection = input.parse::<SPlayerCollection>()?;

  let span = start_span
    .join(player_collection.span)
    .unwrap_or(start_span);
      
  return Ok(
    STokenLocExpr {
      node: TokenLocExpr::LocationPlayerCollection(location, player_collection),
      span
    }
  )
}

fn parse_token_loc_collection_location(input: ParseStream) -> Result<STokenLocExpr> {
  let location_collection = input.parse::<SLocationCollection>()?;
  
  let span = location_collection.span;

  return Ok(
    STokenLocExpr {
      node: TokenLocExpr::LocationCollection(location_collection),
      span
    }
  )
}

fn parse_token_loc_collection_location_player(input: ParseStream) -> Result<STokenLocExpr> {
  let location_collection = input.parse::<SLocationCollection>()?;
  let start_span = location_collection.span;

  input.parse::<kw::of>()?;
  let player = input.parse::<SPlayerExpr>()?;

  let span = start_span
    .join(player.span)
    .unwrap_or(start_span);
      
  return Ok(
    STokenLocExpr {
      node: TokenLocExpr::LocationCollectionPlayer(location_collection, player),
      span
    }
  )
}

fn parse_token_loc_collection_location_player_collection(input: ParseStream) -> Result<STokenLocExpr> {
  let location_collection = input.parse::<SLocationCollection>()?;
  let start_span = location_collection.span;

  input.parse::<kw::of>()?;
  let player_collection = input.parse::<SPlayerCollection>()?;

  let span = start_span
    .join(player_collection.span)
    .unwrap_or(start_span);
      
  return Ok(
    STokenLocExpr {
      node: TokenLocExpr::LocationCollectionPlayerCollection(location_collection, player_collection),
      span
    }
  )
}

// ===========================================================================


// TokenMove
// ===========================================================================
impl Parse for STokenMove {
  fn parse(input: ParseStream) -> Result<Self> {
      parse_with_alternatives(input,
        &[
          parse_token_move_place_quantity,
          parse_token_move_place,
        ]
      )
  }
}

fn parse_token_move_place_quantity(input: ParseStream) -> Result<STokenMove> {
  let place = input.parse::<kw::place>()?;
  let start_span = place.span;

  let quantity = input.parse::<SQuantity>()?;
  let token = input.parse::<SID>()?;
  input.parse::<kw::from>()?;
  let from_tokenloc = input.parse::<STokenLocExpr>()?;
  input.parse::<kw::to>()?;
  let to_tokenloc = input.parse::<STokenLocExpr>()?;

  let span = start_span
    .join(to_tokenloc.span)
    .unwrap_or(start_span);

  return Ok(
    STokenMove {
      node: TokenMove::PlaceQuantity(quantity, token, from_tokenloc, to_tokenloc),
      span
    }
  )
}

fn parse_token_move_place(input: ParseStream) -> Result<STokenMove> {
  let place = input.parse::<kw::place>()?;
  let start_span = place.span;

  let token = input.parse::<SID>()?;
  let from_tokenloc = input.parse::<STokenLocExpr>()?;
  input.parse::<kw::to>()?;
  let to_tokenloc = input.parse::<STokenLocExpr>()?;

  let span = start_span
    .join(to_tokenloc.span)
    .unwrap_or(start_span);

  return Ok(
    STokenMove {
      node: TokenMove::Place(token, from_tokenloc, to_tokenloc),
      span
    }
  )
}

// ===========================================================================


// Rule
// ===========================================================================
impl Parse for SRule {
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
      ]
    )
  }
}

fn parse_create_players(input: ParseStream) -> Result<SRule> {
  let players = input.parse::<kw::players>()?;
  let start_span = players.span();
  input.parse::<Token![:]>()?;
  let content;
  parenthesized!(content in input);
  let players: Punctuated<SID, Token![,]> =
      content.parse_terminated(SID::parse, Token![,])?;
  
  let span = start_span
    .join(content.span())
    .unwrap_or(start_span);


  return Ok(
    SRule {
      node: Rule::CreatePlayer(players.into_iter().collect()),
      span
    }
  )
}

fn parse_create_team(input: ParseStream) -> Result<SRule> {
  let team = input.parse::<kw::team>()?;
  let start_span = team.span();

  let teamname = input.parse::<SID>()?;
  input.parse::<Token![:]>()?;
  let content;
  parenthesized!(content in input);
  let players: Punctuated<SID, Token![,]> =
      content.parse_terminated(SID::parse, Token![,])?;

  let span = start_span
    .join(content.span())
    .unwrap_or(start_span);

  return Ok(
    SRule {
      node: Rule::CreateTeam(teamname, players.into_iter().collect()),
      span
    }
  )
}

fn parse_create_turnorder_random(input: ParseStream) -> Result<SRule> {
  let random = input.parse::<kw::random>()?;
  let start_span = random.span();

  input.parse::<kw::turnorder>()?;
  input.parse::<Token![:]>()?;
  let content;
  parenthesized!(content in input);
  let players: Punctuated<SID, Token![,]> =
      content.parse_terminated(SID::parse, Token![,])?;

  let span = start_span
    .join(content.span())
    .unwrap_or(start_span);

  return Ok(
    SRule {
      node: Rule::CreateTurnorderRandom(players.into_iter().collect()),
      span
    }
  )
}

fn parse_create_turnorder(input: ParseStream) -> Result<SRule> {
  let turnorder = input.parse::<kw::turnorder>()?;
  let start_span = turnorder.span();
  
  input.parse::<Token![:]>()?;
  let content;
  parenthesized!(content in input);
  let players: Punctuated<SID, Token![,]> =
      content.parse_terminated(SID::parse, Token![,])?;

  let span = start_span
    .join(content.span())
    .unwrap_or(start_span);

  return Ok(
    SRule {
      node: Rule::CreateTurnorder(players.into_iter().collect()),
      span
    }
  )
}

fn parse_create_location_collection_on_player_collection(input: ParseStream) -> Result<SRule> {
  let location = input.parse::<kw::location>()?;
  let start_span = location.span();

  let locationcollection = input.parse::<SLocationCollection>()?;
  input.parse::<kw::on>()?;
  let playercollection = input.parse::<SPlayerCollection>()?;

  let span = start_span
    .join(playercollection.span)
    .unwrap_or(start_span);

  return Ok(
    SRule {
      node: Rule::CreateLocationCollectionOnPlayerCollection(
        locationcollection, playercollection
      ),
      span
    }
  )
}

fn parse_create_location_collection_on_team_collection(input: ParseStream) -> Result<SRule> {
  let location = input.parse::<kw::location>()?;
  let start_span = location.span();
  
  let locationcollection = input.parse::<SLocationCollection>()?;
  input.parse::<kw::on>()?;
  input.parse::<kw::teams>()?;
  let teamcollection = input.parse::<STeamCollection>()?;

  let span = start_span
    .join(teamcollection.span)
    .unwrap_or(start_span);

  return Ok(
    SRule {
      node: Rule::CreateLocationCollectionOnTeamCollection(
        locationcollection, teamcollection
      ),
      span
    }
  )
}

fn parse_create_location_collection_on_table(input: ParseStream) -> Result<SRule> {
  let location = input.parse::<kw::location>()?;
  let start_span = location.span();
  
  let locationcollection = input.parse::<SLocationCollection>()?;
  input.parse::<kw::on>()?;
  let table = input.parse::<kw::table>()?;

  let span = start_span
    .join(table.span())
    .unwrap_or(start_span);

  return Ok(
    SRule {
      node: Rule::CreateLocationCollectionOnTable(
        locationcollection
      ),
      span
    }
  ) 
}

fn parse_create_location_on_player_collection(input: ParseStream) -> Result<SRule> {
  let loc = input.parse::<kw::location>()?;
  let start_span = loc.span();

  let location = input.parse::<SID>()?;
  input.parse::<kw::on>()?;
  let playercollection = input.parse::<SPlayerCollection>()?;

  let span = start_span
    .join(playercollection.span)
    .unwrap_or(start_span);

  return Ok(
    SRule {
      node: Rule::CreateLocationOnPlayerCollection(
        location, playercollection
      ),
      span
    }
  )
}

fn parse_create_location_on_team_collection(input: ParseStream) -> Result<SRule> {
  let loc = input.parse::<kw::location>()?;
  let start_span = loc.span();

  let location = input.parse::<SID>()?;
  input.parse::<kw::on>()?;
  input.parse::<kw::teams>()?;
  let teamcollection = input.parse::<STeamCollection>()?;
  
  let span = start_span
    .join(teamcollection.span)
    .unwrap_or(start_span);

  return Ok(
    SRule {
      node: Rule::CreateLocationOnTeamCollection(
        location, teamcollection
      ),
      span
    }
  )
}

fn parse_create_location_on_table(input: ParseStream) -> Result<SRule> {
  let loc = input.parse::<kw::location>()?;
  let start_span = loc.span();

  let location = input.parse::<SID>()?;
  input.parse::<kw::on>()?;
  let table = input.parse::<kw::table>()?;

  let span = start_span
    .join(table.span())
    .unwrap_or(start_span);
  
  return Ok(
    SRule {
      node: Rule::CreateLocationOnTable(
        location
      ),
      span
    }
  )
}

fn parse_create_card_on_location(input: ParseStream) -> Result<SRule> {
  let card = input.parse::<kw::card>()?;
  let start_span = card.span();

  input.parse::<kw::on>()?;
  let location= input.parse::<SID>()?;
  input.parse::<Token![:]>()?;
  let types = input.parse::<STypes>()?;

  let span = start_span
    .join(types.span)
    .unwrap_or(start_span);

  return Ok(
    SRule {
      node: Rule::CreateCardOnLocation(location, types),
      span
    }
  )
}

fn parse_create_token_on_location(input: ParseStream) -> Result<SRule> {
  let token = input.parse::<kw::token>()?;
  let start_span = token.span();

  let amount = input.parse::<SIntExpr>()?;
  let token_type = input.parse::<SID>()?;
  input.parse::<kw::on>()?;
  let location = input.parse::<SID>()?;

  let span = start_span
    .join(location.span)
    .unwrap_or(start_span);

  return Ok(
    SRule {
      node: Rule::CreateTokenOnLocation(
        amount, token_type, location
      ),
      span
    }
  )
}

fn parse_create_precedence(input: ParseStream) -> Result<SRule> {
  let prec = input.parse::<kw::precedence>()?;
  let start_span = prec.span();

  let precedence = input.parse::<SID>()?;
  
  if input.peek(kw::on) {
    input.parse::<kw::on>()?;
    let key = input.parse::<SID>()?;
    let content;
    parenthesized!(content in input);
    let values: Punctuated<SID, Token![,]> =
      content.parse_terminated(SID::parse, Token![,])?;

    let key_value_pairs = values.into_iter().map(|v| (key.clone(), v)).collect();

    let span = start_span
      .join(content.span())
      .unwrap_or(start_span);

    return Ok(
      SRule {
        node: Rule::CreatePrecedence(precedence, key_value_pairs),
        span
      } 
    )
  }

  let content;
  parenthesized!(content in input);
  let mut key_value_pairs = Vec::new();
  while !content.is_empty() {
    let key = content.parse::<SID>()?;
    let in_content;
    parenthesized!(in_content in content);
    let value = in_content.parse::<SID>()?;
    key_value_pairs.push((key, value));

    if content.peek(Token![,]) {
      content.parse::<Token![,]>()?;

      continue
    }
    
    break
  }

  let span = start_span
      .join(content.span())
      .unwrap_or(start_span);

  return Ok(
    SRule {
      node: Rule::CreatePrecedence(precedence, key_value_pairs),
      span
    } 
  )
}

fn parse_create_pointmap(input: ParseStream) -> Result<SRule> {
  let point = input.parse::<kw::pointmap>()?;
  let start_span = point.span();

  let pointmap = input.parse::<SID>()?;
  
  if input.peek(kw::on) {
    input.parse::<kw::on>()?;
    let key = input.parse::<SID>()?;
    let content;
    parenthesized!(content in input);
    let mut key_value_int_triples = Vec::new();
    
    while !content.is_empty() {
      let value = content.parse::<SID>()?;
      content.parse::<Token![:]>()?;
      let int = content.parse::<SIntExpr>()?;
      key_value_int_triples.push((key.clone(), value, int));

      if content.peek(Token![,]) {
        content.parse::<Token![,]>()?;
        
        continue
      }
      
      break
    }

    let span = start_span
      .join(content.span())
      .unwrap_or(start_span);

    return Ok(
      SRule {
        node: Rule::CreatePointMap(pointmap, key_value_int_triples),
        span
      } 
    )
  }

  let content;
  parenthesized!(content in input);
  let mut key_value_int_triples = Vec::new();
  
  while !content.is_empty() {
    let key = content.parse::<SID>()?;
    let in_content;
    parenthesized!(in_content in content);
    let value = in_content.parse::<SID>()?;
    in_content.parse::<Token![:]>()?;
    let int = in_content.parse::<SIntExpr>()?;
    key_value_int_triples.push((key.clone(), value, int));

    if content.peek(Token![,]) {
      content.parse::<Token![,]>()?;
      
      continue
    }
    
    break
  }

  let span = start_span
    .join(content.span())
    .unwrap_or(start_span);


  return Ok(
    SRule {
      node: Rule::CreatePointMap(pointmap, key_value_int_triples),
      span
    } 
  )
}

fn parse_create_combo(input: ParseStream) -> Result<SRule> {
  let com = input.parse::<kw::combo>()?;
  let start_span = com.span();

  let combo = input.parse::<SID>()?;
  input.parse::<Token![where]>()?;
  let filter = input.parse::<SFilterExpr>()?;

  let span = start_span
    .join(filter.span)
    .unwrap_or(start_span);

  return Ok(
    SRule {
      node: Rule::CreateCombo(combo, filter),
      span
    }
  )
}

fn parse_create_memory_table(input: ParseStream) -> Result<SRule> {
  let mem = input.parse::<kw::memory>()?;
  let start_span = mem.span();

  let memory = input.parse::<SID>()?;
  input.parse::<kw::on>()?;
  let table = input.parse::<kw::table>()?;

  let span = start_span
    .join(table.span())
    .unwrap_or(start_span);

  return Ok(
    SRule {
      node: Rule::CreateMemoryTable(memory),
      span
    } 
  )
}

fn parse_create_memory_player_collection(input: ParseStream) -> Result<SRule> {
  let mem = input.parse::<kw::memory>()?;
  let start_span = mem.span();

  let memory = input.parse::<SID>()?;
  input.parse::<kw::on>()?;
  let player_collection = input.parse::<SPlayerCollection>()?;

  let span = start_span
    .join(player_collection.span)
    .unwrap_or(start_span);

  return Ok(
    SRule {
      node: Rule::CreateMemoryPlayerCollection(memory, player_collection),
      span
    } 
  )             
}

fn parse_create_memory_int_table(input: ParseStream) -> Result<SRule> {
  let mem = input.parse::<kw::memory>()?;
  let start_span = mem.span();

  let memory = input.parse::<SID>()?;
  let int = input.parse::<SIntExpr>()?;
  input.parse::<kw::on>()?;
  let table = input.parse::<kw::table>()?;

  let span = start_span
    .join(table.span())
    .unwrap_or(start_span);

  return Ok(
    SRule {
      node: Rule::CreateMemoryIntTable(memory, int),
      span
    } 
  )
}

fn parse_create_memory_int_player_collection(input: ParseStream) -> Result<SRule> {
  let mem = input.parse::<kw::memory>()?;
  let start_span = mem.span();

  let memory = input.parse::<SID>()?;
  let int = input.parse::<SIntExpr>()?;
  input.parse::<kw::on>()?;
  let player_collection = input.parse::<SPlayerCollection>()?;

  let span = start_span
    .join(player_collection.span)
    .unwrap_or(start_span);

  return Ok(
    SRule {
      node: Rule::CreateMemoryIntPlayerCollection(memory, int, player_collection),
      span
    } 
  )  
}

fn parse_create_memory_string_table(input: ParseStream) -> Result<SRule> {
  let mem = input.parse::<kw::memory>()?;
  let start_span = mem.span();

  let memory = input.parse::<SID>()?;
  let string = input.parse::<SStringExpr>()?;
  input.parse::<kw::on>()?;
  let table = input.parse::<kw::table>()?;

  let span = start_span
    .join(table.span())
    .unwrap_or(start_span);

  return Ok(
    SRule {
      node: Rule::CreateMemoryStringTable(memory, string),
      span
    } 
  )
}

fn parse_create_memory_string_player_collection(input: ParseStream) -> Result<SRule> {
  let mem = input.parse::<kw::memory>()?;
  let start_span = mem.span();

  let memory = input.parse::<SID>()?;
  let string = input.parse::<SStringExpr>()?;
  input.parse::<kw::on>()?;
  let player_collection = input.parse::<SPlayerCollection>()?;

  let span = start_span
    .join(player_collection.span)
    .unwrap_or(start_span);

  return Ok(
    SRule {
      node: Rule::CreateMemoryStringPlayerCollection(memory, string, player_collection),
      span
    } 
  )  
}

fn parse_flip_action(input: ParseStream) -> Result<SRule> {
  let flip = input.parse::<kw::flip>()?;
  let start_span = flip.span();

  let cardset = input.parse::<SCardSet>()?;
  input.parse::<kw::to>()?;
  let status  = input.parse::<SStatus>()?;

  let span = start_span
    .join(status.span)
    .unwrap_or(start_span);

  return Ok(
    SRule {
      node: Rule::FlipAction(cardset, status),
      span
    }
  )
}

fn parse_shuffle_action(input: ParseStream) -> Result<SRule> {
  let shuffle = input.parse::<kw::shuffle>()?;
  let start_span = shuffle.span();

  let cardset = input.parse::<SCardSet>()?;

  let span = start_span
    .join(cardset.span)
    .unwrap_or(start_span);
  
  return Ok(
    SRule {
      node: Rule::ShuffleAction(cardset),
      span
    }
  )
}

fn parse_set_player_out_of_stage(input: ParseStream) -> Result<SRule> {
  let set = input.parse::<kw::set>()?;
  let start_span = set.span();

  let player = input.parse::<SPlayerExpr>()?;
  input.parse::<kw::out>()?;
  input.parse::<kw::of>()?;
  let stage = input.parse::<kw::stage>()?;

  let span = start_span
    .join(stage.span())
    .unwrap_or(start_span);

  return Ok(
    SRule {
      node: Rule::PlayerOutOfStageAction(player),
      span
    }
  )
}

fn parse_set_player_out_of_game_succ(input: ParseStream) -> Result<SRule> {
  let set = input.parse::<kw::set>()?;
  let start_span = set.span();

  let player = input.parse::<SPlayerExpr>()?;
  input.parse::<kw::out>()?;
  input.parse::<kw::of>()?;
  input.parse::<kw::game>()?;
  let successful = input.parse::<kw::successful>()?;

  let span = start_span
    .join(successful.span())
    .unwrap_or(start_span);

  return Ok(
    SRule {
      node: Rule::PlayerOutOfGameSuccAction(player),
      span
    }
  )
}

fn parse_set_player_out_of_game_fail(input: ParseStream) -> Result<SRule> {
  let set = input.parse::<kw::set>()?;
  let start_span = set.span();

  let player = input.parse::<SPlayerExpr>()?;
  input.parse::<kw::out>()?;
  input.parse::<kw::of>()?;
  input.parse::<kw::game>()?;
  let fail = input.parse::<kw::fail>()?;

  let span = start_span
    .join(fail.span())
    .unwrap_or(start_span);

  return Ok(
    SRule {
      node: Rule::PlayerOutOfGameFailAction(player),
      span
    }
  )
}

fn parse_set_player_collection_out_of_stage(input: ParseStream) -> Result<SRule> {
  let set = input.parse::<kw::set>()?;
  let start_span = set.span();

  let player_collection = input.parse::<SPlayerCollection>()?;
  input.parse::<kw::out>()?;
  input.parse::<kw::of>()?;
  let stage = input.parse::<kw::stage>()?;

  let span = start_span
    .join(stage.span())
    .unwrap_or(start_span);

  return Ok(
    SRule {
      node: Rule::PlayerCollectionOutOfStageAction(player_collection),
      span
    }
  )
}

fn parse_set_player_collection_out_of_game_succ(input: ParseStream) -> Result<SRule> {
  let set = input.parse::<kw::set>()?;
  let start_span = set.span();

  let player_collection = input.parse::<SPlayerCollection>()?;
  input.parse::<kw::out>()?;
  input.parse::<kw::of>()?;
  input.parse::<kw::game>()?;
  let successful = input.parse::<kw::successful>()?;

  let span = start_span
    .join(successful.span())
    .unwrap_or(start_span);

  return Ok(
    SRule {
      node: Rule::PlayerCollectionOutOfGameSuccAction(player_collection),
      span
    }
  )
}

fn parse_set_player_collection_out_of_game_fail(input: ParseStream) -> Result<SRule> {
  let set = input.parse::<kw::set>()?;
  let start_span = set.span();

  let player_collection = input.parse::<SPlayerCollection>()?;
  input.parse::<kw::out>()?;
  input.parse::<kw::of>()?;
  input.parse::<kw::game>()?;
  let fail = input.parse::<kw::fail>()?;

  let span = start_span
    .join(fail.span())
    .unwrap_or(start_span);

  return Ok(
    SRule {
      node: Rule::PlayerCollectionOutOfGameFailAction(player_collection),
      span
    }
  )
}

fn parse_cycle_action(input: ParseStream) -> Result<SRule> {
  let cycle = input.parse::<kw::cycle>()?;
  let start_span = cycle.span();

  input.parse::<kw::to>()?;
  let player = input.parse::<SPlayerExpr>()?;
  
  let span = start_span
    .join(player.span)
    .unwrap_or(start_span);

  return Ok(
    SRule {
      node: Rule::CycleAction(player),
      span
    }
  )
}

fn parse_bid_action(input: ParseStream) -> Result<SRule> {
  let bid = input.parse::<kw::bid>()?;
  let start_span = bid.span();

  let quantity = input.parse::<SQuantity>()?;
  
  let span = start_span
    .join(quantity.span)
    .unwrap_or(start_span);

  return Ok(
    SRule {
      node: Rule::BidAction(quantity),
      span
    } 
  )
}

fn parse_bid_action_memory(input: ParseStream) -> Result<SRule> {
  let bid = input.parse::<kw::bid>()?;
  let start_span = bid.span();

  let quantity = input.parse::<SQuantity>()?;
  input.parse::<kw::on>()?;
  let memory = input.parse::<SID>()?;

  let span = start_span
    .join(memory.span)
    .unwrap_or(start_span);

  return Ok(
    SRule {
      node: Rule::BidActionMemory(memory, quantity),
      span
    } 
  )
}

fn parse_end_turn(input: ParseStream) -> Result<SRule> {
  let end = input.parse::<kw::end>()?;
  let start_span = end.span();

  let turn = input.parse::<kw::turn>()?;

  let span = start_span
    .join(turn.span())
    .unwrap_or(start_span);

  return Ok(
    SRule {
      node: Rule::EndTurn,
      span
    }
  )
}

fn parse_end_stage(input: ParseStream) -> Result<SRule> {
  let end = input.parse::<kw::end>()?;
  let start_span = end.span();

  let stage = input.parse::<kw::stage>()?;

  let span = start_span
    .join(stage.span())
    .unwrap_or(start_span);

  return Ok(
    SRule {
      node: Rule::EndStage,
      span
    }
  )
}

fn parse_end_game(input: ParseStream) -> Result<SRule> {
  let end = input.parse::<kw::end>()?;
  let start_span = end.span();

  input.parse::<kw::game>()?;
  input.parse::<kw::with>()?;
  input.parse::<kw::winner>()?;
  let player = input.parse::<SPlayerExpr>()?;

  let span = start_span
    .join(player.span)
    .unwrap_or(start_span);

  return Ok(
    SRule {
      node: Rule::EndGameWithWinner(player),
      span
    }
  )
}

fn parse_demand_cardposition_action(input: ParseStream) -> Result<SRule> {
  let demand = input.parse::<kw::demand>()?;
  let start_span = demand.span();

  let cardposition = input.parse::<SCardPosition>()?;

  let span = start_span
    .join(cardposition.span)
    .unwrap_or(start_span);

  return Ok(
    SRule {
      node: Rule::DemandCardPositionAction(cardposition),
      span
    } 
  )
}

fn parse_demand_string_action(input: ParseStream) -> Result<SRule> {
  let demand = input.parse::<kw::demand>()?;
  let start_span = demand.span();

  let string = input.parse::<SStringExpr>()?;

  let span = start_span
    .join(string.span)
    .unwrap_or(start_span);

  return Ok(
    SRule {
      node: Rule::DemandStringAction(string),
      span
    } 
  )
}

fn parse_demand_int_action(input: ParseStream) -> Result<SRule> {
  let demand = input.parse::<kw::demand>()?;
  let start_span = demand.span();

  let int = input.parse::<SIntExpr>()?;

  let span = start_span
    .join(int.span)
    .unwrap_or(start_span);

  return Ok(
    SRule {
      node: Rule::DemandIntAction(int),
      span
    } 
  )
}

fn parse_deal_move(input: ParseStream) -> Result<SRule> {
  let dealmove = input.parse::<SDealMove>()?;
  let span = dealmove.span;

  return Ok(
    SRule {
      node: Rule::DealMove(dealmove),
      span
    }
  )
}

fn parse_classic_move(input: ParseStream) -> Result<SRule> {
  let classicmove = input.parse::<SClassicMove>()?;
  let span = classicmove.span;

  return Ok(
    SRule {
      node: Rule::ClassicMove(classicmove),
      span
    }
  )
}

fn parse_exchange_move(input: ParseStream) -> Result<SRule> {
  let exchangemove = input.parse::<SExchangeMove>()?;
  let span = exchangemove.span;

  return Ok(
    SRule {
      node: Rule::ExchangeMove(exchangemove),
      span
    }
  )
}

fn parse_token_move(input: ParseStream) -> Result<SRule> {
  let tokenmove = input.parse::<STokenMove>()?;
  let span = tokenmove.span;

  return Ok(
    SRule {
      node: Rule::TokenMove(tokenmove),
      span
    }
  )
}

fn parse_score_rule(input: ParseStream) -> Result<SRule> {
  let scorerule = input.parse::<SScoreRule>()?;
  let span = scorerule.span;

  return Ok(
    SRule {
      node: Rule::ScoreRule(scorerule),
      span
    }
  )
}

fn parse_winner_rule(input: ParseStream) -> Result<SRule> {
  let winnerrule = input.parse::<SWinnerRule>()?;
  let span = winnerrule.span;

  return Ok(
    SRule {
      node: Rule::WinnerRule(winnerrule),
      span
    }
  )
}

fn parse_set_memory_collection(input: ParseStream) -> Result<SRule> {
  let memory = input.parse::<SID>()?;
  let start_span = memory.span;

  input.parse::<kw::is>()?;
  let collection = input.parse::<SCollection>()?;

  if matches!(collection.node, Collection::Ambiguous(_)) {
    return Err(input.error("Ambiguous parsing"))
  }

  let span = start_span
    .join(collection.span)
    .unwrap_or(start_span);

  return Ok(
    SRule {
      node: Rule::SetMemoryCollection(memory, collection),
      span
    }
  )
}

fn parse_set_memory_int(input: ParseStream) -> Result<SRule> {
  let memory = input.parse::<SID>()?;
  let start_span = memory.span;

  input.parse::<kw::is>()?;
  let int = input.parse::<SIntExpr>()?;

  let span = start_span
    .join(int.span)
    .unwrap_or(start_span);

  return Ok(
    SRule {
      node: Rule::SetMemoryInt(memory, int),
      span
    }
  )
}

fn parse_set_memory_string(input: ParseStream) -> Result<SRule> {
  let memory = input.parse::<SID>()?;
  let start_span = memory.span;

  input.parse::<kw::is>()?;
  let string = input.parse::<SStringExpr>()?;

  let span = start_span
    .join(string.span)
    .unwrap_or(start_span);

  return Ok(
    SRule {
      node: Rule::SetMemoryString(memory, string),
      span
    }
  )
}

// ===========================================================================


// Types
// ===========================================================================
impl Parse for STypes {
  fn parse(input: ParseStream) -> Result<Self> {
    let mut types = Vec::new();
    let key = input.parse::<SID>()?;
    let start_span = key.span;
  
    let content;
    parenthesized!(content in input);

    let values: Punctuated<SID, Token![,]> =
      content.parse_terminated(SID::parse, Token![,])?;

    types.push((key, values.into_iter().collect()));

    let mut span = start_span
      .join(content.span())
      .unwrap_or(start_span); 

    if input.peek(Token![for]) {
      input.parse::<Token![for]>()?;
      let for_types = input.parse::<STypes>()?;

      span = start_span
      .join(for_types.span)
      .unwrap_or(start_span); 

      types.extend_from_slice(&for_types.node.types);
    }

    return Ok(
      STypes {
        node: Types { types: types },
        span
      }
    )
  }
}

// ===========================================================================


// ScoreRule
// ===========================================================================
impl Parse for SScoreRule {
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

fn parse_score_player_collection_memory(input: ParseStream) -> Result<SScoreRule> {
  let score = input.parse::<kw::score>()?;
  let start_span = score.span();
  
  let int = input.parse::<SIntExpr>()?;
  input.parse::<kw::to>()?;
  let memory = input.parse::<SID>()?;
  input.parse::<kw::of>()?;
  let playercollection = input.parse::<SPlayerCollection>()?;

  let span = start_span
    .join(playercollection.span)
    .unwrap_or(start_span);

  return Ok(
    SScoreRule {
      node: ScoreRule::ScorePlayerCollectionMemory(int, memory, playercollection),
      span
    }
  )
}

fn parse_score_player_memory(input: ParseStream) -> Result<SScoreRule> {
  let score = input.parse::<kw::score>()?;
  let start_span = score.span();
  
  let int = input.parse::<SIntExpr>()?;
  input.parse::<kw::to>()?;
  let memory = input.parse::<SID>()?;
  input.parse::<kw::of>()?;
  let player = input.parse::<SPlayerExpr>()?;

  let span = start_span
    .join(player.span)
    .unwrap_or(start_span);

  return Ok(
    SScoreRule {
      node: ScoreRule::ScorePlayerMemory(int, memory, player),
      span
    }
  )
}

fn parse_score_player_collection(input: ParseStream) -> Result<SScoreRule> {
  let score = input.parse::<kw::score>()?;
  let start_span = score.span();
  
  let int = input.parse::<SIntExpr>()?;
  input.parse::<kw::of>()?;
  let playercollection = input.parse::<SPlayerCollection>()?;

  let span = start_span
    .join(playercollection.span)
    .unwrap_or(start_span);

  return Ok(
    SScoreRule {
      node: ScoreRule::ScorePlayerCollection(int, playercollection),
      span
    }
  )
}

fn parse_score_player(input: ParseStream) -> Result<SScoreRule> {
  let score = input.parse::<kw::score>()?;
  let start_span = score.span();
  
  let int = input.parse::<SIntExpr>()?;
  input.parse::<kw::of>()?;
  let player = input.parse::<SPlayerExpr>()?;

  let span = start_span
    .join(player.span)
    .unwrap_or(start_span);

  return Ok(
    SScoreRule {
      node: ScoreRule::ScorePlayer(int, player),
      span
    }
  )
}

// ===========================================================================


// WinnerRule
// ===========================================================================
impl Parse for SWinnerRule {
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

fn parse_winner_winner_is_lowest_score(input: ParseStream) -> Result<SWinnerRule> {
  let winner = input.parse::<kw::winner>()?;
  let start_span = winner.span();

  input.parse::<kw::is>()?;
  input.parse::<kw::lowest>()?;
  let score = input.parse::<kw::score>()?;

  let span = start_span
    .join(score.span())
    .unwrap_or(start_span);

  return Ok(
    SWinnerRule {
      node: WinnerRule::WinnerLowestScore,
      span
    }
  )
}

fn parse_winner_winner_is_lowest_position(input: ParseStream) -> Result<SWinnerRule> {
  let winner = input.parse::<kw::winner>()?;
  let start_span = winner.span();

  input.parse::<kw::is>()?;
  input.parse::<kw::lowest>()?;
  let position = input.parse::<kw::position>()?;

  let span = start_span
    .join(position.span())
    .unwrap_or(start_span);

  return Ok(
    SWinnerRule {
      node: WinnerRule::WinnerLowestPosition,
      span
    }
  )
}

fn parse_winner_winner_is_lowest_memory(input: ParseStream) -> Result<SWinnerRule> {
  let winner = input.parse::<kw::winner>()?;
  let start_span = winner.span();

  input.parse::<kw::is>()?;
  input.parse::<kw::lowest>()?;

  let memory = input.parse::<SID>()?;

  let span = start_span
    .join(memory.span)
    .unwrap_or(start_span);

  return Ok(
    SWinnerRule {
      node: WinnerRule::WinnerLowestMemory(memory),
      span
    }
  )
}

fn parse_winner_winner_is_highest_score(input: ParseStream) -> Result<SWinnerRule> {
  let winner = input.parse::<kw::winner>()?;
  let start_span = winner.span();

  input.parse::<kw::is>()?;
  input.parse::<kw::highest>()?;
  let score = input.parse::<kw::score>()?;

  let span = start_span
    .join(score.span())
    .unwrap_or(start_span);

  return Ok(
    SWinnerRule {
      node: WinnerRule::WinnerHighestScore,
      span
    }
  )
}

fn parse_winner_winner_is_highest_position(input: ParseStream) -> Result<SWinnerRule> {
  let winner = input.parse::<kw::winner>()?;
  let start_span = winner.span();

  input.parse::<kw::is>()?;
  input.parse::<kw::highest>()?;
  let position = input.parse::<kw::position>()?;

  let span = start_span
    .join(position.span())
    .unwrap_or(start_span);

  return Ok(
    SWinnerRule {
      node: WinnerRule::WinnerHighestPosition,
      span
    }
  )
}

fn parse_winner_winner_is_highest_memory(input: ParseStream) -> Result<SWinnerRule> {
  let winner = input.parse::<kw::winner>()?;
  let start_span = winner.span();

  input.parse::<kw::is>()?;
  input.parse::<kw::highest>()?;

  let memory = input.parse::<SID>()?;

  let span = start_span
    .join(memory.span)
    .unwrap_or(start_span);

  return Ok(
    SWinnerRule {
      node: WinnerRule::WinnerHighestMemory(memory),
      span
    }
  )
}

// ===========================================================================


// SeqStage
// ===========================================================================
impl Parse for SSeqStage {
  fn parse(input: ParseStream) -> Result<Self> {
    let stage = input.parse::<kw::stage>()?;
    let start_span = stage.span();

    let stage = input.parse::<SID>()?;

    input.parse::<Token![for]>()?;
    let player = input.parse::<SPlayerExpr>()?;
    let endcondition = input.parse::<SEndCondition>()?;

    let content;
    braced!(content in input);

    let mut flows = Vec::new();
    while !content.is_empty() {
      let flow = content.parse::<SFlowComponent>()?;

      flows.push(flow);
    }

    let span = start_span
      .join(content.span())
      .unwrap_or(start_span);

    return Ok(
      SSeqStage {
        node: SeqStage {
          stage: stage,
          player: player,
          end_condition: endcondition,
          flows: flows
        },
        span
      }
    )
  }
}

// ===========================================================================


// IfRule
// ===========================================================================
impl Parse for SIfRule {
  fn parse(input: ParseStream) -> Result<Self> {
    let ifkw = input.parse::<Token![if]>()?;
    let start_span = ifkw.span();

    let content;
    parenthesized!(content in input);

    let condition = content.parse::<SBoolExpr>()?;

    let content;
    braced!(content in input);

    let mut flows = Vec::new();
    while !content.is_empty() {
      let flow = content.parse::<SFlowComponent>()?;

      flows.push(flow);
    }

    let span = start_span
      .join(content.span())
      .unwrap_or(start_span);

    return Ok(
      SIfRule {
        node: IfRule { condition, flows },
        span
      }
    )
  }
}

// ===========================================================================


// OptionalRule
// ===========================================================================
impl Parse for SOptionalRule {
  fn parse(input: ParseStream) -> Result<Self> {
    let optional = input.parse::<kw::optional>()?;
    let start_span = optional.span();
    
    let content;
    braced!(content in input);

    let mut flows = Vec::new();
    while !content.is_empty() {
      let flow = content.parse::<SFlowComponent>()?;

      flows.push(flow);
    }

    let span = start_span
      .join(content.span())
      .unwrap_or(start_span);

    return Ok(
      SOptionalRule {
        node: OptionalRule { flows },
        span
      } 
    )
  }
}

// ===========================================================================


// ChoiceRule
// ===========================================================================
impl Parse for SChoiceRule {
  fn parse(input: ParseStream) -> Result<Self> {
    let choose = input.parse::<kw::choose>()?;
    let start_span = choose.span();

    let content;
    braced!(content in input);

    let options: Punctuated<SFlowComponent, kw::or> =
      content.parse_terminated(SFlowComponent::parse, kw::or)?;

    let span = start_span
      .join(content.span())
      .unwrap_or(start_span);

    return Ok(
      SChoiceRule {
        node: ChoiceRule { options: options.into_iter().collect() },
        span
      }  
    )
  }
}

// ===========================================================================


// FlowComponent
// ===========================================================================
impl Parse for SFlowComponent {
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

fn parse_flowcomponent_seqstage(input: ParseStream) -> Result<SFlowComponent> {
  let stage = input.parse::<SSeqStage>()?;
  let span = stage.span;

  return Ok(
    SFlowComponent {
      node: FlowComponent::Stage(stage),
      span
    }
  )
}

fn parse_flowcomponent_ifrule(input: ParseStream) -> Result<SFlowComponent> {
  let ifrule = input.parse::<SIfRule>()?;
  let span = ifrule.span;

  return Ok(
    SFlowComponent {
      node: FlowComponent::IfRule(ifrule),
      span
    }
  )
}

fn parse_flowcomponent_choicerule(input: ParseStream) -> Result<SFlowComponent> {
  let choicerule = input.parse::<SChoiceRule>()?;
  let span = choicerule.span;

  return Ok(
    SFlowComponent {
      node: FlowComponent::ChoiceRule(choicerule),
      span
    }
  )
}

fn parse_flowcomponent_optionalrule(input: ParseStream) -> Result<SFlowComponent> {
  let optionalrule = input.parse::<SOptionalRule>()?;
  let span = optionalrule.span;

  return Ok(
    SFlowComponent {
      node: FlowComponent::OptionalRule(optionalrule),
      span
    }
  )
}

fn parse_flowcomponent_rule(input: ParseStream) -> Result<SFlowComponent> {
  let rule = input.parse::<SRule>()?;
  let start_span = rule.span;

  let sem = input.parse::<Token![;]>()?;

  let span = start_span
    .join(sem.span())
    .unwrap_or(start_span);

  return Ok(
    SFlowComponent {
      node: FlowComponent::Rule(rule),
      span
    }
  )
}

// ===========================================================================


// Game
// ===========================================================================
impl Parse for SGame {
  fn parse(input: ParseStream) -> Result<Self> {
    let mut flows = Vec::new();

    while !input.is_empty() {
      let flow = input.parse::<SFlowComponent>()?;

      flows.push(flow);
    }

    let span = match (flows.first(), flows.last()) {
        (Some(first), Some(last)) => {
            first.span.join(last.span).unwrap_or(first.span)
        }
        _ => input.span(), // empty file (or error recovery)
    };

    return Ok(SGame {
        node: Game { flows },
        span,
    })
  }
}

// ===========================================================================
