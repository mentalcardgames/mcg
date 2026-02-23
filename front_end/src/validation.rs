use std::cell::RefCell;
use std::collections::HashMap;

use pest_consume::Parser;
use crate::ir::{GameFlowError, IrBuilder, SpannedPayload};
use crate::parser::Result;
use crate::semantic::{SemanticError, SemanticVisitor};
use crate::symbols::GameType;
use crate::parser::SymbolTable;
use crate::{ast::ast_spanned::SGame, parser::{CGDSLParser, Rule}, symbols::{SymbolError, SymbolVisitor}, walker::Walker};

pub fn parse_document(text: &str) -> Result<SGame> {
  // 1. Initialize the state your Node alias expects
  let state = RefCell::new(SymbolTable::default());

  // 2. Parsing: pest_consume::parse already returns Result<Nodes, Error<Rule>>
  let nodes = CGDSLParser::parse_with_userdata(Rule::file, text, state)?;

  // 3. Extract Single Node: .single() returns Result<Node, Error<Rule>>
  let node = nodes.single()?;

  // 4. Mapping: mapper returns Result<T, Error<Rule>>
  let parsed_ast = CGDSLParser::file(node)?;

  Ok(parsed_ast)
}

pub fn symbol_validation(game: &SGame) -> std::result::Result<HashMap<GameType, Vec<String>>, Vec<SymbolError>> {
  let mut symbols = SymbolVisitor::new();
  game.walk(&mut symbols);

  match symbols.check_game_type() {
    Some(err) => return Err(err),
    None => {
      Ok(symbols.type_to_variable())
    },
  }
}

pub fn semantic_validation(game: &SGame) -> Option<Vec<SemanticError>> {
  let mut semantic = SemanticVisitor::new();
  game.walk(&mut semantic);

  return semantic.semantic_check()
}

pub fn program_validation(game: &SGame) -> Option<Vec<GameFlowError>> {
  let mut builder: IrBuilder<SpannedPayload> = IrBuilder::default();

  builder.build_ir(game);

  let mut result =  Vec::new();

  if !builder.diagnostics.is_empty() {
    result.extend(builder.diagnostics);
  }

  if let Some(flow_diagnostics) = builder.fsm.diagnostics() {
    result.extend(flow_diagnostics);
  }

  if result.is_empty() {
    return None
  }

  return Some(result)
}
