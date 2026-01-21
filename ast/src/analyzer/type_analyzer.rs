use crate::{analyzer::analyzer_error::AnalyzerError, asts::{ast::*, game_type::GameType}};

pub type TypedVars = Vec<(String, GameType)>;

pub fn ambiguous(ctx: TypedVars) -> Result<(), AnalyzerError> {
  let mut new_ctx = ctx;
  let (next, _) = new_ctx.first().unwrap().clone();
  let ctx_same = new_ctx.clone().into_iter().filter(|(s, _)| *s == next).collect();
  
  if all_no_type(&ctx_same) {
    return Err(AnalyzerError::IDWithNoType { id: next })
  }

  if multiple_options(&ctx_same) {
    return Err(AnalyzerError::IDWithMultipleTypes { id: next })
  }
  
  new_ctx = new_ctx.into_iter().filter(|(s, _)| *s != next).collect::<TypedVars>();

  if new_ctx.is_empty() {
    return Ok(())
  }

  ambiguous(new_ctx)
}

fn all_no_type(ctx: &TypedVars) -> bool {
  ctx
    .iter()
    .filter(|(_, t)| *t != GameType::NoType)
    .collect::<Vec<&(String, GameType)>>()
    .is_empty()
}

fn multiple_options(ctx: &TypedVars) -> bool {
  !ctx
    .iter()
    .filter(
      |(_, ty)|
        *ty != ctx.first().unwrap().1 && *ty != GameType::NoType
    )
    .collect::<Vec<&(String, GameType)>>()
    .is_empty()
}

// IDs
// ===========================================================================
// ===========================================================================
// ===========================================================================
fn ctx_create_id(id: &String, ty: GameType) -> TypedVars {
  vec![
    (id.clone(), ty)
  ]
}

fn ctx_id(id: &String) -> TypedVars {
  vec![
    (id.clone(), GameType::NoType)
  ]
}
// ===========================================================================
// ===========================================================================
// ===========================================================================


pub fn ctx(game: &Game) -> TypedVars {
  ctx_flows(&game.flows)
}

fn ctx_flows(flows: &Vec<FlowComponent>) -> TypedVars {
  flows.iter().map(|f| ctx_flow(f)).collect::<Vec<TypedVars>>().concat()
}

fn ctx_flow(flow: &FlowComponent) -> TypedVars {
  match flow {
    FlowComponent::Stage(seq_stage) => {
      ctx_seq_stage(seq_stage)
    },
    FlowComponent::Rule(rule) => {
      ctx_rule(rule)
    },
    FlowComponent::IfRule(if_rule) => {
      ctx_if_rule(if_rule)
    },
    FlowComponent::ChoiceRule(choice_rule) => {
      ctx_choice_rule(choice_rule)
    },
    FlowComponent::OptionalRule(optional_rule) => {
      ctx_optional_rule(optional_rule)
    },
  }
} 

fn ctx_if_rule(if_rule: &IfRule) -> TypedVars {
  vec![
    ctx_bool_expr(&if_rule.condition),
    ctx_flows(&if_rule.flows)
  ].concat()
}

fn ctx_choice_rule(choice_rule: &ChoiceRule) -> TypedVars {
  ctx_flows(&choice_rule.options)
}

fn ctx_optional_rule(optional_rule: &OptionalRule) -> TypedVars {
  ctx_flows(&optional_rule.flows)
}

fn ctx_seq_stage(seq_stage: &SeqStage) -> TypedVars {
  let mut typed_vars = ctx_flows(&seq_stage.flows);
  typed_vars.extend(ctx_player_expr(&seq_stage.player));
  typed_vars.extend(ctx_end_condition(&seq_stage.end_condition));

  typed_vars
}

fn ctx_end_condition(end_condition: &EndCondition) -> TypedVars {
  match end_condition {
    EndCondition::UntilBool(bool_expr) => {
      ctx_bool_expr(bool_expr)
    },
    EndCondition::UntilBoolAndRep(bool_expr, repititions) => {
      vec![ctx_bool_expr(bool_expr), ctx_repititions(repititions)].concat()
    },
    EndCondition::UntilBoolOrRep(bool_expr, repititions) => {
      vec![ctx_bool_expr(bool_expr), ctx_repititions(repititions)].concat()
    },
    EndCondition::UntilRep(repititions) => {
      ctx_repititions(repititions)
    },
    _ => Vec::new(),
  }
}

fn ctx_repititions(repititions: &Repititions) -> TypedVars {
  ctx_int_expr(&repititions.times)
}

fn ctx_bool_expr(bool_expr: &BoolExpr) -> TypedVars {
  match bool_expr {
    BoolExpr::IntCmp(int_expr, _, int_expr1) => {
      vec![ctx_int_expr(int_expr), ctx_int_expr(int_expr1)].concat()
    },
    BoolExpr::CardSetIsEmpty(card_set) => {
      ctx_card_set(card_set)
    },
    BoolExpr::CardSetIsNotEmpty(card_set) => {
      ctx_card_set(card_set)
    },
    BoolExpr::AmbiguousEq(id, id1) => {
      vec![
        ctx_id(id),
        ctx_id(id1),
      ].concat()
    },
    BoolExpr::AmbiguousNeq(id, id1) => {
      vec![
        ctx_id(id),
        ctx_id(id1),
      ].concat()
    },
    BoolExpr::CardSetEq(card_set, card_set1) => {
      vec![
        ctx_card_set(card_set),
        ctx_card_set(card_set1)
      ].concat()
    },
    BoolExpr::CardSetNeq(card_set, card_set1) => {
      vec![
        ctx_card_set(card_set),
        ctx_card_set(card_set1)
      ].concat()
    },
    BoolExpr::StringEq(string_expr, string_expr1) => {
      vec![
        ctx_string_expr(string_expr),
        ctx_string_expr(string_expr1)
      ].concat()
    },
    BoolExpr::StringNeq(string_expr, string_expr1) => {
      vec![
        ctx_string_expr(string_expr),
        ctx_string_expr(string_expr1)
      ].concat()
    },
    BoolExpr::PlayerEq(player_expr, player_expr1) => {
      vec![
        ctx_player_expr(player_expr),
        ctx_player_expr(player_expr1)
      ].concat()
    },
    BoolExpr::PlayerNeq(player_expr, player_expr1) => {
      vec![
        ctx_player_expr(player_expr),
        ctx_player_expr(player_expr1)
      ].concat()
    },
    BoolExpr::TeamEq(team_expr, team_expr1) => {
      vec![
        ctx_team_expr(team_expr),
        ctx_team_expr(team_expr1)
      ].concat()
    },
    BoolExpr::TeamNeq(team_expr, team_expr1) => {
      vec![
        ctx_team_expr(team_expr),
        ctx_team_expr(team_expr1)
      ].concat()
    },
    BoolExpr::And(bool_expr, bool_expr1) => {
      vec![
        ctx_bool_expr(bool_expr),
        ctx_bool_expr(bool_expr1)
      ].concat()
    },
    BoolExpr::Or(bool_expr, bool_expr1) => {
      vec![
        ctx_bool_expr(bool_expr),
        ctx_bool_expr(bool_expr1)
      ].concat()
    },
    BoolExpr::Not(bool_expr) => {
      ctx_bool_expr(bool_expr)
    },
    BoolExpr::OutOfStagePlayer(player_expr) => {
      ctx_player_expr(player_expr)
    },
    BoolExpr::OutOfGamePlayer(player_expr) => {
      ctx_player_expr(player_expr)
    },
    BoolExpr::OutOfStageCollection(player_collection) => {
      ctx_player_collection(player_collection)
    },
    BoolExpr::OutOfGameCollection(player_collection) => {
      ctx_player_collection(player_collection)
    },
  }
}

fn ctx_int_expr(int_expr: &IntExpr) -> TypedVars {
  match int_expr {
    IntExpr::IntOp(int_expr, _, int_expr1) => {
      vec![
        ctx_int_expr(int_expr),
        ctx_int_expr(int_expr1)
      ].concat()
    },
    IntExpr::IntCollectionAt(int_expr) => {
      ctx_int_expr(int_expr)
    },
    IntExpr::SizeOf(collection) => {
      ctx_collection(collection)
    },
    IntExpr::SumOfIntCollection(int_collection) => {
      ctx_int_collection(int_collection)
    },
    IntExpr::SumOfCardSet(card_set, point_map) => {
      vec![
        ctx_card_set(card_set),
        ctx_id(point_map),
      ].concat()
    },
    IntExpr::MinOf(card_set, point_map) => {
      vec![
        ctx_card_set(card_set),
        ctx_id(point_map),
      ].concat()
    },
    IntExpr::MaxOf(card_set, point_map) => {
      vec![
        ctx_card_set(card_set),
        ctx_id(point_map),
      ].concat()
    },
    IntExpr::MinIntCollection(int_collection) => {
      ctx_int_collection(int_collection)
    },
    IntExpr::MaxIntCollection(int_collection) => {
      ctx_int_collection(int_collection)

    },
    _ => Vec::new(),
  }
}

fn ctx_player_expr(player_expr: &PlayerExpr) -> TypedVars {
  match player_expr {
    PlayerExpr::PlayerName(player_name) => ctx_id(player_name),
    _ => Vec::new(),
  }
}

fn ctx_team_expr(team_expr: &TeamExpr) -> TypedVars {
  match team_expr {
    TeamExpr::TeamName(team_name) => ctx_id(team_name),
    _ => Vec::new(),
  }
}

fn ctx_string_expr(string_expr: &StringExpr) -> TypedVars {
  match string_expr {
    StringExpr::KeyOf(key, card_position) => {
      vec![
        ctx_card_position(card_position),
        ctx_id(key),
      ].concat()
    },
    StringExpr::StringCollectionAt(string_collection, int_expr) => {
      vec![
        ctx_string_collection(string_collection),
        ctx_int_expr(int_expr)
      ].concat()
    },
  }
}

fn ctx_card_set(card_set: &CardSet) -> TypedVars {
  match card_set {
    CardSet::Group(group) => {
      ctx_group(group)
    },
    CardSet::GroupOfPlayer(group, player_expr) => {
      vec![
        ctx_group(group),
        ctx_player_expr(player_expr)
      ].concat()
    },
    CardSet::GroupOfPlayerCollection(group, player_collection) => {
      vec![
        ctx_group(group),
        ctx_player_collection(player_collection)
      ].concat()
    },
  }
}

fn ctx_group(group: &Group) -> TypedVars {
  match group {
    Group::Location(location) => {
      ctx_id(location)
    },
    Group::LocationWhere(location, filter_expr) => {
      vec![
        ctx_filter_expr(filter_expr),
        ctx_id(location)
      ].concat()
    },
    Group::LocationCollection(location_collection) => {
      ctx_location_collection(location_collection)
    },
    Group::LocationCollectionWhere(location_collection, filter_expr) => {
      vec![
        ctx_filter_expr(filter_expr),
        ctx_location_collection(location_collection)
      ].concat()
    },
    Group::ComboInLocation(combo, location) => {
      vec![
        ctx_id(combo),
        ctx_id(location)
      ].concat()
    },
    Group::ComboInLocationCollection(combo, location_collection) => {
      vec![
        ctx_location_collection(location_collection),
        ctx_id(combo),
      ].concat()
    },
    Group::NotComboInLocation(combo, location) => {
      vec![
        ctx_id(combo),
        ctx_id(location)
      ].concat()
    },
    Group::NotComboInLocationCollection(combo, location_collection) => {
      vec![
        ctx_location_collection(location_collection),
        ctx_id(combo),
      ].concat()
    },
    Group::CardPosition(card_position) => {
      ctx_card_position(card_position)
    },
  }
}

fn ctx_card_position(card_position: &CardPosition) -> TypedVars {
  match card_position {
    CardPosition::At(location, int_expr) => {
      vec![
        ctx_id(location),
        ctx_int_expr(int_expr)
      ].concat()
    },
    CardPosition::Top(location) => {
      ctx_id(location)
    },
    CardPosition::Bottom(location) => {
      ctx_id(location)
    },
    CardPosition::Max(card_set, id) => {
      vec![
        ctx_id(id),
        ctx_card_set(card_set),
      ].concat()
    },
    CardPosition::Min(card_set, id) => {
      vec![
        ctx_id(id),
        ctx_card_set(card_set),
      ].concat()
    },
  }
}

fn ctx_filter_expr(filter_expr: &FilterExpr) -> TypedVars {
  match filter_expr {
    FilterExpr::Same(key) => {
      ctx_id(key)
    },
    FilterExpr::Distinct(key) => {
      ctx_id(key)
    },
    FilterExpr::Adjacent(key, precedence) => {
      vec![
        ctx_id(key),
        ctx_id(precedence),
      ].concat()
    },
    FilterExpr::Higher(key, precedence) => {
      vec![
        ctx_id(key),
        ctx_id(precedence),
      ].concat()
    },
    FilterExpr::Lower(key, precedence) => {
      vec![
        ctx_id(key),
        ctx_id(precedence),
      ].concat()
    },
    FilterExpr::Size(_, int_expr) => {
      ctx_int_expr(int_expr)
    },
    FilterExpr::KeyEqString(key, string_expr) => {
      vec![
        ctx_id(key),
        ctx_string_expr(string_expr)
      ].concat()
    },
    FilterExpr::KeyNeqString(key, string_expr) => {
      vec![
        ctx_id(key),
        ctx_string_expr(string_expr)
      ].concat()
    },
    FilterExpr::KeyEqValue(key, value) => {
      vec![
        ctx_id(key),
        ctx_id(value),
      ].concat()
    },
    FilterExpr::KeyNeqValue(key, value) => {
      vec![
        ctx_id(key),
        ctx_id(value),
      ].concat()
    },
    FilterExpr::NotCombo(combo) => {
      ctx_id(combo)
    },
    FilterExpr::Combo(combo) => {
      ctx_id(combo)
    },
    FilterExpr::And(filter_expr, filter_expr1) => {
      vec![
        ctx_filter_expr(filter_expr),
        ctx_filter_expr(filter_expr1),
      ].concat()
    },
    FilterExpr::Or(filter_expr, filter_expr1) => {
      vec![
        ctx_filter_expr(filter_expr),
        ctx_filter_expr(filter_expr1),
      ].concat()
    },
  }
}

fn ctx_collection(collection: &Collection) -> TypedVars {
  match collection {
    Collection::IntCollection(int_collection) => {
      ctx_int_collection(int_collection)
    },
    Collection::StringCollection(string_collection) => {
      ctx_string_collection(string_collection)
    },
    Collection::LocationCollection(location_collection) => {
      ctx_location_collection(location_collection)
    },
    Collection::PlayerCollection(player_collection) => {
      ctx_player_collection(player_collection)
    },
    Collection::TeamCollection(team_collection) => {
      ctx_team_collection(team_collection)
    },
    Collection::CardSet(card_set) => {
      ctx_card_set(card_set)
    },
    Collection::Ambiguous(ids) => {
      ids.iter().map(|id| (id.clone(), GameType::NoType)).collect()
    },
  }
}

fn ctx_int_collection(int_collection: &IntCollection) -> TypedVars {
  int_collection.ints.iter().map(|int_expr| ctx_int_expr(int_expr)).collect::<Vec<TypedVars>>().concat()
}

fn ctx_string_collection(string_collection: &StringCollection) -> TypedVars {
  string_collection.strings.iter().map(|string_expr| ctx_string_expr(string_expr)).collect::<Vec<TypedVars>>().concat()
}

fn ctx_location_collection(location_collection: &LocationCollection) -> TypedVars {
  location_collection.locations.iter().map(|location| (location.clone(), GameType::NoType)).collect::<TypedVars>()
}

fn ctx_player_collection(player_collection: &PlayerCollection) -> TypedVars {
  match player_collection {
    PlayerCollection::Player(player_exprs) => {
      player_exprs.iter().map(|player_expr| ctx_player_expr(player_expr)).collect::<Vec<TypedVars>>().concat()
    },
    _ => Vec::new()
  }
}

fn ctx_team_collection(team_collection: &TeamCollection) -> TypedVars {
  match team_collection {
    TeamCollection::Team(team_exprs) => {
      team_exprs.iter().map(|team_expr| ctx_team_expr(team_expr)).collect::<Vec<TypedVars>>().concat()
    },
    _ => Vec::new()
  }
}


fn ctx_rule(rule: &Rule) -> TypedVars {
  match rule {
    Rule::CreatePlayer(player_names) => {
      player_names.iter().map(|player_name| (player_name.clone(), GameType::Player)).collect()
    },
    Rule::CreateTeam(team_name, player_names) => {
      vec![
        ctx_create_id(team_name, GameType::Team),
        player_names.iter().map(|player_name| (player_name.clone(), GameType::NoType)).collect()
      ].concat()
    },
    Rule::CreateTurnorder(player_names) => {
      player_names.iter().map(|player_name| (player_name.clone(), GameType::NoType)).collect()
    },
    Rule::CreateTurnorderRandom(player_names) => {
      player_names.iter().map(|player_name| (player_name.clone(), GameType::NoType)).collect()
    },
    Rule::CreateLocationOnPlayerCollection(location, player_collection) => {
      vec![
        ctx_create_id(location, GameType::Location),
        ctx_player_collection(player_collection)
      ].concat()
    },
    Rule::CreateLocationOnTeamCollection(location, team_collection) => {
      vec![
        ctx_create_id(location, GameType::Location),
        ctx_team_collection(team_collection)
      ].concat()
    },
    Rule::CreateLocationOnTable(location) => {
      ctx_create_id(location, GameType::Location)
    },
    Rule::CreateLocationCollectionOnPlayerCollection(location_collection, player_collection) => {
      vec![
        location_collection.locations.iter().map(|location| (location.clone(), GameType::Location)).collect(),
        ctx_player_collection(player_collection),
      ].concat()
    },
    Rule::CreateLocationCollectionOnTeamCollection(location_collection, team_collection) => {
      vec![
        location_collection.locations.iter().map(|location| (location.clone(), GameType::Location)).collect(),
        ctx_team_collection(team_collection),
      ].concat()
    },
    Rule::CreateLocationCollectionOnTable(location_collection) => {
      location_collection.locations.iter().map(|location| (location.clone(), GameType::Location)).collect()
    },
    Rule::CreateCardOnLocation(location, types) => {
      vec![
        ctx_id(location),
        types.types
          .iter()
          .map(|(k, vs)| {
            vec![
              ctx_create_id(k, GameType::Key),
              vs.iter().map(|v| (v.clone(), GameType::Value)).collect()
            ].concat() 
          }
          ).collect::<Vec<TypedVars>>().concat(),
      ].concat()
    },
    Rule::CreateTokenOnLocation(int_expr, token, location) => {
      vec![
        ctx_int_expr(int_expr),
        ctx_create_id(token, GameType::Token),
        ctx_id(location),
      ].concat()
    },
    Rule::CreatePrecedence(precedence, items) => {
      vec![
        ctx_create_id(precedence, GameType::Precedence),
        items.iter().map(
          |(k, v)|
          vec![
            ctx_id(k),
            ctx_id(v)
          ].concat()
        ).collect::<Vec<TypedVars>>().concat()
      ].concat()
    },
    Rule::CreateCombo(combo, filter_expr) => {
      vec![
        ctx_create_id(combo, GameType::Combo),
        ctx_filter_expr(filter_expr)
      ].concat()
    },
    Rule::CreateMemoryIntPlayerCollection(memory, int_expr, player_collection) => {
      vec![
        ctx_create_id(memory, GameType::Memory),
        ctx_int_expr(int_expr),
        ctx_player_collection(player_collection),
      ].concat()
    },
    Rule::CreateMemoryStringPlayerCollection(memory, string_expr, player_collection) => {
      vec![
        ctx_create_id(memory, GameType::Memory),
        ctx_string_expr(string_expr),
        ctx_player_collection(player_collection),
      ].concat()
    },
    Rule::CreateMemoryIntTable(memory, int_expr) => {
      vec![
        ctx_create_id(memory, GameType::Memory),
        ctx_int_expr(int_expr),
      ].concat()
    },
    Rule::CreateMemoryStringTable(memory, string_expr) => {
      vec![
        ctx_create_id(memory, GameType::Memory),
        ctx_string_expr(string_expr),
      ].concat()
    },
    Rule::CreateMemoryPlayerCollection(memory, player_collection) => {
      vec![
        ctx_create_id(memory, GameType::Memory),
        ctx_player_collection(player_collection),
      ].concat()
    },
    Rule::CreateMemoryTable(memory) => {
      ctx_create_id(memory, GameType::Memory)
    },
    Rule::CreatePointMap(point_map, items) => {
      vec![
        ctx_create_id(point_map, GameType::PointMap),
        items.iter().map(|(k, v, int)| 
          vec![
            ctx_id(k),
            ctx_id(v),
            ctx_int_expr(int),
          ].concat()
        ).collect::<Vec<TypedVars>>().concat()
      ].concat()
    },
    Rule::FlipAction(card_set, _) => {
      ctx_card_set(card_set)
    },
    Rule::ShuffleAction(card_set) => {
      ctx_card_set(card_set)
    },
    Rule::PlayerOutOfStageAction(player_expr) => {
      ctx_player_expr(player_expr)
    },
    Rule::PlayerOutOfGameSuccAction(player_expr) => {
      ctx_player_expr(player_expr)
    },
    Rule::PlayerOutOfGameFailAction(player_expr) => {
      ctx_player_expr(player_expr)
    },
    Rule::PlayerCollectionOutOfStageAction(player_collection) => {
      ctx_player_collection(player_collection)
    },
    Rule::PlayerCollectionOutOfGameSuccAction(player_collection) => {
      ctx_player_collection(player_collection)
    },
    Rule::PlayerCollectionOutOfGameFailAction(player_collection) => {
      ctx_player_collection(player_collection)
    },
    Rule::SetMemoryInt(memory, int_expr) => {
      vec![
        ctx_id(memory),
        ctx_int_expr(int_expr),
      ].concat()
    },
    Rule::SetMemoryString(memory, string_expr) => {
      vec![
        ctx_id(memory),
        ctx_string_expr(string_expr),
      ].concat()
    },
    Rule::SetMemoryCollection(memory, collection) => {
      vec![
        ctx_id(memory),
        ctx_collection(collection),
      ].concat()
    },
    Rule::CycleAction(player_expr) => {
      ctx_player_expr(player_expr)
    },
    Rule::BidActionMemory(memory, _) => {
      ctx_id(memory)
    },
    Rule::EndGameWithWinner(player_expr) => {
      ctx_player_expr(player_expr)
    },
    Rule::DemandCardPositionAction(card_position) => {
      ctx_card_position(card_position)
    },
    Rule::DemandStringAction(string_expr) => {
      ctx_string_expr(string_expr)
    },
    Rule::DemandIntAction(int_expr) => {
      ctx_int_expr(int_expr)
    },
    Rule::ClassicMove(classic_move) => {
      ctx_classic_move(classic_move)
    },
    Rule::DealMove(deal_move) => {
      ctx_deal_move(deal_move)
    },
    Rule::ExchangeMove(exchange_move) => {
      ctx_exchange_move(exchange_move)
    },
    Rule::TokenMove(token_move) => {
      ctx_token_move(token_move)
    },
    Rule::ScoreRule(score_rule) => {
      ctx_score_rule(score_rule)
    },
    Rule::WinnerRule(winner_rule) => {
      ctx_winner_rule(winner_rule)
    },
    _ => Vec::new(),
  }
}

fn ctx_quantity(quantity: &Quantity) -> TypedVars {
  match quantity {
    Quantity::Int(int_expr) => {
      ctx_int_expr(int_expr)
    },
    Quantity::Quantifier(_) => {
      Vec::new()
    },
    Quantity::IntRange(int_range) => {
      ctx_int_expr(&int_range.int)
    },
  }
}

fn ctx_classic_move(classic_move: &ClassicMove) -> TypedVars {
  match classic_move {
    ClassicMove::Move(card_set, _, card_set1) => {
      vec![
        ctx_card_set(card_set),
        ctx_card_set(card_set1),
      ].concat()
    },
    ClassicMove::MoveQuantity(quantity, card_set, _, card_set1) => {
      vec![
        ctx_quantity(quantity),
        ctx_card_set(card_set),
        ctx_card_set(card_set1),
      ].concat()
    },
  }
}

fn ctx_deal_move(deal_move: &DealMove) -> TypedVars {
  match deal_move {
    DealMove::Deal(card_set, _, card_set1) => {
      vec![
        ctx_card_set(card_set),
        ctx_card_set(card_set1),
      ].concat()
    },
    DealMove::DealQuantity(quantity, card_set, _, card_set1) => {
      vec![
        ctx_quantity(quantity),
        ctx_card_set(card_set),
        ctx_card_set(card_set1),
      ].concat()
    },
  }
}

fn ctx_exchange_move(exchange_move: &ExchangeMove) -> TypedVars {
  match exchange_move {
    ExchangeMove::Exchange(card_set, _, card_set1) => {
      vec![
        ctx_card_set(card_set),
        ctx_card_set(card_set1),
      ].concat()
    },
    ExchangeMove::ExchangeQuantity(quantity, card_set, _, card_set1) => {
      vec![
        ctx_quantity(quantity),
        ctx_card_set(card_set),
        ctx_card_set(card_set1),
      ].concat()
    },
  }
}

fn ctx_token_move(token_move: &TokenMove) -> TypedVars {
  match token_move {
    TokenMove::Place(token, token_loc_expr, token_loc_expr1) => {
      vec![
        ctx_id(token),
        ctx_token_loc_expr(token_loc_expr),
        ctx_token_loc_expr(token_loc_expr1),
      ].concat()
    },
    TokenMove::PlaceQuantity(quantity, token, token_loc_expr, token_loc_expr1) => {
      vec![
        ctx_quantity(quantity),
        ctx_id(token),
        ctx_token_loc_expr(token_loc_expr),
        ctx_token_loc_expr(token_loc_expr1),
      ].concat()
    },
  }
}

fn ctx_token_loc_expr(token_loc_expr: &TokenLocExpr) -> TypedVars {
  match token_loc_expr {
    TokenLocExpr::Location(location) => {
      vec![
        (location.clone(), GameType::NoType)
      ]
    },
    TokenLocExpr::LocationCollection(location_collection) => {
      ctx_location_collection(location_collection)
    },
    TokenLocExpr::LocationPlayer(location, player_expr) => {
      vec![
        ctx_id(location),
        ctx_player_expr(player_expr)
      ].concat()
    },
    TokenLocExpr::LocationCollectionPlayer(location_collection, player_expr) => {
      vec![
        ctx_location_collection(location_collection),
        ctx_player_expr(player_expr)
      ].concat()
    },
    TokenLocExpr::LocationPlayerCollection(location, player_collection) => {
      vec![
        ctx_id(location),
        ctx_player_collection(player_collection)
      ].concat()
    },
    TokenLocExpr::LocationCollectionPlayerCollection(location_collection, player_collection) => {
      vec![
        ctx_location_collection(location_collection),
        ctx_player_collection(player_collection)
      ].concat()
    },
  }
}

fn ctx_score_rule(score_rule: &ScoreRule) -> TypedVars {
  match score_rule {
    ScoreRule::ScorePlayer(int_expr, player_expr) => {
      vec![
        ctx_int_expr(int_expr),
        ctx_player_expr(player_expr),
      ].concat()
    },
    ScoreRule::ScorePlayerMemory(int_expr, memory, player_expr) => {
      vec![
        ctx_int_expr(int_expr),
        ctx_id(memory),
        ctx_player_expr(player_expr),
      ].concat()
    },
    ScoreRule::ScorePlayerCollection(int_expr, player_collection) => {
      vec![
        ctx_int_expr(int_expr),
        ctx_player_collection(player_collection),
      ].concat()
    },
    ScoreRule::ScorePlayerCollectionMemory(int_expr, memory, player_collection) => {
      vec![
        ctx_int_expr(int_expr),
        ctx_id(memory),
        ctx_player_collection(player_collection),
      ].concat()
    },
  }
}

fn ctx_winner_rule(winner_rule: &WinnerRule) -> TypedVars {
  match winner_rule {
    WinnerRule::WinnerPlayer(player_expr) => {
      ctx_player_expr(player_expr)
    },
    WinnerRule::WinnerPlayerCollection(player_collection) => {
      ctx_player_collection(player_collection)
    },
    WinnerRule::WinnerLowestMemory(memory) => {
      ctx_id(memory)
    },
    WinnerRule::WinnerHighestMemory(memory) => {
      ctx_id(memory)
    },
    _ => Vec::new(),
  }
}
