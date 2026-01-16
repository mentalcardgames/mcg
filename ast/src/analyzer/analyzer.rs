use crate::analyzer::analyzer_error::AnalyzerError;
use crate::analyzer::symbol::semantic_op::SemanticOp;

use crate::analyzer::symbol::symbol_analyzer::SymbolAnalyzer;
use crate::keywords::kw::{self as kw};
use crate::dsl_types::DSLType;
use crate::ast::*;


pub struct Analyzer {
  stack: Vec<(usize, SemanticOp)>,
  current: usize,
}

impl Default for Analyzer {
  fn default() -> Self {
      Analyzer { 
        stack: Vec::new(),
        current: 0,
      }
  }
}

impl Analyzer {
  fn add_rule_to_stack(&mut self, semantic_op: SemanticOp) {
    self.stack.push((self.current, semantic_op));
    self.current += 1;
  }

  fn check_id_is_int<T: ToString>(value: &T) -> bool {
    // If ID is int
    if let Ok(_) = value.to_string().trim().parse::<f64>() {
      return true
    }

    return false
  }

  fn check_id_is_custom_keyword<T: ToString>(value: &T) -> bool {
    return kw::in_custom_key_words(value)
  }

  fn check_id_starts_with_lower_or_empty<T: ToString>(value: &T) -> bool {
    if let Some(first_letter) = value.to_string().chars().next() {
      return first_letter.is_lowercase()
    } else {
      return true
    }
  }

  // used for helping the parser
  pub fn check_id<T: ToString>(value: &T) -> Result<(), AnalyzerError> {
    if Self::check_id_starts_with_lower_or_empty(value) {
      return Err(AnalyzerError::IdNotCapitalOrEmpty)
    }
    if Self::check_id_is_int(value) {
      return Err(AnalyzerError::InvalidInteger)
    }
    if Self::check_id_is_custom_keyword(value) {
      return Err(AnalyzerError::ReservedKeyword)
    }

    return Ok(())
  }


  /*
    Do one parsing of Initilized players and used players
    in the correct order.
    Do the same for the other ids.
    => first gather then analyze
    => move ambigious things also inside maybe

    Numerize every Rule:
    > (1, ...), (2, ...), ...
    > Gather all rules with ID ...
    > Filter Rules to satisfy certain things and to rebuild
    the analyzer stack
  
  */


  // CHECKING IF ID IS USED CORRECTLY:
  // > every ID is unique
  // > every ID is initialized somewhere
  // > assigned to its corresponding type
  //
  // AMBIGIUTY IN:
  // > Collection
  // > BoolExpr
  // > SetMemory
  // 
  // CHECKING IF KEY AND VALUE RULES:
  // > rule like: "Rank == Ace" need to be checked
  // 
  // IF ERRORS OCCUR: 
  // > find the closest matching rule for the Error
  // > check for "mis-spells" / "mis-types" and give propable Error
  //
  // ELSE (if something new needs to be analyzed):

  pub fn analyze_rule(&mut self, rule: &Rule) -> Result<(), AnalyzerError> {
    self.build_stack_rule(rule);

    let mut sa = SymbolAnalyzer::default();

    for (_, semantic_op) in self.stack.iter() {
      sa.eval_semantic_op(semantic_op.clone())?;
    }

    return Ok(())
  }

  pub fn analyze_game(&mut self, game: &Game) -> Result<(), AnalyzerError> {
    self.build_stack_game(game);

    let mut sa = SymbolAnalyzer::default();

    for (_, semantic_op) in self.stack.iter() {
      sa.eval_semantic_op(semantic_op.clone())?;
    }

    return Ok(())
  }
  

  fn playernames_to_string(&self, player_names: &Vec<PlayerName>) -> Vec<String> {
    player_names
      .iter()
      .map(|p| p.0.clone())
      .collect()
  }

  fn build_stack_game(&mut self, game: &Game) {
    for flow in game.flows.iter() {
      self.build_stack_flow(flow);
    }
  }

  fn build_stack_flow(&mut self, flow: &FlowComponent) {
    match flow {
        FlowComponent::Stage(seq_stage) => {
          self.add_rule_to_stack(
            SemanticOp::UsePlayer {
              player: seq_stage.player.clone()
            }
          );
          self.add_rule_to_stack(
            SemanticOp::UseEndCondition {
              end_condition: seq_stage.end_condition.clone()
            }
          );

          for f in seq_stage.flows.iter() {
            self.build_stack_flow(f);
          }
        },
        FlowComponent::Rule(rule) => {
          self.build_stack_rule(rule);
        },
        FlowComponent::IfRule(if_rule) => {
          self.add_rule_to_stack(
            SemanticOp::UseBoolExpr {
              bool_expr: if_rule.condition.clone()
            }
          );

          for f in if_rule.flows.iter() {
            self.build_stack_flow(f);
          }
        },
        FlowComponent::ChoiceRule(choice_rule) => {
          for option in choice_rule.options.iter() {
            self.build_stack_flow(option);
          }
        },
        FlowComponent::OptionalRule(optional_rule) => {
          for f in optional_rule.flows.iter() {
            self.build_stack_flow(f);
          }
        },
    }
  }

  fn build_stack_rule(&mut self, rule: &Rule) {
    match rule {
        Rule::CreatePlayer(player_names) => {
          self.add_rule_to_stack(
            SemanticOp::InitializeVec {
              vec_id: self.playernames_to_string(player_names),
              ty: DSLType::Player
            }
          );
        },
        Rule::CreateTeam(team_name, player_names) => {
          self.add_rule_to_stack(
            SemanticOp::Initialize {
              id: team_name.0.clone(),
              ty: DSLType::Team
            }
          );
          self.add_rule_to_stack(
            SemanticOp::UseVec {
              vec_id: self.playernames_to_string(player_names),
              ty: DSLType::Player
            }
          );
        },
        Rule::CreateTurnorder(player_names) => {
          self.add_rule_to_stack(
            SemanticOp::UseVec {
              vec_id: self.playernames_to_string(player_names),
              ty: DSLType::Player
            }
          );
        },
        Rule::CreateTurnorderRandom(player_names) => {
          self.add_rule_to_stack(
            SemanticOp::UseVec {
              vec_id: self.playernames_to_string(player_names),
              ty: DSLType::Player
            }
          );
        },
        Rule::CreateLocationOnPlayerCollection(location, player_collection) => {
          self.add_rule_to_stack(
            SemanticOp::Initialize {
              id: location.0.clone(),
              ty: DSLType::Location
            }
          );
          self.add_rule_to_stack(
            SemanticOp::UseCollection { 
              collection: Collection::PlayerCollection(player_collection.clone()),
            }
          );
        },
        Rule::CreateLocationOnTeamCollection(location, team_collection) => {
          self.add_rule_to_stack(
            SemanticOp::Initialize {
              id: location.0.clone(),
              ty: DSLType::Location
            }
          );
          self.add_rule_to_stack(
            SemanticOp::UseCollection { 
              collection: Collection::TeamCollection(team_collection.clone()),
            }
          );
        },
        Rule::CreateLocationOnTable(location) => {
          self.add_rule_to_stack(
            SemanticOp::Initialize {
              id: location.0.clone(),
              ty: DSLType::Location
            }
          );
        },
        Rule::CreateLocationCollectionOnPlayerCollection(location_collection, player_collection) => {
          self.add_rule_to_stack(
            SemanticOp::InitializeVec {
              vec_id: location_collection.locations.iter().map(|l| l.0.clone()).collect(),
              ty: DSLType::Location,
            }
          );
          self.add_rule_to_stack(
            SemanticOp::UseCollection {
              collection: Collection::PlayerCollection(player_collection.clone())
            }
          );
        },
        Rule::CreateLocationCollectionOnTeamCollection(location_collection, team_collection) => {
          self.add_rule_to_stack(
            SemanticOp::InitializeVec {
              vec_id: location_collection.locations.iter().map(|l| l.0.clone()).collect(),
              ty: DSLType::Location,
            }
          );
          self.add_rule_to_stack(
            SemanticOp::UseCollection {
              collection: Collection::TeamCollection(team_collection.clone())
            }
          );
        },
        Rule::CreateLocationCollectionOnTable(location_collection) => {
          self.add_rule_to_stack(
            SemanticOp::InitializeVec {
              vec_id: location_collection.locations.iter().map(|l| l.0.clone()).collect(),
              ty: DSLType::Location,
            }
          );
        },
        Rule::CreateCardOnLocation(location, types) => {
          self.add_rule_to_stack(
            SemanticOp::Use {
              id: location.0.clone(),
              ty: DSLType::Location
            }
          );
          for (k, vs) in types.types.iter() {
            for v in vs.iter() {
              self.add_rule_to_stack(
                SemanticOp::InitializeKeyValue {
                  key: k.0.clone(),
                  value: v.0.clone()
                }
              );
            }
          }
          
        },
        Rule::CreateTokenOnLocation(int_expr, token, location) => {
          self.add_rule_to_stack(
            SemanticOp::UseInt {
              int_expr: int_expr.clone()
            }
          );
          self.add_rule_to_stack(
            SemanticOp::Initialize {
              id: token.0.clone(),
              ty: DSLType::Token
            }
          );
          self.add_rule_to_stack(
            SemanticOp::Use {
              id: location.0.clone(),
              ty: DSLType::Location
            }
          );
        },
        Rule::CreatePrecedence(precedence, items) => {
          self.add_rule_to_stack(
            SemanticOp::Initialize {
              id: precedence.0.clone(),
              ty: DSLType::Precedence
            }
          );
          for (k, v) in items.iter() {
            self.add_rule_to_stack(
              SemanticOp::Use {
                id: k.0.clone(),
                ty: DSLType::Key
              }
            );
            self.add_rule_to_stack(
              SemanticOp::Use {
                id: v.0.clone(),
                ty: DSLType::Value
              }
            );
          }
        },
        Rule::CreateCombo(combo, _) => {
          self.add_rule_to_stack(
            SemanticOp::Initialize {
              id: combo.0.clone(),
              ty: DSLType::Combo
            }
          );
        },
        Rule::CreateMemoryIntPlayerCollection(memory, int_expr, player_collection) => {
          self.add_rule_to_stack(
            SemanticOp::Initialize {
              id: memory.0.clone(),
              ty: DSLType::Memory
            }
          );
          self.add_rule_to_stack(
            SemanticOp::UseInt {
              int_expr: int_expr.clone()
            }
          );
          self.add_rule_to_stack(
            SemanticOp::UseCollection {
              collection: Collection::PlayerCollection(player_collection.clone())
            }
          );
        },
        Rule::CreateMemoryStringPlayerCollection(memory, string_expr, player_collection) => {
          self.add_rule_to_stack(
            SemanticOp::Initialize {
              id: memory.0.clone(),
              ty: DSLType::Memory
            }
          );
          self.add_rule_to_stack(
            SemanticOp::UseString {
              string_expr: string_expr.clone()
            }
          );
          self.add_rule_to_stack(
            SemanticOp::UseCollection {
              collection: Collection::PlayerCollection(player_collection.clone())
            }
          );
        },
        Rule::CreateMemoryIntTable(memory, int_expr) => {
          self.add_rule_to_stack(
            SemanticOp::Initialize {
              id: memory.0.clone(),
              ty: DSLType::Memory
            }
          );
          self.add_rule_to_stack(
            SemanticOp::UseInt {
              int_expr: int_expr.clone()
            }
          );
        },
        Rule::CreateMemoryStringTable(memory, string_expr) => {
          self.add_rule_to_stack(
            SemanticOp::Initialize {
              id: memory.0.clone(),
              ty: DSLType::Memory
            }
          );
          self.add_rule_to_stack(
            SemanticOp::UseString {
              string_expr: string_expr.clone()
            }
          );
        },
        Rule::CreateMemoryPlayerCollection(memory, player_collection) => {
          self.add_rule_to_stack(
            SemanticOp::Initialize {
              id: memory.0.clone(),
              ty: DSLType::Memory
            }
          );
          self.add_rule_to_stack(
            SemanticOp::UseCollection {
              collection: Collection::PlayerCollection(player_collection.clone())
            }
          );
        },
        Rule::CreateMemoryTable(memory) => {
          self.add_rule_to_stack(
            SemanticOp::Initialize {
              id: memory.0.clone(),
              ty: DSLType::Memory
            }
          );
        },
        Rule::CreatePointMap(point_map, items) => {
          self.add_rule_to_stack(
            SemanticOp::Initialize {
              id: point_map.0.clone(),
              ty: DSLType::PointMap
            }
          );
          for (k, v, _) in items.iter() {
            self.add_rule_to_stack(
              SemanticOp::Use {
                id: k.0.clone(),
                ty: DSLType::Key
              }
            );
            self.add_rule_to_stack(
              SemanticOp::Use {
                id: v.0.clone(),
                ty: DSLType::Value
              }
            );
          }
        },
        Rule::FlipAction(card_set, _) => {
          self.add_rule_to_stack(
            SemanticOp::UseCardSet {
              card_set: card_set.clone()
            }
          );
        },
        Rule::ShuffleAction(card_set) => {
          self.add_rule_to_stack(
            SemanticOp::UseCardSet {
              card_set: card_set.clone()
            }
          );
        },
        Rule::PlayerOutOfStageAction(player_expr) => {
          self.add_rule_to_stack(
            SemanticOp::UsePlayer {
              player: player_expr.clone()
            }
          );
        },
        Rule::PlayerOutOfGameSuccAction(player_expr) => {
          self.add_rule_to_stack(
            SemanticOp::UsePlayer {
              player: player_expr.clone()
            }
          );
        },
        Rule::PlayerOutOfGameFailAction(player_expr) => {
          self.add_rule_to_stack(
            SemanticOp::UsePlayer {
              player: player_expr.clone()
            }
          );
        },
        Rule::PlayerCollectionOutOfStageAction(player_collection) => {
          self.add_rule_to_stack(
            SemanticOp::UseCollection {
              collection: Collection::PlayerCollection(player_collection.clone())
            }
          );
        },
        Rule::PlayerCollectionOutOfGameSuccAction(player_collection) => {
          self.add_rule_to_stack(
            SemanticOp::UseCollection {
              collection: Collection::PlayerCollection(player_collection.clone())
            }
          );
        },
        Rule::PlayerCollectionOutOfGameFailAction(player_collection) => {
          self.add_rule_to_stack(
            SemanticOp::UseCollection {
              collection: Collection::PlayerCollection(player_collection.clone())
            }
          );
        },
        Rule::SetMemoryInt(memory, int_expr) => {
          self.add_rule_to_stack(
            SemanticOp::Initialize {
              id: memory.0.clone(),
              ty: DSLType::Memory
            }
          );
          self.add_rule_to_stack(
            SemanticOp::UseInt { 
              int_expr: int_expr.clone() 
            }
          );
        },
        Rule::SetMemoryString(memory, string_expr) => {
          self.add_rule_to_stack(
            SemanticOp::Initialize {
              id: memory.0.clone(),
              ty: DSLType::Memory
            }
          );
          self.add_rule_to_stack(
            SemanticOp::UseString {
              string_expr: string_expr.clone()
            }
          );
        },
        Rule::SetMemoryCollection(memory, collection) => {
          self.add_rule_to_stack(
            SemanticOp::Initialize {
              id: memory.0.clone(),
              ty: DSLType::Memory
            }
          );
          self.add_rule_to_stack(
            SemanticOp::UseCollection {
              collection: collection.clone()
            }
          );
        },
        Rule::SetMemoryAmbiguous(memory, _) => {
          self.add_rule_to_stack(
            SemanticOp::Initialize {
              id: memory.0.clone(),
              ty: DSLType::Memory
            }
          );
        },
        Rule::CycleAction(player_expr) => {
          self.add_rule_to_stack(
            SemanticOp::UsePlayer {
              player: player_expr.clone()
            }
          );
        },
        Rule::BidActionMemory(memory, _) => {
          self.add_rule_to_stack(
            SemanticOp::Use {
              id: memory.0.clone(),
              ty: DSLType::Memory
            }
          );
        },
        Rule::EndGameWithWinner(player_expr) => {
          self.add_rule_to_stack(
            SemanticOp::UsePlayer {
              player: player_expr.clone()
            }
          );
        },
        Rule::DemandCardPositionAction(card_position) => {
          self.add_rule_to_stack(
            SemanticOp::UseCardPosition { 
              card_position: card_position.clone()
            }
          );
        },
        Rule::DemandStringAction(string_expr) => {
          self.add_rule_to_stack(
            SemanticOp::UseString {
              string_expr: string_expr.clone()
            }
          );
        },
        Rule::DemandIntAction(int_expr) => {
          self.add_rule_to_stack(
            SemanticOp::UseInt {
              int_expr: int_expr.clone()
            }
          );
        },
        Rule::ClassicMove(classic_move) => {
          self.add_rule_to_stack(
            SemanticOp::UseClassicMove {
              classic_move: classic_move.clone()
            }
          );
        },
        Rule::DealMove(deal_move) => {
          self.add_rule_to_stack(
            SemanticOp::UseDealMove {
              deal_move: deal_move.clone()
            }
          );
        },
        Rule::ExchangeMove(exchange_move) => {
          self.add_rule_to_stack(
            SemanticOp::UseExchangeMove { 
              exchange_move: exchange_move.clone()
            }
          );
        },
        Rule::TokenMove(token_move) => {
          self.add_rule_to_stack(
            SemanticOp::UseTokenMove { 
              token_move: token_move.clone()
            }
          );
        },
        Rule::ScoreRule(score_rule) => {
          self.add_rule_to_stack(
            SemanticOp::UseScoreRule {
              score_rule: score_rule.clone()
            }
          );
        },
        Rule::WinnerRule(winner_rule) => {
          self.add_rule_to_stack(
            SemanticOp::UseWinnerRule {
              winner_rule: winner_rule.clone()
            }
          );
        },
        // -------------------------------------------------
        Rule::EndTurn => {},
        Rule::EndStage => {},
        Rule::BidAction(_) => {},
    }
  }  
}
